use std::str::FromStr;
use regex::Regex;
use std::{collections::HashMap};
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;

use crate::planet::PlanetError;

use crate::functions::*;
use crate::functions::constants::*;

// 1. defaults receives the function test, that is FUNC_NAME(attr1, attr2, ...) and returns function instance
//      attributes embeeded into the object attributes. It does not prepare for replacement or anythin.
// 2. replace is the one that received the data map and gets attributes from function and does replacement.
// 3. do_replace: Creates the object and does replacement.
// 4. validate: Validates the function text with respect to the regex for the function, checks function text is
//       fine.

// I need to refactor and extract common operations that can be used in other functions as well and place into
//     the mod inside functions.

lazy_static! {
    pub static ref RE_CONCAT_ATTRS: Regex = Regex::new(r#"("[\w\s-]+")|(\d+)|(\{[\w\s]+\})"#).unwrap();
    pub static ref RE_FORMAT_COLUMNS: Regex = Regex::new(r"(\{[\w\s-]+\})").unwrap();
    pub static ref RE_JOINLIST_ATTRS: Regex = Regex::new(r#"(?P<array>\{[\w\s\d,"-]+\}),[\s+]{0,}(?P<sep>\\{0,1}"[\W]\\{0,1}")"#).unwrap();
    pub static ref RE_LEN_ATTR: Regex = Regex::new(r#"("[\w\s-]+")|(\{[\w\s]+\})"#).unwrap();
    pub static ref RE_SINGLE_ATTR: Regex = Regex::new(r#"("[\w\s-]+")|(\{[\w\s]+\})"#).unwrap();
    pub static ref RE_REPLACE: Regex = Regex::new(r#"REPLACE\([\n\s\t]{0,}"(?P<old_text>[\w\s]+)"[\n\s\t]{0,},[\n\s\t]{0,}(?P<start_num>\d)[\n\s\t]{0,},[\n\s\t]{0,}(?P<num_chars>\d)[\n\s\t]{0,},[\n\s\t]{0,}"(?P<new_text>[\w\s]+)"[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_MID: Regex = Regex::new(r#"MID\([\s\n\t]{0,}((?P<text>"[\w\s]+")|(?P<text_ref>\{[\w\s]+\}))[\s\n\t]{0,},[\s\n\t]{0,}(?P<start_num>\d+)[\s\n\t]{0,},[\s\n\t]{0,}(?P<num_chars>\d+)[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_REPT: Regex = Regex::new(r#"REPT\([\s\n\t]{0,}((?P<text>"[\w\s\W]+")|(?P<text_ref>\{[\w\s]+\}))[\s\n\t]{0,},[\s\n\t]{0,}(?P<number_times>\d+)[\s\t\n]{0,}\)"#).unwrap();
    pub static ref RE_SUBSTITUTE: Regex = Regex::new(r#"SUBSTITUTE\([\s\n\t]{0,}((?P<text>("[\w\s]+"))|(?P<text_ref>(\{[\w\s]+\})))[\s\n\t]{0,},[\s\n\t]{0,}(?P<old_text>"[\w\s]+")[\s\n\t]{0,},[\s\n\t]{0,}(?P<new_text>"[\w\s]+")[\s\t\n]{0,}\)"#).unwrap();
    pub static ref RE_TRIM: Regex = Regex::new(r#"TRIM\([\s\n\t]{0,}((?P<text>"[\w\s]+")|(?P<text_ref>\{[\w\s]+\}))[\s\n\t]{0,}\)"#).unwrap();
}

pub fn concat(
    data_map: &HashMap<String, String>, 
    attributes: Vec<FunctionAttributeItem>
) -> Result<String, PlanetError> {
    let mut attributes_processed: Vec<String> = Vec::new();
    for attribute_item in attributes {
        let is_reference = attribute_item.is_reference;
        let reference_value_wrap = attribute_item.reference_value;
        let value = attribute_item.value.unwrap();
        let mut attribute: String;
        if is_reference == true {
            // I can have reference and need to get data from the data_map_names, or reference already comes
            // with value.
            if reference_value_wrap.is_some() {
                let reference_value = reference_value_wrap.unwrap();
                attribute = reference_value;
            } else {
                attribute = value;
            }
            let function_attr = FunctionAttributeNew::defaults(
                &attribute, Some(true)
            );
            attribute = function_attr.replace(data_map).item_processed.unwrap();
        } else {
            attribute = value;
        }
        attributes_processed.push(attribute);
    }
    let result = attributes_processed.join("");
    return Ok(result)
}

// pub fn concat_validate(
//     function_text: &String, 
//     validate_tuple: (u32, Vec<String>)
//  ) -> (u32, Vec<String>) {
//     let (number_fails, mut failed_functions) = validate_tuple;
//     let expr = &RE_CONCAT_ATTRS;
//     let check = expr.is_match(&function_text);
//     let mut number_fails = number_fails.clone();
//     if check == false {
//         number_fails += 1;
//         failed_functions.push(String::from(FUNCTION_CONCAT));
//     }
//     return (number_fails, failed_functions);
// }

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConcatenateFunction {
    pub function_text: String,
    pub attributes: Vec<String>,
    pub attributes_value_map: Option<HashMap<String, String>>,
    pub data_map: Option<HashMap<String, String>>,
}

impl ConcatenateFunction{
    // CONCAT(arg1, arg2, ...)

    // I need a structure so replace and do_replace have evrything they need.

    pub fn defaults(function_text: &String, data_map: Option<HashMap<String, String>>) -> ConcatenateFunction {
        let mut attributes: Vec<String> = Vec::new();
        for capture in RE_CONCAT_ATTRS.captures_iter(function_text) {
            let attribute = capture.get(0).unwrap().as_str().to_string();
            attributes.push(attribute);
        }
        let obj = Self{
            function_text: function_text.clone(),
            attributes: attributes,
            attributes_value_map: None,
            data_map: data_map,
        };
        return obj
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let concat_obj = ConcatenateFunction::defaults(
            &function_text, None
        );
        let (number_fails, mut failed_functions) = validate_tuple;
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_CONCAT));
        }
        return (number_fails, failed_functions);
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        // I can't have this, since constructor executes REGEX, and I need this fast
        let mut concat_obj = ConcatenateFunction::defaults(
            &function_text,Some(data_map)
        );
        formula = concat_obj.replace(formula);
        return formula
    }
}
impl Function for ConcatenateFunction {
    fn validate(&self) -> bool {
        let expr = RE_CONCAT_ATTRS.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    fn replace(&mut self, formula: String) -> String {
        let data_map = self.data_map.clone().unwrap();
        let mut attributes_processed: Vec<String> = Vec::new();
        let function_text = self.function_text.clone();
        for capture in RE_CONCAT_ATTRS.captures_iter(&function_text) {
            let attribute = capture.get(0).unwrap().as_str().to_string();
            let function_attr = FunctionAttribute::defaults(&attribute, Some(true));
            let attribute_processed = function_attr.replace(data_map.clone()).item_processed.unwrap();
            attributes_processed.push(attribute_processed);
        };
        let attributes_process_ = attributes_processed.clone();
        let mut formula = formula.clone();
        let value = attributes_processed.join("");
        let value = value.as_str();
        formula = formula.replace(self.function_text.as_str(), value);
        // Update attribute_value_map
        let mut map: HashMap<String, String> = HashMap::new();
        let attributes = self.attributes.clone();
        for (i, attribute) in attributes.iter().enumerate() {
            let attribute = attribute.clone();
            let attribute_processed = &attributes_process_[i];
            map.insert(attribute, attribute_processed.clone());
        }
        self.attributes_value_map = Some(map);
        // formula LIB needs to parse and process this, so we need quotes, so handles like a string
        formula = format!("{}{}{}", 
            String::from("\""),
            formula,
            String::from("\""),
        );
        return formula
    }
}

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct ConcatenateFunction {
//     pub function_text: String,
//     pub attributes: Vec<String>,
//     pub attributes_value_map: Option<HashMap<String, String>>,
//     pub data_map: Option<HashMap<String, String>>,
// }

// impl ConcatenateFunction{
//     // CONCAT(arg1, arg2, ...)
//     pub fn defaults(function_text: &String, data_map: Option<HashMap<String, String>>) -> ConcatenateFunction {
//         let mut attributes: Vec<String> = Vec::new();
//         for capture in RE_CONCAT_ATTRS.captures_iter(function_text) {
//             let attribute = capture.get(0).unwrap().as_str().to_string();
//             attributes.push(attribute);
//         }
//         let obj = Self{
//             function_text: function_text.clone(),
//             attributes: attributes,
//             attributes_value_map: None,
//             data_map: data_map,
//         };
//         return obj
//     }
//     pub fn do_validate(
//         function_text: &String, 
//         validate_tuple: (u32, Vec<String>)
//     ) -> (u32, Vec<String>) {
//         let concat_obj = ConcatenateFunction::defaults(
//             &function_text, None
//         );
//         let (number_fails, mut failed_functions) = validate_tuple;
//         let check = concat_obj.validate();
//         let mut number_fails = number_fails.clone();
//         if check == false {
//             number_fails += 1;
//             failed_functions.push(String::from(FUNCTION_CONCAT));
//         }
//         return (number_fails, failed_functions);
//     }
//     pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
//         let data_map = data_map.clone();
//         let mut concat_obj = ConcatenateFunction::defaults(
//             &function_text,Some(data_map)
//         );
//         formula = concat_obj.replace(formula);
//         return formula
//     }
// }
// impl Function for ConcatenateFunction {
//     fn validate(&self) -> bool {
//         let expr = RE_CONCAT_ATTRS.clone();
//         let function_text = self.function_text.clone();
//         let check = expr.is_match(&function_text);
//         return check
//     }
//     fn replace(&mut self, formula: String) -> String {
//         let data_map = self.data_map.clone().unwrap();
//         let mut attributes_processed: Vec<String> = Vec::new();
//         let function_text = self.function_text.clone();
//         for capture in RE_CONCAT_ATTRS.captures_iter(&function_text) {
//             let attribute = capture.get(0).unwrap().as_str().to_string();
//             let function_attr = FunctionAttribute::defaults(&attribute, Some(true));
//             let attribute_processed = function_attr.replace(data_map.clone()).item_processed.unwrap();
//             attributes_processed.push(attribute_processed);
//         };
//         let attributes_process_ = attributes_processed.clone();
//         let mut formula = formula.clone();
//         let value = attributes_processed.join("");
//         let value = value.as_str();
//         formula = formula.replace(self.function_text.as_str(), value);
//         // Update attribute_value_map
//         let mut map: HashMap<String, String> = HashMap::new();
//         let attributes = self.attributes.clone();
//         for (i, attribute) in attributes.iter().enumerate() {
//             let attribute = attribute.clone();
//             let attribute_processed = &attributes_process_[i];
//             map.insert(attribute, attribute_processed.clone());
//         }
//         self.attributes_value_map = Some(map);
//         // formula LIB needs to parse and process this, so we need quotes, so handles like a string
//         formula = format!("{}{}{}", 
//             String::from("\""),
//             formula,
//             String::from("\""),
//         );
//         return formula
//     }
// }


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FormatFunction {
    pub function_text: String,
    pub format: Option<String>,
    pub data_map: Option<HashMap<String, String>>,
}
impl FormatFunction {
    pub fn defaults(function_text: &String, data_map: Option<HashMap<String, String>>) -> FormatFunction {
        // FORMAT("{My Column}-45-pepito")
        let obj = Self{
            format: None,
            function_text: function_text.clone(),
            data_map: data_map,
        };
        return obj
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let concat_obj = FormatFunction::defaults(
            &function_text, None
        );
        let (number_fails, mut failed_functions) = validate_tuple;
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_FORMAT));
        }
        return (number_fails, failed_functions);
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = FormatFunction::defaults(
            &function_text,Some(data_map)
        );
        formula = concat_obj.replace(formula);
        return formula
    }
}
impl Function for FormatFunction {
    fn validate(&self) -> bool {
        let expr = RE_FORMAT_COLUMNS.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    fn replace(&mut self, formula: String) -> String {
        // FORMAT("Hello-{Column A}-45")
        let data_map = self.data_map.clone().unwrap();
        let function_text = self.function_text.clone();
        let mut formula = formula.replace("FORMAT(\"", "").replace("\")", "");
        for capture in RE_FORMAT_COLUMNS.captures_iter(&function_text) {
            let mut attribute = capture.get(0).unwrap().as_str().to_string();
            attribute = attribute.replace("{", "").replace("}", "");
            let item_value = data_map.get(&attribute);
            if item_value.is_some() {
                let item_value = item_value.unwrap();
                let replace_value = format!("{}{}{}", 
                    String::from("{"),
                    &attribute,
                    String::from("}"),
                );
                formula = formula.replace(&replace_value, &item_value);
            }
        }
        self.format = Some(formula.clone());
        formula = format!("\"{}\"", formula);
        return formula
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JoinListFunction {
    pub function_text: String,
    pub array: String,
    pub array_items: Option<Vec<String>>,
    pub sep: String,
    pub data_map: Option<HashMap<String, String>>,
}
impl JoinListFunction{
    pub fn defaults(function_text: &String, data_map: Option<HashMap<String, String>>) -> JoinListFunction {
        // Excel array: {column_a, column_b; column_2_a, column_2_b} (like a table: columns and rows)
        // One dimension, simply separate by commas
        // A database column might have a list: Set, Links, etc...
        // JOINLIST({My Column}, ",")
        // JOINLIST({1, 2, 3}, ",") => "1,2,3"
        let attr_map = RE_JOINLIST_ATTRS.captures(function_text).unwrap();
        // array of what? Deal with them the same way?
        // {1, 2, 3} => array: "{1, 2, 3}" array_items: ["1", "2", "3"]
        // {"a", "b", "c"} => array: "{\"a\", \"b\", \"c\"}" array_items: ["a", "b", "c"]
        // {Column A} => array "{Column A}"" array_items: []
        let array = attr_map.name("array").unwrap().as_str();
        let sep = attr_map.name("sep").unwrap().as_str().replace("\"", "");
        let mut array_items_wrap: Option<Vec<String>> = None;
        if array.find(",").is_some() {
            // {1,2,3} or {"a","b","c"}
            let array = array.replace("{", "").replace("}", "");
            let array_items = array.split(",");
            let array_items: Vec<&str> = array_items.collect();
            let array_items: Vec<String> = array_items.iter().map(|s| s.trim().to_string()).collect();
            array_items_wrap = Some(array_items);
        }        
        let obj = Self{
            function_text: function_text.clone(),
            array: array.to_string(),
            array_items: array_items_wrap,
            sep: sep.to_string(),
            data_map: data_map,
        };
        return obj
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let concat_obj = JoinListFunction::defaults(
            &function_text, None
        );
        let (number_fails, mut failed_functions) = validate_tuple;
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_JOINLIST));
        }
        return (number_fails, failed_functions);
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = JoinListFunction::defaults(
            &function_text,Some(data_map)
        );
        formula = concat_obj.replace(formula);
        return formula
    }
}
impl Function for JoinListFunction {
    fn validate(&self) -> bool {
        let expr = RE_JOINLIST_ATTRS.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    fn replace(&mut self, formula: String) -> String {
        let data_map = self.data_map.clone().unwrap();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();
        let array_items = self.array_items.clone();
        let sep = self.sep.clone();
        let mut replacement_string = String::from("");
        if array_items.is_some() {
            // {1,2,3} or {"a","b","c"}
            let array_items = array_items.unwrap();
            replacement_string = array_items.join(&sep);
        } else {
            // Column A
            // I need to get list of items from data_map
            let column = self.array.replace("{", "").replace("}", "");
            let array_items = data_map.get(&column);
            if array_items.is_some() {
                let array_items = array_items.unwrap();
                let items = array_items.split(",");
                let items: Vec<&str> = items.collect();
                replacement_string = items.join(&sep);
            }
        }
        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        formula = format!("\"{}\"", formula);
        return formula;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LengthFunction {
    pub function_text: String,
    pub attribute: String,
    pub data_map: Option<HashMap<String, String>>,
}
impl LengthFunction {
    pub fn defaults(function_text: &String, data_map: Option<HashMap<String, String>>) -> LengthFunction {
        // LEN("my string") => 8
        // LEN({My Column}) => 23

        let mut obj = Self{
            function_text: function_text.clone(),
            attribute: String::from(""),
            data_map: data_map,
        };
        for capture in RE_LEN_ATTR.captures_iter(function_text) {
            let attribute = capture.get(0).unwrap().as_str().to_string();
            obj.attribute = attribute;
        }

        return obj
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = LengthFunction::defaults(
            &function_text, None
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_LENGTH));
        }
        return (number_fails, failed_functions);
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = LengthFunction::defaults(
            &function_text,Some(data_map)
        );
        formula = concat_obj.replace(formula);
        return formula
    }
}
impl Function for LengthFunction {
    fn validate(&self) -> bool {
        let expr = RE_LEN_ATTR.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    fn replace(&mut self, formula: String) -> String {
        let data_map = self.data_map.clone().unwrap();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();

        let mut replacement_string: String = String::from("");
        for capture in RE_LEN_ATTR.captures_iter(&function_text) {
            let attribute = capture.get(0).unwrap().as_str().to_string();
            let function_attr = FunctionAttribute::defaults(&attribute, Some(true));
            replacement_string = function_attr.replace(data_map.clone()).item_processed.unwrap();
        };
        // let length = replacement_string.len();
        replacement_string = replacement_string.len().to_string();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        return formula;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LowerFunction {
    pub function_text: String,
    pub attribute: String,
    pub data_map: Option<HashMap<String, String>>,
}
impl LowerFunction {
    pub fn defaults(function_text: &String, data_map: Option<HashMap<String, String>>) -> LowerFunction {
        // LOWER("my string")
        // LOWER({My Column})

        let mut obj = Self{
            function_text: function_text.clone(),
            attribute: String::from(""),
            data_map: data_map,
        };
        for capture in RE_SINGLE_ATTR.captures_iter(function_text) {
            let attribute = capture.get(0).unwrap().as_str().to_string();
            obj.attribute = attribute;
        }

        return obj
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = LowerFunction::defaults(
            &function_text, None
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_LOWER));
        }
        return (number_fails, failed_functions);
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = LowerFunction::defaults(
            &function_text,Some(data_map)
        );
        formula = concat_obj.replace(formula);
        return formula
    }
}
impl Function for LowerFunction {
    fn validate(&self) -> bool {
        let expr = RE_SINGLE_ATTR.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    fn replace(&mut self, formula: String) -> String {
        let data_map = self.data_map.clone().unwrap();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();

        let mut replacement_string: String = String::from("");
        for capture in RE_SINGLE_ATTR.captures_iter(&function_text) {
            let attribute = capture.get(0).unwrap().as_str().to_string();
            let function_attr = FunctionAttribute::defaults(&attribute, Some(true));
            replacement_string = function_attr.replace(data_map.clone()).item_processed.unwrap();
        };
        replacement_string = replacement_string.to_lowercase();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        formula = format!("\"{}\"", formula);
        return formula;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpperFunction {
    pub function_text: String,
    pub attribute: String,
    pub data_map: Option<HashMap<String, String>>,
}
impl UpperFunction {
    pub fn defaults(function_text: &String, data_map: Option<HashMap<String, String>>) -> UpperFunction {
        // UPPER("my string")
        // UPPER({My Column})

        let mut obj = Self{
            function_text: function_text.clone(),
            attribute: String::from(""),
            data_map: data_map,
        };
        for capture in RE_SINGLE_ATTR.captures_iter(function_text) {
            let attribute = capture.get(0).unwrap().as_str().to_string();
            obj.attribute = attribute;
        }

        return obj
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = UpperFunction::defaults(
            &function_text, None
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_UPPER));
        }
        return (number_fails, failed_functions);
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = UpperFunction::defaults(
            &function_text,Some(data_map)
        );
        formula = concat_obj.replace(formula);
        return formula
    }
}

impl Function for UpperFunction {
    fn validate(&self) -> bool {
        let expr = RE_SINGLE_ATTR.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    fn replace(&mut self, formula: String) -> String {
        let data_map = self.data_map.clone().unwrap();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();

        let mut replacement_string: String = String::from("");
        for capture in RE_SINGLE_ATTR.captures_iter(&function_text) {
            let attribute = capture.get(0).unwrap().as_str().to_string();
            let function_attr = FunctionAttribute::defaults(&attribute, Some(true));
            replacement_string = function_attr.replace(data_map.clone()).item_processed.unwrap();
        };
        replacement_string = replacement_string.to_uppercase();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        formula = format!("\"{}\"", formula);
        return formula;
    }
}

// REPLACE(old_text, start_num, num_chars, new_text)
// old_text     Required. Text in which you want to replace some characters.
// start_num    Required. The position of the character in old_text that you want to replace with new_text.
// num_chars    Required. The number of characters in old_text that you want REPLACE to replace with new_text.
// new_text     Required. The text that will replace characters in old_text.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReplaceFunction {
    pub function_text: String,
    pub attributes: Vec<String>,
    pub attributes_value_map: Option<HashMap<String, String>>,
    pub old_text: String,
    pub start_num: u32,
    pub num_chars: u32,
    pub new_text: String,
    pub data_map: Option<HashMap<String, String>>,
}
impl ReplaceFunction {
    pub fn defaults(function_text: &String, data_map: Option<HashMap<String, String>>) -> ReplaceFunction {
        // REPLACE(old_text, start_num, num_chars, new_text)
        // REPLACE({My Column}, start_num, num_chars, new_text)

        // RE matches: "old_text", "start_num", "num_chars", "new_text"
        let matches = RE_REPLACE.captures(function_text).unwrap();
        let old_text = matches.name("old_text").unwrap().as_str().to_string();
        let start_num= matches.name("start_num").unwrap().as_str().to_string();
        let num_chars = matches.name("num_chars").unwrap().as_str().to_string();
        let new_text = matches.name("new_text").unwrap().as_str().to_string();
        let mut attributes: Vec<String> = Vec::new();
        attributes.push(old_text.clone());
        attributes.push(start_num.clone());
        attributes.push(num_chars.clone());
        attributes.push(new_text.clone());
        let start_num: u32 = FromStr::from_str(start_num.as_str()).unwrap();
        let num_chars: u32 = FromStr::from_str(num_chars.as_str()).unwrap();

        let obj = Self{
            function_text: function_text.clone(),
            attributes: attributes,
            attributes_value_map: None,
            old_text: old_text,
            new_text: new_text,
            start_num: start_num,
            num_chars: num_chars,
            data_map: data_map,
        };
        
        return obj
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = ReplaceFunction::defaults(
            &function_text, None
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_REPLACE));
        }
        return (number_fails, failed_functions);
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = ReplaceFunction::defaults(
            &function_text,Some(data_map)
        );
        formula = concat_obj.replace(formula);
        return formula
    }

}
impl Function for ReplaceFunction {
    fn validate(&self) -> bool {
        let expr = RE_REPLACE.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    fn replace(&mut self, formula: String) -> String {
        let data_map = self.data_map.clone().unwrap();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();
        let old_text = self.old_text.clone();
        let start_num = self.start_num.clone();
        let number_chars = self.num_chars.clone();
        let new_text = self.new_text.clone();
        let new_text = new_text.as_str();

        let replacement_string: String;

        // attribute processed only in case of old_text, since rest are strings, and ints
        let function_attr = FunctionAttribute::defaults(
            &old_text, 
            Some(true)
        );
        let old_text_processed = function_attr.replace(data_map.clone()).item_processed.unwrap();
        let mut piece: String = String::from("");
        for (i, item) in old_text_processed.chars().enumerate() {
            let i = i as u32;
            let length = *&piece.len();
            let length = length as u32;
            if &i >= &start_num && length < number_chars {
                piece.push(item);
            }
        }
        replacement_string = old_text.replace(&piece, new_text);
        // TODO: attributes_value_map

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        formula = format!("\"{}\"", formula);
        return formula;
    }
}

// MID(text, start_num, num_chars)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MidFunction {
    pub function_text: String,
    pub text: Option<String>,
    pub text_ref: Option<String>,
    pub start_num: Option<u32>,
    pub num_chars: Option<u32>,
    pub data_map: Option<HashMap<String, String>>,
}
impl MidFunction {
    pub fn defaults(function_text: &String, data_map: Option<HashMap<String, String>>) -> MidFunction {
        // MID(text, start_num, num_chars)

        let matches = RE_MID.captures(function_text).unwrap();
        let attr_text = matches.name("text");
        let attr_text_ref = matches.name("text_ref");
        let attr_start_num = matches.name("start_num");
        let attr_num_chars = matches.name("num_chars");

        let mut text_wrap: Option<String> = None;
        let mut text_ref_wrap: Option<String> = None;
        let mut start_num_wrap: Option<u32> = None;
        let mut num_chars_wrap: Option<u32> = None;

        if attr_text.is_some() && attr_start_num.is_some() && attr_num_chars.is_some() {
            text_wrap = Some(attr_text.unwrap().as_str().to_string());
            let start_num: u32 = FromStr::from_str(attr_start_num.unwrap().as_str()).unwrap();
            start_num_wrap = Some(start_num);
            let num_chars: u32 = FromStr::from_str(attr_num_chars.unwrap().as_str()).unwrap();
            num_chars_wrap = Some(num_chars);
        } else if attr_text_ref.is_some() && attr_start_num.is_some() && attr_num_chars.is_some() {
            text_ref_wrap = Some(attr_text_ref.unwrap().as_str().to_string());
            let start_num: u32 = FromStr::from_str(attr_start_num.unwrap().as_str()).unwrap();
            start_num_wrap = Some(start_num);
            let num_chars: u32 = FromStr::from_str(attr_num_chars.unwrap().as_str()).unwrap();
            num_chars_wrap = Some(num_chars);
        }

        let obj = Self{
            function_text: function_text.clone(),
            text: text_wrap,
            text_ref: text_ref_wrap,
            start_num: start_num_wrap,
            num_chars: num_chars_wrap,
            data_map: data_map,
        };
        
        return obj
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = MidFunction::defaults(
            &function_text, None
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_MID));
        }
        return (number_fails, failed_functions);
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = MidFunction::defaults(
            &function_text,Some(data_map)
        );
        formula = concat_obj.replace(formula);
        return formula
    }
}
impl Function for MidFunction {
    fn validate(&self) -> bool {
        let expr = RE_MID.clone();
        let function_text = self.function_text.clone();
        let mut check = expr.is_match(&function_text);
        let num_chars = self.num_chars.clone();
        let start_num = self.start_num.clone();
        if check == false {
            return check
        }
        if num_chars.is_none() || start_num.is_none() {
            check = false;
        }
        return check
    }
    fn replace(&mut self, formula: String) -> String {
        let data_map = self.data_map.clone().unwrap();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();
        let replacement_string: String;

        let text_wrap = self.text.clone();
        let text_ref_wrap = self.text_ref.clone();
        let start_num: u32 = self.start_num.unwrap();
        let start_num = start_num as usize;
        let num_chars: u32 = self.num_chars.unwrap();
        let num_chars = num_chars as usize;
        let mut text: String;
        if text_wrap.is_some() {
            text = text_wrap.unwrap();
            text = text.replace("\"", "");
        } else {
            let text_ref = text_ref_wrap.unwrap();
            let function_attr = FunctionAttribute::defaults(
                &text_ref, 
                Some(true)
            );
            text = function_attr.replace(data_map.clone()).item_processed.unwrap();
            text = text.replace("\"", "");
        }
        let mut text_new = String::from("");
        for (i, char) in text.chars().enumerate() {
            let count = i+1;
            if count >= start_num && count <= num_chars {
                let char_ = char.to_string();
                text_new.push_str(char_.as_str());
            }
        }
        replacement_string = text_new;

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        formula = format!("\"{}\"", formula);
        return formula;
    }
}

// REPT(text, number_times)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReptFunction {
    pub function_text: String,
    pub text: Option<String>,
    pub text_ref: Option<String>,
    pub number_times: Option<u32>,
    pub data_map: Option<HashMap<String, String>>,
}
impl ReptFunction {
    pub fn defaults(function_text: &String, data_map: Option<HashMap<String, String>>) -> ReptFunction {
        // REPT(text, number_times)

        let matches = RE_REPT.captures(function_text).unwrap();
        let attr_text = matches.name("text");
        let attr_text_ref = matches.name("text_ref");
        let attr_number_times = matches.name("number_times");

        let mut text_wrap: Option<String> = None;
        let mut text_ref_wrap: Option<String> = None;
        let mut number_times_wrap: Option<u32> = None;

        if attr_text.is_some() && attr_number_times.is_some() {
            text_wrap = Some(attr_text.unwrap().as_str().to_string());
            let number_times: u32 = FromStr::from_str(attr_number_times.unwrap().as_str()).unwrap();
            number_times_wrap = Some(number_times);
        } else if attr_text_ref.is_some() && attr_number_times.is_some() {
            text_ref_wrap = Some(attr_text_ref.unwrap().as_str().to_string());
            let number_times: u32 = FromStr::from_str(attr_number_times.unwrap().as_str()).unwrap();
            number_times_wrap = Some(number_times);
        }

        let obj = Self{
            function_text: function_text.clone(),
            text: text_wrap,
            text_ref: text_ref_wrap,
            number_times: number_times_wrap,
            data_map: data_map,
        };
        return obj
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = ReptFunction::defaults(
            &function_text, None
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_REPT));
        }
        return (number_fails, failed_functions);
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = ReptFunction::defaults(
            &function_text,Some(data_map)
        );
        formula = concat_obj.replace(formula);
        return formula
    }
}
impl Function for ReptFunction {
    fn validate(&self) -> bool {
        let expr = RE_REPT.clone();
        let function_text = self.function_text.clone();
        let mut check = expr.is_match(&function_text);
        let number_times = self.number_times.clone();
        if check == false {
            return check
        }
        if number_times.is_none() {
            check = false;
        }
        return check
    }
    fn replace(&mut self, formula: String) -> String {
        let data_map = self.data_map.clone().unwrap();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();
        let replacement_string: String;

        let text_wrap = self.text.clone();
        let text_ref_wrap = self.text_ref.clone();
        let number_times: u32 = self.number_times.unwrap();
        let number_times = number_times as usize;
        let mut text: String;
        if text_wrap.is_some() {
            text = text_wrap.unwrap();
            text = text.replace("\"", "");
        } else {
            let text_ref = text_ref_wrap.unwrap();
            let function_attr = FunctionAttribute::defaults(
                &text_ref, 
                Some(true)
            );
            text = function_attr.replace(data_map.clone()).item_processed.unwrap();
            text = text.replace("\"", "");
        }
        let text_str = text.as_str();
        let text_str = text_str.repeat(number_times);
        replacement_string = text_str;

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        formula = format!("\"{}\"", formula);
        return formula;
    }

}

