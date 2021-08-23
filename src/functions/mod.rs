pub mod text;
pub mod constants;

use std::collections::HashMap;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use regex::Regex;
use tr::tr;

use crate::functions::constants::*;
use crate::functions::text::*;
use crate::planet::PlanetError;

// achiever planet functions
pub const FORMULA_FUNCTIONS: [&str; 49] = [
    FUNCTION_CONCAT,
    FUNCTION_FORMAT,
    FUNCTION_JOINLIST,
    FUNCTION_LENGTH,
    FUNCTION_LOWER,
    FUNCTION_UPPER,
    FUNCTION_REPLACE,
    FUNCTION_DATE,
    FUNCTION_DAY,
    FUNCTION_DAYS,
    FUNCTION_HOUR,
    FUNCTION_MONTH,
    FUNCTION_NOW,
    FUNCTION_SECOND,
    FUNCTION_MINUTE,
    FUNCTION_TODAY,
    FUNCTION_WEEK,
    FUNCTION_YEAR,
    FUNCTION_IF,
    FUNCTION_MID,
    FUNCTION_REPT,
    FUNCTION_SUBSTITUTE,
    FUNCTION_TRIM,
    FUNCTION_CEILING,
    FUNCTION_COUNT,
    FUNCTION_COUNTA,
    FUNCTION_COUNTALL,
    FUNCTION_EVEN,
    FUNCTION_EXP,
    FUNCTION_FLOOR,
    FUNCTION_INT,
    FUNCTION_LOG,
    FUNCTION_MAX,
    FUNCTION_MIN,
    FUNCTION_MOD,
    FUNCTION_POWER,
    FUNCTION_ROUND,
    FUNCTION_ROUNDDOWN,
    FUNCTION_ROUNDUP,
    FUNCTION_SQRT,
    FUNCTION_VALUE,
    FUNCTION_CREATED_TIME,
    FUNCTION_DATEADD,
    FUNCTION_DATETDIF,
    FUNCTION_DATETIME_FORMAT,
    FUNCTION_LAST_MODIFIED_TIME,
    FUNCTION_RECORD_ID,
    FUNCTION_TRUE,
    FUNCTION_FALSE,
];

lazy_static! {
    // CONCAT("mine", "-", {My Column}, 45) :: Regex to catch the function attributes in an array
    static ref RE_FORMULA_FUNCTIONS: Regex = Regex::new(r#"([A-Z]+\(.+\))"#).unwrap();
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionsHanler {
    pub function_name: String,
    pub function_text: String,
    pub data_map: HashMap<String, String>,
}
impl FunctionsHanler{
    pub fn do_functions(&self, mut formula: String) -> String {
        let function_name = self.function_name.as_str();
        // Match all achiever functions here. Used by insert and update data to process formula columns.
        match function_name {
            FUNCTION_CONCAT => {
                formula = ConcatenateFunction::do_replace(
                    &self.function_text, self.data_map.clone(), formula);
            },
            FUNCTION_FORMAT => {
                formula = FormatFunction::do_replace(
                    &self.function_text, self.data_map.clone(), formula);
            },
            FUNCTION_JOINLIST => {
                formula = JoinListFunction::do_replace(
                    &self.function_text, self.data_map.clone(), formula);
            },
            FUNCTION_LENGTH => {
                formula = LengthFunction::do_replace(
                    &self.function_text, self.data_map.clone(), formula);
            },
            FUNCTION_LOWER => {
                formula = LowerFunction::do_replace(
                    &self.function_text, self.data_map.clone(), formula);
            },
            _ => {
            }
        }
        return formula
    }
}

pub fn validate_formula(formula: &String) -> Result<bool, PlanetError> {
    let check = true;
    let mut achiever_functions: Vec<String> = Vec::new();
    for function_name_item in FORMULA_FUNCTIONS {
        achiever_functions.push(function_name_item.to_string());
    }
    // Get all function texts in the formula, like ["CONCAT(...)", "ABS(23)", ...]
    let mut formula_functions: Vec<String> = Vec::new();
    let mut function_name_map: HashMap<String, String> = HashMap::new();
    for capture in RE_FORMULA_FUNCTIONS.captures_iter(formula) {
        let function_text = capture.get(0).unwrap().as_str().to_string();
        let function_name = get_function_name(function_text.clone());
        if achiever_functions.contains(&function_name) == true {
            formula_functions.push(function_text.clone());
            function_name_map.insert(function_name, function_text.clone());    
        }
    }

    // Validate all formula_functions (only ones found in formula from all functions in achiever)
    let mut number_fails = 0;
    for function_name in function_name_map.keys() {
        let function_name = function_name.as_str();
        let function_text = function_name_map.get(function_name).unwrap();
        match function_name {
            FUNCTION_CONCAT => {
                number_fails = ConcatenateFunction::do_validate(function_text, &number_fails);
            },
            FUNCTION_FORMAT => {
                number_fails = FormatFunction::do_validate(function_text, &number_fails);
            },
            FUNCTION_JOINLIST => {
                number_fails = JoinListFunction::do_validate(function_text, &number_fails);
            },
            FUNCTION_LENGTH => {
                number_fails = LengthFunction::do_validate(function_text, &number_fails);
            },
            FUNCTION_LOWER => {
                number_fails = LowerFunction::do_validate(function_text, &number_fails);
            },
            _ => {
            }
        }
    }
    if number_fails > 0 {
        return Err(
            PlanetError::new(
                500, 
                Some(tr!("Could not validate formula")),
            )
        );
    }
    return Ok(check);
}

pub fn check_achiever_function(function_text: String) -> bool {
    let mut check = false;
    for function_item in FORMULA_FUNCTIONS {
        let function_name = get_function_name(function_text.clone());
        if function_item.to_lowercase() == function_name.to_string().to_lowercase() {
            check = true;
            break
        }
    }
    return check;
}

pub fn get_function_name(function_text: String) -> String {
    let function_name_pieces = function_text.split("(");
    let function_name_pieces: Vec<&str> = function_name_pieces.collect();
    let function_name = function_name_pieces[0].to_string();
    return function_name
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionAttribute{
    pub item: String,
    pub remove_quotes: Option<bool>,
    pub item_processed: Option<String>,    
}
impl FunctionAttribute {
    pub fn defaults(attribute: &String, remove_quotes: Option<bool>) -> FunctionAttribute {
        let mut remove_quotes_value: bool = false;
        if remove_quotes.is_some() {
            remove_quotes_value = true;
        }
        let obj = Self{
            item: attribute.clone(),
            remove_quotes: Some(remove_quotes_value),
            item_processed: None,
        };
        return obj
    }
    pub fn replace(&self, data_map: HashMap<String, String>) -> Self {
        let mut item = self.item.clone();
        let remove_quotes = self.remove_quotes.unwrap();
        if remove_quotes == true {
            item = item.replace("\"", "");
        }
        let item_string: String;
        let item_find = item.find("{");
        let mut obj = self.clone();
        if item_find.is_some() && item_find.unwrap() == 0 {
            // I have a column, need to get data from data_map
            item = item.replace("{", "").replace("}", "");
            let item_value = data_map.get(&item);
            if item_value.is_some() {
                let item_value = item_value.unwrap().clone();
                item_string = item_value;
                obj.item_processed = Some(item_string);
            }
        } else {
            item_string = item.to_string();
            obj.item_processed = Some(item_string);
        }
        return obj;
    }
}
