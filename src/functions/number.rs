use std::str::FromStr;
use regex::Regex;
use std::{collections::HashMap};
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;
use math::round;

use crate::functions::FunctionAttribute;
use crate::functions::constants::*;


lazy_static! {
    static ref RE_CEILING: Regex = Regex::new(r#"CEILING\(((?P<number>[+-]?[0-9]+\.?[0-9]*|\.[0-9]+)|(?P<number_ref>\{[\w\s]+\}))[\s\n\t]{0,},[\s\n\t]{0,}(?P<significance>[+-]?[0-9]+\.?[0-9]*|\.[0-9]+)\)"#).unwrap();
    static ref RE_FLOOR: Regex = Regex::new(r#"FLOOR\(((?P<number>[+-]?[0-9]+\.?[0-9]*|\.[0-9]+)|(?P<number_ref>\{[\w\s]+\}))[\s\n\t]{0,},[\s\n\t]{0,}(?P<significance>[+-]?[0-9]+\.?[0-9]*|\.[0-9]+)\)"#).unwrap();
    static ref RE_COUNT: Regex = Regex::new(r#"COUNT\((?P<attrs>.+)\)"#).unwrap();
    static ref RE_COUNTA: Regex = Regex::new(r#"COUNTA\((?P<attrs>.+)\)"#).unwrap();
    static ref RE_COUNTALL: Regex = Regex::new(r#"COUNTALL\((?P<attrs>.+)\)"#).unwrap();
    static ref RE_EVEN: Regex = Regex::new(r#"EVEN\((?P<number>[+-]?[0-9]+\.?[0-9]*|\.[0-9]+)\)|EVEN\((?P<number_ref>\{[\w\s]+\})\)"#).unwrap();
}

// CEILING(number, significance)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CeilingFunction {
    pub function_text: String,
    pub number: Option<f64>,
    pub number_ref: Option<String>,
    pub significance: Option<i8>,
}
impl CeilingFunction {
    pub fn defaults(function_text: &String) -> CeilingFunction {
        // CEILING(number, significance)

        let matches = RE_CEILING.captures(function_text).unwrap();
        let attr_number = matches.name("number");
        let attr_number_ref = matches.name("number_ref");
        let attr_significance = matches.name("significance");

        let mut number_wrap: Option<f64> = None;
        let mut number_ref_wrap: Option<String> = None;
        let mut significance_wrap: Option<i8> = None;

        if attr_number.is_some() {
            let number = attr_number.unwrap();
            let number: f64 = FromStr::from_str(number.as_str()).unwrap();
            number_wrap = Some(number);
        } else if attr_number_ref.is_some() {
            number_ref_wrap = Some(attr_number_ref.unwrap().as_str().to_string());
        }
        if attr_significance.is_some() {
            let significance = attr_significance.unwrap();
            let significance: i8 = FromStr::from_str(significance.as_str()).unwrap();
            significance_wrap = Some(significance);
        }

        let obj = Self{
            function_text: function_text.clone(),
            number: number_wrap,
            number_ref: number_ref_wrap,
            significance: significance_wrap,
        };

        return obj
    }
    pub fn validate(&self) -> bool {
        let expr = RE_CEILING.clone();
        let function_text = self.function_text.clone();
        let mut check = expr.is_match(&function_text);
        let number = self.number.clone();
        let number_ref = self.number_ref.clone();
        let significance = self.significance.clone();
        if check == false {
            return check
        }
        if number.is_none() && number_ref.is_none() {
            check = false;
        }
        if significance.is_none() {
            check = false
        }
        return check
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = CeilingFunction::defaults(
            &function_text, 
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_CEILING));
        }
        return (number_fails, failed_functions);
    }
    pub fn replace(&mut self, formula: String, data_map: HashMap<String, String>) -> String {
        let data_map = data_map.clone();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();
        let replacement_string: String;

        let number_wrap = self.number.clone();
        let number_ref_wrap = self.number_ref.clone();
        let significance = self.significance.clone().unwrap() - 1;
        let number: f64;

        if number_wrap.is_some() {
            number = number_wrap.unwrap();
        } else {
            let number_ref = number_ref_wrap.unwrap();
            let function_attr = FunctionAttribute::defaults(
                &number_ref, 
                Some(true)
            );
            let number_str = function_attr.replace(data_map.clone()).item_processed.unwrap();
            number = FromStr::from_str(number_str.as_str()).unwrap();
        }
        let number = round::ceil(number, significance);

        replacement_string = number.to_string();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        formula = format!("\"{}\"", formula);
        return formula;
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = CeilingFunction::defaults(
            &function_text, 
        );
        formula = concat_obj.replace(formula, data_map.clone());
        return formula
    }
}

// FLOOR(number, significance)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FloorFunction {
    pub function_text: String,
    pub number: Option<f64>,
    pub number_ref: Option<String>,
    pub significance: Option<i8>,
}
impl FloorFunction {
    pub fn defaults(function_text: &String) -> FloorFunction {
        // FLOOR(number, significance)

        let matches = RE_FLOOR.captures(function_text).unwrap();
        let attr_number = matches.name("number");
        let attr_number_ref = matches.name("number_ref");
        let attr_significance = matches.name("significance");

        let mut number_wrap: Option<f64> = None;
        let mut number_ref_wrap: Option<String> = None;
        let mut significance_wrap: Option<i8> = None;

        if attr_number.is_some() {
            let number = attr_number.unwrap();
            let number: f64 = FromStr::from_str(number.as_str()).unwrap();
            number_wrap = Some(number);
        } else if attr_number_ref.is_some() {
            number_ref_wrap = Some(attr_number_ref.unwrap().as_str().to_string());
        }
        if attr_significance.is_some() {
            let significance = attr_significance.unwrap();
            let significance: i8 = FromStr::from_str(significance.as_str()).unwrap();
            significance_wrap = Some(significance);
        }

        let obj = Self{
            function_text: function_text.clone(),
            number: number_wrap,
            number_ref: number_ref_wrap,
            significance: significance_wrap,
        };

        return obj
    }
    pub fn validate(&self) -> bool {
        let expr = RE_FLOOR.clone();
        let function_text = self.function_text.clone();
        let mut check = expr.is_match(&function_text);
        let number = self.number.clone();
        let number_ref = self.number_ref.clone();
        let significance = self.significance.clone();
        if check == false {
            return check
        }
        if number.is_none() && number_ref.is_none() {
            check = false;
        }
        if significance.is_none() {
            check = false
        }
        return check
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = FloorFunction::defaults(
            &function_text, 
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_FLOOR));
        }
        return (number_fails, failed_functions);
    }
    pub fn replace(&mut self, formula: String, data_map: HashMap<String, String>) -> String {
        let data_map = data_map.clone();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();
        let replacement_string: String;

        let number_wrap = self.number.clone();
        let number_ref_wrap = self.number_ref.clone();
        let significance = self.significance.clone().unwrap() - 1;
        let number: f64;

        if number_wrap.is_some() {
            number = number_wrap.unwrap();
        } else {
            let number_ref = number_ref_wrap.unwrap();
            let function_attr = FunctionAttribute::defaults(
                &number_ref, 
                Some(true)
            );
            let number_str = function_attr.replace(data_map.clone()).item_processed.unwrap();
            number = FromStr::from_str(number_str.as_str()).unwrap();
        }
        let number = round::floor(number, significance);

        replacement_string = number.to_string();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        formula = format!("\"{}\"", formula);
        return formula;
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = FloorFunction::defaults(
            &function_text, 
        );
        formula = concat_obj.replace(formula, data_map.clone());
        return formula
    }
}