// SUBSTITUTE(text, old_text, new_text)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubstituteFunction {
    pub function_text: String,
    pub text: Option<String>,
    pub text_ref: Option<String>,
    pub old_text: Option<String>,
    pub new_text: Option<String>,
    pub data_map: Option<HashMap<String, String>>,
}
impl SubstituteFunction {
    pub fn defaults(function_text: &String, data_map: Option<HashMap<String, String>>) -> SubstituteFunction {
        // SUBSTITUTE(text, old_text, new_text)

        let matches = RE_SUBSTITUTE.captures(function_text).unwrap();
        let attr_text = matches.name("text");
        let attr_text_ref = matches.name("text_ref");
        let attr_old_text = matches.name("old_text");
        let attr_new_text = matches.name("new_text");

        let mut text_wrap: Option<String> = None;
        let mut text_ref_wrap: Option<String> = None;
        let mut old_text_wrap: Option<String> = None;
        let mut new_text_wrap: Option<String> = None;

        if attr_text.is_some() && attr_old_text.is_some() && attr_new_text.is_some() {
            text_wrap = Some(attr_text.unwrap().as_str().to_string());
            old_text_wrap = Some(attr_old_text.unwrap().as_str().to_string());
            new_text_wrap = Some(attr_new_text.unwrap().as_str().to_string());
        } else if attr_text_ref.is_some() && attr_old_text.is_some() && attr_new_text.is_some() {
            text_ref_wrap = Some(attr_text_ref.unwrap().as_str().to_string());
            old_text_wrap = Some(attr_old_text.unwrap().as_str().to_string());
            new_text_wrap = Some(attr_new_text.unwrap().as_str().to_string());
        }

        let obj = Self{
            function_text: function_text.clone(),
            text: text_wrap,
            text_ref: text_ref_wrap,
            old_text: old_text_wrap,
            new_text: new_text_wrap,
            data_map: data_map,
        };

        return obj
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = SubstituteFunction::defaults(
            &function_text, None
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_SUBSTITUTE));
        }
        return (number_fails, failed_functions);
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = SubstituteFunction::defaults(
            &function_text,Some(data_map)
        );
        formula = concat_obj.replace(formula);
        return formula
    }
}
impl Function for SubstituteFunction {
    fn validate(&self) -> bool {
        let expr = RE_SUBSTITUTE.clone();
        let function_text = self.function_text.clone();
        let mut check = expr.is_match(&function_text);
        let text = self.text.clone();
        let text_ref = self.text_ref.clone();
        let old_text = self.old_text.clone();
        let new_text = self.new_text.clone();
        if check == false {
            return check
        }
        if new_text.is_none() || old_text.is_none() {
            check = false;
        }
        if text.is_none() && text_ref.is_none() {
            check = false;
        }
        return check
    }
    fn replace(&mut self, formula: String) -> String {
        let data_map = self.data_map.clone().unwrap();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();
        let replacement_string: String;

        let text_wrap = self.text.clone();
        let text_ref_wrap = self.text_ref.clone();
        let old_text_wrap = self.old_text.clone();
        let new_text_wrap = self.new_text.clone();
        let mut text: String;
        let old_text = old_text_wrap.unwrap().replace("\"", "");
        let old_text = old_text.as_str();
        let new_text = new_text_wrap.unwrap().replace("\"", "");
        let new_text = new_text.as_str();

        if text_wrap.is_some() {
            text = text_wrap.unwrap();
            text = text.replace("\"", "");
        } else {
            let text_ref = text_ref_wrap.unwrap();
            let function_attr = FunctionAttribute::defaults(
                &text_ref, 
                Some(true)
            );
            text = function_attr.replace(data_map.clone()).item_processed.unwrap();
            text = text.replace("\"", "");
        }
        text = text.replace(old_text, new_text);

        replacement_string = text;

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        formula = format!("\"{}\"", formula);
        return formula;
    }
}

