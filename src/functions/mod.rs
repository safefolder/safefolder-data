pub mod constants;
pub mod text;
pub mod date;
pub mod structure;
pub mod number;

use std::collections::HashMap;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use regex::Regex;
use tr::tr;

use crate::functions::constants::*;
use crate::functions::text::*;
use crate::functions::date::*;
use crate::functions::number::*;
use crate::functions::structure::*;
use crate::planet::PlanetError;
use crate::storage::table::DbData;

// achiever planet functions
pub const FORMULA_FUNCTIONS: [&str; 50] = [
    FUNCTION_CONCAT,
    FUNCTION_FORMAT,
    FUNCTION_JOINLIST,
    FUNCTION_LENGTH,
    FUNCTION_LOWER,
    FUNCTION_UPPER,
    FUNCTION_REPLACE,
    FUNCTION_DATE,
    FUNCTION_DATEFMT,
    FUNCTION_DAY,
    FUNCTION_DAYS,
    FUNCTION_HOUR,
    FUNCTION_MONTH,
    FUNCTION_NOW,
    FUNCTION_SECOND,
    FUNCTION_MINUTE,
    FUNCTION_TODAY,
    FUNCTION_WEEK,
    FUNCTION_WEEKDAY,
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
    FUNCTION_DATEDIF,
    FUNCTION_LAST_MODIFIED_TIME,
    FUNCTION_RECORD_ID,
    FUNCTION_TRUE,
    FUNCTION_FALSE,
];