// COUNT(value1, value2, ...))
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CountFunction {
    pub function_text: String,
    pub attrs: Option<String>,
}
impl CountFunction {
    pub fn defaults(function_text: &String) -> CountFunction {
        // COUNT(value1, value2, ...))

        let matches = RE_COUNT.captures(function_text).unwrap();
        let attrs = matches.name("attrs");

        let mut attrs_wrap: Option<String> = None;
        if attrs.is_some() {
            let attrs = attrs.unwrap().as_str().to_string();
            attrs_wrap = Some(attrs);
        }

        let obj = Self{
            function_text: function_text.clone(),
            attrs: attrs_wrap,
        };

        return obj
    }
    pub fn validate(&self) -> bool {
        let expr = RE_COUNT.clone();
        let function_text = self.function_text.clone();
        let mut check = expr.is_match(&function_text);
        if check == false {
            return check
        }
        let attrs_wrap = self.attrs.clone();
        if attrs_wrap.is_none() {
            check = false
        }
        return check
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = CountFunction::defaults(
            &function_text, 
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_COUNT));
        }
        return (number_fails, failed_functions);
    }
    pub fn replace(&mut self, formula: String, data_map: HashMap<String, String>) -> String {
        let data_map = data_map.clone();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();
        let replacement_string: String;

        let attrs = self.attrs.clone().unwrap();
        let items: Vec<&str> = attrs.split(",").collect();
        let mut number_items: Vec<String> = Vec::new();
        for mut item in items {
            item = item.trim();
            let is_string = item.to_string().find("\"");
            let is_ref = item.to_string().find("{");
            if is_ref.is_some() {
                let item_ = &item.trim().to_string();
                let function_attr = FunctionAttribute::defaults(
                    item_, 
                    Some(true)
                );
                let result = function_attr.replace(data_map.clone());
                let result = result.item_processed.clone();
                let result = result.unwrap();
                let result = result.as_str();
                let is_string = result.to_string().find("\"");
                if is_string.is_none() {
                    number_items.push(result.to_string());
                }
            } else {
                if is_string.is_none() {
                    number_items.push(item.to_string());
                }
            }
        }
        let count = number_items.len();

        replacement_string = count.to_string();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        return formula;
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = CountFunction::defaults(
            &function_text, 
        );
        formula = concat_obj.replace(formula, data_map.clone());
        return formula
    }
}

