use std::str::FromStr;
use regex::Regex;
use std::{collections::HashMap};
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;
// use chrono::{DateTime, Datelike, FixedOffset, Local, NaiveDate, NaiveDateTime, Timelike, Utc};
use chrono::{DateTime, Datelike, Timelike, Utc, NaiveDate, Date};

use crate::functions::FunctionAttribute;

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
    static ref RE_SECOND: Regex = Regex::new(r#"SECOND\((?P<date>"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\+\d{2}:\d{2}")|SECOND\((?P<date_ref>\{[\w\s]+\})\)"#).unwrap();
    static ref RE_MINUTE: Regex = Regex::new(r#"MINUTE\((?P<date>"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\+\d{2}:\d{2}")|MINUTE\((?P<date_ref>\{[\w\s]+\})\)"#).unwrap();
    static ref RE_HOUR: Regex = Regex::new(r#"HOUR\((?P<date>"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\+\d{2}:\d{2}")|HOUR\((?P<date_ref>\{[\w\s]+\})\)"#).unwrap();
    static ref RE_DAY: Regex = Regex::new(r#"DAY\((?P<date>"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\+\d{2}:\d{2}")|DAY\((?P<date_ref>\{[\w\s]+\})\)"#).unwrap();
    static ref RE_WEEK: Regex = Regex::new(r#"WEEK\((?P<date>"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\+\d{2}:\d{2}")|WEEK\((?P<date_ref>\{[\w\s]+\})\)"#).unwrap();
    static ref RE_WEEKDAY: Regex = Regex::new(r#"WEEKDAY\((?P<date>"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\+\d{2}:\d{2}")|WEEKDAY\((?P<date_ref>\{[\w\s]+\})\)"#).unwrap();
    static ref RE_MONTH: Regex = Regex::new(r#"MONTH\((?P<date>"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\+\d{2}:\d{2}")|MONTH\((?P<date_ref>\{[\w\s]+\})\)"#).unwrap();
    static ref RE_YEAR: Regex = Regex::new(r#"YEAR\((?P<date>"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\+\d{2}:\d{2}")|YEAR\((?P<date_ref>\{[\w\s]+\})\)"#).unwrap();
    static ref RE_NOW: Regex = Regex::new(r#"NOW\(\)"#).unwrap();
    static ref RE_TODAY: Regex = Regex::new(r#"TODAY\(\)"#).unwrap();
    static ref RE_DAYS: Regex = Regex::new(r#"(DAYS\((?P<end_date>"\d{1,2}-[a-zA-Z]{3}-\d{4}"),[\s]+(?P<start_date>"\d{1,2}-[a-zA-Z]{3}-\d{4}"))\)|DAYS\(((?P<end_date_ref>\{[\w\s]+\}),[\s]+(?P<start_date_ref>\{[\w\s]+\}))\)"#).unwrap();
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
    Second,
    Minute,
    Hour,
    Day,
    Week,
    WeekDay,
    Month,
    Year,
}

// DAY($date_str)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DateParseFunction {
    pub function_text: String,
    pub date_string: Option<String>,
    pub date_ref: Option<String>,
    pub parse_option: DateParseOption,
}
impl DateParseFunction {
    pub fn defaults(function_text: &String, date_parse_option: DateParseOption) -> DateParseFunction {
        // DAY("2021-08-21T00:00:00+00:00") : This has date as in constructor
        // DAY({Column Date}) : This has no date in constructor
        // SECOND, MINUTE, HOUR, WEEK, MONTH and YEAR functions handler

        let matches = RE_DAY.captures(function_text).unwrap();
        let mut date_string_wrap: Option<String> = None;
        let mut date_ref_wrap: Option<String> = None;
        let date_parse_option = date_parse_option.clone();
        if matches.name("date").is_some() {
            let date = matches.name("date").unwrap().as_str().to_string();
            date_string_wrap = Some(date);
        } else {
            let date = matches.name("date_ref").unwrap().as_str().to_string();
            date_ref_wrap = Some(date)
        }
        let obj = Self{
            function_text: function_text.clone(),
            date_string: date_string_wrap,
            date_ref: date_ref_wrap,
            parse_option: date_parse_option,
        };
        return obj
    }
    pub fn validate(&self, date_parse_option: DateParseOption) -> bool {
        let expr: Regex;
        match date_parse_option {
            DateParseOption::Second => {
                expr = RE_SECOND.clone();
            },
            DateParseOption::Minute => {
                expr = RE_MINUTE.clone();
            },
            DateParseOption::Hour => {
                expr = RE_HOUR.clone();
            },
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
        let check = expr.is_match(&function_text);
        return check
    }
    pub fn do_validate(function_text: &String, date_parse_option: DateParseOption, number_fails: &u32) -> u32 {
        let obj = DateParseFunction::defaults(
            &function_text, date_parse_option.clone()
        );
        let check = obj.validate(date_parse_option.clone());
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
        }
        return number_fails;
    }
    pub fn replace(&mut self, formula: String, data_map: HashMap<String, String>) -> String {
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();
        let mut replacement_string = String::from("");
        let date_wrap = self.date_string.clone();
        let mut date: String;
        let date_parse_option = self.parse_option.clone();
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
        let date_obj_wrap = DateTime::parse_from_rfc3339(&date);
        let mut is_string_output = false;
        if date_obj_wrap.is_ok() {
            let date_obj = date_obj_wrap.unwrap();
            match date_parse_option {
                DateParseOption::Second => {
                    let second = date_obj.second();
                    replacement_string = second.to_string();
                },
                DateParseOption::Minute => {
                    let minute = date_obj.minute();
                    replacement_string = minute.to_string();
                },
                DateParseOption::Hour => {
                    let hour = date_obj.hour();
                    replacement_string = hour.to_string();
                },
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
    pub fn do_validate(function_text: &String, number_fails: &u32) -> u32 {
        let obj = NowFunction::defaults(
            &function_text
        );
        let check = obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
        }
        return number_fails;
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
    pub fn do_validate(function_text: &String, number_fails: &u32) -> u32 {
        let obj = TodayFunction::defaults(
            &function_text
        );
        let check = obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
        }
        return number_fails;
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
    pub fn do_validate(function_text: &String, number_fails: &u32) -> u32 {
        let obj = DaysFunction::defaults(
            &function_text
        );
        let check = obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
        }
        return number_fails;
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