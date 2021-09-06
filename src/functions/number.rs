use std::str::FromStr;
use regex::{Regex, Captures};
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
    static ref RE_EXP: Regex = Regex::new(r#"EXP\((?P<number>[+-]?[0-9]+\.?[0-9]*|\.[0-9]+)\)|EXP\((?P<number_ref>\{[\w\s]+\})\)"#).unwrap();
    static ref RE_INT: Regex = Regex::new(r#"INT\((?P<number>[+-]?[0-9]+\.?[0-9]*|\.[0-9]+)\)|INT\((?P<number_ref>\{[\w\s]+\})\)"#).unwrap();
    static ref RE_LOG: Regex = Regex::new(r#"LOG\((?P<number>[+-]?[0-9]+\.?[0-9]*|\.[0-9]+)[\n\s\t]{0,},{0,}[\n\s\t]{0,}(?P<base>\d+){0,}\)|LOG\((?P<number_ref>\{[\w\s]+\})[\n\s\t]{0,},{0,}[\n\s\t]{0,}(?P<base_ref>\d+){0,}\)"#).unwrap();
    static ref RE_MOD: Regex = Regex::new(r#"MOD\((?P<number>[+-]?[0-9]+\.?[0-9]*|\.[0-9]+)[\n\s\t]{0,},[\n\s\t]{0,}(?P<divisor>\d+){0,}\)|MOD\((?P<number_ref>\{[\w\s]+\})[\n\s\t]{0,},[\n\s\t]{0,}(?P<divisor_ref>\d+){0,}\)"#).unwrap();
    static ref RE_POWER: Regex = Regex::new(r#"POWER\((?P<number>[+-]?[0-9]+\.?[0-9]*|\.[0-9]+)[\n\s\t]{0,},[\n\s\t]{0,}(?P<power>[+-]?[0-9]+\.?[0-9]*|\.[0-9]+){0,}\)|POWER\((?P<number_ref>\{[\w\s]+\})[\n\s\t]{0,},[\n\s\t]{0,}(?P<power_ref>[+-]?[0-9]+\.?[0-9]*|\.[0-9]+){0,}\)"#).unwrap();
    static ref RE_ROUND: Regex = Regex::new(r#"ROUND\((?P<number>[+-]?[0-9]+\.?[0-9]*|\.[0-9]+)[\n\s\t]{0,},[\n\s\t]{0,}(?P<digits>\d+){0,}\)|ROUND\((?P<number_ref>\{[\w\s]+\})[\n\s\t]{0,},[\n\s\t]{0,}(?P<digits_ref>\d+){0,}\)"#).unwrap();
    static ref RE_ROUND_UP: Regex = Regex::new(r#"ROUNDUP\((?P<number>[+-]?[0-9]+\.?[0-9]*|\.[0-9]+)[\n\s\t]{0,},[\n\s\t]{0,}(?P<digits>\d+){0,}\)|ROUNDUP\((?P<number_ref>\{[\w\s]+\})[\n\s\t]{0,},[\n\s\t]{0,}(?P<digits_ref>\d+){0,}\)"#).unwrap();
    static ref RE_ROUND_DOWN: Regex = Regex::new(r#"ROUNDDOWN\((?P<number>[+-]?[0-9]+\.?[0-9]*|\.[0-9]+)[\n\s\t]{0,},[\n\s\t]{0,}(?P<digits>\d+){0,}\)|ROUNDDOWN\((?P<number_ref>\{[\w\s]+\})[\n\s\t]{0,},[\n\s\t]{0,}(?P<digits_ref>\d+){0,}\)"#).unwrap();
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

// EXP(number)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExpFunction {
    pub function_text: String,
    pub number: Option<f64>,
    pub number_ref: Option<String>,
}
impl ExpFunction {
    pub fn defaults(function_text: &String) -> ExpFunction {
        // EXP(number)
        // EXP(1) => 2.71828183

        let matches = RE_EXP.captures(function_text).unwrap();
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
        let expr = RE_EXP.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = ExpFunction::defaults(
            &function_text, 
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_EXP));
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
        let number_result: f64;
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

        number_result = number.exp();

        let replacement_string = number_result.to_string();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        return formula;
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = ExpFunction::defaults(
            &function_text, 
        );
        formula = concat_obj.replace(formula, data_map.clone());
        return formula
    }
}

// INT(number)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IntFunction {
    pub function_text: String,
    pub number: Option<f64>,
    pub number_ref: Option<String>,
}
impl IntFunction {
    pub fn defaults(function_text: &String) -> IntFunction {
        // INT(number)
        // INT(8.9) => 8

        let matches = RE_INT.captures(function_text).unwrap();
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
        let expr = RE_INT.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = IntFunction::defaults(
            &function_text, 
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_INT));
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
        number = number.trunc();
        let number_str = number.to_string();
        let number_str = number_str.as_str();
        let number_result: isize = FromStr::from_str(number_str).unwrap();

        let replacement_string = number_result.to_string();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        return formula;
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = IntFunction::defaults(
            &function_text, 
        );
        formula = concat_obj.replace(formula, data_map.clone());
        return formula
    }
}

// LOG(number)
// LOG(number, [base])
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogFunction {
    pub function_text: String,
    pub number: Option<f64>,
    pub number_ref: Option<String>,
    pub base: Option<usize>,
    pub base_ref: Option<usize>,
}
impl LogFunction {
    pub fn defaults(function_text: &String) -> LogFunction {
        // LOG(number)
        // LOG(number, [base])
        // LOG(10) = 1
        // LOG(8, 2) = 3

        let matches = RE_LOG.captures(function_text).unwrap();
        let attr_number = matches.name("number");
        let attr_number_ref = matches.name("number_ref");
        let attr_base = matches.name("base");
        let attr_base_ref = matches.name("base_ref");

        let mut number_wrap: Option<f64> = None;
        let mut number_ref_wrap: Option<String> = None;
        let mut base_wrap: Option<usize> = None;
        let mut base_ref_wrap: Option<usize> = None;

        if attr_number.is_some() {
            let number: f64 = FromStr::from_str(attr_number.unwrap().as_str()).unwrap();
            number_wrap = Some(number);
        } else if attr_number_ref.is_some() {
            let number_ref = attr_number_ref.unwrap().as_str().to_string();
            number_ref_wrap = Some(number_ref);
        }
        if attr_base.is_some() {
            let base = attr_base.unwrap().as_str();
            let base: usize = FromStr::from_str(base).unwrap();
            base_wrap = Some(base);
        }
        if attr_base_ref.is_some() {
            let base = attr_base_ref.unwrap().as_str();
            let base_ref: usize = FromStr::from_str(base).unwrap();
            base_ref_wrap = Some(base_ref);
        }

        let obj = Self{
            function_text: function_text.clone(),
            number: number_wrap,
            number_ref: number_ref_wrap,
            base: base_wrap,
            base_ref: base_ref_wrap,
        };

        return obj
    }
    pub fn validate(&self) -> bool {
        let expr = RE_LOG.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = LogFunction::defaults(
            &function_text, 
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_LOG));
        }
        return (number_fails, failed_functions);
    }
    pub fn replace(&mut self, formula: String, data_map: HashMap<String, String>) -> String {
        let data_map = data_map.clone();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();

        let number_wrap = self.number.clone();
        let number_ref_wrap = self.number_ref.clone();
        let base_wrap = self.base.clone();
        let base_ref_wrap = self.base_ref.clone();
        let mut number: f64 = 0.0;
        let mut base: f64 = 10.0;
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
        if base_wrap.is_some() {
            let base_: usize = base_wrap.unwrap();
            base = FromStr::from_str(base_.to_string().as_str()).unwrap();
        }
        if base_ref_wrap.is_some() {
            let base_: usize = base_ref_wrap.unwrap();
            base = FromStr::from_str(base_.to_string().as_str()).unwrap();
        }
        number = number.log(base);
        let number_str = number.to_string();
        let number_str = number_str.as_str();
        let number_result: isize = FromStr::from_str(number_str).unwrap();

        let replacement_string = number_result.to_string();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        return formula;
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = LogFunction::defaults(
            &function_text, 
        );
        formula = concat_obj.replace(formula, data_map.clone());
        return formula
    }
}

// MOD(number, divisor)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModFunction {
    pub function_text: String,
    pub number: Option<f64>,
    pub number_ref: Option<String>,
    pub divisor: Option<usize>,
    pub divisor_ref: Option<usize>,
}
impl ModFunction {
    pub fn defaults(function_text: &String) -> ModFunction {
        // MOD(number, divisor)

        let matches = RE_MOD.captures(function_text).unwrap();
        let attr_number = matches.name("number");
        let attr_number_ref = matches.name("number_ref");
        let attr_divisor = matches.name("divisor");
        let attr_divisor_ref = matches.name("divisor_ref");

        let mut number_wrap: Option<f64> = None;
        let mut number_ref_wrap: Option<String> = None;
        let mut divisor_wrap: Option<usize> = None;
        let mut divisor_ref_wrap: Option<usize> = None;

        if attr_number.is_some() {
            let number: f64 = FromStr::from_str(attr_number.unwrap().as_str()).unwrap();
            number_wrap = Some(number);
        } else if attr_number_ref.is_some() {
            let number_ref = attr_number_ref.unwrap().as_str().to_string();
            number_ref_wrap = Some(number_ref);
        }
        if attr_divisor.is_some() {
            let divisor = attr_divisor.unwrap().as_str();
            let divisor: usize = FromStr::from_str(divisor).unwrap();
            divisor_wrap = Some(divisor);
        }
        if attr_divisor_ref.is_some() {
            let divisor = attr_divisor_ref.unwrap().as_str();
            let divisor_ref: usize = FromStr::from_str(divisor).unwrap();
            divisor_ref_wrap = Some(divisor_ref);
        }

        let obj = Self{
            function_text: function_text.clone(),
            number: number_wrap,
            number_ref: number_ref_wrap,
            divisor: divisor_wrap,
            divisor_ref: divisor_ref_wrap,
        };

        return obj
    }
    pub fn validate(&self) -> bool {
        let expr = RE_MOD.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = ModFunction::defaults(
            &function_text, 
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_MOD));
        }
        return (number_fails, failed_functions);
    }
    pub fn replace(&mut self, formula: String, data_map: HashMap<String, String>) -> String {
        let data_map = data_map.clone();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();

        let number_wrap = self.number.clone();
        let number_ref_wrap = self.number_ref.clone();
        let divisor_wrap = self.divisor.clone();
        let divisor_ref_wrap = self.divisor_ref.clone();
        let mut number: f64 = 0.0;
        let mut divisor: f64 = 10.0;
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
        if divisor_wrap.is_some() {
            let divisor_: usize = divisor_wrap.unwrap();
            divisor = FromStr::from_str(divisor_.to_string().as_str()).unwrap();
        }
        if divisor_ref_wrap.is_some() {
            let divisor_: usize = divisor_ref_wrap.unwrap();
            divisor = FromStr::from_str(divisor_.to_string().as_str()).unwrap();
        }
        number = number%divisor;
        let number_str = number.to_string();
        let number_str = number_str.as_str();
        let number_result: isize = FromStr::from_str(number_str).unwrap();

        let replacement_string = number_result.to_string();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        return formula;
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = ModFunction::defaults(
            &function_text, 
        );
        formula = concat_obj.replace(formula, data_map.clone());
        return formula
    }
}

