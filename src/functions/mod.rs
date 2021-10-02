pub mod constants;
pub mod text;
pub mod date;
pub mod structure;
pub mod number;
pub mod collections;

// use std::time::Instant;
use std::str::FromStr;
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
    static ref RE_FORMULA_QUERY: Regex = Regex::new(r#"(?P<assign>\{[\s\w]+\}[\s\t]{0,}(?P<log_op>=|>|<|>=|<=)[\s\t]{0,}.+)|(?P<op>AND|OR|NOT|XOR)\((?P<attrs>.+)\)"#).unwrap();
    static ref RE_FORMULA_FIELD_FUNCTIONS: Regex = Regex::new(r#"(?P<func>[A-Z]+[("\d)\w,.\s{}-]+)"#).unwrap();
    static ref RE_FUNCTION_ATTRS_OLD: Regex = Regex::new(r#"("[\w\s-]+")|(\{[\w\s]+\})|([A-Z]+\(["\w\s]+\))|([+-]?[0-9]+\.?[0-9]*|\.[0-9]+)"#).unwrap();
    static ref RE_FUNCTION_ATTRS: Regex = Regex::new(r#"[A-Z]+\((?P<attrs>.+)\)"#).unwrap();
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
        // eprintln!("FunctionsHandler.do_functions :: data_map: {:#?}", &data_map);
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

// pub fn validate_formula(
//     formula: &String,
//     formula_format: &String,
// ) -> Result<bool, PlanetError> {
//     let check = true;
//     // let formula_obj = Formula::defaults(None, None);
//     // let function_text_list = formula_obj.validate(&formula, &formula_format)?;

//     // 1. Get all functions in the formula
//     // (23 + EXP(32) + EXP({Column A}) + ABS(EXP(98))) / 2
//     // => 
//     // (23 + $func1 + $func2 + $func3) / 2

//     // I user CompiledFormulaField with functions and formula itself
//     // This would give me a compiled version of any formula field which is fast to process
//     // Then I need to validate specific each function. For this to work, 
//     // I need a function or mode to return all functions inside function, so I have 

//     // ABS(EXP(98))
//     // ABS function should validate ABS and EXP, right? Since it is what it is being sent.

//     // What we do right now, is validate EXP(98), then ABS({value from EXP(98)})
//     // I can have get_all_functions in a FormulaFieldCompiled
//     // It will give me all the functions inside functions, having the function_text, which 
//     // I can send to the regex bellow. I don't need to execute, only to validate.

//     // let expr = &RE_FORMULA_FIELD_FUNCTIONS;
//     // let functions = expr.captures_iter(formula);
//     // let mut count = 1;
//     // for function in functions {
//     //     let function_holder = format!("$func{}", &count);
//     //     eprintln!("validate_formula :: function_holder: {}", &function_holder);
//     //     count += 1;
//     // }

//     // compile formula field
//     // It will compile formula and validate all functions referenced in the formula. Will raise error
//     // in case of validation problem.
//     // FormulaFieldCompiled::defaults(
//     //     formula, 
//     //     formula_format,
//     // )?;

//     // Validate all formula_functions (only ones found in formula from all functions in achiever)
//     // let mut number_fails: u32 = 0;
//     // let mut failed_functions: Vec<String> = Vec::new();
//     // let mut validate_tuple = (number_fails, failed_functions);
//     // let function_text_list: Vec<String> = Vec::new();
//     // for function_text in &function_text_list {
//     //     let parts: Vec<&str> = function_text.split("(").collect();
//     //     let function_name = parts[0];
//     //     match function_name {
//     //         // FUNCTION_CONCAT => {
//     //         //     validate_tuple = function_validate(
//     //         //         function_text, validate_tuple, &RE_CONCAT_ATTRS, FUNCTION_CONCAT
//     //         //     );
//     //         // },
//     //         // FUNCTION_FORMAT => {
//     //         //     validate_tuple = FormatFunction::do_validate(function_text, validate_tuple);
//     //         // },
//     //         // FUNCTION_JOINLIST => {
//     //         //     validate_tuple = JoinListFunction::do_validate(function_text, validate_tuple);
//     //         // },
//     //         // FUNCTION_LENGTH => {
//     //         //     validate_tuple = LengthFunction::do_validate(function_text, validate_tuple);
//     //         // },
//     //         // FUNCTION_LOWER => {
//     //         //     validate_tuple = LowerFunction::do_validate(function_text, validate_tuple);
//     //         // },
//     //         // FUNCTION_UPPER => {
//     //         //     validate_tuple = UpperFunction::do_validate(function_text, validate_tuple);
//     //         // },
//     //         // FUNCTION_REPLACE => {
//     //         //     validate_tuple = ReplaceFunction::do_validate(function_text, validate_tuple);
//     //         // },
//     //         // FUNCTION_MID => {
//     //         //     validate_tuple = MidFunction::do_validate(function_text, validate_tuple);
//     //         // },
//     //         // FUNCTION_REPT => {
//     //         //     validate_tuple = ReptFunction::do_validate(function_text, validate_tuple);
//     //         // },
//     //         // FUNCTION_SUBSTITUTE => {
//     //         //     validate_tuple = SubstituteFunction::do_validate(function_text, validate_tuple);
//     //         // },
//     //         // FUNCTION_TRIM => {
//     //         //     validate_tuple = TrimFunction::do_validate(function_text, validate_tuple);
//     //         // },
//     //         // FUNCTION_CEILING => {
//     //         //     validate_tuple = CeilingFunction::do_validate(function_text, validate_tuple);
//     //         // },
//     //         // FUNCTION_FLOOR => {
//     //         //     validate_tuple = FloorFunction::do_validate(function_text, validate_tuple);
//     //         // },
//     //         // FUNCTION_COUNT => {
//     //         //     validate_tuple = CountFunction::do_validate(function_text, validate_tuple);
//     //         // },
//     //         // FUNCTION_COUNTA => {
//     //         //     validate_tuple = CountAFunction::do_validate(function_text, validate_tuple);
//     //         // },
//     //         // FUNCTION_COUNTALL => {
//     //         //     validate_tuple = CountAllFunction::do_validate(function_text, validate_tuple);
//     //         // },
//     //         // FUNCTION_EVEN => {
//     //         //     validate_tuple = EvenFunction::do_validate(function_text, validate_tuple);
//     //         // },
//     //         // FUNCTION_EXP => {
//     //         //     validate_tuple = ExpFunction::do_validate(function_text, validate_tuple);
//     //         // },
//     //         // FUNCTION_INT => {
//     //         //     validate_tuple = IntFunction::do_validate(function_text, validate_tuple);
//     //         // },
//     //         // FUNCTION_LOG => {
//     //         //     validate_tuple = LogFunction::do_validate(function_text, validate_tuple);
//     //         // },
//     //         // FUNCTION_MOD => {
//     //         //     validate_tuple = ModFunction::do_validate(function_text, validate_tuple);
//     //         // },
//     //         // FUNCTION_POWER => {
//     //         //     validate_tuple = PowerFunction::do_validate(function_text, validate_tuple);
//     //         // },
//     //         // FUNCTION_ROUND => {
//     //         //     validate_tuple = RoundFunction::do_validate(function_text, validate_tuple, RoundOption::Basic);
//     //         // },
//     //         // FUNCTION_ROUNDUP => {
//     //         //     validate_tuple = RoundFunction::do_validate(function_text, validate_tuple, RoundOption::Up);
//     //         // },
//     //         // FUNCTION_ROUNDDOWN => {
//     //         //     validate_tuple = RoundFunction::do_validate(function_text, validate_tuple, RoundOption::Down);
//     //         // },
//     //         // FUNCTION_SQRT => {
//     //         //     validate_tuple = SqrtFunction::do_validate(function_text, validate_tuple);
//     //         // },
//     //         // FUNCTION_VALUE => {
//     //         //     validate_tuple = ValueFunction::do_validate(function_text, validate_tuple);
//     //         // },
//     //         // FUNCTION_TRUE => {
//     //         //     validate_tuple = BooleanFunction::do_validate(function_text, validate_tuple, BooleanOption::True);
//     //         // },
//     //         // FUNCTION_FALSE => {
//     //         //     validate_tuple = BooleanFunction::do_validate(function_text, validate_tuple, BooleanOption::True);
//     //         // },
//     //         // FUNCTION_DATE => {
//     //         //     validate_tuple = DateFunction::do_validate(function_text, validate_tuple);
//     //         // },
//     //         // FUNCTION_SECOND => {
//     //         //     validate_tuple = DateTimeParseFunction::do_validate(function_text, 
//     //         //         DateTimeParseOption::Second, validate_tuple);
//     //         // },
//     //         // FUNCTION_MINUTE => {
//     //         //     validate_tuple = DateTimeParseFunction::do_validate(function_text, 
//     //         //         DateTimeParseOption::Minute, validate_tuple);
//     //         // },
//     //         // FUNCTION_HOUR => {
//     //         //     validate_tuple = DateTimeParseFunction::do_validate(function_text, 
//     //         //         DateTimeParseOption::Hour, validate_tuple);
//     //         // },
//     //         // FUNCTION_DAY => {
//     //         //     validate_tuple = DateParseFunction::do_validate(function_text, DateParseOption::Day, validate_tuple);
//     //         // },
//     //         // FUNCTION_WEEK => {
//     //         //     validate_tuple = DateParseFunction::do_validate(function_text, DateParseOption::Week, validate_tuple);
//     //         // },
//     //         // FUNCTION_WEEKDAY => {
//     //         //     validate_tuple = DateParseFunction::do_validate(function_text, DateParseOption::WeekDay, validate_tuple);
//     //         // },
//     //         // FUNCTION_MONTH => {
//     //         //     validate_tuple = DateParseFunction::do_validate(function_text, DateParseOption::Month, validate_tuple);
//     //         // },
//     //         // FUNCTION_YEAR => {
//     //         //     validate_tuple = DateParseFunction::do_validate(function_text, DateParseOption::Year, validate_tuple);
//     //         // },
//     //         // FUNCTION_NOW => {
//     //         //     validate_tuple = NowFunction::do_validate(function_text, validate_tuple);
//     //         // },
//     //         // FUNCTION_TODAY => {
//     //         //     validate_tuple = TodayFunction::do_validate(function_text, validate_tuple);
//     //         // },
//     //         // FUNCTION_DAYS => {
//     //         //     validate_tuple = DaysFunction::do_validate(function_text, validate_tuple);
//     //         // },
//     //         // FUNCTION_DATEADD => {
//     //         //     validate_tuple = DateAddDiffFunction::do_validate(function_text, 
//     //         //         DateDeltaOperation::Add, validate_tuple);
//     //         // },
//     //         // FUNCTION_DATEFMT => {
//     //         //     validate_tuple = DateFormatFunction::do_validate(function_text, validate_tuple);
//     //         // },
//     //         // FUNCTION_IF => {
//     //         //     validate_tuple = IfFunction::do_validate(function_text, validate_tuple);
//     //         // },
//     //         // FUNCTION_MIN => {
//     //         //     validate_tuple = StatsFunction::do_validate(function_text, validate_tuple, StatOption::Min);
//     //         // },
//     //         // FUNCTION_MAX => {
//     //         //     validate_tuple = StatsFunction::do_validate(function_text, validate_tuple, StatOption::Max);
//     //         // },
//     //         _ => {
//     //             number_fails += 1;
//     //         }
//     //     }
//     // }
//     // number_fails = validate_tuple.0;
//     // failed_functions = validate_tuple.1;
//     // if number_fails > 0 {
//     //     let failed_functions_str = failed_functions.join(", ");
//     //     return Err(
//     //         PlanetError::new(
//     //             500, 
//     //             Some(tr!("Could not validate formula. Failed functions: {}", &failed_functions_str)),
//     //         )
//     //     );
//     // }
//     return Ok(check);
// }

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
pub struct FunctionAttributeNew{
    pub item: String,
    pub remove_quotes: Option<bool>,
    pub item_processed: Option<String>,    
}
impl FunctionAttributeNew {
    pub fn defaults(attribute: &String, remove_quotes: Option<bool>) -> FunctionAttributeNew {
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
    pub fn replace(&self, data_map: &HashMap<String, String>) -> Self {
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
        // let t_1 = Instant::now();
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
        // eprintln!("Formula.execute :: perf : header: {} µs", &t_1.elapsed().as_micros());
        // let t_process_formula_1 = Instant::now();
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
        // eprintln!("Formula.execute :: perf : process formula: {} µs", &t_process_formula_1.elapsed().as_micros());
        // let t_process_formula_2 = Instant::now();
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
        // eprintln!("Formula.execute :: perf : process formula (2): {} µs", &t_process_formula_2.elapsed().as_micros());
        // This injects references without achiever functions
        if data_map.is_some() && table.is_some() {
            // let t_inyect_1 = Instant::now();
            // eprintln!("Formula.execute :: formula before inyect: *{}*", &formula_string);
            let formula_wrap = self.inyect_data_formula(&formula_string);
            // eprintln!("Formula.execute :: formula after inyect: *{:?}*", &formula_wrap);
            // eprintln!("Formula.execute :: perf : inyect formula: {} µs", &t_inyect_1.elapsed().as_micros());
            if formula_wrap.is_some() {
                formula_string = formula_wrap.unwrap();
                // let t_string_1 = Instant::now();
                formula_string = self.process_string_matches(&formula_string);
                // eprintln!("Formula.execute :: perf : string matches: {} µs", &t_string_1.elapsed().as_micros());
                // eprintln!("Formula.execute :: formula_new: {}", &formula_string);
                formula_string = format!("={}", &formula_string);
                // let t_exec_1 = Instant::now();
                let formula_ = parse_formula::parse_string_to_formula(
                    &formula_string, 
                    None::<NoCustomFunction>
                );
                // eprintln!("Formula.execute :: formula_: {:?}", &formula_);
                let result = calculate::calculate_formula(formula_, None::<NoReference>);
                // eprintln!("Formula.execute :: calcuated formula_: {:?}", &result);
                let result = calculate::result_to_string(result);
                // eprintln!("Formula.execute :: perf : exec: {} µs", &t_exec_1.elapsed().as_micros());
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AttributeType {
    Text,
    Number,
    Bool,
    Date,
}

// {Field} = "pepito"
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttributeAssign(
    pub String, 
    pub FormulaOperator, 
    pub String
);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FormulaQueryCompiled {
    pub function: Option<CompiledFunction>,
    pub assignment: Option<FunctionAttributeItem>,
}
impl FormulaQueryCompiled {
    pub fn defaults(function: Option<CompiledFunction>, assignment: Option<FunctionAttributeItem>) -> Self {
        let obj = Self{
            function: function,
            assignment: assignment,
        };
        return obj
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FormulaFieldCompiled {
    pub functions: Option<HashMap<String, CompiledFunction>>,
    pub formula: String,
}
impl FormulaFieldCompiled {
    pub fn defaults(
        formula: &String,
        formula_format: &String,
        field_type_map: &HashMap<String, String>,
    ) -> Result<Self, PlanetError> {
        // If I have an error in compilation, then does not validate. Compilation uses validate of functions.
        // This function is the one does compilation from string formula to FormulaFieldCompiled
        let formula_origin = formula.clone();
        eprintln!("FormulaFieldCompiled :: formula_origin: {:?}", &formula_origin);
        eprintln!("FormulaFieldCompiled :: formula_format: {:?}", &formula_format);
        let expr = &RE_FORMULA_FIELD_FUNCTIONS;
        let mut formula_processed = formula_origin.clone();
        let formula_format = formula_format.clone();

        let mut formula_compiled = FormulaFieldCompiled{
            functions: None,
            formula: String::from(""),
        };
        // Start processing formula into compiled structure
        let mut compiled_functions: Vec<CompiledFunction> = Vec::new();
        let function_list = expr.captures_iter(&formula_origin);

        let mut count = 1;
        let mut compiled_functions_map: HashMap<String, CompiledFunction> = HashMap::new();
        for capture in function_list {
            eprintln!("FormulaFieldCompiled :: capture: {:?}", &capture);
            let function_text = capture.get(0).unwrap().as_str();
            let function_placeholder = format!("$func{}", &count);
            eprintln!("FormulaFieldCompiled :: function_text: {}", function_text);
            eprintln!("FormulaFieldCompiled :: function_placeholder: {}", function_placeholder);
            let main_function = compile_function_text(
                function_text, 
                &formula_format,
                field_type_map
            )?;
            compiled_functions.push(main_function.clone());
            compiled_functions_map.insert(function_placeholder.clone(), main_function.clone());
            formula_processed = formula_processed.replace(function_text, function_placeholder.as_str());
            eprintln!("FormulaFieldCompiled :: formula_processed: {:#?}", &formula_processed);
            count += 1;
        }

        // TODO: Apply also to the functions linked inside this function as attributes
        for (function_key, function) in &compiled_functions_map {
            eprintln!("FormulaFieldCompiled :: function_key: {}", &function_key);
            let function = function.clone();
            let function_text = function.text.unwrap();
            let function_text = function_text.as_str();
            let function_name = function.name;
            let function_name = function_name.as_str();
            let validate = validate_function_text(function_name, function_text)?;
            eprintln!("FormulaFieldCompiled :: validate: {}", &validate);
            if validate == false {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!("This function is not correctly formatted: {}", function_text)),
                    )
                );
            }
        }

        formula_compiled.functions = Some(compiled_functions_map);
        formula_compiled.formula = formula_processed;

        return Ok(formula_compiled)
    }
}

pub fn compile_function_text(
    function_text: &str,
    formula_format: &String,
    field_type_map: &HashMap<String, String>,
) -> Result<CompiledFunction, PlanetError> {
    let formula_format = formula_format.clone();
    let field_type_map = field_type_map.clone();
    eprintln!("compile_function_text :: field_type_map: {:#?}", &field_type_map);
    let parts: Vec<&str> = function_text.split("(").collect();
    let function_name = parts[0];
    eprintln!("compile_function_text :: function_name: {}", function_name);
    // function_name: CONCAT for example
    let mut main_function: CompiledFunction = CompiledFunction::defaults(
        &function_name.to_string());
    main_function.text = Some(function_text.to_string());
    let mut main_function_attrs: Vec<FunctionAttributeItem> = Vec::new();
    let expr = &RE_FUNCTION_ATTRS;
    let attr_map = expr.captures(function_text).unwrap();
    let attrs = attr_map.name("attrs");
    if attrs.is_some() {
        let attrs = attrs.unwrap().as_str();
        eprintln!("compile_function_text :: [new] attrs: {}", &attrs);
        let captured_attrs: Vec<&str> = attrs.split(",").collect();
        for mut attr in captured_attrs {
            eprintln!("compile_function_text :: attr: *{}*", attr);
            let mut attribute_type: AttributeType = AttributeType::Text;
            let mut function_attribute = FunctionAttributeItem::defaults(
                &String::from(""), 
                attribute_type
            );
            let replaced_text: String;
            if attr.find("{").is_some() {
                // Reference
                // I have attribute name and also the field type -> attribute_type from table
                let attr_string = attr.to_string();
                let attr_string = attr_string.replace("{", "").replace("}", "");
                function_attribute.name = attr_string.clone();
                let field_type = field_type_map.get(&attr_string);
                if field_type.is_some() {
                    let field_type = field_type.unwrap().clone();
                    attribute_type = get_attribute_type(&field_type, Some(formula_format.clone()));
                    function_attribute.attr_type = attribute_type;
                    function_attribute.is_reference = true;
                }
            } else if attr.find("(").is_some() {
                // function
                eprintln!("compile_function_text :: function_text: {}", &attr);
                let linked_function = compile_function_text(
                     attr, &formula_format, &field_type_map
                )?;
                let linked_function_text = linked_function.text.clone().unwrap();
                function_attribute.function = Some(linked_function);
                function_attribute.name = linked_function_text;
                // eprintln!("compile_function_text :: linked_function: {:#?}", &linked_function);
            } else {
                // Normal attribute, text, date, number
                // Attribute type: Text, Number, Bool (Not possible, through function), Date
                if attr.find("\"").is_some() {
                    // Could be text or date, time. How deal with it?
                    // TODO: Have date strings to check to resolve if we have a date, time or text
                    function_attribute.attr_type = AttributeType::Text;
                    replaced_text = attr.replace("\"", "");
                    attr = replaced_text.as_str();
                } else {
                    function_attribute.attr_type = AttributeType::Number;
                }
                // Set value
                function_attribute.value = Some(attr.to_string());
            }
            main_function_attrs.push(function_attribute);    
        }
    }
    main_function.attributes = Some(main_function_attrs);
    return Ok(main_function)
}

pub fn validate_function_text(function_name: &str, function_text: &str) -> Result<bool, PlanetError> {
    let check: bool;
    match function_name {
        FUNCTION_CONCAT => {
            check = *&RE_CONCAT_ATTRS.is_match(function_text);
        },
        FUNCTION_TRIM => {
            check = *&RE_TRIM.is_match(function_text);
        },
        _ => {
            check = true;
        }
    }
    return Ok(check)
}

// Score for this is validate all functions in a text formula, but design  might change with compilation
// of formulas
pub fn function_validate_tuple(
    function_text: &String, 
    validate_tuple: (u32, Vec<String>),
    expr: &Regex,
    function_name: &str,
 ) -> (u32, Vec<String>) {
    let (number_fails, mut failed_functions) = validate_tuple;
    let expr = expr;
    let check = expr.is_match(&function_text);
    let mut number_fails = number_fails.clone();
    if check == false {
        number_fails += 1;
        failed_functions.push(String::from(function_name));
    }
    return (number_fails, failed_functions);
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompiledFunction {
    pub name: String,
    pub text: Option<String>,
    pub attributes: Option<Vec<FunctionAttributeItem>>,
}
impl CompiledFunction {
    pub fn defaults(name: &String) -> Self {
        let name = name.clone();
        let obj = Self{
            name: name,
            attributes: None,
            text: None,
        };
        return obj;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FormulaOperator {
    Eq,
    Greater,
    Smaller,
    GreaterOrEqual,
    SmallerOrEqual,
}

// attributes:
// {Column}
// "value" or value
// (3 + 4)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionAttributeItem {
    pub is_reference: bool,
    pub reference_value: Option<String>,
    pub assignment: Option<AttributeAssign>,
    pub name: String,
    pub value: Option<String>,
    pub attr_type: AttributeType,
    pub function: Option<CompiledFunction>,
}
impl FunctionAttributeItem {
    pub fn defaults(name: &String, attr_type: AttributeType) -> Self {
        let name = name.clone();
        let attr_type = attr_type.clone();
        let obj = Self{
            is_reference: false,
            reference_value: None,
            assignment: None,
            name: name,
            value: None,
            attr_type: attr_type,
            function: None,
        };
        return obj
    }
}

pub fn execute_formula_query(
    formula: &FormulaQueryCompiled, 
    data_map: &HashMap<String, String>
) -> Result<bool, PlanetError> {
    let mut check: bool = true;
    // eprint!("execute_formula :: formula: {:#?}", formula);
    // In case we have function, like AND, OR, NOT, XOR and other references and functions inside
    let function = formula.function.clone();
    if function.is_some() {
        let function = function.unwrap();
        let result = execute_function(&function, data_map)?;
        if result.check.unwrap() == false {
            check = false;
        }
    }
    // In case we have 1 direct assignment, like {Column A} = "mine" or {Column B} > 67.8
    let assignment = formula.assignment.clone();
    if assignment.is_some() {
        let assignment = assignment.unwrap();
        let is_reference = assignment.is_reference;
        if is_reference == true {
            let attr_assignment = assignment.assignment.unwrap();
            check = check_assignment(attr_assignment, assignment.attr_type, data_map)?;
        }
    }
    return Ok(check)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionResult {
    pub text: Option<String>,
    pub date: Option<String>,
    pub check: Option<bool>,
    pub number: Option<usize>,
}

pub fn execute_function(
    function: &CompiledFunction, 
    data_map: &HashMap<String, String>
) -> Result<FunctionResult, PlanetError> {
    // Here we match which function and we call function directly. We get result as a value and return
    let function_name = function.name.clone();
    let function_name = function_name.as_str();
    let attributes = function.attributes.clone();
    let attributes = attributes.unwrap();
    // eprintln!("execute_function :: function_name: {} attributes: {:#?}", function_name, &attributes);
    let mut function_result = FunctionResult{
        text: None,
        date: None,
        check: None,
        number: None,
    };
    // Here I map all the functions, ones used by formula queries and ones as formula field
    match function_name {
        "AND" => {
            function_result.check = Some(and(data_map, &attributes)?);
        },
        "OR" => {
            function_result.check = Some(or(data_map, &attributes)?);
        },
        "NOT" => {
            function_result.check = Some(not(data_map, &attributes)?);
        },
        "XOR" => {
            function_result.check = Some(xor(data_map, &attributes)?);
        },
        _ => {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("Function \"{}\" not supported", &function_name)),
                )
            );
        }
    }
    return Ok(function_result)
}

pub fn compile_formula_query(
    formula: &String, 
    db_table: &DbTable,
    table_name: &String,
    table: Option<DbData>,
    field_type_map: Option<HashMap<String, String>>,
    field_name_map: Option<HashMap<String, String>>,
) -> Result<FormulaQueryCompiled, PlanetError> {
    // eprintln!("compile_formula_query...");
    let db_table = db_table.clone();
    let expr = &RE_FORMULA_QUERY;
    let formula_str = formula.as_str();
    let formula_str = formula_str.replace("\n", "");
    let expr_map = expr.captures(&formula_str);
    let mut formula_compiled = FormulaQueryCompiled::defaults(
        None, None
    );
    let field_name_map_: HashMap<String, String>;
    let field_type_map_: HashMap<String, String>;
    if table.is_some() {
        let table = table.unwrap();
        field_type_map_ = DbTable::get_field_type_map(&table)?;
        field_name_map_ = DbTable::get_field_name_map(&db_table, table_name)?;
    } else if field_type_map.is_some() && field_name_map.is_some() {
        let field_type_map = field_type_map.unwrap();
        field_type_map_ = field_type_map;
        let field_name_map = field_name_map.unwrap();
        field_name_map_ = field_name_map
    } else {
        // This means that table is None, field_type_map as well, we raise error letting know about problem
        return Err(
            PlanetError::new(
                500, 
                Some(tr!("Either the db table instance needs to be informed or the 
                map for the field types")),
            )
        );
    }
    
    // "Status": "Select",
    // My Field": "Small Text",
    // If I could field name => field id, then could get id for the field names
    // eprintln!("compile_formula_query :: field_name_map: {:#?}", &field_name_map);
    // eprintln!("compile_formula_query :: field_type_map: {:#?}", &field_type_map);

    if expr_map.is_some() {
        let expr_map = expr_map.unwrap();
        let has_assign = *&expr_map.name("assign").is_some();
        // eprintln!("compile_formula_query :: has_assign: {}", &has_assign);
        if has_assign {
            let assign = *&expr_map.name("assign").unwrap().as_str();
            let assign = assign.to_string();
            // capture log_op which is =, <, >, etc...
            // I need to process these possible cases:
            // {My Column} = "pepito"
            // {My Column} = 98.89
            // {My Column} = TRIM(" pepito ")
            // {My Column} > 98
            let (items, attribute_operator) = parse_assign_operator(
                &assign, &formula
            )?;
            let (reference_name, items_new) = get_assignment_reference(
                &items, 
                field_name_map_
            )?;
            let field_type = field_type_map_.get(&reference_name);
            let mut attribute_type: AttributeType = AttributeType::Text;
            if field_type.is_some() {
                let field_type = field_type.unwrap();
                attribute_type = get_attribute_type(field_type, None);
            }
            let mut function_attribute = FunctionAttributeItem::defaults(
                &reference_name, 
                attribute_type
            );
            function_attribute.is_reference = true;
            function_attribute.assignment = Some(
                AttributeAssign(
                    items_new[0].clone(), 
                    attribute_operator, 
                    items_new[1].clone()
                )
            );
            formula_compiled.assignment = Some(function_attribute);
        } else {
            let op = *&expr_map.name("op").unwrap().as_str();
            // attrs: {My Field}   =   "pepito"  , {Status}  =   "c4vhm0gsmpv7omu4aqg0"  ,   {Mine}  =    98.3
            let attributes_str = *&expr_map.name("attrs").unwrap().as_str();
            let op_string = String::from(op);
            // eprintln!("compile_formula_query :: op: {} attributes: {}", &op_string, &attributes_str);
            let mut main_function: CompiledFunction = CompiledFunction::defaults(&op_string);
            let mut main_function_attrs: Vec<FunctionAttributeItem> = Vec::new();
            let attributes_source: Vec<&str> = attributes_str.split(",").collect();
            // let mut item_replaced: String;
            for attr_source in attributes_source {
                //|  {Status}  =   "c4vhm0gsmpv7omu4aqg0"  |
                // Operators can be "=", "<", ">", ">=", "<="
                
                // eprintln!("compile_formula_query :: attr_source: *{}*", attr_source);
                let (items, attribute_operator) = parse_assign_operator(
                    attr_source, &formula
                )?;

                let (reference_name, items_new) = get_assignment_reference(
                    &items, 
                    field_name_map_.clone()
                )?;
                // ["$id", "my value"]
                // eprintln!("compile_formula_query :: reference_name: {}", &reference_name);
                // eprintln!("compile_formula_query :: items_new: {:?}", &items_new);
                let field_type = field_type_map_.get(&reference_name);
                let mut attribute_type: AttributeType = AttributeType::Text;
                if field_type.is_some() {
                    let field_type = field_type.unwrap();
                    // eprintln!("compile_formula_query :: field_type: {}", &field_type);
                    attribute_type = get_attribute_type(field_type, None);
                    // eprintln!("compile_formula_query :: attribute_type: {:?}", &attribute_type);
                }
                let mut function_attribute = FunctionAttributeItem::defaults(
                    &reference_name, 
                    attribute_type
                );
                function_attribute.is_reference = true;
                function_attribute.assignment = Some(
                    AttributeAssign(
                        items_new[0].clone(), 
                        attribute_operator, 
                        items_new[1].clone()
                    )
                );
                // eprintln!("compile_formula_query :: function_attribute: {:#?}", &function_attribute);
                main_function_attrs.push(function_attribute);
            }
            main_function.attributes = Some(main_function_attrs);
            // formula_function = main_function;
            formula_compiled.function = Some(main_function);
        }
        // eprintln!("compile_formula_query :: formula_compiled: {:#?}", &formula_compiled);
        return Ok(formula_compiled)
    } else {
        return Err(
            PlanetError::new(
                500, 
                Some(tr!("Could not validate formula. Formula: {}", &formula)),
            )
        );
    }
}

pub fn fetch_logical_op(attribute: &str) -> &str {
    let mut log_op: &str = "";
    let has_equal = attribute.find("=").is_some();
    let has_greater = attribute.find(">").is_some();
    let has_smaller = attribute.find("<").is_some();
    let has_equal_or_greater = attribute.find("=>").is_some();
    let has_smaller_or_equal = attribute.find("<=").is_some();
    if has_equal == true {
        log_op = "=";
    } else {
        if has_greater == true && has_equal_or_greater == false {
            log_op = ">";
        } else if has_greater == true && has_equal_or_greater == true {
            log_op = "=>";
        } else if has_smaller == true && has_smaller_or_equal == false {
            log_op = "<";
        } else if has_smaller == true && has_smaller_or_equal == true {
            log_op = "<=";
        }
    }
    return log_op
}

pub fn get_attribute_type(field_type: &String, formula_format: Option<String>) -> AttributeType {
    let field_type = field_type.clone();
    let field_type = field_type.to_lowercase();
    let field_type = field_type.as_str();
    let attr_type: AttributeType;
    match field_type {
        "checkbox" => {attr_type = AttributeType::Bool},
        "small text" => {attr_type = AttributeType::Text},
        "long text" => {attr_type = AttributeType::Text},
        "select" => {attr_type = AttributeType::Text},
        "number" => {attr_type = AttributeType::Number},
        _ => {attr_type = AttributeType::Text},
    }
    if field_type == "formula" && formula_format.is_some() {
        let formula_format = formula_format.unwrap().to_lowercase();
        let formula_format = formula_format.as_str();
        return match formula_format {
            "text" => AttributeType::Text,
            "number" => AttributeType::Number,
            "date" => AttributeType::Date,
            "bool" => AttributeType::Bool,
            _ => AttributeType::Text,
        }
    }
    return attr_type
}

pub fn get_assignment_reference(
    items: &Vec<String>, 
    field_name_map: HashMap<String, String>
) -> Result<(String, Vec<String>), PlanetError> {
    let mut reference_name: String = String::from("");
    let mut items_new: Vec<String> = Vec::new();
    let mut item_replaced: String;
    for (count, item) in items.iter().enumerate() {
        let mut item = item.clone();
        // |  {Status}  |
        eprintln!("compile_formula_query :: item: *{}*", item);
        // let mut item_ = *item;
        let item_ = item.trim();
        item = item_.to_string();
        eprintln!("compile_formula_query :: item: *{}*", item);
        if count == 0 {
            // {Column A} => $column_id
            // let mut item_string = item_.to_string();
            item = item.replace("{", "}").replace("}", "");
            reference_name = item.clone();
            let column_id = &field_name_map.get(&item).unwrap();
            let column_id = column_id.clone();
            // eprintln!("compile_formula_query :: column_id: {}", column_id);
            item = column_id.clone();
        }
        item_replaced = item.replace("\"", "");
        items_new.push(item_replaced);
    }
    return Ok((reference_name, items_new));
}

pub fn parse_assign_operator(
    attr_source: &str, 
    formula: &String
) -> Result<(Vec<String>, FormulaOperator), PlanetError> {
    let log_op = fetch_logical_op(attr_source);
    let items: Vec<&str>;
    let attribute_operator: FormulaOperator;
    match log_op {
        "=" => {
            items = attr_source.split(log_op).collect();
            attribute_operator = FormulaOperator::Eq;
        },
        "=>" => {
            items = attr_source.split(log_op).collect();
            attribute_operator = FormulaOperator::GreaterOrEqual;
        },
        "<=" => {
            items = attr_source.split(log_op).collect();
            attribute_operator = FormulaOperator::SmallerOrEqual;
        },
        ">" => {
            items = attr_source.split(log_op).collect();
            attribute_operator = FormulaOperator::Greater;
        },
        "<" => {
            items = attr_source.split(log_op).collect();
            attribute_operator = FormulaOperator::Smaller;
        },
        _ => {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("Could not validate formula. Formula: {}", &formula)),
                )
            );
        }
    }
    let mut items_string: Vec<String> = Vec::new();
    for item_str in items {
        let item_string: String = item_str.to_string();
        items_string.push(item_string);
    }    
    return Ok((items_string, attribute_operator))
}

pub fn check_assignment(
    attr_assignment: AttributeAssign,
    attr_type: AttributeType,
    data_map: &HashMap<String, String>,
) -> Result<bool, PlanetError> {
    let column_id = attr_assignment.0;
    let column_id = column_id.as_str();
    let db_value = data_map.get(column_id).unwrap();
    let op = attr_assignment.1;
    let mut value = attr_assignment.2;
    let check: bool;
    // We have case when we try to compare dates, but is not supported, functions would need to be used.
    // Greater and smaller is used for numbers
    // TODO: Match for bool {Column} = true. How???? TRUE() right?
    match attr_type {
        AttributeType::Text | AttributeType::Date => {
            match op {
                FormulaOperator::Eq => {
                    value = value.replace("\"", "");
                    check = check_string_equal(&db_value, &value)?;
                },
                _ => {
                    return Err(
                        PlanetError::new(
                            500, 
                            Some(tr!("Assignment for string only support equal operator.")),
                        )
                    );
                },
            }    
        },
        AttributeType::Number => {
            let value = value.as_str();
            let value: f64 = FromStr::from_str(value).unwrap();
            let db_value: f64 = FromStr::from_str(db_value).unwrap();
            check = check_float_compare(&value, &db_value, op)?;
        },
        _ => {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("Assignment type not supported. We only support 
                    Text, Number, Bool and Date")),
                )
            );
        }
    }
    return Ok(check)
}