// COUNTA(value1, value2, ...))
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CountAFunction {
    pub function_text: String,
    pub attrs: Option<String>,
}
impl CountAFunction {
    pub fn defaults(function_text: &String) -> CountAFunction {
        // COUNTA(value1, value2, ...))
        // We take as empty null and ""

        let matches = RE_COUNTA.captures(function_text).unwrap();
        let attrs = matches.name("attrs");

        let mut attrs_wrap: Option<String> = None;
        if attrs.is_some() {
            let attrs = attrs.unwrap().as_str().to_string();
            attrs_wrap = Some(attrs);
        }

        let obj = Self{
            function_text: function_text.clone(),
            attrs: attrs_wrap,
        };

        return obj
    }
    pub fn validate(&self) -> bool {
        let expr = RE_COUNTA.clone();
        let function_text = self.function_text.clone();
        let mut check = expr.is_match(&function_text);
        if check == false {
            return check
        }
        let attrs_wrap = self.attrs.clone();
        if attrs_wrap.is_none() {
            check = false
        }
        return check
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = CountAFunction::defaults(
            &function_text, 
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_COUNTA));
        }
        return (number_fails, failed_functions);
    }
    pub fn replace(&mut self, formula: String, data_map: HashMap<String, String>) -> String {
        let data_map = data_map.clone();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();
        let replacement_string: String;

        let attrs = self.attrs.clone().unwrap();
        let items: Vec<&str> = attrs.split(",").collect();
        let mut number_items: Vec<String> = Vec::new();
        for mut item in items {
            item = item.trim();
            let is_string = item.to_string().find("\"");
            let is_null = item.to_lowercase() == "null";
            let is_ref = item.to_string().find("{");
            if is_ref.is_some() {
                let item_ = &item.trim().to_string();
                let function_attr = FunctionAttribute::defaults(
                    item_, 
                    Some(true)
                );
                let result = function_attr.replace(data_map.clone());
                let result = result.item_processed.clone();
                let result = result.unwrap();
                let result = result.as_str();
                let is_string = result.to_string().find("\"");
                if is_string.is_some() {
                    if item != "\"\"" {
                        number_items.push(result.to_string());
                    }
                } else {
                    if is_null == false {
                        number_items.push(item.to_string());
                    }
                }
            } else {
                if is_string.is_some() {
                    if item != "\"\"" {
                        number_items.push(item.to_string());
                    }
                } else {
                    if is_null == false {
                        number_items.push(item.to_string());
                    }
                }
            }
        }
        let count = number_items.len();

        replacement_string = count.to_string();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        return formula;
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = CountAFunction::defaults(
            &function_text, 
        );
        formula = concat_obj.replace(formula, data_map.clone());
        return formula
    }
}

