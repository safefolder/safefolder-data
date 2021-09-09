pub mod constants;
pub mod text;
pub mod date;
pub mod structure;
pub mod number;
pub mod collections;

use std::collections::HashMap;
use colored::Colorize;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use regex::Regex;
use tr::tr;
use xlformula_engine::{calculate, parse_formula, NoReference, NoCustomFunction};

use crate::storage::table::{DbData, DbTable};
use crate::functions::constants::*;
use crate::functions::text::*;
use crate::functions::date::*;
use crate::functions::number::*;
use crate::functions::collections::*;
use crate::functions::structure::*;
use crate::planet::PlanetError;
use crate::storage::constants::*;

lazy_static! {
    static ref RE_ACHIEVER_FUNCTIONS: Regex = Regex::new(r#"(?P<func>[A-Z]+\(.+\))|(?P<func_empty>[A-Z]+\(\))"#).unwrap();
    static ref RE_ACHIEVER_FUNCTIONS_PARTS : Regex = Regex::new(r#"(?P<func>[A-Z]+\([\w\s\d"\-\+:,\{\}.€\$=;]+\))|(?P<func_empty>[A-Z]+\(\))"#).unwrap();
    static ref RE_FORMULA_VALID: Regex = Regex::new(r#"(?im:\{[\w\s]+\})"#).unwrap();
    static ref RE_EMBED_FUNC: Regex = Regex::new(r#"\((?P<func_embed>[A-Z]+)"#).unwrap();
}

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

pub trait Function {
    fn validate(&self) -> bool;
    fn replace(&mut self, formula: String) -> String;
}

lazy_static! {
    // CONCAT("mine", "-", {My Column}, 45) :: Regex to catch the function attributes in an array
    static ref RE_FORMULA_FUNCTIONS: Regex = Regex::new(r#"([a-zA-Z]+\(.+\))"#).unwrap();
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionsHanler {
    pub function_name: String,
    pub function_text: String,
    pub data_map: Option<HashMap<String, String>>,
    pub table: Option<DbData>,
}
impl FunctionsHanler{
    pub fn do_functions(&self, mut formula: String) -> String {
        let function_name = self.function_name.as_str();
        // Match all achiever functions here. Used by insert and update data to process formula columns.
        let data_map: HashMap<String, String>;
        let data_map_wrap = self.data_map.clone();
        let table_wrap = self.table.clone();
        if data_map_wrap.is_none() {
            data_map = HashMap::new();
        } else {
            data_map = data_map_wrap.unwrap();
        }
        match function_name {
            FUNCTION_CONCAT => {
                formula = ConcatenateFunction::do_replace(
                    &self.function_text, data_map.clone(), formula);
            },
            FUNCTION_FORMAT => {
                formula = FormatFunction::do_replace(
                    &self.function_text, data_map.clone(), formula);
            },
            FUNCTION_JOINLIST => {
                formula = JoinListFunction::do_replace(
                    &self.function_text, data_map.clone(), formula);
            },
            FUNCTION_LENGTH => {
                formula = LengthFunction::do_replace(
                    &self.function_text, data_map.clone(), formula);
            },
            FUNCTION_LOWER => {
                formula = LowerFunction::do_replace(
                    &self.function_text, data_map.clone(), formula);
            },
            FUNCTION_UPPER => {
                formula = UpperFunction::do_replace(
                    &self.function_text, data_map.clone(), formula);
            },
            FUNCTION_REPLACE => {
                formula = ReplaceFunction::do_replace(
                    &self.function_text, data_map.clone(), formula);
            },
            FUNCTION_MID => {
                formula = MidFunction::do_replace(
                    &self.function_text, data_map.clone(), formula);
            },
            FUNCTION_REPT => {
                formula = ReptFunction::do_replace(
                    &self.function_text, data_map.clone(), formula);
            },
            FUNCTION_SUBSTITUTE => {
                formula = SubstituteFunction::do_replace(
                    &self.function_text, data_map.clone(), formula);
            },
            FUNCTION_TRIM => {
                formula = TrimFunction::do_replace(
                    &self.function_text, data_map.clone(), formula);
            },
            FUNCTION_CEILING => {
                formula = CeilingFunction::do_replace(
                    &self.function_text, data_map.clone(), formula);
            },
            FUNCTION_FLOOR => {
                formula = FloorFunction::do_replace(
                    &self.function_text, data_map.clone(), formula);
            },
            FUNCTION_COUNT => {
                formula = CountFunction::do_replace(
                    &self.function_text, data_map.clone(), formula);
            },
            FUNCTION_COUNTA => {
                formula = CountAFunction::do_replace(
                    &self.function_text, data_map.clone(), formula);
            },
            FUNCTION_COUNTALL => {
                formula = CountAllFunction::do_replace(
                    &self.function_text, formula);
            },
            FUNCTION_EVEN => {
                formula = EvenFunction::do_replace(
                    &self.function_text, data_map.clone(), formula);
            },
            FUNCTION_EXP => {
                formula = ExpFunction::do_replace(
                    &self.function_text, data_map.clone(), formula);
            },
            FUNCTION_INT => {
                formula = IntFunction::do_replace(
                    &self.function_text, data_map.clone(), formula);
            },
            FUNCTION_LOG => {
                formula = LogFunction::do_replace(
                    &self.function_text, data_map.clone(), formula);
            },
            FUNCTION_MOD => {
                formula = ModFunction::do_replace(
                    &self.function_text, data_map.clone(), formula);
            },
            FUNCTION_POWER => {
                formula = PowerFunction::do_replace(
                    &self.function_text, data_map.clone(), formula);
            },
            FUNCTION_ROUND => {
                formula = RoundFunction::do_replace(
                    &self.function_text, data_map.clone(), formula, RoundOption::Basic);
            },
            FUNCTION_ROUNDUP => {
                formula = RoundFunction::do_replace(
                    &self.function_text, data_map.clone(), formula, RoundOption::Up);
            },
            FUNCTION_ROUNDDOWN => {
                formula = RoundFunction::do_replace(
                    &self.function_text, data_map.clone(), formula, RoundOption::Down);
            },
            FUNCTION_SQRT => {
                formula = SqrtFunction::do_replace(
                    &self.function_text, data_map.clone(), formula);
            },
            FUNCTION_VALUE => {
                formula = ValueFunction::do_replace(
                    &self.function_text, data_map.clone(), formula);
            },
            FUNCTION_TRUE => {
                formula = BooleanFunction::do_replace(
                    &self.function_text, formula, BooleanOption::True);
            },
            FUNCTION_FALSE => {
                formula = BooleanFunction::do_replace(
                    &self.function_text, formula, BooleanOption::False);
            },
            FUNCTION_DATE => {
                formula = DateFunction::do_replace(
                    &self.function_text, formula);
            },
            FUNCTION_SECOND => {
                formula = DateTimeParseFunction::do_replace(
                    &self.function_text, DateTimeParseOption::Second, data_map.clone(), formula);
            },
            FUNCTION_MINUTE => {
                formula = DateTimeParseFunction::do_replace(
                    &self.function_text, DateTimeParseOption::Minute, data_map.clone(), formula);
            },
            FUNCTION_HOUR => {
                formula = DateTimeParseFunction::do_replace(
                    &self.function_text, DateTimeParseOption::Hour, data_map.clone(), formula);
            },
            FUNCTION_DAY => {
                formula = DateParseFunction::do_replace(
                    &self.function_text, DateParseOption::Day, data_map.clone(), formula);
            },
            FUNCTION_WEEK => {
                formula = DateParseFunction::do_replace(
                    &self.function_text, DateParseOption::Week, data_map.clone(), formula);
            },
            FUNCTION_WEEKDAY => {
                formula = DateParseFunction::do_replace(
                    &self.function_text, DateParseOption::WeekDay, data_map.clone(), formula);
            },
            FUNCTION_MONTH => {
                formula = DateParseFunction::do_replace(
                    &self.function_text, DateParseOption::Month, data_map.clone(), formula);
            },
            FUNCTION_YEAR => {
                formula = DateParseFunction::do_replace(
                    &self.function_text, DateParseOption::Year, data_map.clone(), formula);
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
                    &self.function_text, data_map.clone(), formula);
            },
            FUNCTION_DATEADD => {
                formula = DateAddDiffFunction::do_replace(
                    &self.function_text, DateDeltaOperation::Add, data_map.clone(), formula);
            },
            FUNCTION_DATEFMT => {
                formula = DateFormatFunction::do_replace(
                    &self.function_text, data_map.clone(), formula);
            },
            FUNCTION_IF => {
                if table_wrap.is_some() {
                    let table = table_wrap.unwrap().clone();
                    formula = IfFunction::do_replace(
                        &self.function_text, formula, data_map.clone(), &table);
                }
            },
            FUNCTION_MIN => {
                formula = StatsFunction::do_replace(
                    &self.function_text, data_map.clone(), formula, StatOption::Min);
            },
            FUNCTION_MAX => {
                formula = StatsFunction::do_replace(
                    &self.function_text, data_map.clone(), formula, StatOption::Max);
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
            FUNCTION_INT => {
                validate_tuple = IntFunction::do_validate(function_text, validate_tuple);
            },
            FUNCTION_LOG => {
                validate_tuple = LogFunction::do_validate(function_text, validate_tuple);
            },
            FUNCTION_MOD => {
                validate_tuple = ModFunction::do_validate(function_text, validate_tuple);
            },
            FUNCTION_POWER => {
                validate_tuple = PowerFunction::do_validate(function_text, validate_tuple);
            },
            FUNCTION_ROUND => {
                validate_tuple = RoundFunction::do_validate(function_text, validate_tuple, RoundOption::Basic);
            },
            FUNCTION_ROUNDUP => {
                validate_tuple = RoundFunction::do_validate(function_text, validate_tuple, RoundOption::Up);
            },
            FUNCTION_ROUNDDOWN => {
                validate_tuple = RoundFunction::do_validate(function_text, validate_tuple, RoundOption::Down);
            },
            FUNCTION_SQRT => {
                validate_tuple = SqrtFunction::do_validate(function_text, validate_tuple);
            },
            FUNCTION_VALUE => {
                validate_tuple = ValueFunction::do_validate(function_text, validate_tuple);
            },
            FUNCTION_TRUE => {
                validate_tuple = BooleanFunction::do_validate(function_text, validate_tuple, BooleanOption::True);
            },
            FUNCTION_FALSE => {
                validate_tuple = BooleanFunction::do_validate(function_text, validate_tuple, BooleanOption::True);
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
            FUNCTION_MIN => {
                validate_tuple = StatsFunction::do_validate(function_text, validate_tuple, StatOption::Min);
            },
            FUNCTION_MAX => {
                validate_tuple = StatsFunction::do_validate(function_text, validate_tuple, StatOption::Max);
            },
            _ => {
                number_fails += 1;
            }
        }
    }
    number_fails = validate_tuple.0;
    failed_functions = validate_tuple.1;
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


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Formula {
    item_data: Option<HashMap<String, String>>, 
    table: Option<DbData>,
}
impl Formula{
    pub fn defaults(
        item_data: Option<HashMap<String, String>>, 
        table: Option<DbData>
    ) -> Formula {
        let obj = Self{
            item_data: item_data,
            table: table,
        };

        // eprintln!("Formula.defaults :: obj: {:#?}", &obj);

        return obj
    }
    pub fn validate_data(&self, formula: &String) -> Result<bool, PlanetError> {
        let expr = RE_FORMULA_VALID.clone();
        let mut check = true;
        let data_map = self.item_data.clone().unwrap();
        for capture in expr.captures_iter(formula.as_str()) {
            let field_ref = capture.get(0).unwrap().as_str();
            let field_ref = field_ref.replace("{", "").replace("}", "");
            let field_ref_value = data_map.get(&field_ref);
            if field_ref_value.is_none() {
                check = false;
                break;
            }
        }
        return Ok(check)
    }
    pub fn inyect_data_formula(&self, formula: &String) -> Option<String> {
        let table_wrap = self.table.clone();
        let data_map_wrap = self.item_data.clone();
        let mut formula = formula.clone();
        let formula_str = formula.clone();
        let formula_str = formula_str.as_str();
        if table_wrap.is_some() && data_map_wrap.is_some() {
            let table = table_wrap.unwrap();
            let data_map = data_map_wrap.unwrap();
            // This replaces the column data with its value and return the formula to be processed
            let field_type_map = DbTable::get_field_type_map(&table);
            if field_type_map.is_ok() {
                let field_type_map = field_type_map.unwrap();
                let expr = RE_FORMULA_VALID.clone();
                // let mut formula = formula.clone();
                for capture in expr.captures_iter(formula_str) {
                    let field_ref = capture.get(0).unwrap().as_str();
                    let field_ref = field_ref.replace("{", "").replace("}", "");
                    let field_ref_value = data_map.get(&field_ref);
                    if field_ref_value.is_some() {
                        let field_ref_value = field_ref_value.unwrap();
                        // Check is we have string field_type or not string one
                        let field_type = field_type_map.get(&field_ref.to_string());
                        if field_type.is_some() {
                            let field_type = field_type.unwrap().clone();
                            let replace_string: String;
                            match field_type.as_str() {
                                FIELD_SMALL_TEXT => {
                                    replace_string = format!("\"{}\"", &field_ref_value);
                                },
                                FIELD_LONG_TEXT => {
                                    replace_string = format!("\"{}\"", &field_ref_value);
                                },
                                _ => {
                                    replace_string = field_ref_value.clone();
                                }
                            }
                            let field_to_replace = format!("{}{}{}", 
                                String::from("{"),
                                &field_ref,
                                String::from("}"),
                            );
                            formula = formula.replace(&field_to_replace, &replace_string);
                        }
                    }
                }
                return Some(formula);
            }
            return None;
        } else {
            return None;
        }
    }
    pub fn execute(self, formula: &String) -> Result<String, PlanetError> {
        // First process the achiever functions, then rest
        let expr = RE_ACHIEVER_FUNCTIONS_PARTS.clone();
        let formula_str = formula.as_str();
        let mut formula = formula.clone();
        let data_map = self.item_data.clone();
        let table = self.table.clone();
        let has_matches = expr.captures(&formula_str);
        let mut sequence: u8 = 1;
        let max_counter = 20;
        while has_matches.is_some() {
            eprintln!("Formula.execute :: [inside] has_matches: {}", has_matches.is_some());
            let mut only_not_achive_functions = false;
            let mut not_achiever_counter = 0;
            for capture in expr.captures_iter(formula_str) {
                // AND, OR, NOT, XOR function names would be skipped, since handled by Lib
                let function_text = capture.get(0).unwrap().as_str();
                eprintln!("Formula.execute :: REGEX text item: {}", &function_text);
                let function_text_string = function_text.to_string();
                let check_achiever = check_achiever_function(function_text_string.clone());
                eprintln!("Formula.execute :: check_achiever: {}", &check_achiever);
                if check_achiever == true {
                    let function_name = get_function_name(function_text_string.clone());
                    eprintln!("Formula.execute :: function_name: {}", &function_name);
                    let handler = FunctionsHanler{
                        function_name: function_name.clone(),
                        function_text: function_text_string,
                        data_map: data_map.clone(),
                        table: table.clone(),
                    };
                    formula = handler.do_functions(formula.clone());
                } else {
                    eprintln!("Formula.execute :: not achieve func: {}", &function_text);
                    not_achiever_counter += 1;
                }
            }
            // formula_str = formula.as_str().clone();
            if not_achiever_counter == 1 {
                only_not_achive_functions = true;
            }
            let has_matches = expr.captures(&formula.as_str());
            eprintln!("Formula.execute :: [seq.{sequence}] formula:{formula} has_matches: {matches}", 
                sequence=&sequence, formula=&formula, matches=&has_matches.is_some());
            sequence += 1;
            if only_not_achive_functions == true {
                break
            }
            if sequence > max_counter {
                break
            }
            if has_matches.is_none() {
                break
            }
        }
        let formula_str_ = formula.as_str();
        // Check function again, for case of IF with non achieve functions inside like AND, OR, etc...
        let expr = RE_ACHIEVER_FUNCTIONS.clone();
        let has_matches = expr.captures(&formula_str_);
        if has_matches.is_some() {
            // IF formula with AND, OR, NOT, etc... global functions from Lib used
            for capture in expr.captures_iter(formula_str) {
                let function_text = capture.get(0).unwrap().as_str();
                let function_text_string = function_text.to_string();
                let check_achiever = check_achiever_function(function_text_string.clone());
                if check_achiever == true {
                    let function_name = get_function_name(function_text_string.clone());
                    let handler = FunctionsHanler{
                        function_name: function_name.clone(),
                        function_text: function_text_string,
                        data_map: data_map.clone(),
                        table: table.clone(),
                    };
                    formula = handler.do_functions(formula.clone());
                }
            }
        }
        // This injects references without achiever functions
        if data_map.is_some() && table.is_some() {
            let formula_wrap = self.inyect_data_formula(&formula);
            if formula_wrap.is_some() {
                formula = formula_wrap.unwrap();
                eprintln!("Formula.execute :: [LIB] formula: {:#?}", &formula);
                formula = format!("={}", &formula);
                let formula_ = parse_formula::parse_string_to_formula(
                    &formula, 
                    None::<NoCustomFunction>
                );
                let result = calculate::calculate_formula(formula_, None::<NoReference>);
                let result = calculate::result_to_string(result);
                eprintln!("Formula.execute :: [LIB] result: **{}**", &result.blue());
                return Ok(result);
            } else {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!("Formula could not execute due to bad format in the references like {My Reference} or
                        data problem on the target data associated with the reference.")),
                    )
                );
            }
        } else {
            // eprintln!("Formula.execute :: no references since no data received");
            // I have no data and table information, skip references, simply execute formula I have
            let formula_ = parse_formula::parse_string_to_formula(
                &formula, 
                None::<NoCustomFunction>
            );
            let result = calculate::calculate_formula(formula_, None::<NoReference>);
            let result = calculate::result_to_string(result);
            // eprintln!("Formula.execute :: result: {:#?}", &result);
            return Ok(result);
        }        
    }
}
