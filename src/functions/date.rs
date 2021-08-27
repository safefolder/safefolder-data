use std::{str::FromStr};
use regex::{Regex, Captures};
use std::{collections::HashMap};
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;
use chrono::{DateTime, Datelike, FixedOffset, NaiveDate, Timelike, Utc, Duration};
use tr::tr;
use ordinal::Ordinal;

use crate::{functions::FunctionAttribute, planet::PlanetError};
use crate::functions::constants::*;

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
    static ref RE_DAY: Regex = Regex::new(r#"DAY\((?P<date>"\d{4}-\d{2}-\d{2}T{0,1}[\s]{0,1}\d{2}:\d{2}:\d{2}(\+\d{2}:\d{2}){0,1}")\)|DAY\((?P<date_alt>"\d{2}-[a-zA-Z]{3}-\d{4}([\s]{0,1}\d{2}:\d{2}:\d{2}){0,1}(\+\d{2}:\d{2}){0,1}")\)|DAY\((?P<date_ref>\{[\w\s]+\})\)"#).unwrap();
    static ref RE_WEEK: Regex = Regex::new(r#"WEEK\((?P<date>"\d{4}-\d{2}-\d{2}T{0,1}[\s]{0,1}\d{2}:\d{2}:\d{2}(\+\d{2}:\d{2}){0,1}")\)|WEEK\((?P<date_alt>"\d{2}-[a-zA-Z]{3}-\d{4}([\s]{0,1}\d{2}:\d{2}:\d{2}){0,1}(\+\d{2}:\d{2}){0,1}")\)|WEEK\((?P<date_ref>\{[\w\s]+\})\)"#).unwrap();
    static ref RE_WEEKDAY: Regex = Regex::new(r#"WEEKDAY\((?P<date>"\d{4}-\d{2}-\d{2}T{0,1}[\s]{0,1}\d{2}:\d{2}:\d{2}(\+\d{2}:\d{2}){0,1}")\)|WEEKDAY\((?P<date_alt>"\d{2}-[a-zA-Z]{3}-\d{4}([\s]{0,1}\d{2}:\d{2}:\d{2}){0,1}(\+\d{2}:\d{2}){0,1}")\)|WEEKDAY\((?P<date_ref>\{[\w\s]+\})\)"#).unwrap();
    static ref RE_MONTH: Regex = Regex::new(r#"MONTH\((?P<date>"\d{4}-\d{2}-\d{2}T{0,1}[\s]{0,1}\d{2}:\d{2}:\d{2}(\+\d{2}:\d{2}){0,1}")\)|MONTH\((?P<date_alt>"\d{2}-[a-zA-Z]{3}-\d{4}([\s]{0,1}\d{2}:\d{2}:\d{2}){0,1}(\+\d{2}:\d{2}){0,1}")\)|MONTH\((?P<date_ref>\{[\w\s]+\})\)"#).unwrap();
    static ref RE_YEAR: Regex = Regex::new(r#"YEAR\((?P<date>"\d{4}-\d{2}-\d{2}T{0,1}[\s]{0,1}\d{2}:\d{2}:\d{2}(\+\d{2}:\d{2}){0,1}")\)|YEAR\((?P<date_alt>"\d{2}-[a-zA-Z]{3}-\d{4}([\s]{0,1}\d{2}:\d{2}:\d{2}){0,1}(\+\d{2}:\d{2}){0,1}")\)|YEAR\((?P<date_ref>\{[\w\s]+\})\)"#).unwrap();
    static ref RE_HOUR: Regex = Regex::new(r#"HOUR\((?P<date>"\d{4}-\d{2}-\d{2}T{0,1}[\s]{0,1}\d{2}:\d{2}:\d{2}(\+\d{2}:\d{2}){0,1}")\)|HOUR\((?P<date_alt>"\d{2}-[a-zA-Z]{3}-\d{4}([\s]{0,1}\d{2}:\d{2}:\d{2}){0,1}(\+\d{2}:\d{2}){0,1}")\)|HOUR\((?P<date_ref>\{[\w\s]+\})\)"#).unwrap();
    static ref RE_MINUTE: Regex = Regex::new(r#"MINUTE\((?P<date>"\d{4}-\d{2}-\d{2}T{0,1}[\s]{0,1}\d{2}:\d{2}:\d{2}(\+\d{2}:\d{2}){0,1}")\)|MINUTE\((?P<date_alt>"\d{2}-[a-zA-Z]{3}-\d{4}([\s]{0,1}\d{2}:\d{2}:\d{2}){0,1}(\+\d{2}:\d{2}){0,1}")\)|MINUTE\((?P<date_ref>\{[\w\s]+\})\)"#).unwrap();
    static ref RE_SECOND: Regex = Regex::new(r#"SECOND\((?P<date>"\d{4}-\d{2}-\d{2}T{0,1}[\s]{0,1}\d{2}:\d{2}:\d{2}(\+\d{2}:\d{2}){0,1}")\)|SECOND\((?P<date_alt>"\d{2}-[a-zA-Z]{3}-\d{4}([\s]{0,1}\d{2}:\d{2}:\d{2}){0,1}(\+\d{2}:\d{2}){0,1}")\)|SECOND\((?P<date_ref>\{[\w\s]+\})\)"#).unwrap();
    static ref RE_NOW: Regex = Regex::new(r#"NOW\(\)"#).unwrap();
    static ref RE_TODAY: Regex = Regex::new(r#"TODAY\(\)"#).unwrap();
    static ref RE_DAYS: Regex = Regex::new(r#"(DAYS\((?P<end_date>"\d{1,2}-[a-zA-Z]{3}-\d{4}"),[\s]+(?P<start_date>"\d{1,2}-[a-zA-Z]{3}-\d{4}"))\)|DAYS\(((?P<end_date_ref>\{[\w\s]+\}),[\s]+(?P<start_date_ref>\{[\w\s]+\}))\)"#).unwrap();
    static ref RE_DATEADD: Regex = Regex::new(r#"DATEADD\((?P<date>"\d{1,2}-[a-zA-Z]{3}-\d{4}"),[\s]+(?P<number>\d+),[\s]+(?P<units>("days")|("months")|("milliseconds")|("seconds")|("minutes")|("hours")|("weeks")|("quarters")|("years"))\)|DATEADD\((?P<datetime>"\d{1,2}-[a-zA-Z]{3}-\d{4}[\s]{1}\d{1,2}:\d{1,2}:\d{1,2}"),[\s]+(?P<dt_number>\d+),[\s]+(?P<dt_units>("days")|("months")|("milliseconds")|("seconds")|("minutes")|("hours")|("weeks")|("quarters")|("years"))\)|DATEADD\((?P<date_ref>\{[\w\s]+\}),[\s]+(?P<ref_number>\d{1,2}),[\s]+(?P<ref_units>("days")|("months")|("milliseconds")|("seconds")|("minutes")|("hours")|("weeks")|("quarters")|("years"))\)"#).unwrap();
    static ref RE_DATEDIF: Regex = Regex::new(r#"DATEDIF\((?P<date>"\d{1,2}-[a-zA-Z]{3}-\d{4}"),[\s]+(?P<number>\d+),[\s]+(?P<units>("days")|("months")|("milliseconds")|("seconds")|("minutes")|("hours")|("weeks")|("quarters")|("years"))\)|DATEDIF\((?P<datetime>"\d{1,2}-[a-zA-Z]{3}-\d{4}[\s]{1}\d{1,2}:\d{1,2}:\d{1,2}"),[\s]+(?P<dt_number>\d+),[\s]+(?P<dt_units>("days")|("months")|("milliseconds")|("seconds")|("minutes")|("hours")|("weeks")|("quarters")|("years"))\)|DATEDIF\((?P<date_ref>\{[\w\s]+\}),[\s]+(?P<ref_number>\d{1,2}),[\s]+(?P<ref_units>("days")|("months")|("milliseconds")|("seconds")|("minutes")|("hours")|("weeks")|("quarters")|("years"))\)"#).unwrap();
    static ref RE_DATEFMT: Regex = Regex::new(r#"DATEFMT\((?P<date>"\d{1,2}-[a-zA-Z]{3}-\d{4}"),[\s]+(?P<format>"[\{a-zA-Z-/,:_\s\}].+")\)|DATEFMT\((?P<datetime>"\d{1,2}-[a-zA-Z]{3}-\d{4}\s{1}\d{2}:\d{2}:\d{2}"),[\s]+(?P<dt_format>"[\{a-zA-Z-/,:_\s\}].+")\)|DATEFMT\((?P<ref>\{[\w\s]+\}),[\s]+(?P<ref_format>"[\{a-zA-Z-/,:_\s\}].+")\)"#).unwrap();
}

// DATE(year,month,day)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DateFunction {
    pub function_text: String,
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

        let obj = Self{
            function_text: function_text.clone(),
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
    pub fn do_validate(function_text: &String, validate_tuple: (u32, Vec<String>)) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = DateFunction::defaults(
            &function_text, 
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_DATE));
        }
        return (number_fails, failed_functions);
    }
    pub fn replace(&mut self, formula: String) -> String {
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();
        let year = self.year;
        let month = self.month;
        let day = self.day;
        let date_only = NaiveDate::parse_from_str(
            format!(
                "{year}-{month}-{day}",
                year=year,
                month=month,
                day=day,
            ).as_str(), 
            "%Y-%m-%d"
        ).unwrap();
        let month_short = date_only.format("%b").to_string();
        let replacement_string: String = format!(
            "{day}-{month_short}-{year}",
            day=day,
            month_short=month_short,
            year=year,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DateParseOption {
    Day,
    Week,
    WeekDay,
    Month,
    Year,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DateTimeParseOption {
    Second,
    Minute,
    Hour,
}

// MINUTE($date_str)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DateTimeParseFunction {
    pub function_text: String,
    pub date_string: Option<String>,
    pub date_ref: Option<String>,
    pub parse_option: DateTimeParseOption,
    pub mode: String,
    pub is_iso: bool,
    pub is_human_time: bool,
    pub only_date: bool,
}
impl DateTimeParseFunction {
    pub fn defaults(function_text: &String, date_parse_option: DateTimeParseOption) -> DateTimeParseFunction {
        // Applies to MINUTE, HOUR, SECOND

        let matches: Captures;
        match date_parse_option {
            DateTimeParseOption::Hour => {
                matches = RE_HOUR.captures(function_text).unwrap();
            },
            DateTimeParseOption::Minute => {
                matches = RE_MINUTE.captures(function_text).unwrap();
            },
            DateTimeParseOption::Second => {
                matches = RE_SECOND.captures(function_text).unwrap();
            },
        }
        let mut date_string_wrap: Option<String> = None;
        let mut date_ref_wrap: Option<String> = None;
        let date_parse_option = date_parse_option.clone();
        if matches.name("date").is_some() {
            let date = matches.name("date").unwrap().as_str().to_string();
            date_string_wrap = Some(date);
        } else if matches.name("date_alt").is_some() {
            let date = matches.name("date_alt").unwrap().as_str().to_string();
            date_string_wrap = Some(date);
        } else if matches.name("date_ref").is_some() {
            let date = matches.name("date_ref").unwrap().as_str().to_string();
            date_ref_wrap = Some(date)
        }

        let mut is_iso: bool = false;
        let mut is_human_time: bool = false;
        let mut only_date: bool = false;
        let mut mode: &str = "";
        if date_string_wrap.is_some() {
            let date_string = date_string_wrap.unwrap();
            if date_string.find("T").is_some() {
                is_iso = true;
                mode = "iso";
            } else if date_string.find(" ").is_some() {
                is_human_time = true;
                mode = "human_time"
            } else {
                only_date = true;
                mode = "only_date"
            }
            date_string_wrap = Some(date_string);
        }

        let obj = Self{
            function_text: function_text.clone(),
            date_string: date_string_wrap,
            date_ref: date_ref_wrap,
            parse_option: date_parse_option,
            mode: mode.to_string(),
            is_iso: is_iso,
            is_human_time: is_human_time,
            only_date: only_date
        };
        return obj
    }
    pub fn validate(&self, date_parse_option: DateTimeParseOption) -> bool {
        let expr: Regex;
        match date_parse_option {
            DateTimeParseOption::Hour => {
                expr = RE_HOUR.clone();
            },
            DateTimeParseOption::Minute => {
                expr = RE_MINUTE.clone();
            },
            DateTimeParseOption::Second => {
                expr = RE_SECOND.clone();
            },
        }
        let function_text = self.function_text.clone();
        let mut check = expr.is_match(&function_text);
        if check == false {
            return false;
        }
        // Validate I can parse the dates for all options
        let date_string = self.date_string.clone();
        let mode = self.mode.clone();
        let mode = mode.as_str();
        if date_string.is_some() {
            let date_string = date_string.unwrap();
            if mode == "iso" {
                let date_object = get_date_object_iso(&date_string);
                if date_object.is_err() {
                    check = false;
                }    
            } else if mode == "human_time" {
                let date_object = get_date_object_human_time(&date_string);
                if date_object.is_err() {
                    check = false;
                }

            }
        }
        return check
    }
    pub fn do_validate(
        function_text: &String, 
        date_parse_option: DateTimeParseOption, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let obj = DateTimeParseFunction::defaults(
            &function_text, date_parse_option.clone()
        );
        let check = obj.validate(date_parse_option.clone());
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            match date_parse_option {
                DateTimeParseOption::Hour => {
                    failed_functions.push(String::from(FUNCTION_HOUR));
                },
                DateTimeParseOption::Minute => {
                    failed_functions.push(String::from(FUNCTION_MINUTE));
                },
                DateTimeParseOption::Second => {
                    failed_functions.push(String::from(FUNCTION_SECOND));
                },
            }
        }
        return (number_fails, failed_functions);
    }
    pub fn replace(&mut self, formula: String, data_map: HashMap<String, String>) -> String {
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();
        let mut replacement_string = String::from("");
        let date_wrap = self.date_string.clone();
        let mut date: String;
        let date_parse_option = self.parse_option.clone();
        let mode = self.mode.clone();
        let mode = mode.as_str();
        if date_wrap.is_some() {
            // We don't have references for the date, inyected as string
            date = date_wrap.unwrap();
        } else {
            // We have reference for date in a column
            let date_ = self.date_ref.clone().unwrap();
            let function_attr = FunctionAttribute::defaults(&date_, Some(true));
            date = function_attr.replace(data_map.clone()).item_processed.unwrap();
        }
        date = date.replace("\"", "");

        if mode == "iso" {
            let date_obj_wrap = get_date_object_iso(&date);
            if date_obj_wrap.is_ok() {
                let date_obj = date_obj_wrap.unwrap();
                match date_parse_option {
                    DateTimeParseOption::Hour => {
                        let hour = date_obj.hour();
                        replacement_string = hour.to_string();
                    },
                    DateTimeParseOption::Minute => {
                        let minute = date_obj.minute();
                        replacement_string = minute.to_string();
                    },
                    DateTimeParseOption::Second => {
                        let second = date_obj.second();
                        replacement_string = second.to_string();
                    }, 
                }
            }
        } else if mode == "human_time" {
            let date_obj_wrap = get_date_object_human_time(&date);
            
            if date_obj_wrap.is_ok() {
                let date_obj = date_obj_wrap.unwrap();
                match date_parse_option {
                    DateTimeParseOption::Hour => {
                        let hour = date_obj.hour();
                        replacement_string = hour.to_string();
                    },
                    DateTimeParseOption::Minute => {
                        let minute = date_obj.minute();
                        replacement_string = minute.to_string();
                    },
                    DateTimeParseOption::Second => {
                        let second = date_obj.second();
                        replacement_string = second.to_string();
                    }, 
                }
            }
        }
        
        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        return formula;
    }
    pub fn do_replace(
        function_text: &String, 
        date_parse_option: DateTimeParseOption, 
        data_map: HashMap<String, String>, 
        mut formula: String
    ) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = DateTimeParseFunction::defaults(
            &function_text, 
            date_parse_option
        );
        formula = concat_obj.replace(formula, data_map.clone());
        return formula
    }
}


// DAY($date_str)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DateParseFunction {
    pub function_text: String,
    pub date_string: Option<String>,
    pub date_ref: Option<String>,
    pub parse_option: DateParseOption,
    pub mode: String,
    pub is_iso: bool,
    pub is_human_time: bool,
    pub only_date: bool,
}
impl DateParseFunction {
    pub fn defaults(function_text: &String, date_parse_option: DateParseOption) -> DateParseFunction {
        // DAY("2021-08-25T06:26:00+00:00")
        // DAY({Column Date}) : This is any of the ones here.
        // DAY("2021-08-25 06:26:00")
        // DAY("25-Aug-2021 06:26:00")
        // DAY("25-AUG-2021 06:26:00")
        // DAY("25-Aug-2021")
        // DAY("25-AUG-2021")
        // DAY({Column Date}) : This has no date in constructor
        // WEEK, WEEKDAY, MONTH and YEAR functions handler with same date formats

        let matches: Captures;
        match date_parse_option {
            DateParseOption::Day => {
                matches = RE_DAY.captures(function_text).unwrap();
            },
            DateParseOption::Week => {
                matches = RE_WEEK.captures(function_text).unwrap();
            },
            DateParseOption::WeekDay => {
                matches = RE_WEEKDAY.captures(function_text).unwrap();
            },
            DateParseOption::Month => {
                matches = RE_MONTH.captures(function_text).unwrap();
            },
            DateParseOption::Year => {
                matches = RE_YEAR.captures(function_text).unwrap();
            },
        }
        let mut date_string_wrap: Option<String> = None;
        let mut date_ref_wrap: Option<String> = None;
        let date_parse_option = date_parse_option.clone();
        if matches.name("date").is_some() {
            let date = matches.name("date").unwrap().as_str().to_string();
            date_string_wrap = Some(date);
        } else if matches.name("date_alt").is_some() {
            let date = matches.name("date_alt").unwrap().as_str().to_string();
            date_string_wrap = Some(date);
        } else if matches.name("date_ref").is_some() {
            let date = matches.name("date_ref").unwrap().as_str().to_string();
            date_ref_wrap = Some(date)
        }

        let mut is_iso: bool = false;
        let mut is_human_time: bool = false;
        let mut only_date: bool = false;
        let mut mode: &str = "";
        if date_string_wrap.is_some() {
            let date_string = date_string_wrap.unwrap();
            if date_string.find("T").is_some() {
                is_iso = true;
                mode = "iso";
            } else if date_string.find(" ").is_some() {
                is_human_time = true;
                mode = "human_time"
            } else {
                only_date = true;
                mode = "only_date"
            }
            date_string_wrap = Some(date_string);
        }

        let obj = Self{
            function_text: function_text.clone(),
            date_string: date_string_wrap,
            date_ref: date_ref_wrap,
            parse_option: date_parse_option,
            mode: mode.to_string(),
            is_iso: is_iso,
            is_human_time: is_human_time,
            only_date: only_date
        };
        return obj
    }
    pub fn validate(&self, date_parse_option: DateParseOption) -> bool {
        let expr: Regex;
        match date_parse_option {
            DateParseOption::Day => {
                expr = RE_DAY.clone();
            },
            DateParseOption::Week => {
                expr = RE_WEEK.clone();
            },
            DateParseOption::WeekDay => {
                expr = RE_WEEKDAY.clone();
            },
            DateParseOption::Month => {
                expr = RE_MONTH.clone();
            },
            DateParseOption::Year => {
                expr = RE_YEAR.clone();
            },
        }
        let function_text = self.function_text.clone();
        let mut check = expr.is_match(&function_text);
        eprintln!("DateParseFunction.validate :: basic check: {}", &check);
        if check == false {
            return false;
        }
        // Validate I can parse the dates for all options
        let date_string = self.date_string.clone();
        let mode = self.mode.clone();
        let mode = mode.as_str();
        if date_string.is_some() {
            let date_string = date_string.unwrap();
            eprintln!("DateParseFunction.validate :: date_string: {:#?}", &date_string);
            if mode == "iso" {
                let date_object = get_date_object_iso(&date_string);
                eprintln!("DateParseFunction.validate :: iso : date_object: {:#?}", &date_object);
                if date_object.is_err() {
                    check = false;
                }    
            } else if mode == "human_time" {
                let date_object = get_date_object_human_time(&date_string);
                eprintln!("DateParseFunction.validate :: human time : date_object: {:#?}", &date_object);
                if date_object.is_err() {
                    check = false;
                }

            } else {
                let date_object = get_date_object_only_date(&date_string);
                eprintln!("DateParseFunction.validate :: date only : date_object: {:#?}", &date_object);
                if date_object.is_err() {
                    check = false;
                }
            }
        }
        return check
    }
    pub fn do_validate(
        function_text: &String, 
        date_parse_option: DateParseOption, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let obj = DateParseFunction::defaults(
            &function_text, date_parse_option.clone()
        );
        let check = obj.validate(date_parse_option.clone());
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            match date_parse_option {
                DateParseOption::Day => {
                    failed_functions.push(String::from(FUNCTION_DAY));
                },
                DateParseOption::Week => {
                    failed_functions.push(String::from(FUNCTION_WEEK));
                },
                DateParseOption::WeekDay => {
                    failed_functions.push(String::from(FUNCTION_WEEKDAY));
                },
                DateParseOption::Month => {
                    failed_functions.push(String::from(FUNCTION_MONTH));
                },
                DateParseOption::Year => {
                    failed_functions.push(String::from(FUNCTION_YEAR));
                },
            }
        }
        return (number_fails, failed_functions);
    }
    pub fn replace(&mut self, formula: String, data_map: HashMap<String, String>) -> String {
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();
        let mut replacement_string = String::from("");
        let date_wrap = self.date_string.clone();
        let mut date: String;
        let date_parse_option = self.parse_option.clone();
        let mode = self.mode.clone();
        let mode = mode.as_str();
        if date_wrap.is_some() {
            // We don't have references for the date, inyected as string
            date = date_wrap.unwrap();
        } else {
            // We have reference for date in a column
            let date_ = self.date_ref.clone().unwrap();
            let function_attr = FunctionAttribute::defaults(&date_, Some(true));
            date = function_attr.replace(data_map.clone()).item_processed.unwrap();
        }
        date = date.replace("\"", "");

        let mut is_string_output = false;
        if mode == "iso" {
            let date_obj_wrap = get_date_object_iso(&date);
            if date_obj_wrap.is_ok() {
                let date_obj = date_obj_wrap.unwrap();
                match date_parse_option {
                    DateParseOption::Day => {
                        let day = date_obj.day();
                        replacement_string = day.to_string();
                    },
                    DateParseOption::Week => {
                        let week = date_obj.iso_week().week();
                        replacement_string = week.to_string();
                    },
                    DateParseOption::WeekDay => {
                        let weekday = date_obj.weekday();
                        replacement_string = weekday.to_string();
                        is_string_output = true;
                    }, 
                    DateParseOption::Month => {
                        let month = date_obj.month();
                        replacement_string = month.to_string();
                    },
                    DateParseOption::Year => {
                        let year = date_obj.year();
                        replacement_string = year.to_string();
                    }, 
                }
            }
        } else if mode == "human_time" {
            let date_obj_wrap = get_date_object_human_time(&date);
            
            if date_obj_wrap.is_ok() {
                let date_obj = date_obj_wrap.unwrap();
                match date_parse_option {
                    DateParseOption::Day => {
                        let day = date_obj.day();
                        replacement_string = day.to_string();
                    },
                    DateParseOption::Week => {
                        let week = date_obj.iso_week().week();
                        replacement_string = week.to_string();
                    },
                    DateParseOption::WeekDay => {
                        let weekday = date_obj.weekday();
                        replacement_string = weekday.to_string();
                        is_string_output = true;
                    }, 
                    DateParseOption::Month => {
                        let month = date_obj.month();
                        replacement_string = month.to_string();
                    },
                    DateParseOption::Year => {
                        let year = date_obj.year();
                        replacement_string = year.to_string();
                    }, 
                }
            }
        } else if mode == "only_date" {
            let date_obj_wrap = get_date_object_only_date(&date);
            
            if date_obj_wrap.is_ok() {
                let date_obj = date_obj_wrap.unwrap();
                eprintln!("DateParseFunction.replace :: date_obj: {:?}", &date_obj);
                match date_parse_option {
                    DateParseOption::Day => {
                        let day = date_obj.day();
                        replacement_string = day.to_string();
                    },
                    DateParseOption::Week => {
                        let week = date_obj.iso_week().week();
                        replacement_string = week.to_string();
                    },
                    DateParseOption::WeekDay => {
                        let weekday = date_obj.weekday();
                        replacement_string = weekday.to_string();
                        is_string_output = true;
                    }, 
                    DateParseOption::Month => {
                        let month = date_obj.month();
                        replacement_string = month.to_string();
                    },
                    DateParseOption::Year => {
                        let year = date_obj.year();
                        replacement_string = year.to_string();
                    }, 
                }
            }
        }
        
        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        if is_string_output == true {
            formula = format!("\"{}\"", formula); 
        }
        return formula;
    }
    pub fn do_replace(
        function_text: &String, 
        date_parse_option: DateParseOption, 
        data_map: HashMap<String, String>, 
        mut formula: String
    ) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = DateParseFunction::defaults(
            &function_text, 
            date_parse_option
        );
        formula = concat_obj.replace(formula, data_map.clone());
        return formula
    }
}

// NOW()
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NowFunction {
    pub function_text: String,
}
impl NowFunction {
    pub fn defaults(function_text: &String) -> NowFunction {
        // NOW()
        let obj = Self{
            function_text: function_text.clone(),
        };
        return obj
    }
    pub fn validate(&self) -> bool {
        let expr: Regex = RE_NOW.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    pub fn do_validate(function_text: &String, validate_tuple: (u32, Vec<String>)) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let obj = NowFunction::defaults(
            &function_text
        );
        let check = obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_NOW));
        }
        return (number_fails, failed_functions);
    }
    pub fn replace(&mut self, formula: String) -> String {
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();
        let now_date_obj = Utc::now();
        let replacement_string = now_date_obj.to_rfc3339();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        formula = format!("\"{}\"", formula);
        return formula;
    }
    pub fn do_replace(
        function_text: &String, 
        mut formula: String
    ) -> String {
        let mut obj = NowFunction::defaults(
            &function_text, 
        );
        formula = obj.replace(formula);
        return formula
    }
}

// TODAY()
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TodayFunction {
    pub function_text: String,
}
impl TodayFunction {
    pub fn defaults(function_text: &String) -> TodayFunction {
        // NOW()
        let obj = Self{
            function_text: function_text.clone(),
        };
        return obj
    }
    pub fn validate(&self) -> bool {
        let expr: Regex = RE_TODAY.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    pub fn do_validate(function_text: &String, validate_tuple: (u32, Vec<String>)) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let obj = TodayFunction::defaults(
            &function_text
        );
        let check = obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_TODAY));
        }
        return (number_fails, failed_functions);
    }
    pub fn replace(&mut self, formula: String) -> String {
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();
        let today_date_obj = Utc::today();
        let month_short = &today_date_obj.format("%b").to_string();
        let day = &today_date_obj.day();
        let year = &today_date_obj.year();
        let replacement_string = format!("{day}-{month_short}-{year}", 
            day=day,
            month_short=month_short,
            year=year
        );
        // 24-AUG-2021
        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        formula = format!("\"{}\"", formula);
        return formula;
    }
    pub fn do_replace(
        function_text: &String, 
        mut formula: String
    ) -> String {
        let mut obj = TodayFunction::defaults(
            &function_text, 
        );
        formula = obj.replace(formula);
        return formula
    }
}

// DAYS(end_date, start_date)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DaysFunction {
    pub function_text: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub start_date_ref: Option<String>,
    pub end_date_ref: Option<String>,
}
impl DaysFunction {
    pub fn defaults(function_text: &String) -> DaysFunction {
        // DAYS("12-SEP-2021", "5-FEB-2021")
        // DAYS({Column A}, {Column B})
        let matches = RE_DAYS.captures(&function_text).unwrap();
        let start_date = matches.name("start_date");
        let end_date = matches.name("end_date");
        let start_date_ref = matches.name("start_date_ref");
        let end_date_ref = matches.name("end_date_ref");
        let mut start_date_wrap: Option<String> = None;
        let mut end_date_wrap: Option<String> = None;
        let mut start_date_ref_wrap: Option<String> = None;
        let mut end_date_ref_wrap: Option<String> = None;
        if start_date.is_some() {
            start_date_wrap = Some(start_date.unwrap().as_str().to_string())
        }
        if end_date.is_some() {
            end_date_wrap = Some(end_date.unwrap().as_str().to_string())
        }
        if start_date_ref.is_some() {
            start_date_ref_wrap = Some(start_date_ref.unwrap().as_str().to_string())
        }
        if end_date_ref.is_some() {
            end_date_ref_wrap = Some(end_date_ref.unwrap().as_str().to_string())
        }
        let obj = Self{
            function_text: function_text.clone(),
            start_date: start_date_wrap,
            end_date: end_date_wrap,
            start_date_ref: start_date_ref_wrap,
            end_date_ref: end_date_ref_wrap,
        };
        return obj
    }
    pub fn validate(&self) -> bool {
        let expr: Regex = RE_DAYS.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    pub fn do_validate(function_text: &String, validate_tuple: (u32, Vec<String>)) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let obj = DaysFunction::defaults(
            &function_text
        );
        let check = obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_DAYS));
        }
        return (number_fails, failed_functions);
    }
    pub fn replace(&mut self, formula: String, data_map: HashMap<String, String>) -> String {
        let data_map = data_map.clone();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();
        let start_date = self.start_date.clone();
        let end_date = self.end_date.clone();
        let start_date_ref = self.start_date_ref.clone();
        let end_date_ref = self.end_date_ref.clone();
        let number_days: i64;
        if start_date.is_some() == true {
            let start_date = start_date.unwrap();
            let end_date = end_date.unwrap();
            let start_date = start_date.replace("\"", "");
            let end_date = end_date.replace("\"", "");
            let start_date_obj = NaiveDate::parse_from_str(
                start_date.as_str(), 
                "%d-%b-%Y"
            ).unwrap();
            let end_date_obj = NaiveDate::parse_from_str(
                end_date.as_str(), 
                "%d-%b-%Y"
            ).unwrap();
            let duration = end_date_obj.signed_duration_since(start_date_obj);
            number_days = duration.num_days();
        } else {
            let start_date_ref = start_date_ref.unwrap();
            let end_date_ref = end_date_ref.unwrap();
            let function_attr = FunctionAttribute::defaults(&start_date_ref, 
                Some(true));
            let start_date_ref_processed = function_attr.replace(data_map.clone()).item_processed.unwrap();
            let function_attr = FunctionAttribute::defaults(&end_date_ref, 
                Some(true));
            let end_date_ref_processed = function_attr.replace(data_map.clone()).item_processed.unwrap();
            let start_date_obj = NaiveDate::parse_from_str(
                start_date_ref_processed.as_str(), 
                "%d-%b-%Y"
            ).unwrap();
            let end_date_obj = NaiveDate::parse_from_str(
                end_date_ref_processed.as_str(), 
                "%d-%b-%Y"
            ).unwrap();
            let duration = end_date_obj.signed_duration_since(start_date_obj);
            number_days = duration.num_days();
        }

        let replacement_string = number_days.to_string();
        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        return formula;
    }
    pub fn do_replace(
        function_text: &String, 
        data_map: HashMap<String, String>,
        mut formula: String
    ) -> String {
        let data_map = data_map.clone();
        let mut obj = DaysFunction::defaults(
            &function_text, 
        );
        formula = obj.replace(formula, data_map.clone());
        return formula
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DateUnits {
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
    Days,
    Weeks,
    Months,
    Quarters,
    Years,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DateDeltaOperation {
    Add,
    Diff,
}

// DATEADD(date, number, units)
// DATEDIF(date, number, units)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DateAddDiffFunction {
    pub function_text: String,
    pub date: Option<String>,
    pub date_ref: Option<String>,
    pub number: Option<u32>,
    pub units: Option<DateUnits>,
    pub operation: DateDeltaOperation,
}
impl DateAddDiffFunction {
    pub fn defaults(function_text: &String, operation: DateDeltaOperation) -> DateAddDiffFunction {
        let matches: Captures;
        match operation {
            DateDeltaOperation::Add => {
                matches = RE_DATEADD.captures(&function_text).unwrap();
            },
            DateDeltaOperation::Diff => {
                matches = RE_DATEDIF.captures(&function_text).unwrap();
            },
        }
        // date
        let attr_date = matches.name("date");
        let attr_number = matches.name("number");
        let attr_units = matches.name("units");
        // date with time
        let attr_datetime = matches.name("datetime");
        let attr_dt_number = matches.name("dt_number");
        let attr_dt_units = matches.name("dt_units");
        // reference to column with date
        let attr_date_ref = matches.name("date_ref");
        let attr_ref_number = matches.name("ref_number");
        let attr_ref_units = matches.name("ref_units");
        
        let mut date_wrap: Option<String> = None;
        let mut date_ref_wrap: Option<String> = None;
        let mut number_wrap: Option<u32> = None;
        let mut units_wrap: Option<DateUnits> = None;
        let mut units_str_wrap: Option<&str> = None;
        if attr_date.is_some() && attr_number.is_some() && attr_units.is_some() {
            date_wrap = Some(attr_date.unwrap().as_str().to_string());
            number_wrap = Some(FromStr::from_str(attr_number.unwrap().as_str()).unwrap());
            units_str_wrap = Some(attr_units.unwrap().as_str());
        } else if attr_datetime.is_some() && attr_dt_number.is_some() && attr_dt_units.is_some() {
            date_wrap = Some(attr_datetime.unwrap().as_str().to_string());
            number_wrap = Some(FromStr::from_str(attr_dt_number.unwrap().as_str()).unwrap());
            units_str_wrap = Some(attr_dt_units.unwrap().as_str());
        } else if attr_date_ref.is_some() && attr_ref_number.is_some() && attr_ref_units.is_some() {
            date_ref_wrap = Some(attr_date_ref.unwrap().as_str().to_string());
            number_wrap = Some(FromStr::from_str(attr_ref_number.unwrap().as_str()).unwrap());
            units_str_wrap = Some(attr_ref_units.unwrap().as_str());
        }
        if units_str_wrap.is_some() {
            let units_str = units_str_wrap.unwrap();
            let units_str = units_str.replace("\"", "");
            let units_str = units_str.as_str();
            let units = match units_str {
                "milliseconds" => DateUnits::Milliseconds,
                "seconds" => DateUnits::Seconds,
                "minutes" => DateUnits::Minutes,
                "hours" => DateUnits::Hours,
                "days" => DateUnits::Days,
                "weeks" => DateUnits::Weeks,
                "months" => DateUnits::Months,
                "quarters" => DateUnits::Quarters,
                "years" => DateUnits::Years,
                _ => DateUnits::Days,
            };
            units_wrap = Some(units);
        }
        let obj = Self{
            function_text: function_text.clone(),
            date: date_wrap,
            date_ref: date_ref_wrap,
            number: number_wrap,
            units: units_wrap,
            operation: operation,
        };
        return obj
    }
    pub fn validate(&self) -> bool {
        let operation = self.operation.clone();
        let expr: Regex;
        match operation {
            DateDeltaOperation::Add => {
                expr = RE_DATEADD.clone();
            },
            DateDeltaOperation::Diff => {
                expr = RE_DATEDIF.clone();
            },
        }
        let function_text = self.function_text.clone();
        let mut check = expr.is_match(&function_text);
        if check == false {
            return check
        }
        // parse date, datetime
        let date = self.date.clone().unwrap();
        let has_time = *&date.find(" ").is_some();
        if has_time == true {
            // We have date and time
            let date_obj_wrap = get_date_object_human_time(&date);
            if date_obj_wrap.is_err() {
                check = false;
            }
        } else {
            let date_obj_wrap = get_date_object_only_date(&date);
            if date_obj_wrap.is_err() {
                check = false;
            }
        }
        return check
    }
    pub fn do_validate(
        function_text: &String, 
        operation: DateDeltaOperation, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let operation_ = operation.clone();
        let obj = DateAddDiffFunction::defaults(
            &function_text, operation
        );
        let check = obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            match operation_ {
                DateDeltaOperation::Add => {
                    failed_functions.push(String::from(FUNCTION_DATEADD));
                },
                DateDeltaOperation::Diff => {
                    failed_functions.push(String::from(FUNCTION_DATEDIF));
                },
            }
        }
        return (number_fails, failed_functions);
    }
    pub fn replace(&mut self, formula: String, data_map: HashMap<String, String>) -> String {
        let data_map = data_map.clone();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();
        let operation = self.operation.clone();
        let date = self.date.clone();
        let number = self.number.clone().unwrap();
        let number: i64 = FromStr::from_str(number.to_string().as_str()).unwrap();
        let units = self.units.clone().unwrap();
        let replacement_string: String;
        let new_date: DateTime<FixedOffset>;
        let date_obj: DateTime<FixedOffset>;
        let has_time: bool;
        if date.is_some() {
            let date = date.unwrap();
            has_time = *&date.find(" ").is_some();
            if has_time == true {
                date_obj = get_date_object_human_time(&date).unwrap();
            } else {
                date_obj = get_date_object_only_date(&date).unwrap();
            }
        } else {
            let date_ref = self.date_ref.clone().unwrap();
            // I need to get from data_map the date string
            let function_attr = FunctionAttribute::defaults(
                &date_ref, 
                Some(true)
            );
            let date = function_attr.replace(data_map.clone()).item_processed.unwrap();
            has_time = *&date.find(" ").is_some();
            if has_time == true {
                date_obj = get_date_object_human_time(&date).unwrap();
            } else {
                date_obj = get_date_object_only_date(&date).unwrap();
            }
        }
        // I could use operation to get date_obj????
        match units {
            DateUnits::Milliseconds => {
                match operation {
                    DateDeltaOperation::Add => {
                        new_date = date_obj + Duration::microseconds(number);
                    },
                    DateDeltaOperation::Diff => {
                        new_date = date_obj - Duration::microseconds(number);
                    },
                }
            },
            DateUnits::Seconds => {
                match operation {
                    DateDeltaOperation::Add => {
                        new_date = date_obj + Duration::seconds(number);
                    },
                    DateDeltaOperation::Diff => {
                        new_date = date_obj - Duration::seconds(number);
                    },
                }
            },
            DateUnits::Minutes => {
                match operation {
                    DateDeltaOperation::Add => {
                        new_date = date_obj + Duration::minutes(number);
                    },
                    DateDeltaOperation::Diff => {
                        new_date = date_obj - Duration::minutes(number);
                    },
                }
            },
            DateUnits::Hours => {
                match operation {
                    DateDeltaOperation::Add => {
                        new_date = date_obj + Duration::hours(number);
                    },
                    DateDeltaOperation::Diff => {
                        new_date = date_obj - Duration::hours(number);
                    },
                }
            },
            DateUnits::Days => {
                match operation {
                    DateDeltaOperation::Add => {
                        new_date = date_obj + Duration::days(number);
                    },
                    DateDeltaOperation::Diff => {
                        new_date = date_obj - Duration::days(number);
                    },
                }
            },
            DateUnits::Weeks => {
                match operation {
                    DateDeltaOperation::Add => {
                        new_date = date_obj + Duration::weeks(number);
                    },
                    DateDeltaOperation::Diff => {
                        new_date = date_obj - Duration::weeks(number);
                    },
                }
            },
            DateUnits::Months => {
                // Here same day in a month (+/-)
                match operation {
                    DateDeltaOperation::Add => {
                        // I can have date or datetime
                        let month = date_obj.month() + 1;
                        new_date = date_obj.with_month(month).unwrap();
                    },
                    DateDeltaOperation::Diff => {
                        let month = date_obj.month() - 1;
                        new_date = date_obj.with_month(month).unwrap();
                    },
                }
            },
            DateUnits::Quarters => {
                // Same day in three months (+/-)
                match operation {
                    DateDeltaOperation::Add => {
                        let month = date_obj.month() + 3;
                        new_date = date_obj.with_month(month).unwrap();
                    },
                    DateDeltaOperation::Diff => {
                        let month = date_obj.month() - 3;
                        new_date = date_obj.with_month(month).unwrap();
                    },
                }
            },
            DateUnits::Years => {
                match operation {
                    DateDeltaOperation::Add => {
                        new_date = date_obj + Duration::days(number*365);
                    },
                    DateDeltaOperation::Diff => {
                        new_date = date_obj - Duration::days(number*365);
                    },
                }
            },
        }
        if has_time == true {
            replacement_string = new_date.to_rfc3339();
        } else {
            replacement_string = new_date.format("%d-%b-%Y").to_string();
        }
        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        formula = format!("\"{}\"", formula);
        return formula;
    }
    pub fn do_replace(
        function_text: &String, 
        operation: DateDeltaOperation, 
        data_map: HashMap<String, String>,
        mut formula: String
    ) -> String {
        let data_map = data_map.clone();
        let mut obj = DateAddDiffFunction::defaults(
            &function_text, operation
        );
        formula = obj.replace(formula, data_map.clone());
        return formula
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DateFormatFunction {
    pub function_text: String,
    pub date: Option<String>,
    pub datetime: Option<String>,
    pub date_ref: Option<String>,
    pub format: Option<String>,
}
impl DateFormatFunction {
    pub fn defaults(function_text: &String) -> DateFormatFunction {
        let matches= RE_DATEFMT.captures(&function_text).unwrap();

        // date
        let date = matches.name("date");
        let datetime = matches.name("datetime");
        let date_ref = matches.name("ref");
        // format
        let fmt = matches.name("format");
        let dt_fmt = matches.name("dt_format");
        let ref_fmt = matches.name("ref_format");

        let mut date_wrap: Option<String> = None;
        let mut datetime_wrap: Option<String> = None;
        let mut date_ref_wrap: Option<String> = None;
        let mut format_wrap: Option<String> = None;

        if date.is_some() && fmt.is_some() {
            date_wrap = Some(date.unwrap().as_str().to_string());
            format_wrap = Some(fmt.unwrap().as_str().to_string());
        } else if datetime.is_some() && dt_fmt.is_some() {
            datetime_wrap = Some(datetime.unwrap().as_str().to_string());
            format_wrap = Some(dt_fmt.unwrap().as_str().to_string());
        } else if date_ref.is_some() && ref_fmt.is_some() {
            date_ref_wrap = Some(date_ref.unwrap().as_str().to_string());
            format_wrap = Some(ref_fmt.unwrap().as_str().to_string());
        }

        let obj = Self{
            function_text: function_text.clone(),
            date: date_wrap,
            datetime: datetime_wrap,
            date_ref: date_ref_wrap,
            format: format_wrap,
        };
        return obj
    }
    pub fn get_rust_format(&self, date_obj: DateTime<FixedOffset>) -> String {
        let mut format = self.format.clone().unwrap();
        let mut fmt_map: HashMap<&str, &str> = HashMap::new();
        fmt_map.insert("M", "%_m");
        fmt_map.insert("Mo", "");
        fmt_map.insert("MM", "%m");
        fmt_map.insert("MMM", "%b");
        fmt_map.insert("MMMM", "%B");
        fmt_map.insert("D", "%_d");
        fmt_map.insert("Do", "");
        fmt_map.insert("DD", "%d");
        fmt_map.insert("d", "%w");
        fmt_map.insert("ddd", "%a");
        fmt_map.insert("dddd", "%A");
        fmt_map.insert("w", "%_W");
        fmt_map.insert("ww", "%W");
        fmt_map.insert("YY", "%y");
        fmt_map.insert("YYYY", "%Y");
        fmt_map.insert("H", "%k");
        fmt_map.insert("HH", "%H");
        fmt_map.insert("h", "%l");
        fmt_map.insert("hh", "%I");
        fmt_map.insert("A", "%P");
        fmt_map.insert("a", "%p");
        fmt_map.insert("m", "%_M");
        fmt_map.insert("mm", "%M");
        fmt_map.insert("s", "%_S");
        fmt_map.insert("ss", "%S");
        fmt_map.insert("S", "%.f");
        fmt_map.insert("Z", "%:z");
        fmt_map.insert("ZZ", "%z");
        fmt_map.insert("X", "%s");
        fmt_map.insert("x", "%s");

        // additional processing: x, ww, Mo, Do
        for fmt_key in fmt_map.keys() {
            let fmt_key = fmt_key.clone();
            let mut find_key = String::from("{");
            find_key.push_str(fmt_key);
            find_key.push_str("}");
            let has_key = format.find(find_key.as_str());
            let value = fmt_map.get(fmt_key);
            if has_key.is_some() && value.is_some() {
                let value = value.unwrap();
                if fmt_key == "x" {
                    // Timestamp in miliseconds
                    let value_int: u64 = FromStr::from_str(value).unwrap();
                    let value_str = (value_int*1000).to_string();
                    let value_str = value_str.as_str();
                    format = format.replace(find_key.as_str(), value_str);
                } else if fmt_key == "ww" {
                    // Week number starts with 01 instead of 00
                    let mut value_int: u64 = FromStr::from_str(value).unwrap();
                    value_int += 1;
                    let value_str = value_int.to_string();
                    let mut value_str = value_str.as_str();
                    let value_str_: String = format!("0{}", value_str).clone();
                    if value_str.len() < 2 {
                        value_str = value_str_.as_str();
                    }
                    format = format.replace(find_key.as_str(), value_str);
                } else if fmt_key == "Mo" {
                    // Number of month as 1st, 2nd 13th, 31st, etc...
                    let number_month = date_obj.month();
                    let ord_number_month = Ordinal(number_month).to_string();
                    let ord_number_month = ord_number_month.as_str();
                    format = format.replace(find_key.as_str(), ord_number_month);
                } else if fmt_key == "Do" {
                    // Number of day as 1st, 2nd 13th, 31st, etc...
                    let number_day = date_obj.day();
                    let ord_number_day = Ordinal(number_day).to_string();
                    let ord_number_day = ord_number_day.as_str();
                    format = format.replace(find_key.as_str(), ord_number_day);
                } else {
                    format = format.replace(find_key.as_str(), value);
                }
            }
        }
        return format;
    }
    pub fn validate(&self) -> bool {
        let expr: Regex = RE_DATEFMT.clone();
        let function_text = self.function_text.clone();
        let mut check = expr.is_match(&function_text);
        let date = self.date.clone();
        let datetime = self.datetime.clone();
        let format = self.format.clone();
        if date.is_none() && datetime.is_none() {
            check = false;
        }
        if format.is_none() {
            check = false;
        }
        if check == false {
            return check
        }

        let mut date_obj_wrap: Option<DateTime<FixedOffset>> = None;
        if date.is_some() && datetime.is_none() {
            // I have date
            let date = date.unwrap();
            let date_obj_wrap_ = get_date_object_only_date(&date);
            if date_obj_wrap_.is_err() {
                return false;
            }
            date_obj_wrap = Some(date_obj_wrap_.unwrap());
        } else if datetime.is_some() && date.is_none() {
            // I have date with time
            let date = datetime.unwrap();
            let date_obj_wrap_ = get_date_object_human_time(&date);
            if date_obj_wrap_.is_err() {
                return false;
            }
            date_obj_wrap = Some(date_obj_wrap_.unwrap());
        }

        let format = self.get_rust_format(date_obj_wrap.unwrap());
        let date_obj = date_obj_wrap.unwrap();
        let date_string = date_obj.format(format.as_str()).to_string();
        if date_string.len() == 0 {
            check = false;
        }
        return check
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>),
    ) -> (u32, Vec<String>) {
        let (number_fails, failed_functions) = validate_tuple.clone();
        let obj = DateFormatFunction::defaults(
            &function_text
        );
        let check = obj.validate();
        let mut number_fails = number_fails.clone();
        let mut failed_functions = failed_functions.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_DATEFMT));
        }
        return (number_fails, failed_functions);
    }
    pub fn replace(&mut self, formula: String, data_map: HashMap<String, String>) -> String {
        let data_map = data_map.clone();
        let function_text = self.function_text.clone();
        let date = self.date.clone();
        let datetime = self.datetime.clone();
        let date_ref = self.date_ref.clone();
        let mut formula = formula.clone();

        let date_str: String;
        let mut date_obj_wrap: Option<DateTime<FixedOffset>> = None;
        // let has_time: bool = false;
        if date_ref.is_some() {
            eprintln!("DateFmt.replace :: data map length: {}", &data_map.len());
            // let date_ref = date_ref.unwrap();
            // let function_attr = FunctionAttribute::defaults(
            //     &date_ref, 
            //     Some(true)
            // );
            // date_str = function_attr.replace(data_map.clone()).item_processed.unwrap();
            // What I have at database?????? Need to check with table config
            // Might be only date, or datetime
            // TODO: Do when we have the date fields
        } else {
            if date.is_some() && datetime.is_none() {
                eprintln!("");
                date_str = date.unwrap();
                let date_obj_wrap_ = get_date_object_only_date(&date_str);
                date_obj_wrap = Some(date_obj_wrap_.unwrap());
            } else if datetime.is_some() && date.is_none() {
                date_str = datetime.unwrap();
                let date_obj_wrap_ = get_date_object_human_time(&date_str);
                date_obj_wrap = Some(date_obj_wrap_.unwrap());
            }
        }
        let date_obj = date_obj_wrap.unwrap();
        let format = self.get_rust_format(date_obj);
        let date_string = date_obj.format(format.as_str()).to_string();
        let replacement_string: String = date_string;

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        return formula;
    }
    pub fn do_replace(
        function_text: &String, 
        data_map: HashMap<String, String>,
        mut formula: String
    ) -> String {
        let data_map = data_map.clone();
        let mut obj = DateFormatFunction::defaults(
            &function_text
        );
        formula = obj.replace(formula, data_map.clone());
        return formula
    }
}