// TRIM(text)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrimFunction {
    pub function_text: String,
    pub text: Option<String>,
    pub text_ref: Option<String>,
    pub data_map: Option<HashMap<String, String>>,
}
impl TrimFunction {
    pub fn defaults(function_text: &String, data_map: Option<HashMap<String, String>>) -> TrimFunction {
        // TRIM(text)

        let matches = RE_TRIM.captures(function_text).unwrap();
        let attr_text = matches.name("text");
        let attr_text_ref = matches.name("text_ref");

        let mut text_wrap: Option<String> = None;
        let mut text_ref_wrap: Option<String> = None;

        if attr_text.is_some() {
            text_wrap = Some(attr_text.unwrap().as_str().to_string());
        } else if attr_text_ref.is_some() {
            text_ref_wrap = Some(attr_text_ref.unwrap().as_str().to_string());
        }

        let obj = Self{
            function_text: function_text.clone(),
            text: text_wrap,
            text_ref: text_ref_wrap,
            data_map: data_map,
        };

        return obj
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = TrimFunction::defaults(
            &function_text, None
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_TRIM));
        }
        return (number_fails, failed_functions);
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = TrimFunction::defaults(
            &function_text,Some(data_map)
        );
        formula = concat_obj.replace(formula);
        return formula
    }
}

