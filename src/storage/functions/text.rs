use regex::Regex;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// The mapping CONCAT to struct Concatenate would need to be done in a match
// In future we make functions modular, so far is a POC way of doing. I will divide functions by 
// string functions, date functions, etc...

// CONCAT({Column 1}, "-", {Column 2}) => "{Column 1}-{Column 2}" as string replaced in formula
// Name & " - " & Age
// {Column 1} & "-" & {Column 2} : This version only needs reference substitution and not the Concatenate struct
// handling

// 1. I get all functions that meet [A-Z][0-9](), that is, need to be upper.
// 2. I check in a match the case. Default is process default ones: AND, OR, etc...
// 3. For each occurance of function, I instantiate the function


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConcatenateFunction {
    pub items: Vec<String>,
    pub function_text: String,
}
impl ConcatenateFunction{
    // function_text would be CONCAT(arg1, arg2, ...)
    pub fn defaults(function_text: &String, data_map: HashMap<String, String>) -> ConcatenateFunction {
        let data_map = data_map.clone();
        // I use regex in order to come up with vector of items
        // I parse the items and return object
        let items: Vec<String> = Vec::new();
        // I can have as items {Field A}, "A string", or a number like 43, etc...
        // References are obtained from data_map and rest converted to string
        let expr = Regex::new(r#"("[\w\s]+")|(\d+)|(\{[\w\s]+\})"#).unwrap();
        for capture in expr.captures_iter(function_text) {
            let item = capture.get(0).unwrap().as_str();
            eprintln!("ConcatenateFunction.defaults :: item: {}", item);
            let item_string: String;
            if item.find("{") == 0 {
                // I have a column, need to get data from data_map
                item = item.replace("{", "").replace("}", "");
                let item_value = data_map.get(&item);
                if item_value.is_some() {
                    let item_value = item_value.unwrap().clone();
                    item_string = item_value;
                } else {
                    continue;
                }
            } else {
                item_string = item.to_string();
            }
            items.push(item_string);
        }
        let obj = Self{
            items: items,
            function_text: *function_text,
        };
        eprintln!("ConcatenateFunction.defaults :: obj: {:#?}", &obj);
        return obj
    }
    pub fn replace(&self, formula: &String) -> String {
        let formula = formula.clone();
        let value = self.items.join("");
        eprintln!("ConcatenateFunction.replace :: value: {}", &value);
        formula = formula.replace(self.function_text, value);
        eprintln!("ConcatenateFunction.replace :: formula: {}", &formula);
        return formula
    }
}