lazy_static! {
    // CONCAT("mine", "-", {My Column}, 45) :: Regex to catch the function attributes in an array
    static ref RE_FORMULA_FUNCTIONS: Regex = Regex::new(r#"([a-zA-Z]+\(.+\))"#).unwrap();
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionsHanler {
    pub function_name: String,
    pub function_text: String,
    pub data_map: HashMap<String, String>,
    pub table: DbData,
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
            FUNCTION_UPPER => {
                formula = UpperFunction::do_replace(
                    &self.function_text, self.data_map.clone(), formula);
            },
            FUNCTION_REPLACE => {
                formula = ReplaceFunction::do_replace(
                    &self.function_text, self.data_map.clone(), formula);
            },
            FUNCTION_MID => {
                formula = MidFunction::do_replace(
                    &self.function_text, self.data_map.clone(), formula);
            },
            FUNCTION_REPT => {
                formula = ReptFunction::do_replace(
                    &self.function_text, self.data_map.clone(), formula);
            },
            FUNCTION_SUBSTITUTE => {
                formula = SubstituteFunction::do_replace(
                    &self.function_text, self.data_map.clone(), formula);
            },
            FUNCTION_TRIM => {
                formula = TrimFunction::do_replace(
                    &self.function_text, self.data_map.clone(), formula);
            },
            FUNCTION_CEILING => {
                formula = CeilingFunction::do_replace(
                    &self.function_text, self.data_map.clone(), formula);
            },
            FUNCTION_FLOOR => {
                formula = FloorFunction::do_replace(
                    &self.function_text, self.data_map.clone(), formula);
            },
            FUNCTION_COUNT => {
                formula = CountFunction::do_replace(
                    &self.function_text, self.data_map.clone(), formula);
            },
            FUNCTION_COUNTA => {
                formula = CountAFunction::do_replace(
                    &self.function_text, self.data_map.clone(), formula);
            },
            FUNCTION_COUNTALL => {
                formula = CountAllFunction::do_replace(
                    &self.function_text, formula);
            },
            FUNCTION_EVEN => {
                formula = EvenFunction::do_replace(
                    &self.function_text, self.data_map.clone(), formula);
            },
            FUNCTION_EXP => {
                formula = ExpFunction::do_replace(
                    &self.function_text, self.data_map.clone(), formula);
            },
            FUNCTION_DATE => {
                formula = DateFunction::do_replace(
                    &self.function_text, formula);
            },
            FUNCTION_SECOND => {
                formula = DateTimeParseFunction::do_replace(
                    &self.function_text, DateTimeParseOption::Second, self.data_map.clone(), formula);
            },
            FUNCTION_MINUTE => {
                formula = DateTimeParseFunction::do_replace(
                    &self.function_text, DateTimeParseOption::Minute, self.data_map.clone(), formula);
            },
            FUNCTION_HOUR => {
                formula = DateTimeParseFunction::do_replace(
                    &self.function_text, DateTimeParseOption::Hour, self.data_map.clone(), formula);
            },
            FUNCTION_DAY => {
                formula = DateParseFunction::do_replace(
                    &self.function_text, DateParseOption::Day, self.data_map.clone(), formula);
            },
            FUNCTION_WEEK => {
                formula = DateParseFunction::do_replace(
                    &self.function_text, DateParseOption::Week, self.data_map.clone(), formula);
            },
            FUNCTION_WEEKDAY => {
                formula = DateParseFunction::do_replace(
                    &self.function_text, DateParseOption::WeekDay, self.data_map.clone(), formula);
            },
            FUNCTION_MONTH => {
                formula = DateParseFunction::do_replace(
                    &self.function_text, DateParseOption::Month, self.data_map.clone(), formula);
            },
            FUNCTION_YEAR => {
                formula = DateParseFunction::do_replace(
                    &self.function_text, DateParseOption::Year, self.data_map.clone(), formula);
            },
            FUNCTION_NOW => {
                formula = NowFunction::do_replace(
                    &self.function_text, formula);
            },
            FUNCTION_TODAY => {
                formula = TodayFunction::do_replace(
                    &self.function_text, formula);
            },
            FUNCTION_DAYS => {
                formula = DaysFunction::do_replace(
                    &self.function_text, self.data_map.clone(), formula);
            },
            FUNCTION_DATEADD => {
                formula = DateAddDiffFunction::do_replace(
                    &self.function_text, DateDeltaOperation::Add, self.data_map.clone(), formula);
            },
            FUNCTION_DATEFMT => {
                formula = DateFormatFunction::do_replace(
                    &self.function_text, self.data_map.clone(), formula);
            },
            FUNCTION_IF => {
                formula = IfFunction::do_replace(
                    &self.function_text, formula, self.data_map.clone(), &self.table.clone());
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
        } else {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("Function \"{}\" does not exist. Check your spelling.", &function_name)),
                )
            );
        }
    }

    // Validate all formula_functions (only ones found in formula from all functions in achiever)
    let mut number_fails: u32 = 0;
    let mut failed_functions: Vec<String> = Vec::new();
    let mut validate_tuple = (number_fails, failed_functions);
    for function_name in function_name_map.keys() {
        let function_name = function_name.as_str();
        eprintln!("validate_formula :: ** function_name: {}", &function_name);
        let function_text = function_name_map.get(function_name).unwrap();
        match function_name {
            FUNCTION_CONCAT => {
                validate_tuple = ConcatenateFunction::do_validate(function_text, validate_tuple);
            },
            FUNCTION_FORMAT => {
                validate_tuple = FormatFunction::do_validate(function_text, validate_tuple);
            },
            FUNCTION_JOINLIST => {
                validate_tuple = JoinListFunction::do_validate(function_text, validate_tuple);
            },
            FUNCTION_LENGTH => {
                validate_tuple = LengthFunction::do_validate(function_text, validate_tuple);
            },
            FUNCTION_LOWER => {
                validate_tuple = LowerFunction::do_validate(function_text, validate_tuple);
            },
            FUNCTION_UPPER => {
                validate_tuple = UpperFunction::do_validate(function_text, validate_tuple);
            },
            FUNCTION_REPLACE => {
                validate_tuple = ReplaceFunction::do_validate(function_text, validate_tuple);
            },
            FUNCTION_MID => {
                validate_tuple = MidFunction::do_validate(function_text, validate_tuple);
            },
            FUNCTION_REPT => {
                validate_tuple = ReptFunction::do_validate(function_text, validate_tuple);
            },
            FUNCTION_SUBSTITUTE => {
                validate_tuple = SubstituteFunction::do_validate(function_text, validate_tuple);
            },
            FUNCTION_TRIM => {
                validate_tuple = TrimFunction::do_validate(function_text, validate_tuple);
            },
            FUNCTION_CEILING => {
                validate_tuple = CeilingFunction::do_validate(function_text, validate_tuple);
            },
            FUNCTION_FLOOR => {
                validate_tuple = FloorFunction::do_validate(function_text, validate_tuple);
            },
            FUNCTION_COUNT => {
                validate_tuple = CountFunction::do_validate(function_text, validate_tuple);
            },
            FUNCTION_COUNTA => {
                validate_tuple = CountAFunction::do_validate(function_text, validate_tuple);
            },
            FUNCTION_COUNTALL => {
                validate_tuple = CountAllFunction::do_validate(function_text, validate_tuple);
            },
            FUNCTION_EVEN => {
                validate_tuple = EvenFunction::do_validate(function_text, validate_tuple);
            },
            FUNCTION_EXP => {
                validate_tuple = ExpFunction::do_validate(function_text, validate_tuple);
            },
            FUNCTION_DATE => {
                validate_tuple = DateFunction::do_validate(function_text, validate_tuple);
            },
            FUNCTION_SECOND => {
                validate_tuple = DateTimeParseFunction::do_validate(function_text, 
                    DateTimeParseOption::Second, validate_tuple);
            },
            FUNCTION_MINUTE => {
                validate_tuple = DateTimeParseFunction::do_validate(function_text, 
                    DateTimeParseOption::Minute, validate_tuple);
            },
            FUNCTION_HOUR => {
                validate_tuple = DateTimeParseFunction::do_validate(function_text, 
                    DateTimeParseOption::Hour, validate_tuple);
            },
            FUNCTION_DAY => {
                validate_tuple = DateParseFunction::do_validate(function_text, DateParseOption::Day, validate_tuple);
            },
            FUNCTION_WEEK => {
                validate_tuple = DateParseFunction::do_validate(function_text, DateParseOption::Week, validate_tuple);
            },
            FUNCTION_WEEKDAY => {
                validate_tuple = DateParseFunction::do_validate(function_text, DateParseOption::WeekDay, validate_tuple);
            },
            FUNCTION_MONTH => {
                validate_tuple = DateParseFunction::do_validate(function_text, DateParseOption::Month, validate_tuple);
            },
            FUNCTION_YEAR => {
                validate_tuple = DateParseFunction::do_validate(function_text, DateParseOption::Year, validate_tuple);
            },
            FUNCTION_NOW => {
                validate_tuple = NowFunction::do_validate(function_text, validate_tuple);
            },
            FUNCTION_TODAY => {
                validate_tuple = TodayFunction::do_validate(function_text, validate_tuple);
            },
            FUNCTION_DAYS => {
                validate_tuple = DaysFunction::do_validate(function_text, validate_tuple);
            },
            FUNCTION_DATEADD => {
                validate_tuple = DateAddDiffFunction::do_validate(function_text, 
                    DateDeltaOperation::Add, validate_tuple);
            },
            FUNCTION_DATEFMT => {
                validate_tuple = DateFormatFunction::do_validate(function_text, validate_tuple);
            },
            FUNCTION_IF => {
                validate_tuple = IfFunction::do_validate(function_text, validate_tuple);
            },
            _ => {
                number_fails += 1;
            }
        }
    }
    number_fails = validate_tuple.0;
    failed_functions = validate_tuple.1;
    eprintln!("validate_formula :: number_fails: {}", &number_fails);
    if number_fails > 0 {
        let failed_functions_str = failed_functions.join(", ");
        return Err(
            PlanetError::new(
                500, 
                Some(tr!("Could not validate formula. Failed functions: {}", &failed_functions_str)),
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
