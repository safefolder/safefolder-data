use regex::Regex;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConcatenateFunction {
    pub items: Vec<String>,
    pub function_text: String,
}
impl ConcatenateFunction{
    // function_text would be CONCAT(arg1, arg2, ...)
    pub fn defaults(function_text: &String, data_map: HashMap<String, String>) -> ConcatenateFunction {
        let data_map = data_map.clone();
        let mut items: Vec<String> = Vec::new();
        // References are obtained from data_map and rest converted to string
        let expr = Regex::new(r#"("[\w\s-]+")|(\d+)|(\{[\w\s]+\})"#).unwrap();
        for capture in expr.captures_iter(function_text) {
            let mut item = capture.get(0).unwrap().as_str().to_string();
            item = item.replace("\"", "");
            let item_string: String;
            let item_find = item.find("{");
            if item_find.is_some() && item_find.unwrap() == 0 {
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
            function_text: function_text.clone(),
        };
        return obj
    }
    pub fn replace(&self, formula: String) -> String {
        let items = self.items.clone();
        let mut formula = formula.clone();
        let value = items.join("");
        let value = value.as_str();
        formula = formula.replace(self.function_text.as_str(), value);
        return formula
    }
    pub fn init_do(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let concat_obj = ConcatenateFunction::defaults(
            &function_text, 
            data_map
        );
        formula = concat_obj.replace(formula);
        return formula
    }
}