// POWER(number, power)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PowerFunction {
    pub function_text: String,
    pub number: Option<f64>,
    pub number_ref: Option<String>,
    pub power: Option<f64>,
    pub power_ref: Option<f64>,
}
impl PowerFunction {
    pub fn defaults(function_text: &String) -> PowerFunction {
        // POWER(number, power)

        let matches = RE_POWER.captures(function_text).unwrap();
        let attr_number = matches.name("number");
        let attr_number_ref = matches.name("number_ref");
        let attr_power = matches.name("power");
        let attr_power_ref = matches.name("power_ref");

        let mut number_wrap: Option<f64> = None;
        let mut number_ref_wrap: Option<String> = None;
        let mut power_wrap: Option<f64> = None;
        let mut power_ref_wrap: Option<f64> = None;

        if attr_number.is_some() {
            let number: f64 = FromStr::from_str(attr_number.unwrap().as_str()).unwrap();
            number_wrap = Some(number);
        } else if attr_number_ref.is_some() {
            let number_ref = attr_number_ref.unwrap().as_str().to_string();
            number_ref_wrap = Some(number_ref);
        }
        if attr_power.is_some() {
            let power = attr_power.unwrap().as_str();
            let power: f64 = FromStr::from_str(power).unwrap();
            power_wrap = Some(power);
        }
        if attr_power_ref.is_some() {
            let power = attr_power_ref.unwrap().as_str();
            let power_ref: f64 = FromStr::from_str(power).unwrap();
            power_ref_wrap = Some(power_ref);
        }

        let obj = Self{
            function_text: function_text.clone(),
            number: number_wrap,
            number_ref: number_ref_wrap,
            power: power_wrap,
            power_ref: power_ref_wrap,
        };

        return obj
    }
    pub fn validate(&self) -> bool {
        let expr = RE_POWER.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = PowerFunction::defaults(
            &function_text, 
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_POWER));
        }
        return (number_fails, failed_functions);
    }
    pub fn replace(&mut self, formula: String, data_map: HashMap<String, String>) -> String {
        let data_map = data_map.clone();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();

        let number_wrap = self.number.clone();
        let number_ref_wrap = self.number_ref.clone();
        let power_wrap = self.power.clone();
        let power_ref_wrap = self.power_ref.clone();
        let mut number: f64 = 0.0;
        let mut power: f64 = 10.0;
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
        if power_wrap.is_some() {
            let power_: f64 = power_wrap.unwrap();
            power = FromStr::from_str(power_.to_string().as_str()).unwrap();
        }
        if power_ref_wrap.is_some() {
            let power_: f64 = power_ref_wrap.unwrap();
            power = FromStr::from_str(power_.to_string().as_str()).unwrap();
        }
        number = number.powf(power);
        let number_str = number.to_string();
        let number_str = number_str.as_str();
        let number_result: f64 = FromStr::from_str(number_str).unwrap();

        let replacement_string = number_result.to_string();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        return formula;
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = PowerFunction::defaults(
            &function_text, 
        );
        formula = concat_obj.replace(formula, data_map.clone());
        return formula
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RoundOption {
    Basic,
    Up,
    Down,
}

// ROUND(number, digits)
// ROUNDUP(number, digits)
// ROUNDDOWN(number, digits)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoundFunction {
    pub function_text: String,
    pub number: Option<f64>,
    pub number_ref: Option<String>,
    pub digits: Option<i8>,
    pub digits_ref: Option<i8>,
    pub option: RoundOption,
}
impl RoundFunction {
    pub fn defaults(function_text: &String, option: RoundOption) -> RoundFunction {
        // ROUND(number, digits)
        // ROUNDUP(number, digits)
        // ROUNDDOWN(number, digits)

        let matches: Captures;
        match option {
            RoundOption::Basic => {
                matches = RE_ROUND.captures(function_text).unwrap();
            },
            RoundOption::Up => {
                matches = RE_ROUND_UP.captures(function_text).unwrap();
            },
            RoundOption::Down => {
                matches = RE_ROUND_DOWN.captures(function_text).unwrap();
            },
        }
        let attr_number = matches.name("number");
        let attr_number_ref = matches.name("number_ref");
        let attr_digits = matches.name("digits");
        let attr_digits_ref = matches.name("digits_ref");

        let mut number_wrap: Option<f64> = None;
        let mut number_ref_wrap: Option<String> = None;
        let mut digits_wrap: Option<i8> = None;
        let mut digits_ref_wrap: Option<i8> = None;

        if attr_number.is_some() {
            let number: f64 = FromStr::from_str(attr_number.unwrap().as_str()).unwrap();
            number_wrap = Some(number);
        } else if attr_number_ref.is_some() {
            let number_ref = attr_number_ref.unwrap().as_str().to_string();
            number_ref_wrap = Some(number_ref);
        }
        if attr_digits.is_some() {
            let digits = attr_digits.unwrap().as_str();
            let digits: i8 = FromStr::from_str(digits).unwrap();
            digits_wrap = Some(digits);
        }
        if attr_digits_ref.is_some() {
            let digits = attr_digits_ref.unwrap().as_str();
            let digits_ref: i8 = FromStr::from_str(digits).unwrap();
            digits_ref_wrap = Some(digits_ref);
        }

        let obj = Self{
            function_text: function_text.clone(),
            number: number_wrap,
            number_ref: number_ref_wrap,
            digits: digits_wrap,
            digits_ref: digits_ref_wrap,
            option: option,
        };

        return obj
    }
    pub fn validate(&self) -> bool {
        let expr: Regex;
        match self.option {
            RoundOption::Basic => {
                expr = RE_ROUND.clone();
            },
            RoundOption::Up => {
                expr = RE_ROUND_UP.clone();
            },
            RoundOption::Down => {
                expr = RE_ROUND_DOWN.clone();
            },
        }
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>),
        option: RoundOption
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = RoundFunction::defaults(
            &function_text, option.clone()
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            match option {
                RoundOption::Basic => {
                    failed_functions.push(String::from(FUNCTION_ROUND));
                },
                RoundOption::Up => {
                    failed_functions.push(String::from(FUNCTION_ROUNDUP));
                },
                RoundOption::Down => {
                    failed_functions.push(String::from(FUNCTION_ROUNDDOWN));
                },
            }
            
        }
        return (number_fails, failed_functions);
    }
    pub fn replace(&mut self, formula: String, data_map: HashMap<String, String>) -> String {
        let data_map = data_map.clone();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();
        let option = self.option.clone();

        let number_wrap = self.number.clone();
        let number_ref_wrap = self.number_ref.clone();
        let digits_wrap = self.digits.clone();
        let digits_ref_wrap = self.digits_ref.clone();
        let mut number: f64 = 0.0;
        let mut digits: i8 = 2;
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
        if digits_wrap.is_some() {
            let digits_: i8 = digits_wrap.unwrap();
            digits = FromStr::from_str(digits_.to_string().as_str()).unwrap();
        }
        if digits_ref_wrap.is_some() {
            let digits_: i8 = digits_ref_wrap.unwrap();
            digits = FromStr::from_str(digits_.to_string().as_str()).unwrap();
        }
        match option {
            RoundOption::Basic => {
                number = round::half_away_from_zero(number, digits);
            },
            RoundOption::Up => {
                number = round::ceil(number, digits);
            },
            RoundOption::Down => {
                number = round::floor(number, digits);
            },
        }
        let number_str = number.to_string();
        let number_str = number_str.as_str();
        let number_result: f64 = FromStr::from_str(number_str).unwrap();

        let replacement_string = number_result.to_string();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        return formula;
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String, option: RoundOption) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = RoundFunction::defaults(
            &function_text, option
        );
        formula = concat_obj.replace(formula, data_map.clone());
        return formula
    }
}