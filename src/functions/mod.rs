pub mod constants;
pub mod text;
pub mod date;
pub mod structure;
pub mod number;
pub mod collections;

use std::time::Instant;
use std::collections::HashMap;
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
    static ref RE_FORMULA_FUNCTIONS: Regex = Regex::new(r#"([a-zA-Z]+\(.+\))"#).unwrap();
    static ref RE_ACHIEVER_FUNCTIONS: Regex = Regex::new(r#"(?P<func>[A-Z]+\(.+\))|(?P<func_empty>[A-Z]+\(\))"#).unwrap();
    static ref RE_ACHIEVER_FUNCTIONS_PARTS : Regex = Regex::new(r#"(?P<func>[A-Z]+\([\w\s\d"\-\+:,\{\}.€\$=;]+\))|(?P<func_empty>[A-Z]+\(\))"#).unwrap();
    static ref RE_FORMULA_VALID: Regex = Regex::new(r#"(?im:\{[\w\s]+\})"#).unwrap();
    static ref RE_EMBED_FUNC: Regex = Regex::new(r#"\((?P<func_embed>[A-Z]+)"#).unwrap();
    static ref RE_STRING_MATCH: Regex = Regex::new(r#"(?P<string_match>"[\w\s]+"[\s\n\t]{0,}[=><][\s\n\t]{0,}"[\w\s]+")"#).unwrap();
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
        eprintln!("FunctionsHandler.do_functions :: data_map: {:#?}", &data_map);
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

pub fn validate_formula(formula: &String, formula_format: &String) -> Result<bool, PlanetError> {
    let check = true;
    let formula_obj = Formula::defaults(None, None);
    let function_text_list = formula_obj.validate(&formula, &formula_format)?;

    // Validate all formula_functions (only ones found in formula from all functions in achiever)
    let mut number_fails: u32 = 0;
    let mut failed_functions: Vec<String> = Vec::new();
    let mut validate_tuple = (number_fails, failed_functions);
    for function_text in &function_text_list {
        let parts: Vec<&str> = function_text.split("(").collect();
        let function_name = parts[0];
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
pub enum FormulaProcessMode {
    Validate,
    Execute
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
    fn process_formula_str(
        &self, 
        expr: &Regex, 
        formula_string: String, 
        mut function_text_list: Vec<String>,
        replaced_text: Option<&str>,
        mode: FormulaProcessMode,
    ) -> Result<(String, Vec<String>, bool, bool), PlanetError> {
        let data_map = self.item_data.clone();
        let table = self.table.clone();
        let mut achiever_functions: Vec<String> = Vec::new();
        let formula_str = formula_string.as_str();
        let replaced_text = replaced_text.unwrap_or_default();
        for function_name_item in FORMULA_FUNCTIONS {
            achiever_functions.push(function_name_item.to_string());
        }
        let mut formula_new_string = formula_str.clone().to_string();
        let mut only_not_achive_functions = false;
        let mut not_achiever_counter = 0;
        // let t_1 = Instant::now();
        let capture_list = expr.captures_iter(formula_str);
        // eprintln!("Formula.process_formula_str :: capture_list : {} ms", &t_1.elapsed().as_millis());
        for capture in capture_list {
            let function_text = capture.get(0).unwrap().as_str();
            let function_text_string = function_text.to_string();
            let check_achiever = check_achiever_function(function_text_string.clone());
            if check_achiever == true {
                let function_name = get_function_name(function_text_string.clone());
                if achiever_functions.contains(&function_name) == true {
                    
                    match mode {
                        FormulaProcessMode::Validate => {
                            function_text_list.push(function_text_string);
                            formula_new_string = formula_new_string.replace(function_text, replaced_text);
                        },
                        FormulaProcessMode::Execute => {
                            let handler = FunctionsHanler{
                                function_name: function_name.clone(),
                                function_text: function_text_string,
                                data_map: data_map.clone(),
                                table: table.clone(),
                            };
                            formula_new_string = handler.do_functions(formula_new_string.clone());
                        },
                    }
                } else {
                    return Err(
                        PlanetError::new(
                            500, 
                            Some(tr!("Function \"{}\" does not exist. Check your spelling.", &function_name)),
                        )
                    );
                }
            } else {
                not_achiever_counter += 1;
            }
        }
        let has_matches = expr.clone().captures(formula_new_string.as_str()).is_some();
        if not_achiever_counter == 1 {
            only_not_achive_functions = true;
        }
        return Ok(
            (
                formula_new_string,
                function_text_list,
                has_matches,
                only_not_achive_functions,
            )
        )
    }
    pub fn validate(&self, formula: &String, formula_format: &String) -> Result<Vec<String>, PlanetError> {
        let expr = &RE_ACHIEVER_FUNCTIONS_PARTS;
        let formula = formula.clone();
        let formula_str = formula.as_str();
        let mut has_matches = expr.captures(formula_str).is_some();
        let mut sequence: u8 = 1;
        let max_counter = 20;
        let mut achiever_functions: Vec<String> = Vec::new();
        for function_name_item in FORMULA_FUNCTIONS {
            achiever_functions.push(function_name_item.to_string());
        }
        let mut function_text_list: Vec<String> = Vec::new();
        let formula_format_str = formula_format.as_str();
        let replaced_text: &str;
        match formula_format_str {
            FORMULA_FORMAT_TEXT => {
                replaced_text = "\"\"";
            },
            FORMULA_FORMAT_NUMBER => {
                replaced_text = "1"
            },
            FORMULA_FORMAT_DATE => {
                replaced_text = "01-Jan-2021"
            },
            _ => {
                replaced_text = "\"\"";
            },
        }
        let mut formula_string: String = formula_str.to_string();
        let mut only_not_achive_functions: bool;
        while has_matches == true {
            let tuple = self.process_formula_str(
                expr, 
                formula_string.clone(), 
                function_text_list.clone(), 
                Some(replaced_text),
                FormulaProcessMode::Validate,
            )?;
            formula_string = tuple.0;
            function_text_list = tuple.1;
            has_matches = tuple.2;
            only_not_achive_functions = tuple.3;
            sequence += 1;
            if only_not_achive_functions == true {
                break
            }
            if sequence > max_counter {
                break
            }
            if has_matches == false {
                break
            }
        }
        let formula_str_ = formula_string.as_str();
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
                    function_text_list.push(function_text_string);
                }
            }
        }
        return Ok(function_text_list)
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
    fn process_string_matches(&self, formula: &String) -> String {
        // let t_1 = Instant::now();
        let formula = formula.clone();
        let formula_str = formula.as_str();
        let expr = &RE_STRING_MATCH;
        let mut formula_new = formula.clone();
        // let t_capture_list = Instant::now();
        let list_captures = expr.captures_iter(formula_str);
        // eprintln!("Formula.process_string_matches :: perf : capture_list: {} ms", &t_capture_list.elapsed().as_millis());
        // eprintln!("Formula.process_string_matches :: perf : header: {} ms", &t_1.elapsed().as_millis());
        for capture in list_captures {
            // let t_item_1 = Instant::now();
            let item = capture.get(0).unwrap().as_str();
            // let equal_greater = item.find("=>");
            // let smaller_equal = item.find("<=");
            let equal = item.find("=");
            if equal.is_some() {
                let fields: Vec<&str> = item.split("=").collect();
                let name = fields[0].trim();
                let value = fields[1].trim();
                if name != value {
                    formula_new = formula_new.replace(item, "1=2");
                } else {
                    formula_new = formula_new.replace(item, "1=1");
                }
            }
            // eprintln!("Formula.process_string_matches :: perf : item {}: {} ms", &item, &t_item_1.elapsed().as_millis());
        }
        // eprintln!("Formula.process_string_matches :: perf : total: {} ms", &t_1.elapsed().as_millis());
        return formula_new
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
                            // eprintln!("Formula.inyect_data_formula :: field_type: {}", &field_type);
                            let replace_string: String;
                            // field formula????
                            // TODO: Add fields as you implement them
                            match field_type.as_str() {
                                FIELD_SMALL_TEXT => {
                                    replace_string = format!("\"{}\"", &field_ref_value);
                                },
                                FIELD_LONG_TEXT => {
                                    replace_string = format!("\"{}\"", &field_ref_value);
                                },
                                FIELD_SELECT => {
                                    replace_string = format!("\"{}\"", &field_ref_value);
                                },
                                FIELD_NUMBER => {
                                    replace_string = format!("{}", &field_ref_value);
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
        let t_1 = Instant::now();
        let expr = &RE_ACHIEVER_FUNCTIONS_PARTS;
        let formula_str = formula.as_str();
        let mut formula_string = formula.clone();
        let data_map = self.item_data.clone();
        let table = self.table.clone();
        let mut has_matches = expr.captures(&formula_str).is_some();
        let mut sequence: u8 = 1;
        let max_counter = 20;
        let mut only_not_achive_functions;
        let mut function_text_list: Vec<String> = Vec::new();
        // eprintln!("Formula.execute :: achiever functions?: {}", &has_matches);
        eprintln!("Formula.execute :: perf : header: {} µs", &t_1.elapsed().as_micros());
        let t_process_formula_1 = Instant::now();
        while has_matches == true {
            let tuple = self.process_formula_str(
                expr, 
                formula_string.clone(), 
                function_text_list.clone(), 
                None,
                FormulaProcessMode::Execute,
            )?;
            formula_string = tuple.0;
            function_text_list = tuple.1;
            has_matches = tuple.2;
            only_not_achive_functions = tuple.3;

            sequence += 1;
            if only_not_achive_functions == true {
                break
            }
            if sequence > max_counter {
                break
            }
            if has_matches == false {
                break
            }
        }
        eprintln!("Formula.execute :: perf : process formula: {} µs", &t_process_formula_1.elapsed().as_micros());
        let t_process_formula_2 = Instant::now();
        let formula_str_ = formula_string.as_str();
        // Check function again, for case of IF with non achieve functions inside like AND, OR, etc...
        let expr = &RE_ACHIEVER_FUNCTIONS;
        let has_matches = expr.captures(&formula_str_).is_some();
        // eprintln!("Formula.execute :: [2] achiever functions?: {}", &has_matches);
        if has_matches == true {
            let tuple = self.process_formula_str(
                expr, 
                formula_string.clone(), 
                function_text_list.clone(), 
                None,
                FormulaProcessMode::Execute,
            )?;
            formula_string = tuple.0;
        }
        eprintln!("Formula.execute :: perf : process formula (2): {} µs", &t_process_formula_2.elapsed().as_micros());
        // This injects references without achiever functions
        if data_map.is_some() && table.is_some() {
            let t_inyect_1 = Instant::now();
            // eprintln!("Formula.execute :: formula before inyect: *{}*", &formula_string);
            let formula_wrap = self.inyect_data_formula(&formula_string);
            // eprintln!("Formula.execute :: formula after inyect: *{:?}*", &formula_wrap);
            eprintln!("Formula.execute :: perf : inyect formula: {} µs", &t_inyect_1.elapsed().as_micros());
            if formula_wrap.is_some() {
                formula_string = formula_wrap.unwrap();
                let t_string_1 = Instant::now();
                formula_string = self.process_string_matches(&formula_string);
                eprintln!("Formula.execute :: perf : string matches: {} µs", &t_string_1.elapsed().as_micros());
                // eprintln!("Formula.execute :: formula_new: {}", &formula_string);
                formula_string = format!("={}", &formula_string);
                let t_exec_1 = Instant::now();
                let formula_ = parse_formula::parse_string_to_formula(
                    &formula_string, 
                    None::<NoCustomFunction>
                );
                // eprintln!("Formula.execute :: formula_: {:?}", &formula_);
                let result = calculate::calculate_formula(formula_, None::<NoReference>);
                // eprintln!("Formula.execute :: calcuated formula_: {:?}", &result);
                let result = calculate::result_to_string(result);
                eprintln!("Formula.execute :: perf : exec: {} µs", &t_exec_1.elapsed().as_micros());
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
            // I have no data and table information, skip references, simply execute formula I have
            let formula_ = parse_formula::parse_string_to_formula(
                &formula, 
                None::<NoCustomFunction>
            );
            let result = calculate::calculate_formula(formula_, None::<NoReference>);
            let result = calculate::result_to_string(result);
            return Ok(result);
        }        
    }
}