impl Function for TrimFunction {
    fn validate(&self) -> bool {
        let expr = RE_TRIM.clone();
        let function_text = self.function_text.clone();
        let mut check = expr.is_match(&function_text);
        let text = self.text.clone();
        let text_ref = self.text_ref.clone();
        if check == false {
            return check
        }
        if text.is_none() && text_ref.is_none() {
            check = false;
        }
        return check
    }
    fn replace(&mut self, formula: String) -> String {
        let data_map = self.data_map.clone().unwrap();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();
        let replacement_string: String;

        let text_wrap = self.text.clone();
        let text_ref_wrap = self.text_ref.clone();
        let mut text: String;

        if text_wrap.is_some() {
            text = text_wrap.unwrap();
            text = text.replace("\"", "");
        } else {
            let text_ref = text_ref_wrap.unwrap();
            let function_attr = FunctionAttribute::defaults(
                &text_ref, 
                Some(true)
            );
            text = function_attr.replace(data_map.clone()).item_processed.unwrap();
            text = text.replace("\"", "");
        }
        text = text.trim().to_string();

        replacement_string = text;

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        formula = format!("\"{}\"", formula);
        return formula;
    }
}

pub fn check_string_equal(name: &String, value: &String) -> Result<bool, PlanetError> {
    let check: bool;
    if name.to_lowercase() == value.to_lowercase() {
        check = true
    } else {
        check = false
    }
    return Ok(check)
}

pub fn check_str_equal(name: &str, value: &str) -> Result<bool, PlanetError> {
    let check: bool;
    if name.to_lowercase() == value.to_lowercase() {
        check = true
    } else {
        check = false
    }
    return Ok(check)
}