pub fn has_month_short(date_str: &String) -> bool {
    let date_str = date_str.clone().to_lowercase();
    let months: Vec<&str> = [
        "jan",
        "feb",
        "mar",
        "apr",
        "may",
        "jun",
        "jul",
        "aug",
        "sep",
        "oct",
        "nov",
        "dec",
    ].to_vec();
    let mut check: bool = false;
    for month in months {
        let has_month = date_str.find(&month);
        if has_month.is_some() {
            check = true;
            break
        }
    }
    return check
}

pub fn get_standard_date_short_month(date_str: &String) -> String {
    let mut date_str = date_str.clone();
    // 12-AUG-2021 12:00:00 => 12-Aug-2021 12:00:00
    // 12-AUG-2021 => 12-Aug-2021
    let mut map: HashMap<&str, &str> = HashMap::new();
    map.insert("JAN", "Jan");
    map.insert("FEB", "Feb");
    map.insert("MAR", "Mar");
    map.insert("APR", "Apr");
    map.insert("MAY", "May");
    map.insert("JUN", "Jun");
    map.insert("JUL", "Jul");
    map.insert("AUG", "Aug");
    map.insert("SEP", "Sep");
    map.insert("OCT", "Oct");
    map.insert("NOV", "Nov");
    map.insert("DEC", "Dec");
    for key in map.keys() {
        let key = key.clone();
        let has_item = date_str.find(key);
        if has_item.is_some() {
            let replaced_item = map.get(key).unwrap().clone();
            date_str = date_str.replace(key, replaced_item);
            break
        }
    }
    return date_str
}

