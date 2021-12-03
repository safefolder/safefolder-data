use lazy_static::lazy_static;
use regex::{Regex};
use std::{collections::HashMap};
use crate::functions::*;

lazy_static! {
    static ref RE_IF: Regex = Regex::new(r#"IF\([\n\s\t]{0,}(?P<condition>\{[\w\s]+\}[\s]{0,}(=|<|>|<=|>=)[\s]{0,}((\d+)|("[\w\s]+"))),[\s\n\t]{0,}(?P<expr_true>(\d+)|("[\w\s]+")),[\s\n\t]{0,}(?P<expr_false>(\d+)|("[\w\s]+"))[\s\n\t]{0,}\)|IF\([\s\n\t]{0,}(?P<log_condition>(AND\([\s\n\t]{0,}[\w\W\s\n\t]{1,}[\s\n\t]{0,}\)|OR\([\s\n\t]{0,}[\w\W\s\n\t]{1,}[\s\n\t]{0,}\)|NOT\([\s\n\t]{0,}[\w\W\s\n\t]{1,}[\s\n\t]{0,}\)|XOR\([\s\n\t]{0,}[\w\W\s\n\t]{1,}[\s\n\t]{0,}\))),[\s\n\t]{0,}(?P<log_expr_true>(\d+)|("[\w\s]+")),[\s\n\t]{0,}(?P<log_expr_false>(\d+)|("[\w\s]+"))[\s\n\t]{0,}\)"#).unwrap();
    static ref RE_IF_REPLACED: Regex = Regex::new(r#"IF\([\s\n\t]{0,}(?P<condition>([a-zA-Z0-9_<>=\s]+)|(AND\([\w\W\s\n\t]{1,}\))|(OR\([\w\W\s\n\t]{1,}\))|(NOT\([\w\W\s\n\t]{1,}\))|(XOR\([\w\W\s\n\t]{1,}\)))[\s\n\t]{0,},[\s\n\t]{0,}(?P<expr_true>("[\w\s]+")|(\d+))[\s\n\t]{0,},[\s\n\t]{0,}(?P<expr_false>("[\w\s]+")|(\d+))[\s\n\t]{0,}\)"#).unwrap();
    static ref RE_AND: Regex = Regex::new(r#"AND\([\n\s\t]{0,}(?P<attrs>.+)[\n\s\t]{0,}\)"#).unwrap();
}

pub trait StructureFunction {
    fn handle(
        &mut self,
    ) -> Result<FunctionParse, PlanetError>;
    fn execute(&self) -> Result<String, PlanetError>;
}

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct IfFormulaFieldCompiled {
//     pub condition: FormulaFieldCompiled,
//     pub true_formula: FormulaFieldCompiled,
//     pub false_formula: FormulaFieldCompiled,
// }
// impl IfFormulaFieldCompiled{
//     pub fn defaults(
//         formula: &String,
//         formula_format: &String,
//         field_type_map: &HashMap<String, String>,
//     ) -> Result<Self, PlanetError> {
//         // IF({My Column} = 3, "Mine", "Yours")
//         // IF({My Column} = 3, 34, 53)
//         // IF({My Column} > 3, "Mine", "Yours")
//         // IF({My Column}>3, "Mine", "Yours")
//         // IF({My Column}>3, "Mine", "Yours")
//         // IF({My Column} >= 3, "Mine", "Yours")
//         // IF({My Column} = "pepito mio", "Mine", "Yours")
//         // IF(AND({My Column} = 23, {My Other Column} > 4, {My Text Column} = "pepito"),"mine","yours")
//         let formula_str = formula.as_str();
//         let expr = &RE_IF;
//         let validate = Some(expr.is_match(formula_str)).unwrap();
//         if !validate {
//             return Err(
//                 PlanetError::new(
//                     500, 
//                     Some(tr!("function {} not having the expected format, validation error", &formula_str)),
//                 )
//             );    
//         }
//         let matches = expr.captures(formula_str).unwrap();
//         let condition = matches.name("condition");
//         let expr_true = matches.name("expr_true");
//         let expr_false = matches.name("expr_false");
//         let log_condition = matches.name("log_condition");
//         let log_expr_true = matches.name("log_expr_true");
//         let log_expr_false = matches.name("log_expr_false");
//         let mut condition_formula: String = String::from("");
//         let mut true_formula: String = String::from("");
//         let mut false_formula: String = String::from("");
//         if condition.is_some() && expr_true.is_some() && expr_false.is_some() {
//             condition_formula = condition.unwrap().as_str().to_string();
//             true_formula = expr_true.unwrap().as_str().to_string();
//             false_formula = expr_false.unwrap().as_str().to_string();
//         }
//         if log_condition.is_some() && log_expr_true.is_some() && log_expr_false.is_some() {
//             condition_formula = log_condition.unwrap().as_str().to_string();
//             true_formula = log_expr_true.unwrap().as_str().to_string();
//             false_formula = log_expr_false.unwrap().as_str().to_string();
//         }
//         // {Column} = 5, etc...
//         let condition_compiled = FormulaFieldCompiled::defaults(
//             &condition_formula,
//             formula_format,
//             field_type_map,
//         )?;
//         let true_compiled = FormulaFieldCompiled::defaults(
//             &true_formula,
//             formula_format,
//             field_type_map,
//         )?;
//         let false_compiled = FormulaFieldCompiled::defaults(
//             &false_formula,
//             formula_format,
//             field_type_map,
//         )?;
//         let obj = Self{
//             condition: condition_compiled,
//             true_formula: true_compiled,
//             false_formula: false_compiled,
//         };
//         return Ok(obj)
//     }
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct If {
//     function: Option<FunctionParse>,
//     data_map: Option<HashMap<String, String>>,
//     attributes: Option<Vec<FunctionAttributeItem>>
// }
// impl If {
//     pub fn defaults(
//         function: Option<FunctionParse>, 
//         data_map: Option<HashMap<String, String>>
//     ) -> Self {
//         return Self{function: function, data_map: data_map, attributes: None};
//     }
// }
// impl IfFunction for If {
//     fn handle(
//         &mut self, 
//     ) -> Result<FunctionParse, PlanetError> {
        // IF({My Column} = 3, "Mine", "Yours")
        // IF({My Column} = 3, TRIM(" Mine "), "Yours") ????
        // IF({My Column} = 3, 34, 53)
        // IF({My Column} > 3, "Mine", "Yours")
        // IF({My Column}>3, "Mine", "Yours")
        // IF({My Column}>3, "Mine", "Yours")
        // IF({My Column} >= 3, "Mine", "Yours")
        // IF({My Column} = "pepito mio", "Mine", "Yours")
        // IF(AND({My Column} = 23, {My Other Column} > 4, {My Text Column} = "pepito"),"mine","yours")
        // The true and false statements are formulas which are compiled and executed
        // What to do if we have AND, OR, NOT, XOR? => Convert the condition into logical, then process that.

        // I need to have clue on how to do this
        // FunctionParse: Has name, attributes, compiled attributes and validate
        // Seems attributes is [], since the attributes are compiled formulas

        // IF({My Column} = 3, "Mine", "Yours")
        // handle can compile the three formulas???
        // Spec is true and false are formulas instead of text/number values
        // process_function deals with FunctionParse and is one to handle and execute all functions, including IF

        // Workflow for normal functions
        // 1. config takes formula text
        // 2. config calls FormulaFieldCompiled sending fomula text
        // 3. FormulaFieldCompiled compiles the formula and calls process_function for each function in the 
        //    formula.
        // 4. process_function calls function handle to compile attributes and execute function when compiled
        //    attributes present.

        // Workflow for IF function

        // We have same as above for the IF function, since it may be embedded into a formula in different ways
        // Feature that a compiled attribute (FunctionAttributeItem) may have a compiled formula for complex 
        //    structures like IF. Not possible since formula -> functions -> attribute items. Attributes items
        //    cannot link to formula.

        // Hint: I could make that the attribute may be a function (not a formula), or a value

        // 1. config takes formula text
        // 2. Checks has IF(

        // let function_parse = &self.function.clone().unwrap();
        // let data_map = self.data_map.clone();
        // let expr = &RE_IF;
        // let mut function = function_parse.clone();
        // let data_map_wrap = data_map.clone();
        // let (
        //     function_text_wrap, 
        //     function_text, 
        //     compiled_attributes,
        //     mut function_result,
        //     data_map,
        // ) = prepare_function_parse(function_parse, data_map.clone());
        // if function_text_wrap.is_some() {
        //     function.validate = Some(expr.is_match(function_text.as_str()));
        //     if function.validate.unwrap() {
                // let formula_field = IfFormulaFieldCompiled::defaults(
                //     &function_text, 
                //     formula_format, 
                //     field_type_map
                // )?;
                // What I do with attributes????

                // let matches = expr.captures(function_text.as_str()).unwrap();
                // let condition = matches.name("condition");
                // let expr_true = matches.name("expr_true");
                // let expr_false = matches.name("expr_false");
                // let log_condition = matches.name("log_condition");
                // let log_expr_true = matches.name("log_expr_true");
                // let log_expr_false = matches.name("log_expr_false");
                // let mut attributes_: Vec<String> = Vec::new();
                // if condition.is_some() && expr_true.is_some() && expr_false.is_some() {
                //     let condition = condition.unwrap().as_str().to_string();
                //     let expr_true = expr_true.unwrap().as_str().to_string();
                //     let expr_false = expr_false.unwrap().as_str().to_string();
                //     attributes_.push(condition);
                //     attributes_.push(expr_true);
                //     attributes_.push(expr_false);
                // }
                // if log_condition.is_some() && log_expr_true.is_some() && log_expr_false.is_some() {
                //     let condition = log_condition.unwrap().as_str().to_string();
                //     let expr_true = log_expr_true.unwrap().as_str().to_string();
                //     let expr_false = log_expr_false.unwrap().as_str().to_string();
                //     attributes_.push(condition);
                //     attributes_.push(expr_true);
                //     attributes_.push(expr_false);
                // }
                
                // function.attributes = Some(attributes_);
    //         }
    //     }
    //     if data_map_wrap.is_some() {
    //         self.attributes = Some(compiled_attributes);
    //         self.data_map = Some(data_map);
    //         function_result.text = Some(self.execute()?);
    //         function.result = Some(function_result.clone());
    //     }
    //     return Ok(function)
    // }
    // fn execute(&self) -> Result<String, PlanetError> {
        // let attributes = self.attributes.clone().unwrap();
        // let data_map = &self.data_map.clone().unwrap();
        // let attribute_item = attributes[0].clone();

        // let formula_obj = Formula::defaults(
        //     Some(insert_data_map),
        //     Some(table.clone()),
        // );
        // let formula_wrap = formula_obj.inyect_data_formula(&formula);
        // let formula_if = formula_wrap.unwrap();
        // let replacement_string: String;

        // let matches = RE_IF_REPLACED.captures(&formula_if);
        // let matches = matches.unwrap();
        // let processed_condition = matches.name("condition").unwrap().as_str().to_string();
        // let formula_obj = Formula::defaults(
        //     self.data_map.clone(),
        //     self.table.clone(),
        // );
        // let result = formula_obj.execute(&processed_condition).unwrap();

        // if result == "TRUE" {
        //     let formula_obj = Formula::defaults(
        //         self.data_map.clone(), 
        //         self.table.clone()
        //     );
        //     let result = formula_obj.execute(&expr_true).unwrap();
        //     replacement_string = result;
        // } else {
        //     let formula_obj = Formula::defaults(
        //         self.data_map.clone(), 
        //         self.table.clone()
        //     );
        //     let result = formula_obj.execute(&expr_false).unwrap();
        //     replacement_string = result;
        // }

//         return Ok(String::from(""))
//     }
// }

// IF({Sales} > 50, "Win", "Loose")
// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct IfFunction {
//     pub function_text: String,
//     pub condition: Option<String>,
//     pub expr_true: Option<String>,
//     pub expr_false: Option<String>,
//     pub data_map: Option<HashMap<String, String>>,
//     pub table: Option<DbData>,
// }
// impl IfFunction {
//     pub fn defaults(function_text: &String, data_map: Option<HashMap<String, String>>, table: Option<DbData>) -> IfFunction {
//         // IF({My Column} = 3, "Mine", "Yours")
//         // IF({My Column} > 3, "Mine", "Yours")
//         // IF({My Column}>3, "Mine", "Yours")
//         // IF({My Column}>3,"Mine","Yours")
//         // IF({My Column} >= 3, "Mine", "Yours")
//         // IF({My Column} = "pepito mio", "Mine", "Yours")
//         // IF(AND({My Column} = 23, {My Other Column} > 4, {My Text Column} = "pepito"),"mine","yours")
//         let matches = RE_IF.captures(function_text).unwrap();
//         let condition = matches.name("condition");
//         let expr_true = matches.name("expr_true");
//         let expr_false = matches.name("expr_false");
//         let log_condition = matches.name("log_condition");
//         let log_expr_true = matches.name("log_expr_true");
//         let log_expr_false = matches.name("log_expr_false");

//         let mut condition_wrap: Option<String> = None;
//         let mut expr_true_wrap: Option<String> = None;
//         let mut expr_false_wrap: Option<String> = None;

//         if condition.is_some() && expr_true.is_some() && expr_false.is_some() {
//             condition_wrap = Some(condition.unwrap().as_str().to_string());
//             expr_true_wrap = Some(expr_true.unwrap().as_str().to_string());
//             expr_false_wrap = Some(expr_false.unwrap().as_str().to_string());
//         }
//         if log_condition.is_some() && log_expr_true.is_some() && log_expr_false.is_some() {
//             condition_wrap = Some(log_condition.unwrap().as_str().to_string());
//             expr_true_wrap = Some(log_expr_true.unwrap().as_str().to_string());
//             expr_false_wrap = Some(log_expr_false.unwrap().as_str().to_string());
//         }

//         let obj = Self{
//             function_text: function_text.clone(),
//             condition: condition_wrap,
//             expr_true: expr_true_wrap,
//             expr_false: expr_false_wrap,
//             data_map: data_map,
//             table: table,
//         };
        
//         return obj
//     }
//     pub fn do_validate(
//         function_text: &String, 
//         validate_tuple: (u32, Vec<String>),
//     ) -> (u32, Vec<String>) {
//         let (number_fails, mut failed_functions) = validate_tuple;
//         let concat_obj = IfFunction::defaults(
//             &function_text, None, None
//         );
//         let check = concat_obj.validate();
//         let mut number_fails = number_fails.clone();
//         if check == false {
//             number_fails += 1;
//             failed_functions.push(String::from(FUNCTION_IF));
//         }
//         return (number_fails, failed_functions);
//     }
//     pub fn do_replace(
//         function_text: &String, 
//         mut formula: String,
//         insert_data_map: HashMap<String, String>,
//         table: &DbData,
//     ) -> String {
//         let table = table.clone();
//         let mut concat_obj = IfFunction::defaults(
//             &function_text, Some(insert_data_map), Some(table)
//         );
//         formula = concat_obj.replace(formula);
//         return formula
//     }
// }
// impl Function for IfFunction {
//     fn validate(&self) -> bool {
//         let expr = RE_IF.clone();
//         let function_text = self.function_text.clone();
//         let check = expr.is_match(&function_text);
//         return check
//     }
//     fn replace(&mut self, formula: String) -> String {
//         // let insert_data_map = insert_data_map.clone();
//         let insert_data_map = self.data_map.clone().unwrap();
//         let function_text = self.function_text.clone();
//         let mut formula = formula.clone();
//         // let table = table.clone();
//         let table = self.table.clone().unwrap();

//         let expr_true_wrap = self.expr_true.clone();
//         let expr_false_wrap = self.expr_false.clone();
//         let expr_true = expr_true_wrap.unwrap();
//         let expr_false = expr_false_wrap.unwrap();

//         let formula_obj = Formula::defaults(
//             Some(insert_data_map),
//             Some(table.clone()),
//         );
//         let formula_wrap = formula_obj.inyect_data_formula(&formula);
//         let formula_if = formula_wrap.unwrap();
//         let replacement_string: String;

//         let matches = RE_IF_REPLACED.captures(&formula_if);
//         let matches = matches.unwrap();
//         let processed_condition = matches.name("condition").unwrap().as_str().to_string();
//         let formula_obj = Formula::defaults(
//             self.data_map.clone(),
//             self.table.clone(),
//         );
//         let result = formula_obj.execute(&processed_condition).unwrap();

//         if result == "TRUE" {
//             let formula_obj = Formula::defaults(
//                 self.data_map.clone(), 
//                 self.table.clone()
//             );
//             let result = formula_obj.execute(&expr_true).unwrap();
//             replacement_string = result;
//         } else {
//             let formula_obj = Formula::defaults(
//                 self.data_map.clone(), 
//                 self.table.clone()
//             );
//             let result = formula_obj.execute(&expr_false).unwrap();
//             replacement_string = result;
//         }

//         formula = formula.replace(function_text.as_str(), replacement_string.as_str());
//         formula = format!("\"{}\"", formula);
//         return formula;
//     }
// }


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
        // AND({My Field}="pepito", {Status}="c4vhm0gsmpv7omu4aqg0")
        // 2.
        // AND(
        //    OR({This Field}=78, {Other Field}="hola"),
        //    {This Way}=TRIM(" other ")
        // )
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_AND;
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
                let matches = &expr.captures(function_text.as_str()).unwrap();
                let attrs = matches.name("attrs");
                let mut attributes_: Vec<String> = Vec::new();
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
        return Ok(String::from(""))
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
