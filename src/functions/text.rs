use std::str::FromStr;
use regex::Regex;
use std::{collections::HashMap};
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
    static ref RE_FORMAT_COLUMNS: Regex = Regex::new(r"(\{[\w\s-]+\})").unwrap();
    static ref RE_JOINLIST_ATTRS: Regex = Regex::new(r#"(?P<array>\{[\w\s\d,"-]+\}),[\s+]{0,}(?P<sep>\\{0,1}"[\W]\\{0,1}")"#).unwrap();
    static ref RE_LEN_ATTR: Regex = Regex::new(r#"("[\w\s-]+")|(\{[\w\s]+\})"#).unwrap();
    static ref RE_SINGLE_ATTR: Regex = Regex::new(r#"("[\w\s-]+")|(\{[\w\s]+\})"#).unwrap();
    static ref RE_REPLACE: Regex = Regex::new(r#"REPLACE\("(?P<old_text>[\w\s]+)",[\s]+(?P<start_num>\d),[\s]+(?P<num_chars>\d),[\s]+"(?P<new_text>[\w\s]+)"\)"#).unwrap();
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
        let expr = RE_CONCAT_ATTRS.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    pub fn do_validate(function_text: &String, number_fails: &i32) -> i32 {
        let concat_obj = ConcatenateFunction::defaults(
            &function_text, 
        );
        let check = concat_obj.validate();
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
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = ConcatenateFunction::defaults(
            &function_text, 
        );
        formula = concat_obj.replace(formula, data_map.clone());
        return formula
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FormatFunction {
    pub function_text: String,
    pub format: Option<String>,
}
impl FormatFunction{
    pub fn defaults(function_text: &String) -> FormatFunction {
        // FORMAT("{My Column}-45-pepito")
        let obj = Self{
            format: None,
            function_text: function_text.clone(),
        };
        return obj
    }
    pub fn validate(&self) -> bool {
        let expr = RE_FORMAT_COLUMNS.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    pub fn do_validate(function_text: &String, number_fails: &i32) -> i32 {
        let concat_obj = FormatFunction::defaults(
            &function_text, 
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
        }
        return number_fails;
    }
    pub fn replace(&mut self, formula: String, data_map: HashMap<String, String>) -> String {
        // FORMAT("Hello-{Column A}-45")
        let data_map = data_map.clone();
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
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = FormatFunction::defaults(
            &function_text, 
        );
        formula = concat_obj.replace(formula, data_map.clone());
        return formula
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JoinListFunction {
    pub function_text: String,
    pub array: String,
    pub array_items: Option<Vec<String>>,
    pub sep: String,
}
impl JoinListFunction{
    pub fn defaults(function_text: &String) -> JoinListFunction {
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
        };
        return obj
    }
    pub fn validate(&self) -> bool {
        let expr = RE_JOINLIST_ATTRS.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    pub fn do_validate(function_text: &String, number_fails: &i32) -> i32 {
        let concat_obj = JoinListFunction::defaults(
            &function_text, 
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
        }
        return number_fails;
    }
    pub fn replace(&mut self, formula: String, data_map: HashMap<String, String>) -> String {
        let data_map = data_map.clone();
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
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = JoinListFunction::defaults(
            &function_text, 
        );
        formula = concat_obj.replace(formula, data_map.clone());
        return formula
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LengthFunction {
    pub function_text: String,
    pub attribute: String,
}
impl LengthFunction {
    pub fn defaults(function_text: &String) -> LengthFunction {
        // LEN("my string") => 8
        // LEN({My Column}) => 23

        let mut obj = Self{
            function_text: function_text.clone(),
            attribute: String::from(""),
        };
        for capture in RE_LEN_ATTR.captures_iter(function_text) {
            let attribute = capture.get(0).unwrap().as_str().to_string();
            obj.attribute = attribute;
        }

        return obj
    }
    pub fn validate(&self) -> bool {
        let expr = RE_LEN_ATTR.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    pub fn do_validate(function_text: &String, number_fails: &i32) -> i32 {
        let concat_obj = LengthFunction::defaults(
            &function_text, 
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
        }
        return number_fails;
    }
    pub fn replace(&mut self, formula: String, data_map: HashMap<String, String>) -> String {
        let data_map = data_map.clone();
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
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = LengthFunction::defaults(
            &function_text, 
        );
        formula = concat_obj.replace(formula, data_map.clone());
        return formula
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LowerFunction {
    pub function_text: String,
    pub attribute: String,
}
impl LowerFunction {
    pub fn defaults(function_text: &String) -> LowerFunction {
        // LOWER("my string")
        // LOWER({My Column})

        let mut obj = Self{
            function_text: function_text.clone(),
            attribute: String::from(""),
        };
        for capture in RE_SINGLE_ATTR.captures_iter(function_text) {
            let attribute = capture.get(0).unwrap().as_str().to_string();
            obj.attribute = attribute;
        }

        return obj
    }
    pub fn validate(&self) -> bool {
        let expr = RE_SINGLE_ATTR.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    pub fn do_validate(function_text: &String, number_fails: &i32) -> i32 {
        let concat_obj = LowerFunction::defaults(
            &function_text, 
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
        }
        return number_fails;
    }
    pub fn replace(&mut self, formula: String, data_map: HashMap<String, String>) -> String {
        let data_map = data_map.clone();
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
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = LowerFunction::defaults(
            &function_text, 
        );
        formula = concat_obj.replace(formula, data_map.clone());
        return formula
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpperFunction {
    pub function_text: String,
    pub attribute: String,
}
impl UpperFunction {
    pub fn defaults(function_text: &String) -> UpperFunction {
        // UPPER("my string")
        // UPPER({My Column})

        let mut obj = Self{
            function_text: function_text.clone(),
            attribute: String::from(""),
        };
        for capture in RE_SINGLE_ATTR.captures_iter(function_text) {
            let attribute = capture.get(0).unwrap().as_str().to_string();
            obj.attribute = attribute;
        }

        return obj
    }
    pub fn validate(&self) -> bool {
        let expr = RE_SINGLE_ATTR.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    pub fn do_validate(function_text: &String, number_fails: &i32) -> i32 {
        let concat_obj = UpperFunction::defaults(
            &function_text, 
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
        }
        return number_fails;
    }
    pub fn replace(&mut self, formula: String, data_map: HashMap<String, String>) -> String {
        let data_map = data_map.clone();
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
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = UpperFunction::defaults(
            &function_text, 
        );
        formula = concat_obj.replace(formula, data_map.clone());
        return formula
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
    pub start_num: i32,
    pub num_chars: i32,
    pub new_text: String,

}
impl ReplaceFunction {
    pub fn defaults(function_text: &String) -> ReplaceFunction {
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
        let start_num: i32 = FromStr::from_str(start_num.as_str()).unwrap();
        let num_chars: i32 = FromStr::from_str(num_chars.as_str()).unwrap();

        let obj = Self{
            function_text: function_text.clone(),
            attributes: attributes,
            attributes_value_map: None,
            old_text: old_text,
            new_text: new_text,
            start_num: start_num,
            num_chars: num_chars,
        };
        
        return obj
    }
    pub fn validate(&self) -> bool {
        let expr = RE_REPLACE.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    pub fn do_validate(function_text: &String, number_fails: &i32) -> i32 {
        let concat_obj = ReplaceFunction::defaults(
            &function_text, 
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
        }
        return number_fails;
    }
    pub fn replace(&mut self, formula: String, data_map: HashMap<String, String>) -> String {
        let data_map = data_map.clone();
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
            let i = i as i32;
            let length = *&piece.len();
            let length = length as i32;
            if &i >= &start_num && length < number_chars {
                piece.push(item);
            }
        }
        replacement_string = old_text.replace(&piece, new_text);

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        formula = format!("\"{}\"", formula);
        return formula;
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = ReplaceFunction::defaults(
            &function_text, 
        );
        formula = concat_obj.replace(formula, data_map.clone());
        return formula
    }
}