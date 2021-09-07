use std::str::FromStr;
use regex::{Regex, Captures};
use std::{collections::HashMap};
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;

use crate::functions::FunctionAttribute;
use crate::functions::constants::*;


lazy_static! {
    static ref RE_MIN: Regex = Regex::new(r#"MIN\((?P<sequence>[\d\s,.-]+)\)|MIN\((?P<sequence_ref>\{[\w\s]+\})\)"#).unwrap();
    static ref RE_MAX: Regex = Regex::new(r#"MAX\((?P<sequence>[\d\s,.-]+)\)|MAX\((?P<sequence_ref>\{[\w\s]+\})\)"#).unwrap();
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StatOption {
    Min,
    Max,
}

// MIN(1,2,3,4)
// MAX(1,2,3,4)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatsFunction {
    pub function_text: String,
    pub sequence: Option<String>,
    pub sequence_ref: Option<String>,
    pub option: StatOption,
}
impl StatsFunction {
    pub fn defaults(function_text: &String, option: StatOption) -> StatsFunction {
        // MIN(1,2,3,4)
        // MIN({My Column}) (Having Set, or other collection of numbers)
        // MIN(-4, 1.9, 2.34)
        // MAX(1,2,3,4)

        let matches: Captures;
        match option {
            StatOption::Min => {
                matches = RE_MIN.captures(function_text).unwrap();
            },
            StatOption::Max => {
                matches = RE_MAX.captures(function_text).unwrap();
            },
        }

        let attr_sequence = matches.name("sequence");
        let attr_sequence_ref = matches.name("sequence_ref");

        let mut sequence_wrap: Option<String> = None;
        let mut sequence_ref_wrap: Option<String> = None;

        if attr_sequence.is_some() {
            let sequence: String = attr_sequence.unwrap().as_str().to_string();
            sequence_wrap = Some(sequence)
        }
        if attr_sequence_ref.is_some() {
            let sequence_ref: String = attr_sequence_ref.unwrap().as_str().to_string();
            sequence_ref_wrap = Some(sequence_ref)
        }

        let obj = Self{
            function_text: function_text.clone(),
            sequence: sequence_wrap,
            sequence_ref: sequence_ref_wrap,
            option: option,
        };

        return obj
    }
    pub fn validate(&self) -> bool {
        let expr: Regex;
        let option = self.option.clone();
        match option {
            StatOption::Min => {
                expr = RE_MIN.clone();
            },
            StatOption::Max => {
                expr = RE_MAX.clone();
            },
        }
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>),
        option: StatOption
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = StatsFunction::defaults(
            &function_text, option.clone()
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            match option {
                StatOption::Min => {
                    failed_functions.push(String::from(FUNCTION_MIN));
                },
                StatOption::Max => {
                    failed_functions.push(String::from(FUNCTION_MAX));
                },
            }
            
        }
        return (number_fails, failed_functions);
    }
    pub fn replace(&mut self, formula: String, data_map: HashMap<String, String>) -> String {
        let data_map = data_map.clone();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();
        let replacement_string: String;
        let option = self.option.clone();

        let sequence_wrap = self.sequence.clone();
        let sequence_ref_wrap = self.sequence_ref.clone();
        let mut sequence: String;
        if sequence_wrap.is_some() {
            sequence = sequence_wrap.unwrap();
        } else {
            let sequence_ref = sequence_ref_wrap.unwrap();
            let function_attr = FunctionAttribute::defaults(
                &sequence_ref, 
                Some(true)
            );
            sequence = function_attr.replace(data_map.clone()).item_processed.unwrap();
            // To do when I have Set field, since design can change
        }
        sequence = sequence.replace("\"", "");
        let mut sequence_list: Vec<f64> = Vec::new();
        let sequence_str_list: Vec<&str> = sequence.split(",").collect();
        for mut item in sequence_str_list {
            let has_dot = item.clone().find(".");
            item = item.trim();
            let mut item_string: String = item.to_string();
            if has_dot.is_none() {
                item_string.push_str(".0");
            }
            let item_number: f64 = FromStr::from_str(item_string.as_str()).unwrap();
            sequence_list.push(item_number);
        }
        let stat_result: f64;
        match option {
            StatOption::Min => {
                let mut min: f64 = sequence_list[0];
                for item in sequence_list {
                    if item < min {
                        min = item
                    }
                }
                stat_result = min;
            },
            StatOption::Max => {
                let mut max: f64 = sequence_list[0];
                for item in sequence_list {
                    if item > max {
                        max = item
                    }
                }                
                stat_result = max;
            },
        }

        replacement_string = stat_result.to_string();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        return formula;
    }
    pub fn do_replace(
        function_text: &String, 
        data_map: HashMap<String, String>, 
        mut formula: String,
        option: StatOption
    ) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = StatsFunction::defaults(
            &function_text, option
        );
        formula = concat_obj.replace(formula, data_map.clone());
        return formula
    }
}