pub fn get_date_object_human_time(date_string: &String) -> Result<DateTime<FixedOffset>, PlanetError> {
    let date_string = date_string.clone();
    let mut date_string = date_string.replace("\"", "");
    // DAY("2021-08-25 06:26:00")
    // DAY("25-Aug-2021 06:26:00")
    // DAY("25-AUG-2021 06:26:00")

    // need map AUG -> Aug
    // Check str month in order to parse
    let has_short_month = has_month_short(&date_string);
    if has_short_month {
        let mut date_string_ = get_standard_date_short_month(&date_string);
        date_string_ = format!("{}+0000", &date_string_);
        // date_string now is "25-Aug-2021 06:26:00"
        let date_obj = DateTime::parse_from_str(
            &date_string_, 
            "%d-%b-%Y %H:%M:%S%z"
        );
        if date_obj.is_ok() {
            return Ok(date_obj.unwrap())
        }
    } else {
        // DAY("2021-08-25 06:26:00")
        // let no_timezone = NaiveDateTime::parse_from_str("2015-09-05 23:56:04", "%Y-%m-%d %H:%M:%S");
        date_string = format!("{}+0000", &date_string);
        let date_obj = DateTime::parse_from_str(
            &date_string, 
            "%Y-%m-%d %H:%M:%S%z"
        );
        if date_obj.is_ok() {
            return Ok(date_obj.unwrap())
        }
    }
    return Err(
        PlanetError::new(
            500, 
            Some(tr!("Could not parse date")),
        )
    );
}

pub fn get_date_object_only_date(date_string: &String) -> Result<DateTime<FixedOffset>, PlanetError> {
    let date_string = date_string.clone();
    let date_string = date_string.replace("\"", "");
    let mut date_string_ = get_standard_date_short_month(&date_string);
    date_string_ = format!("{} 00:00:00+0000", &date_string_);
    let date_obj = DateTime::parse_from_str(
        &date_string_, 
        "%d-%b-%Y %H:%M:%S%z"
    );
    if date_obj.is_ok() {
        return Ok(date_obj.unwrap())
    }
    return Err(
        PlanetError::new(
            500, 
            Some(tr!("Could not parse date")),
        )
    );
}

pub fn get_date_object_iso(date_string: &String) -> Result<DateTime<FixedOffset>, PlanetError> {
    let date_string = date_string.clone();
    let date_string = date_string.replace("\"", "");
    let date_obj_wrap = DateTime::parse_from_rfc3339(&date_string);
    if date_obj_wrap.is_ok() {
        return Ok(date_obj_wrap.unwrap())
    }
    return Err(
        PlanetError::new(
            500, 
            Some(tr!("Could not parse date")),
        )
    );
}