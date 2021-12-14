use std::{str::FromStr};
use regex::{Regex, Captures};
use std::{collections::HashMap};
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;
use chrono::{DateTime, Datelike, FixedOffset, NaiveDate, Utc, Duration, Timelike};
use tr::tr;
use ordinal::Ordinal;
use crate::functions::*;

lazy_static! {
    pub static ref RE_DATE: Regex = Regex::new(r#"^DATE\([\s\n\t]{0,}(?P<year>(([\s\d+-/*]*)|(\{[\w\s]+\})|([A-Z]+\(.[^)]*\))))[\s\n\t]{0,},[\s\n\t]{0,}(?P<month>(([\s\d+-/*]+)|(\{[\w\s]+\})|([A-Z]+\(.[^)]*\))))[\s\n\t]{0,},[\s\n\t]{0,}(?P<day>(([\s\d+-/*]+)|(\{[\w\s]+\})|([A-Z]+\(.[^)]*\))))[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_DAY: Regex = Regex::new(r#"^DAY\([\s\n\t]{0,}(?P<date>(("\d{4}-\d{2}-\d{2}T{0,1}[\s]{0,1}\d{2}:\d{2}:\d{2}(\+\d{2}:\d{2}){0,1}")|([A-Z]+\(.[^)]*\))))[\s\n\t]{0,}\)|DAY\([\s\n\t]{0,}(?P<date_alt>"\d{2}-[a-zA-Z]{3}-\d{4}([\s]{0,1}\d{2}:\d{2}:\d{2}){0,1}(\+\d{2}:\d{2}){0,1}")[\s\n\t]{0,}\)|^DAY\([\s\n\t]{0,}(?P<date_ref>\{[\w\s]+\})[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_WEEK: Regex = Regex::new(r#"^WEEK\([\s\n\t]{0,}(?P<date>(("\d{4}-\d{2}-\d{2}T{0,1}[\s]{0,1}\d{2}:\d{2}:\d{2}(\+\d{2}:\d{2}){0,1}")|([A-Z]+\(.[^)]*\))))[\s\n\t]{0,}\)|WEEK\([\s\n\t]{0,}(?P<date_alt>"\d{2}-[a-zA-Z]{3}-\d{4}([\s]{0,1}\d{2}:\d{2}:\d{2}){0,1}(\+\d{2}:\d{2}){0,1}")[\s\n\t]{0,}\)|^WEEK\([\s\n\t]{0,}(?P<date_ref>\{[\w\s]+\})[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_WEEKDAY: Regex = Regex::new(r#"^WEEKDAY\([\s\n\t]{0,}(?P<date>(("\d{4}-\d{2}-\d{2}T{0,1}[\s]{0,1}\d{2}:\d{2}:\d{2}(\+\d{2}:\d{2}){0,1}")|([A-Z]+\(.[^)]*\))))[\s\n\t]{0,}\)|WEEKDAY\([\s\n\t]{0,}(?P<date_alt>"\d{2}-[a-zA-Z]{3}-\d{4}([\s]{0,1}\d{2}:\d{2}:\d{2}){0,1}(\+\d{2}:\d{2}){0,1}")[\s\n\t]{0,}\)|^WEEKDAY\([\s\n\t]{0,}(?P<date_ref>\{[\w\s]+\})[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_MONTH: Regex = Regex::new(r#"^MONTH\([\s\n\t]{0,}(?P<date>(("\d{4}-\d{2}-\d{2}T{0,1}[\s]{0,1}\d{2}:\d{2}:\d{2}(\+\d{2}:\d{2}){0,1}")|([A-Z]+\(.[^)]*\))))[\s\n\t]{0,}\)|MONTH\([\s\n\t]{0,}(?P<date_alt>"\d{2}-[a-zA-Z]{3}-\d{4}([\s]{0,1}\d{2}:\d{2}:\d{2}){0,1}(\+\d{2}:\d{2}){0,1}")[\s\n\t]{0,}\)|^MONTH\([\s\n\t]{0,}(?P<date_ref>\{[\w\s]+\})[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_YEAR: Regex = Regex::new(r#"^YEAR\([\s\n\t]{0,}(?P<date>(("\d{4}-\d{2}-\d{2}T{0,1}[\s]{0,1}\d{2}:\d{2}:\d{2}(\+\d{2}:\d{2}){0,1}")|([A-Z]+\(.[^)]*\))))[\s\n\t]{0,}\)|YEAR\([\s\n\t]{0,}(?P<date_alt>"\d{2}-[a-zA-Z]{3}-\d{4}([\s]{0,1}\d{2}:\d{2}:\d{2}){0,1}(\+\d{2}:\d{2}){0,1}")[\s\n\t]{0,}\)|^YEAR\([\s\n\t]{0,}(?P<date_ref>\{[\w\s]+\})[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_HOUR: Regex = Regex::new(r#"^HOUR\([\s\n\t]{0,}(?P<date>(("\d{4}-\d{2}-\d{2}T{0,1}[\s]{0,1}\d{2}:\d{2}:\d{2}(\+\d{2}:\d{2}){0,1}")|([A-Z]+\(.[^)]*\))))[\s\n\t]{0,}\)|HOUR\([\s\n\t]{0,}(?P<date_alt>"\d{2}-[a-zA-Z]{3}-\d{4}([\s]{0,1}\d{2}:\d{2}:\d{2}){0,1}(\+\d{2}:\d{2}){0,1}")[\s\n\t]{0,}\)|^HOUR\([\s\n\t]{0,}(?P<date_ref>\{[\w\s]+\})[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_MINUTE: Regex = Regex::new(r#"MINUTE\([\s\n\t]{0,}(?P<date>(("\d{4}-\d{2}-\d{2}T{0,1}[\s]{0,1}\d{2}:\d{2}:\d{2}(\+\d{2}:\d{2}){0,1}")|([A-Z]+\(.[^)]*\))))[\s\n\t]{0,}\)|MINUTE\([\s\n\t]{0,}(?P<date_alt>"\d{2}-[a-zA-Z]{3}-\d{4}([\s]{0,1}\d{2}:\d{2}:\d{2}){0,1}(\+\d{2}:\d{2}){0,1}")[\s\n\t]{0,}\)|^MINUTE\([\s\n\t]{0,}(?P<date_ref>\{[\w\s]+\})[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_SECOND: Regex = Regex::new(r#"SECOND\([\s\n\t]{0,}(?P<date>(("\d{4}-\d{2}-\d{2}T{0,1}[\s]{0,1}\d{2}:\d{2}:\d{2}(\+\d{2}:\d{2}){0,1}")|([A-Z]+\(.[^)]*\))))[\s\n\t]{0,}\)|SECOND\([\s\n\t]{0,}(?P<date_alt>"\d{2}-[a-zA-Z]{3}-\d{4}([\s]{0,1}\d{2}:\d{2}:\d{2}){0,1}(\+\d{2}:\d{2}){0,1}")[\s\n\t]{0,}\)|^SECOND\([\s\n\t]{0,}(?P<date_ref>\{[\w\s]+\})[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_NOW: Regex = Regex::new(r#"^NOW\(\)"#).unwrap();
    pub static ref RE_TODAY: Regex = Regex::new(r#"^TODAY\(\)"#).unwrap();
    pub static ref RE_DAYS: Regex = Regex::new(r#"(^DAYS\([\s\n\t]{0,}(?P<start_date>(("\d{1,2}-[a-zA-Z]{3}-\d{4}")|([A-Z]+\(.[^)]*\))))[\s\n\t]{0,},[\s\n\t]{0,}(?P<end_date>(("\d{1,2}-[a-zA-Z]{3}-\d{4}")|([A-Z]+\(.[^)]*\)))))[\s\n\t]{0,}\)|^DAYS\([\s\n\t]{0,}((?P<start_date_ref>\{[\w\s]+\})[\s\n\t]{0,},[\s\n\t]{0,}(?P<end_date_ref>\{[\w\s]+\}))[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_DATEADD: Regex = Regex::new(r#"^DATEADD\([\s\n\t]{0,}(?P<date>(("\d{1,2}-[a-zA-Z]{3}-\d{4}")|([A-Z]+\(.[^)]*\))))[\s\n\t]{0,},[\s\n\t]{0,}(?P<number>(([+\-*/\d]+)|([A-Z]+\(.[^)]*\))))[\s\n\t]{0,},[\s\n\t]{0,}(?P<units>("days")|("months")|("milliseconds")|("seconds")|("minutes")|("hours")|("weeks")|("quarters")|("years"))[\s\n\t]{0,}\)|^DATEADD\([\s\n\t]{0,}(?P<datetime>"\d{1,2}-[a-zA-Z]{3}-\d{4}[\s]{1}\d{1,2}:\d{1,2}:\d{1,2}")[\s\n\t]{0,},[\s\n\t]{0,}(?P<dt_number>\d+)[\s\n\t]{0,},[\s\n\t]{0,}(?P<dt_units>("days")|("months")|("milliseconds")|("seconds")|("minutes")|("hours")|("weeks")|("quarters")|("years"))[\s\n\t]{0,}\)|^DATEADD\([\s\n\t]{0,}(?P<date_ref>\{[\w\s]+\})[\s\n\t]{0,},[\s\n\t]{0,}(?P<ref_number>\d{1,2})[\s\n\t]{0,},[\s\n\t]{0,}(?P<ref_units>("days")|("months")|("milliseconds")|("seconds")|("minutes")|("hours")|("weeks")|("quarters")|("years"))[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_DATEDIF: Regex = Regex::new(r#"^DATEDIF\([\s\n\t]{0,}(?P<date>(("\d{1,2}-[a-zA-Z]{3}-\d{4}")|([A-Z]+\(.[^)]*\))))[\s\n\t]{0,},[\s\n\t]{0,}(?P<number>(([+\-*/\d]+)|([A-Z]+\(.[^)]*\))))[\s\n\t]{0,},[\s\n\t]{0,}(?P<units>("days")|("months")|("milliseconds")|("seconds")|("minutes")|("hours")|("weeks")|("quarters")|("years"))[\s\n\t]{0,}\)|^DATEDIF\([\s\n\t]{0,}(?P<datetime>"\d{1,2}-[a-zA-Z]{3}-\d{4}[\s]{1}\d{1,2}:\d{1,2}:\d{1,2}")[\s\n\t]{0,},[\s\n\t]{0,}(?P<dt_number>\d+)[\s\n\t]{0,},[\s\n\t]{0,}(?P<dt_units>("days")|("months")|("milliseconds")|("seconds")|("minutes")|("hours")|("weeks")|("quarters")|("years"))[\s\n\t]{0,}\)|^DATEDIF\([\s\n\t]{0,}(?P<date_ref>\{[\w\s]+\})[\s\n\t]{0,},[\s\n\t]{0,}(?P<ref_number>\d{1,2})[\s\n\t]{0,},[\s\n\t]{0,}(?P<ref_units>("days")|("months")|("milliseconds")|("seconds")|("minutes")|("hours")|("weeks")|("quarters")|("years"))[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_DATEFMT: Regex = Regex::new(r#"^DATEFMT\([\s\n\t]{0,}(?P<date>(("\d{1,2}-[a-zA-Z]{3}-\d{4}")|([A-Z]+\(.[^)]*\))))[\s\n\t]{0,},[\s\n\t]{0,}(?P<format>"[\{a-zA-Z-/,:_\s\}].+")[\s\n\t]{0,}\)|^DATEFMT\([\s\n\t]{0,}(?P<datetime>"\d{1,2}-[a-zA-Z]{3}-\d{4}\s{1}\d{2}:\d{2}:\d{2}")[\s\n\t]{0,},[\s\n\t]{0,}(?P<dt_format>"[\{a-zA-Z-/,:_\s\}].+")[\s\n\t]{0,}\)|^DATEFMT\([\s\n\t]{0,}(?P<ref>\{[\w\s]+\})[\s\n\t]{0,},[\s\n\t]{0,}(?P<ref_format>"[\{a-zA-Z-/,:_\s\}].+")[\s\n\t]{0,}\)"#).unwrap();
}

pub trait DateFunction {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError>;
    fn execute(&self) -> Result<String, PlanetError>;
}

pub trait DateTimeFunction {
    fn handle(&mut self, date_time_option: DateTimeParseOption) -> Result<FunctionParse, PlanetError>;
    fn execute(&self, date_time_option: DateTimeParseOption) -> Result<String, PlanetError>;
}

pub trait DateParseFunction {
    fn handle(&mut self, date_option: DateParseOption) -> Result<FunctionParse, PlanetError>;
    fn execute(&self, date_option: DateParseOption) -> Result<String, PlanetError>;
}

pub trait DateAddDiffFunction {
    fn handle(&mut self, operation: DateDeltaOperation) -> Result<FunctionParse, PlanetError>;
    fn execute(&self, operation: DateDeltaOperation) -> Result<String, PlanetError>;
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Date {
    function: Option<FunctionParse>,
    data_map: Option<HashMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>
}
impl Date {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<HashMap<String, String>>
    ) -> Self {
        return Self{function: function, data_map: data_map, attributes: None};
    }
}
impl DateFunction for Date {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // DATE(year,month,day)
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_DATE;
        let mut function = function_parse.clone();
        let data_map_wrap = data_map.clone();
        let (
            function_text_wrap, 
            function_text, 
            compiled_attributes,
            mut function_result,
            data_map,
        ) = prepare_function_parse(function_parse, data_map.clone());
        if function_text_wrap.is_some() {
            function.validate = Some(expr.is_match(function_text.as_str()));
            if function.validate.unwrap() {
                let matches = &expr.captures(function_text.as_str()).unwrap();
                let year = matches.name("year").unwrap();
                let month = matches.name("month").unwrap();
                let day = matches.name("day").unwrap();
                let mut attributes_: Vec<String> = Vec::new();
                attributes_.push(year.as_str().to_string());
                attributes_.push(month.as_str().to_string());
                attributes_.push(day.as_str().to_string());
                function.attributes = Some(attributes_)
            }
        }
        if data_map_wrap.is_some() {
            self.attributes = Some(compiled_attributes);
            self.data_map = Some(data_map);
            function_result.text = Some(self.execute()?);
            function.result = Some(function_result.clone());
        }
        return Ok(function)
    }
    fn execute(&self) -> Result<String, PlanetError> {
        let attributes = self.attributes.clone().unwrap();
        let data_map = &self.data_map.clone().unwrap();
        let year_item = attributes[0].clone();
        let year_value = year_item.get_value(data_map)?;
        let month_item = attributes[1].clone();
        let month_value = month_item.get_value(data_map)?;
        let day_item = attributes[2].clone();
        let day_value = day_item.get_value(data_map)?;
        let date_only = NaiveDate::parse_from_str(
            format!(
                "{year}-{month}-{day}",
                year=&year_value,
                month=&month_value,
                day=&day_value,
            ).as_str(), 
            "%Y-%m-%d"
        ).unwrap();
        let month_short = date_only.format("%b").to_string();
        let replacement_string: String = format!(
            "{day}-{month_short}-{year}",
            day=&day_value,
            month_short=month_short,
            year=&year_value,
        );
        return Ok(replacement_string)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DateTimeParse {
    function: Option<FunctionParse>,
    data_map: Option<HashMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>
}
impl DateTimeParse {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<HashMap<String, String>>
    ) -> Self {
        return Self{function: function, data_map: data_map, attributes: None};
    }
}
impl DateTimeFunction for DateTimeParse {
    fn handle(&mut self, date_parse_option: DateTimeParseOption) -> Result<FunctionParse, PlanetError> {
        // SECOND($date_str)
        // MINUTE($date_str)
        // HOUR($date_str)
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr_minute = &RE_MINUTE;
        let expr_second = &RE_SECOND;
        let expr_hour = &RE_HOUR;
        let mut function = function_parse.clone();
        let data_map_wrap = data_map.clone();
        let (
            function_text_wrap, 
            function_text, 
            compiled_attributes,
            mut function_result,
            data_map,
        ) = prepare_function_parse(function_parse, data_map.clone());
        if function_text_wrap.is_some() {
            // function.validate = Some(expr.is_match(function_text.as_str()));
            match date_parse_option {
                DateTimeParseOption::Second => {
                    function.validate = Some(expr_second.is_match(function_text.as_str()));
                }
                DateTimeParseOption::Minute => {
                    function.validate = Some(expr_minute.is_match(function_text.as_str()));
                }
                DateTimeParseOption::Hour => {
                    function.validate = Some(expr_hour.is_match(function_text.as_str()));
                }
            }
            if function.validate.unwrap() {
                let matches: Captures;
                match date_parse_option {
                    DateTimeParseOption::Hour => {
                        matches = expr_hour.captures(function_text.as_str()).unwrap();
                    },
                    DateTimeParseOption::Minute => {
                        matches = expr_minute.captures(function_text.as_str()).unwrap();
                    },
                    DateTimeParseOption::Second => {
                        matches = expr_second.captures(function_text.as_str()).unwrap();
                    },
                }
                let mut attributes_: Vec<String> = Vec::new();
                let mut date_string_wrap: Option<String> = None;
                let mut date: String = String::from("");
                if matches.name("date").is_some() {
                    date = matches.name("date").unwrap().as_str().to_string();
                    date_string_wrap = Some(date.clone());
                } else if matches.name("date_alt").is_some() {
                    date = matches.name("date_alt").unwrap().as_str().to_string();
                    date_string_wrap = Some(date.clone());
                } else if matches.name("date_ref").is_some() {
                    date = matches.name("date_ref").unwrap().as_str().to_string();
                }
                attributes_.push(date);
                let mut mode: &str = "";
                if date_string_wrap.is_some() {
                    let date_string = date_string_wrap.unwrap();
                    if date_string.find("T").is_some() {
                        mode = "\"iso\"";
                        let date_object = get_date_object_iso(&date_string);
                        if date_object.is_err() {
                            function.validate = Some(false);
                        }
                    } else if date_string.find(" ").is_some() {
                        mode = "\"human_time\"";
                        let date_object = get_date_object_human_time(&date_string);
                        if date_object.is_err() {
                            function.validate = Some(false);
                        }
                    } else {
                        mode = "\"only_date\"";
                    }
                }
                attributes_.push(mode.to_string());
                function.attributes = Some(attributes_);
            }
        }
        if data_map_wrap.is_some() {
            self.attributes = Some(compiled_attributes);
            self.data_map = Some(data_map);
            function_result.text = Some(self.execute(date_parse_option)?);
            function.result = Some(function_result.clone());
        }
        return Ok(function)
    }
    fn execute(&self, date_parse_option: DateTimeParseOption) -> Result<String, PlanetError> {
        let attributes = self.attributes.clone().unwrap();
        let data_map = &self.data_map.clone().unwrap();
        let date_item = attributes[0].clone();
        let mut date = date_item.get_value(data_map)?;
        let mode_item = attributes[1].clone();
        let mode = mode_item.get_value(data_map)?;
        let mode = mode.as_str();
        let mut replacement_string = String::from("");
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
        return Ok(replacement_string)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DateParse {
    function: Option<FunctionParse>,
    data_map: Option<HashMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>
}
impl DateParse {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<HashMap<String, String>>
    ) -> Self {
        return Self{function: function, data_map: data_map, attributes: None};
    }
}
impl DateParseFunction for DateParse {
    fn handle(&mut self, date_parse_option: DateParseOption) -> Result<FunctionParse, PlanetError> {
        // DAY("2021-08-25T06:26:00+00:00")
        // DAY({Column Date}) : This is any of the ones here.
        // DAY("2021-08-25 06:26:00")
        // DAY("25-Aug-2021 06:26:00")
        // DAY("25-AUG-2021 06:26:00")
        // DAY("25-Aug-2021")
        // DAY("25-AUG-2021")
        // DAY({Column Date}) : This has no date in constructor
        // WEEK, WEEKDAY, MONTH and YEAR functions handler with same date formats
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr_day = &RE_DAY;
        let expr_week = &RE_WEEK;
        let expr_week_day = &RE_WEEKDAY;
        let expr_month = &RE_MONTH;
        let expr_year = &RE_YEAR;
        let mut function = function_parse.clone();
        let data_map_wrap = data_map.clone();
        let (
            function_text_wrap, 
            function_text, 
            compiled_attributes,
            mut function_result,
            data_map,
        ) = prepare_function_parse(function_parse, data_map.clone());
        if function_text_wrap.is_some() {
            match date_parse_option {
                DateParseOption::Day => {
                    function.validate = Some(expr_day.is_match(function_text.as_str()));
                }
                DateParseOption::Week => {
                    function.validate = Some(expr_week.is_match(function_text.as_str()));
                }
                DateParseOption::WeekDay => {
                    function.validate = Some(expr_week_day.is_match(function_text.as_str()));
                }
                DateParseOption::Month => {
                    function.validate = Some(expr_month.is_match(function_text.as_str()));
                }
                DateParseOption::Year => {
                    function.validate = Some(expr_year.is_match(function_text.as_str()));
                }
            }
            if function.validate.unwrap() {
                let matches: Captures;
                match date_parse_option {
                    DateParseOption::Day => {
                        matches = expr_day.captures(function_text.as_str()).unwrap();
                    },
                    DateParseOption::Week => {
                        matches = expr_week.captures(function_text.as_str()).unwrap();
                    },
                    DateParseOption::WeekDay => {
                        matches = expr_week_day.captures(function_text.as_str()).unwrap();
                    },
                    DateParseOption::Month => {
                        matches = expr_month.captures(function_text.as_str()).unwrap();
                    },
                    DateParseOption::Year => {
                        matches = expr_year.captures(function_text.as_str()).unwrap();
                    },
                }
                let mut attributes_: Vec<String> = Vec::new();
                let mut date_string_wrap: Option<String> = None;
                let mut date: String = String::from("");
                if matches.name("date").is_some() {
                    date = matches.name("date").unwrap().as_str().to_string();
                    date_string_wrap = Some(date.clone());
                } else if matches.name("date_alt").is_some() {
                    date = matches.name("date_alt").unwrap().as_str().to_string();
                    date_string_wrap = Some(date.clone());
                } else if matches.name("date_ref").is_some() {
                    date = matches.name("date_ref").unwrap().as_str().to_string();
                }
                attributes_.push(date);
                let mut mode: &str = "";
                if date_string_wrap.is_some() {
                    let date_string = date_string_wrap.unwrap();
                    if date_string.find("T").is_some() {
                        mode = "\"iso\"";
                        let date_object = get_date_object_iso(&date_string);
                        if date_object.is_err() {
                            function.validate = Some(false);
                        }
                    } else if date_string.find(" ").is_some() {
                        mode = "\"human_time\"";
                        let date_object = get_date_object_human_time(&date_string);
                        if date_object.is_err() {
                            function.validate = Some(false);
                        }
                    } else {
                        mode = "\"only_date\"";
                        let date_object = get_date_object_only_date(&date_string);
                        if date_object.is_err() {
                            function.validate = Some(false);
                        }
                    }
                }
                attributes_.push(mode.to_string());
                function.attributes = Some(attributes_);
            }
        }
        if data_map_wrap.is_some() {
            self.attributes = Some(compiled_attributes);
            self.data_map = Some(data_map);
            function_result.text = Some(self.execute(date_parse_option)?);
            function.result = Some(function_result.clone());
        }
        return Ok(function)
    }
    fn execute(&self, date_parse_option: DateParseOption) -> Result<String, PlanetError> {
        let attributes = self.attributes.clone().unwrap();
        let data_map = &self.data_map.clone().unwrap();
        let date_item = attributes[0].clone();
        let mut date = date_item.get_value(data_map)?;
        let mode_item = attributes[1].clone();
        let mode = mode_item.value.unwrap_or_default();
        let mode = mode.as_str();
        let mut replacement_string = String::from("");
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
                // eprintln!("DateParseFunction.replace :: date_obj: {:?}", &date_obj);
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
        if is_string_output {
            replacement_string = prepare_string_attribute(replacement_string);
        }
        return Ok(replacement_string)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Now {
    function: Option<FunctionParse>,
    data_map: Option<HashMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>
}
impl Now {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<HashMap<String, String>>
    ) -> Self {
        return Self{function: function, data_map: data_map, attributes: None};
    }
}
impl DateFunction for Now {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_NOW;
        let mut function = function_parse.clone();
        let data_map_wrap = data_map.clone();
        let (
            function_text_wrap, 
            function_text, 
            compiled_attributes,
            mut function_result,
            data_map,
        ) = prepare_function_parse(function_parse, data_map.clone());
        if function_text_wrap.is_some() {
            function.validate = Some(expr.is_match(function_text.as_str()));
            if function.validate.unwrap() {
                let attributes_: Vec<String> = Vec::new();
                function.attributes = Some(attributes_);
            }
        }
        if data_map_wrap.is_some() {
            self.attributes = Some(compiled_attributes);
            self.data_map = Some(data_map);
            function_result.text = Some(self.execute()?);
            function.result = Some(function_result.clone());
        }
        return Ok(function)
    }
    fn execute(&self) -> Result<String, PlanetError> {
        let now_date_obj = Utc::now();
        let replacement_string = now_date_obj.to_rfc3339();
        return Ok(replacement_string);
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Today {
    function: Option<FunctionParse>,
    data_map: Option<HashMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>
}
impl Today {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<HashMap<String, String>>
    ) -> Self {
        return Self{function: function, data_map: data_map, attributes: None};
    }
}
impl DateFunction for Today {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_TODAY;
        let mut function = function_parse.clone();
        let data_map_wrap = data_map.clone();
        let (
            function_text_wrap, 
            function_text, 
            compiled_attributes,
            mut function_result,
            data_map,
        ) = prepare_function_parse(function_parse, data_map.clone());
        if function_text_wrap.is_some() {
            function.validate = Some(expr.is_match(function_text.as_str()));
            if function.validate.unwrap() {
                let attributes_: Vec<String> = Vec::new();
                function.attributes = Some(attributes_);
            }
        }
        if data_map_wrap.is_some() {
            self.attributes = Some(compiled_attributes);
            self.data_map = Some(data_map);
            function_result.text = Some(self.execute()?);
            function.result = Some(function_result.clone());
        }
        return Ok(function)
    }
    fn execute(&self) -> Result<String, PlanetError> {
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
        return Ok(replacement_string)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Days {
    function: Option<FunctionParse>,
    data_map: Option<HashMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>
}
impl Days {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<HashMap<String, String>>
    ) -> Self {
        return Self{function: function, data_map: data_map, attributes: None};
    }
}
impl DateFunction for Days {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // DAYS("12-SEP-2021", "5-FEB-2021")
        // DAYS({Column A}, {Column B})
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_DAYS;
        let mut function = function_parse.clone();
        let data_map_wrap = data_map.clone();
        let (
            function_text_wrap, 
            function_text, 
            compiled_attributes,
            mut function_result,
            data_map,
        ) = prepare_function_parse(function_parse, data_map.clone());
        if function_text_wrap.is_some() {
            function.validate = Some(expr.is_match(function_text.as_str()));
            if function.validate.unwrap() {
                let mut attributes_: Vec<String> = Vec::new();
                let matches = &expr.captures(&function_text).unwrap();
                let start_date = matches.name("start_date");
                let end_date = matches.name("end_date");
                let start_date_ref = matches.name("start_date_ref");
                let end_date_ref = matches.name("end_date_ref");
                if start_date.is_some() {
                    let start_date_string = start_date.unwrap().as_str().to_string();
                    attributes_.push(start_date_string);
                }
                if end_date.is_some() {
                    let end_date_string = end_date.unwrap().as_str().to_string();
                    attributes_.push(end_date_string);
                }
                if start_date_ref.is_some() {
                    let start_date_ref_string = start_date_ref.unwrap().as_str().to_string();
                    attributes_.push(start_date_ref_string);
                }
                if end_date_ref.is_some() {
                    let end_date_ref_string = end_date_ref.unwrap().as_str().to_string();
                    attributes_.push(end_date_ref_string);
                }
                function.attributes = Some(attributes_);
            }
        }
        if data_map_wrap.is_some() {
            self.attributes = Some(compiled_attributes);
            self.data_map = Some(data_map);
            function_result.text = Some(self.execute()?);
            function.result = Some(function_result.clone());
        }
        return Ok(function)
    }
    fn execute(&self) -> Result<String, PlanetError> {
        let attributes = self.attributes.clone().unwrap();
        let data_map = &self.data_map.clone().unwrap();
        let start_date_item = attributes[0].clone();
        let start_date_value = start_date_item.get_value(data_map)?;
        let end_date_item = attributes[1].clone();
        let end_date_value = end_date_item.get_value(data_map)?;
        let number_days: i64;
        let start_date_obj = NaiveDate::parse_from_str(
            start_date_value.as_str(), 
            "%d-%b-%Y"
        ).unwrap();
        let end_date_obj = NaiveDate::parse_from_str(
            end_date_value.as_str(), 
            "%d-%b-%Y"
        ).unwrap();
        let duration = end_date_obj.signed_duration_since(start_date_obj);
        number_days = duration.num_days();
        let replacement_string = number_days.to_string();
        return Ok(replacement_string)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DateAddDiff {
    function: Option<FunctionParse>,
    data_map: Option<HashMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>
}
impl DateAddDiff {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<HashMap<String, String>>
    ) -> Self {
        return Self{function: function, data_map: data_map, attributes: None};
    }
}
impl DateAddDiffFunction for DateAddDiff {
    fn handle(&mut self, operation: DateDeltaOperation) -> Result<FunctionParse, PlanetError> {
        // DATEADD(date, number, units)
        // DATEDIF(date, number, units)
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr_add = &RE_DATEADD;
        let expr_diff = &RE_DATEDIF;
        let mut function = function_parse.clone();
        let data_map_wrap = data_map.clone();
        let (
            function_text_wrap, 
            function_text, 
            compiled_attributes,
            mut function_result,
            data_map,
        ) = prepare_function_parse(function_parse, data_map.clone());
        if function_text_wrap.is_some() {
            match operation {
                DateDeltaOperation::Add => {
                    function.validate = Some(expr_add.is_match(function_text.as_str()));
                }
                DateDeltaOperation::Diff => {
                    function.validate = Some(expr_diff.is_match(function_text.as_str()));
                }
            }            
            if function.validate.unwrap() {
                let mut attributes_: Vec<String> = Vec::new();
                let matches: Captures;
                match operation {
                    DateDeltaOperation::Add => {
                        matches = expr_add.captures(&function_text).unwrap();
                    },
                    DateDeltaOperation::Diff => {
                        matches = expr_diff.captures(&function_text).unwrap();
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

                if attr_date.is_some() && attr_number.is_some() && attr_units.is_some() {
                    let date_string = attr_date.unwrap().as_str().to_string();
                    let number_string = attr_number.unwrap().as_str().to_string();
                    let units_string = attr_units.unwrap().as_str().to_string();
                    attributes_.push(date_string.clone());
                    attributes_.push(number_string);
                    attributes_.push(units_string);
                    let date_obj_wrap = get_date_object_only_date(&date_string);
                    if date_obj_wrap.is_err() {
                        function.validate = Some(false);
                    }
                } else if attr_datetime.is_some() && attr_dt_number.is_some() && attr_dt_units.is_some() {
                    let date_string = attr_datetime.unwrap().as_str().to_string();
                    let number_string = attr_dt_number.unwrap().as_str().to_string();
                    let units_string = attr_dt_units.unwrap().as_str().to_string();
                    attributes_.push(date_string.clone());
                    attributes_.push(number_string);
                    attributes_.push(units_string);
                    let date_obj_wrap = get_date_object_human_time(&date_string);
                    if date_obj_wrap.is_err() {
                        function.validate = Some(false);
                    }
                } else if attr_date_ref.is_some() && attr_ref_number.is_some() && attr_ref_units.is_some() {
                    let date_string = attr_date_ref.unwrap().as_str().to_string();
                    let number_string = attr_ref_number.unwrap().as_str().to_string();
                    let units_string = attr_ref_units.unwrap().as_str().to_string();
                    attributes_.push(date_string);
                    attributes_.push(number_string);
                    attributes_.push(units_string);
                }
                function.attributes = Some(attributes_);
            }
        }
        if data_map_wrap.is_some() {
            self.attributes = Some(compiled_attributes);
            self.data_map = Some(data_map);
            function_result.text = Some(self.execute(operation)?);
            function.result = Some(function_result.clone());
        }
        return Ok(function)
    }
    fn execute(&self, operation: DateDeltaOperation) -> Result<String, PlanetError> {
        let attributes = self.attributes.clone().unwrap();
        let data_map = &self.data_map.clone().unwrap();
        let date_item = attributes[0].clone();
        let date_value = date_item.get_value(data_map)?;
        let number_item = attributes[1].clone();
        let number_value = number_item.get_value(data_map)?;
        let units_item = attributes[2].clone();
        let units_value = units_item.get_value(data_map)?;
        let replacement_string: String;
        let new_date: DateTime<FixedOffset>;
        let date_obj: DateTime<FixedOffset>;
        let has_time: bool;
        let number: i64 = FromStr::from_str(number_value.as_str()).unwrap();
        let units_str = units_value.as_str();
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
        has_time = *&date_value.find(" ").is_some();
        if has_time == true {
            date_obj = get_date_object_human_time(&date_value).unwrap();
        } else {
            date_obj = get_date_object_only_date(&date_value).unwrap();
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
        return Ok(replacement_string)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DateFormat {
    function: Option<FunctionParse>,
    data_map: Option<HashMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>
}
impl DateFormat {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<HashMap<String, String>>
    ) -> Self {
        return Self{function: function, data_map: data_map, attributes: None};
    }
}
impl DateFunction for DateFormat {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_DATEFMT;
        let mut function = function_parse.clone();
        let data_map_wrap = data_map.clone();
        let (
            function_text_wrap, 
            function_text, 
            compiled_attributes,
            mut function_result,
            data_map,
        ) = prepare_function_parse(function_parse, data_map.clone());
        if function_text_wrap.is_some() {
            function.validate = Some(expr.is_match(function_text.as_str()));
            if function.validate.unwrap() {
                let mut attributes_: Vec<String> = Vec::new();
                let matches= &expr.captures(&function_text).unwrap();

                // date
                let date = matches.name("date");
                let datetime = matches.name("datetime");
                let date_ref = matches.name("ref");
                // format
                let fmt = matches.name("format");
                let dt_fmt = matches.name("dt_format");
                let ref_fmt = matches.name("ref_format");
        
                let mut mode: String = String::from("date");
                if date.is_some() && fmt.is_some() {
                    let date_string = date.unwrap().as_str().to_string();
                    let format_string = fmt.unwrap().as_str().to_string();
                    let date_obj_wrap_ = get_date_object_only_date(&date_string);
                    if date_obj_wrap_.is_err() {
                        function.validate = Some(false);
                    }
                    attributes_.push(date_string.clone());
                    attributes_.push(format_string);
                    attributes_.push(mode);
                } else if datetime.is_some() && dt_fmt.is_some() {
                    let date_string = datetime.unwrap().as_str().to_string();
                    let format_string = dt_fmt.unwrap().as_str().to_string();
                    let date_obj_wrap_ = get_date_object_human_time(&date_string);
                    if date_obj_wrap_.is_err() {
                        function.validate = Some(false);
                    }        
                    mode = String::from("datetime");
                    attributes_.push(date_string.clone());
                    attributes_.push(format_string);
                    attributes_.push(mode);
                } else if date_ref.is_some() && ref_fmt.is_some() {
                    let date_string = date_ref.unwrap().as_str().to_string();
                    let format_string = ref_fmt.unwrap().as_str().to_string();
                    attributes_.push(date_string);
                    attributes_.push(format_string);
                    attributes_.push(mode);
                }
                function.attributes = Some(attributes_);
            }
        }
        if data_map_wrap.is_some() {
            self.attributes = Some(compiled_attributes);
            self.data_map = Some(data_map);
            function_result.text = Some(self.execute()?);
            function.result = Some(function_result.clone());
        }
        return Ok(function)
    }
    fn execute(&self) -> Result<String, PlanetError> {
        let attributes = self.attributes.clone().unwrap();
        let data_map = &self.data_map.clone().unwrap();
        let date_item = attributes[0].clone();
        let date_value = date_item.get_value(data_map)?;
        let format_item = attributes[1].clone();
        let format_value = format_item.get_value(data_map)?;
        let mode_item = attributes[2].clone();
        let mode_value = mode_item.get_value(data_map)?;
        let date_obj_wrap: Option<DateTime<FixedOffset>>;
        if mode_value == String::from("date") {
            date_obj_wrap = Some(get_date_object_only_date(&date_value).unwrap());
        } else {
            date_obj_wrap = Some(get_date_object_human_time(&date_value).unwrap());
        }
        let date_obj = date_obj_wrap.unwrap();
        let format = get_rust_date_format(date_obj, format_value);
        let date_string = date_obj.format(format.as_str()).to_string();
        return Ok(date_string)
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

fn get_rust_date_format(date_obj: DateTime<FixedOffset>, mut format: String) -> String {
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
