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