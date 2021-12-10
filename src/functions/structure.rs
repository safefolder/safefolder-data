use lazy_static::lazy_static;
use regex::{Regex};
use std::{collections::HashMap};
use crate::functions::*;

lazy_static! {
    static ref RE_IF: Regex = Regex::new(r#"IF\([\n\s\t]{0,}(?P<condition>\{[\w\s]+\}[\s]{0,}(=|<|>|<=|>=)[\s]{0,}((\d+)|("[\w\s]+"))),[\s\n\t]{0,}(?P<expr_true>(\d+)|("[\w\s]+")),[\s\n\t]{0,}(?P<expr_false>(\d+)|("[\w\s]+"))[\s\n\t]{0,}\)|IF\([\s\n\t]{0,}(?P<log_condition>(AND\([\s\n\t]{0,}[\w\W\s\n\t]{1,}[\s\n\t]{0,}\)|OR\([\s\n\t]{0,}[\w\W\s\n\t]{1,}[\s\n\t]{0,}\)|NOT\([\s\n\t]{0,}[\w\W\s\n\t]{1,}[\s\n\t]{0,}\)|XOR\([\s\n\t]{0,}[\w\W\s\n\t]{1,}[\s\n\t]{0,}\))),[\s\n\t]{0,}(?P<log_expr_true>(\d+)|("[\w\s]+")),[\s\n\t]{0,}(?P<log_expr_false>(\d+)|("[\w\s]+"))[\s\n\t]{0,}\)"#).unwrap();
    static ref RE_IF_REPLACED: Regex = Regex::new(r#"IF\([\s\n\t]{0,}(?P<condition>([a-zA-Z0-9_<>=\s]+)|(AND\([\w\W\s\n\t]{1,}\))|(OR\([\w\W\s\n\t]{1,}\))|(NOT\([\w\W\s\n\t]{1,}\))|(XOR\([\w\W\s\n\t]{1,}\)))[\s\n\t]{0,},[\s\n\t]{0,}(?P<expr_true>("[\w\s]+")|(\d+))[\s\n\t]{0,},[\s\n\t]{0,}(?P<expr_false>("[\w\s]+")|(\d+))[\s\n\t]{0,}\)"#).unwrap();
    static ref RE_LOGIC_ATTRS: Regex = Regex::new(r#"(?P<op>AND|OR|NOT|XOR)\([\n\s\t]{0,}(?P<attrs>.+)[\n\s\t]{0,}\)"#).unwrap();
    static ref RE_LOGIC_NAME_VALUE: Regex = Regex::new(r#"((?P<name>{*"*[\w\s\d]+}*"*)(?P<op>[=<>])*(?P<value>"*[\w\s]+"*))"#).unwrap();
}

pub trait StructureFunction {
    fn handle(
        &mut self,
    ) -> Result<FunctionParse, PlanetError>;
    fn execute(&self) -> Result<String, PlanetError>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct If {
    function: Option<FunctionParse>,
    data_map: Option<HashMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>
}
impl If {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<HashMap<String, String>>
    ) -> Self {
        return Self{function: function, data_map: data_map, attributes: None};
    }
}
impl StructureFunction for If {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // IF({My Column} = 3, "Mine", "Yours")
        // IF({My Column} = 3, 34, 53)
        // IF({My Column} > 3, "Mine", "Yours")
        // IF({My Column}>3, "Mine", "Yours")
        // IF({My Column}>3, "Mine", "Yours")
        // IF({My Column} >= 3, "Mine", "Yours")
        // IF({My Column} = "pepito mio", "Mine", "Yours")
        // IF(AND({My Column} = 23, {My Other Column} > 4, {My Text Column} = "pepito"),"mine","yours")
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_IF;
        let mut function = function_parse.clone();
        let data_map_wrap = data_map.clone();
        let (
            function_text_wrap,
            function_text,
            compiled_attributes,
            mut function_result,
            data_map,
        ) = prepare_function_parse(function_parse, data_map.clone());
        if function_text_wrap.is_some() {
            function.validate = Some(expr.is_match(function_text.as_str()));
            if function.validate.unwrap() {
                let matches = expr.captures(function_text.as_str()).unwrap();
                let condition = matches.name("condition");
                let expr_true = matches.name("expr_true");
                let expr_false = matches.name("expr_false");
                let log_condition = matches.name("log_condition");
                let log_expr_true = matches.name("log_expr_true");
                let log_expr_false = matches.name("log_expr_false");
                let mut attributes_: Vec<String> = Vec::new();
                if condition.is_some() {
                    let condition = condition.unwrap().as_str().to_string();
                    let expr_true = expr_true.unwrap().as_str().to_string();
                    let expr_false = expr_false.unwrap().as_str().to_string();
                    attributes_.push(condition);
                    attributes_.push(expr_true);
                    attributes_.push(expr_false);
                } else {
                    let condition = log_condition.unwrap().as_str().to_string();
                    let expr_true = log_expr_true.unwrap().as_str().to_string();
                    let expr_false = log_expr_false.unwrap().as_str().to_string();
                    attributes_.push(condition);
                    attributes_.push(expr_true);
                    attributes_.push(expr_false);
                }
                function.attributes = Some(attributes_);
            }
        }
        if data_map_wrap.is_some() {
            self.attributes = Some(compiled_attributes);
            self.data_map = Some(data_map);
            function_result.text = Some(self.execute()?);
            function.result = Some(function_result.clone());
        }
        return Ok(function)
    }
    fn execute(&self) -> Result<String, PlanetError> {
        let attributes = self.attributes.clone().unwrap();
        let data_map = &self.data_map.clone().unwrap();
        let condition_item = attributes[0].clone();
        let condition_value = condition_item.get_value(data_map)?;
        let expr_true_item = attributes[1].clone();
        let expr_false_item = attributes[2].clone();
        let is_assignment = condition_item.assignment.is_some();
        let mut check = false;
        if is_assignment {
            // check if assignment is met
            check = condition_item.check_assignment(data_map);
        } else {
            // Execute formula
            if condition_value == String::from("1") {
                check = true;
            }
        }
        let result: String;
        if check {
            // Return expr_true
            result = expr_true_item.get_value(data_map)?;
        } else {
            // Return expr_false
            result = expr_false_item.get_value(data_map)?;
        }
        return Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct And {
    function: Option<FunctionParse>,
    data_map: Option<HashMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>
}
impl And {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<HashMap<String, String>>
    ) -> Self {
        return Self{function: function, data_map: data_map, attributes: None};
    }
}
impl StructureFunction for And {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // 1.
        // We do 1) so far, no embeddings
        // AND({My Field}="pepito", {Status}="c4vhm0gsmpv7omu4aqg0")
        // 2.
        // AND(
        //    OR({This Field}=78, {Other Field}="hola"),
        //    {This Way}=TRIM(" other ")
        // )
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_LOGIC_ATTRS;
        let mut function = function_parse.clone();
        let data_map_wrap = data_map.clone();
        let (
            function_text_wrap,
            function_text,
            compiled_attributes,
            mut function_result,
            data_map,
        ) = prepare_function_parse(function_parse, data_map.clone());
        if function_text_wrap.is_some() {
            function.validate = Some(expr.is_match(function_text.as_str()));
            if function.validate.unwrap() {
                let matches = expr.captures(function_text.as_str()).unwrap();
                let op = matches.name("op");
                if op.is_some() {
                    let op = op.unwrap().as_str().to_string();
                    if op != String::from("AND") {
                        function.validate = Some(false);
                        return Ok(function)
                    }
                }
                let attrs = matches.name("attrs");
                let mut attributes_: Vec<String> = Vec::new();
                if attrs.is_some() {
                    let attrs = attrs.unwrap().as_str().to_string();
                    attributes_ = formula_attr_collection(attrs)?;
                }
                function.attributes = Some(attributes_);
            }
        }
        if data_map_wrap.is_some() {
            self.attributes = Some(compiled_attributes);
            self.data_map = Some(data_map);
            function_result.text = Some(self.execute()?);
            function.result = Some(function_result.clone());
        }
        return Ok(function)
    }
    fn execute(&self) -> Result<String, PlanetError> {
        let attributes = self.attributes.clone().unwrap();
        let data_map = &self.data_map.clone().unwrap();
        let mut result: String = String::from("1");
        for attribute in attributes {
            // I can have assignment or logic formula to execute
            let assignment = attribute.assignment.clone();
            if assignment.is_some() {
                let check = attribute.check_assignment(data_map);
                if check == false {
                    result = String::from("0");
                    break
                }
            } else {
                // Like OR, NOT, etc...
                let formula_result = attribute.get_value(data_map)?;
                if formula_result == String::from("0") {
                    result = String::from("0");
                    break;
                }
            }
        }
        return Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Or {
    function: Option<FunctionParse>,
    data_map: Option<HashMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>
}
impl Or {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<HashMap<String, String>>
    ) -> Self {
        return Self{function: function, data_map: data_map, attributes: None};
    }
}
impl StructureFunction for Or {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // 1.
        // We do 1) so far, no embeddings
        // AND({My Field}="pepito", {Status}="c4vhm0gsmpv7omu4aqg0")
        // 2.
        // AND(
        //    OR({This Field}=78, {Other Field}="hola"),
        //    {This Way}=TRIM(" other ")
        // )
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_LOGIC_ATTRS;
        let mut function = function_parse.clone();
        let data_map_wrap = data_map.clone();
        let (
            function_text_wrap,
            function_text,
            compiled_attributes,
            mut function_result,
            data_map,
        ) = prepare_function_parse(function_parse, data_map.clone());
        if function_text_wrap.is_some() {
            function.validate = Some(expr.is_match(function_text.as_str()));
            if function.validate.unwrap() {
                let matches = expr.captures(function_text.as_str()).unwrap();
                let op = matches.name("op");
                if op.is_some() {
                    let op = op.unwrap().as_str().to_string();
                    if op != String::from("OR") {
                        function.validate = Some(false);
                        return Ok(function)
                    }
                }
                let attrs = matches.name("attrs");
                let mut attributes_: Vec<String> = Vec::new();
                if attrs.is_some() {
                    let attrs = attrs.unwrap().as_str().to_string();
                    attributes_ = formula_attr_collection(attrs)?;
                }
                function.attributes = Some(attributes_);
            }
        }
        if data_map_wrap.is_some() {
            self.attributes = Some(compiled_attributes);
            self.data_map = Some(data_map);
            function_result.text = Some(self.execute()?);
            function.result = Some(function_result.clone());
        }
        return Ok(function)
    }
    fn execute(&self) -> Result<String, PlanetError> {
        let attributes = self.attributes.clone().unwrap();
        let data_map = &self.data_map.clone().unwrap();
        let mut result: String = String::from("0");
        for attribute in attributes {
            // I can have assignment or logic formula to execute
            let assignment = attribute.assignment.clone();
            if assignment.is_some() {
                let check = attribute.check_assignment(data_map);
                if check == true {
                    result = String::from("1");
                    break
                }
            } else {
                // Like OR, NOT, etc...
                let formula_result = attribute.get_value(data_map)?;
                if formula_result == String::from("1") {
                    result = String::from("1");
                    break
                }
            }
        }
        return Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Not {
    function: Option<FunctionParse>,
    data_map: Option<HashMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>
}
impl Not {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<HashMap<String, String>>
    ) -> Self {
        return Self{function: function, data_map: data_map, attributes: None};
    }
}
impl StructureFunction for Not {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // 1.
        // We do 1) so far, no embeddings
        // AND({My Field}="pepito", {Status}="c4vhm0gsmpv7omu4aqg0")
        // 2.
        // AND(
        //    OR({This Field}=78, {Other Field}="hola"),
        //    {This Way}=TRIM(" other ")
        // )
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_LOGIC_ATTRS;
        let mut function = function_parse.clone();
        let data_map_wrap = data_map.clone();
        let (
            function_text_wrap,
            function_text,
            compiled_attributes,
            mut function_result,
            data_map,
        ) = prepare_function_parse(function_parse, data_map.clone());
        if function_text_wrap.is_some() {
            function.validate = Some(expr.is_match(function_text.as_str()));
            if function.validate.unwrap() {
                let matches = expr.captures(function_text.as_str()).unwrap();
                let op = matches.name("op");
                if op.is_some() {
                    let op = op.unwrap().as_str().to_string();
                    if op != String::from("NOT") {
                        function.validate = Some(false);
                        return Ok(function)
                    }
                }
                let attrs = matches.name("attrs");
                let mut attributes_: Vec<String> = Vec::new();
                if attrs.is_some() {
                    let attrs = attrs.unwrap().as_str().to_string();
                    attributes_ = formula_attr_collection(attrs)?;
                }
                function.attributes = Some(attributes_);
            }
        }
        if data_map_wrap.is_some() {
            self.attributes = Some(compiled_attributes);
            self.data_map = Some(data_map);
            function_result.text = Some(self.execute()?);
            function.result = Some(function_result.clone());
        }
        return Ok(function)
    }
    fn execute(&self) -> Result<String, PlanetError> {
        let attributes = self.attributes.clone().unwrap();
        let data_map = &self.data_map.clone().unwrap();
        let mut result: String = String::from("1");
        for attribute in attributes {
            // I can have assignment or logic formula to execute
            let assignment = attribute.assignment.clone();
            if assignment.is_some() {
                let check = attribute.check_assignment(data_map);
                if check == true {
                    result = String::from("0");
                    break
                }
            } else {
                // Like OR, NOT, etc...
                let formula_result = attribute.get_value(data_map)?;
                if formula_result == String::from("1") {
                    result = String::from("0");
                    break
                }
            }
        }
        return Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Xor {
    function: Option<FunctionParse>,
    data_map: Option<HashMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>
}
impl Xor {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<HashMap<String, String>>
    ) -> Self {
        return Self{function: function, data_map: data_map, attributes: None};
    }
}
impl StructureFunction for Xor {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // 1.
        // We do 1) so far, no embeddings
        // AND({My Field}="pepito", {Status}="c4vhm0gsmpv7omu4aqg0")
        // 2.
        // AND(
        //    OR({This Field}=78, {Other Field}="hola"),
        //    {This Way}=TRIM(" other ")
        // )
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_LOGIC_ATTRS;
        let mut function = function_parse.clone();
        let data_map_wrap = data_map.clone();
        let (
            function_text_wrap,
            function_text,
            compiled_attributes,
            mut function_result,
            data_map,
        ) = prepare_function_parse(function_parse, data_map.clone());
        if function_text_wrap.is_some() {
            function.validate = Some(expr.is_match(function_text.as_str()));
            if function.validate.unwrap() {
                let matches = expr.captures(function_text.as_str()).unwrap();
                let op = matches.name("op");
                if op.is_some() {
                    let op = op.unwrap().as_str().to_string();
                    if op != String::from("XOR") {
                        function.validate = Some(false);
                        return Ok(function)
                    }
                }
                let attrs = matches.name("attrs");
                let mut attributes_: Vec<String> = Vec::new();
                if attrs.is_some() {
                    let attrs = attrs.unwrap().as_str().to_string();
                    attributes_ = formula_attr_collection(attrs)?;
                }
                function.attributes = Some(attributes_);
            }
        }
        if data_map_wrap.is_some() {
            self.attributes = Some(compiled_attributes);
            self.data_map = Some(data_map);
            function_result.text = Some(self.execute()?);
            function.result = Some(function_result.clone());
        }
        return Ok(function)
    }
    fn execute(&self) -> Result<String, PlanetError> {
        let attributes = self.attributes.clone().unwrap();
        let data_map = &self.data_map.clone().unwrap();
        let mut result: String = String::from("0");
        let mut count = 0;
        for attribute in attributes {
            // I can have assignment or logic formula to execute
            let assignment = attribute.assignment.clone();
            if assignment.is_some() {
                let check = attribute.check_assignment(data_map);
                if check == true {
                    result = String::from("1");
                    count += 1;
                }
            } else {
                // Like OR, NOT, etc...
                let formula_result = attribute.get_value(data_map)?;
                if formula_result == String::from("1") {
                    result = String::from("1");
                    count += 1;
                }
            }
        }
        if count == 1 {
            result = String::from("1");
        }    
        return Ok(result)
    }
}

pub fn and(
    data_map: &HashMap<String, String>, 
    attributes: &Vec<FunctionAttributeItem>
) -> Result<bool, PlanetError> {
    // AND({My Field}="pepito", {Status}="c4vhm0gsmpv7omu4aqg0")

    // 2.
    // AND(
    //    OR({This Field}=78, {Other Field}="hola"),
    //    {This Way}=TRIM(" other ")
    // )

    let mut check_all = true;
    for attribute in attributes {
        let assignment = attribute.assignment.clone();
        let attr_type = attribute.attr_type.clone();
        if assignment.is_some() {
            let assignment = assignment.unwrap();
            let check = check_assignment(assignment, attr_type, data_map)?;
            if check == false {
                check_all = false;
                break
            }
        }
    }
    return Ok(check_all)
}

pub fn or(
    data_map: &HashMap<String, String>, 
    attributes: &Vec<FunctionAttributeItem>
) -> Result<bool, PlanetError> {
    let mut check = false;
    for attribute in attributes {
        let assignment = attribute.assignment.clone();
        let attr_type = attribute.attr_type.clone();
        if assignment.is_some() {
            let assignment = assignment.unwrap();
            let check_item = check_assignment(assignment, attr_type, data_map)?;
            if check_item == true {
                check = true;
                break
            }
        }
    }
    return Ok(check)
}

pub fn not(
    data_map: &HashMap<String, String>, 
    attributes: &Vec<FunctionAttributeItem>
) -> Result<bool, PlanetError> {
    let mut check_all = true;
    for attribute in attributes {
        let assignment = attribute.assignment.clone();
        let attr_type = attribute.attr_type.clone();
        if assignment.is_some() {
            let assignment = assignment.unwrap();
            let check_item = check_assignment(assignment, attr_type, data_map)?;
            if check_item == true {
                check_all = false;
                break
            }
        }
    }
    return Ok(check_all)
}

pub fn xor(
    data_map: &HashMap<String, String>, 
    attributes: &Vec<FunctionAttributeItem>
) -> Result<bool, PlanetError> {
    // Only 1 of the items needs to be true
    // Many true, we return false, which is not the case on the OR
    let mut check = false;
    let mut count = 0;
    for attribute in attributes {
        let assignment = attribute.assignment.clone();
        let attr_type = attribute.attr_type.clone();
        if assignment.is_some() {
            let assignment = assignment.unwrap();
            let check_item = check_assignment(assignment, attr_type, data_map)?;
            if check_item == true {
                check = true;
                count += 1;
            }
        }
    }
    if count == 1 {
        check = true;
    }
    return Ok(check)
}
