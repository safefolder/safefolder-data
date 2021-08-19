use regex::Regex;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;

use crate::functions::FunctionAttribute;

// 1. defaults receives the function test, that is FUNC_NAME(attr1, attr2, ...) and returns function instance
//      attributes embeeded into the object attributes. It does not prepare for replacement or anythin.
// 2. replace is the one that received the data map and gets attributes from function and does replacement.
// 3. do_replace: Creates the object and does replacement.
// 4. validate: Validates the function text with respect to the regex for the function, checks function text is
//       fine.

// I need to refactor and extract common operations that can be used in other functions as well and place into
//     the mod inside functions.

lazy_static! {
    // CONCAT("mine", "-", {My Column}, 45) :: Regex to catch the function attributes in an array
    static ref RE_CONCAT_ATTRS: Regex = Regex::new(r#"("[\w\s-]+")|(\d+)|(\{[\w\s]+\})"#).unwrap();
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConcatenateFunction {
    pub function_text: String,
    pub attributes: Vec<String>,
    pub attributes_value_map: Option<HashMap<String, String>>,
}

impl ConcatenateFunction{
    // CONCAT(arg1, arg2, ...)
    pub fn defaults(function_text: &String) -> ConcatenateFunction {
        let mut attributes: Vec<String> = Vec::new();
        for capture in RE_CONCAT_ATTRS.captures_iter(function_text) {
            let attribute = capture.get(0).unwrap().as_str().to_string();
            attributes.push(attribute);
        }
        let obj = Self{
            function_text: function_text.clone(),
            attributes: attributes,
            attributes_value_map: None,
        };
        return obj
    }
    pub fn validate(&self) -> bool {
        // We validate the function text respect to the regular expression, is match?
        let expr = RE_CONCAT_ATTRS.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        eprintln!("ConcatenateFunction.validate :: validate: {}", &check);
        return check
    }
    pub fn do_validate(function_text: &String, number_fails: &i32) -> i32 {
        let concat_obj = ConcatenateFunction::defaults(
            &function_text, 
        );
        let check = concat_obj.validate();
        eprintln!("ConcatenateFunction.do_validate :: check: {}", &check);
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
        }
        return number_fails;
    }
    pub fn replace(&mut self, formula: String, data_map: HashMap<String, String>) -> String {
        let data_map = data_map.clone();
        let mut attributes_processed: Vec<String> = Vec::new();
        let function_text = self.function_text.clone();
        for capture in RE_CONCAT_ATTRS.captures_iter(&function_text) {
            let attribute = capture.get(0).unwrap().as_str().to_string();
            eprintln!("ConcatenateFunction.replace :: attribute: {}", &attribute);
            let function_attr = FunctionAttribute::defaults(&attribute, Some(true));
            let attribute_processed = function_attr.replace(data_map.clone()).item_processed.unwrap();
            eprintln!("ConcatenateFunction.replace :: attribute_processed: {}", &attribute_processed);
            attributes_processed.push(attribute_processed);
        };
        let attributes_process_ = attributes_processed.clone();
        eprintln!("ConcatenateFunction.replace :: attributes_processed: {:?}", &attributes_processed);
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
        eprintln!("ConcatenateFunction.replace :: obj: {:#?}", self.clone());
        // formula LIB needs to parse and process this, so we need quotes, so handles like a string
        formula = format!("{}{}{}", 
            String::from("\""),
            formula,
            String::from("\""),
        );
        return formula
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = ConcatenateFunction::defaults(
            &function_text, 
        );
        formula = concat_obj.replace(formula, data_map.clone());
        return formula
    }
}


// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct FormatFunction {
//     pub format: String,
//     pub function_text: String,
// }
// impl FormatFunction{
//     pub fn defaults(function_text: &String, data_map: HashMap<String, String>) -> FormatFunction {
//         let data_map = data_map.clone();
//         let mut formula = function_text.replace("FORMAT(\"", "").replace("\")", "");
//         let expr = Regex::new(r"(\{[\w\s-]+\})").unwrap();
//         for capture in expr.captures_iter(function_text) {
//             let mut item = capture.get(0).unwrap().as_str().to_string();
//             item = item.replace("{", "").replace("}", "");
//             let item_value = data_map.get(&item);
//             if item_value.is_some() {
//                 let item_value = item_value.unwrap();
//                 let replace_value = format!("{}{}{}", 
//                     String::from("{"),
//                     &item,
//                     String::from("}"),
//                 );
//                 formula = formula.replace(&replace_value, &item_value);
//             }
//         }
//         let obj = Self{
//             format: formula.clone(),
//             function_text: function_text.clone(),
//         };
//         return obj
//     }
//     pub fn replace(&self, formula: String) -> String {
//         let mut formula = formula.clone();
//         formula = formula.replace(self.function_text.as_str(), self.format.as_str());
//         return formula
//     }
//     pub fn init_do(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
//         let data_map = data_map.clone();
//         let concat_obj = FormatFunction::defaults(
//             &function_text, 
//             data_map
//         );
//         formula = concat_obj.replace(formula);
//         return formula
//     }
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct JoinListFunction {
//     pub list_item: String,
//     pub sep: String,
//     pub function_text: String,
// }
// impl JoinListFunction{
//     pub fn defaults(function_text: &String, data_map: HashMap<String, String>) -> JoinListFunction {
//         // This receives a list of items and creates a string with a join sep, like "1,2,3,4"
//         // List could be a column but also an array like {1,2,3}
//         // JOINLIST({My Column}, ",")
//         // JOINLIST({1, 2, 3}, ",")
//         // Regex to get {Column}, "{sep}"
//         let expr = Regex::new(r#"(\{[\w\s,-]+\})|("[\W]")"#).unwrap();
//         let sep: String;
//         for capture in expr.captures_iter(function_text) {
//             let mut item = capture.get(0).unwrap().as_str().to_string();
//             if item.find("\"").is_some() {
//                 sep = item.clone();
//             }
//         }
//         let obj = Self{
//             list_column: String::from(""),
//             sep: sep,
//             function_text: function_text.clone(),
//         };
//         eprintln!("JoinListFunction.defaults :: obj: {:#?}", &obj);
//         return obj
//     }
//     pub fn replace(&self, formula: String) -> String {
//     }
//     pub fn init_do(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
//     }
// }