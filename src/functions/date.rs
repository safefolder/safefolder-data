use std::str::FromStr;
use regex::Regex;
use std::{collections::HashMap};
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;
// use chrono::{DateTime, Datelike, FixedOffset, Local, NaiveDate, NaiveDateTime, Timelike, Utc};

// use crate::functions::FunctionAttribute;

// 1. defaults receives the function test, that is FUNC_NAME(attr1, attr2, ...) and returns function instance
//      attributes embeeded into the object attributes. It does not prepare for replacement or anythin.
// 2. replace is the one that received the data map and gets attributes from function and does replacement.
// 3. do_replace: Creates the object and does replacement.
// 4. validate: Validates the function text with respect to the regex for the function, checks function text is
//       fine.

// I need to refactor and extract common operations that can be used in other functions as well and place into
//     the mod inside functions.

lazy_static! {
    static ref RE_DATE: Regex = Regex::new(r#"DATE\((?P<year>\d+),[\s]+(?P<month>\d+),[\s]+(?P<day>\d+)\)"#).unwrap();
}

// DATE(year,month,day)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DateFunction {
    pub function_text: String,
    pub attributes: Vec<String>,
    pub attributes_value_map: Option<HashMap<String, String>>,
    pub year: i32,
    pub month: u32,
    pub day: u32,
}
impl DateFunction {
    pub fn defaults(function_text: &String) -> DateFunction {
        // DATE(year,month,day)
        // DATE(2021, 8, 28)

        let matches = RE_DATE.captures(function_text).unwrap();
        let year = matches.name("year").unwrap();
        let month = matches.name("month").unwrap();
        let day = matches.name("day").unwrap();
        let year: i32 = FromStr::from_str(year.as_str()).unwrap();
        let month: u32 = FromStr::from_str(month.as_str()).unwrap();
        let day: u32 = FromStr::from_str(day.as_str()).unwrap();

        let attributes: Vec<String> = Vec::new();
        let obj = Self{
            function_text: function_text.clone(),
            attributes: attributes,
            attributes_value_map: None,
            year: year,
            month: month,
            day: day,
        };
        
        return obj
    }
    pub fn validate(&self) -> bool {
        let expr = RE_DATE.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    pub fn do_validate(function_text: &String, number_fails: &u32) -> u32 {
        let concat_obj = DateFunction::defaults(
            &function_text, 
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
        }
        return number_fails;
    }
    pub fn replace(&mut self, formula: String) -> String {
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();
        let year = self.year;
        let month = self.month;
        let day = self.day;
        let replacement_string: String = format!(
            "{year}-{month}-{day}T00:00:00+00:00",
            year=year,
            month=month,
            day=day
        );

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        formula = format!("\"{}\"", formula);
        return formula;
    }
    pub fn do_replace(function_text: &String, mut formula: String) -> String {
        let mut concat_obj = DateFunction::defaults(
            &function_text, 
        );
        formula = concat_obj.replace(formula);
        return formula
    }
}