// COUNTALL(value1, value2, ...))
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CountAllFunction {
    pub function_text: String,
    pub attrs: Option<String>,
}
impl CountAllFunction {
    pub fn defaults(function_text: &String) -> CountAllFunction {
        // COUNTALL(value1, value2, ...))
        // We count all, including nulls and empty values

        let matches = RE_COUNTALL.captures(function_text).unwrap();
        let attrs = matches.name("attrs");

        let mut attrs_wrap: Option<String> = None;
        if attrs.is_some() {
            let attrs = attrs.unwrap().as_str().to_string();
            attrs_wrap = Some(attrs);
        }

        let obj = Self{
            function_text: function_text.clone(),
            attrs: attrs_wrap,
        };

        return obj
    }
    pub fn validate(&self) -> bool {
        let expr = RE_COUNTALL.clone();
        let function_text = self.function_text.clone();
        let mut check = expr.is_match(&function_text);
        if check == false {
            return check
        }
        let attrs_wrap = self.attrs.clone();
        if attrs_wrap.is_none() {
            check = false
        }
        return check
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = CountAllFunction::defaults(
            &function_text, 
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_COUNTALL));
        }
        return (number_fails, failed_functions);
    }
    pub fn replace(&mut self, formula: String) -> String {
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();
        let replacement_string: String;

        let attrs = self.attrs.clone().unwrap();
        let items: Vec<&str> = attrs.split(",").collect();
        let mut number_items: Vec<String> = Vec::new();
        for mut item in items {
            item = item.trim();
            number_items.push(item.to_string());
        }
        let count = number_items.len();

        replacement_string = count.to_string();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        return formula;
    }
    pub fn do_replace(function_text: &String, mut formula: String) -> String {
        let mut concat_obj = CountAllFunction::defaults(
            &function_text, 
        );
        formula = concat_obj.replace(formula);
        return formula
    }
}

// EVEN(number)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EvenFunction {
    pub function_text: String,
    pub number: Option<f64>,
    pub number_ref: Option<String>,
}
impl EvenFunction {
    pub fn defaults(function_text: &String) -> EvenFunction {
        // EVEN(number)
        // EVEN(1.5) => 2
        // EVEN(3) => 4
        // EVEN(2) => 2
        // EVEN(-1) => -2
        // EVEN({Column}) => 2

        let matches = RE_EVEN.captures(function_text).unwrap();
        let attr_number = matches.name("number");
        let attr_number_ref = matches.name("number_ref");

        let mut number_wrap: Option<f64> = None;
        let mut number_ref_wrap: Option<String> = None;

        if attr_number.is_some() {
            let number: f64 = FromStr::from_str(attr_number.unwrap().as_str()).unwrap();
            number_wrap = Some(number);
        } else if attr_number_ref.is_some() {
            let number_ref = attr_number_ref.unwrap().as_str().to_string();
            number_ref_wrap = Some(number_ref);
        }

        let obj = Self{
            function_text: function_text.clone(),
            number: number_wrap,
            number_ref: number_ref_wrap,
        };

        return obj
    }
    pub fn validate(&self) -> bool {
        let expr = RE_EVEN.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = EvenFunction::defaults(
            &function_text, 
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_EVEN));
        }
        return (number_fails, failed_functions);
    }
    pub fn replace(&mut self, formula: String, data_map: HashMap<String, String>) -> String {
        let data_map = data_map.clone();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();

        let number_wrap = self.number.clone();
        let number_ref_wrap = self.number_ref.clone();
        let mut number: f64 = 0.0;
        let mut rounded_int: i32;
        if number_wrap.is_some() {
            number = number_wrap.unwrap();
        } else if number_ref_wrap.is_some() {
            let number_ref = number_ref_wrap.unwrap();
            let function_attr = FunctionAttribute::defaults(
                &number_ref, 
                Some(true)
            );
            let result = function_attr.replace(data_map.clone());
            let result = result.item_processed.clone();
            number = FromStr::from_str(result.unwrap().as_str()).unwrap();
        }

        let rounded = number.round();
        rounded_int = FromStr::from_str(rounded.to_string().as_str()).unwrap();
        let is_even = rounded_int%2 == 0;
        if is_even == false && rounded_int > 0 {
            rounded_int += 1;
        } else if is_even == true && rounded_int < 0 {
            rounded_int -= 1;
        }

        let replacement_string = rounded_int.to_string();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        return formula;
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = EvenFunction::defaults(
            &function_text, 
        );
        formula = concat_obj.replace(formula, data_map.clone());
        return formula
    }
}