use std::str::FromStr;
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use colored::Colorize;
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use regex::{Regex};

use crate::planet::{PlanetError};
use crate::commands::folder::config::*;
use crate::storage::columns::*;
use crate::storage::constants::*;

lazy_static! {
    pub static ref RE_DURATION: Regex = Regex::new(r#"^(?P<hour>\d+):(?P<minute>\d+)(:(?P<second>\d+))*(.(?P<micro>\d+))*$"#).unwrap();
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DateColumn {
    pub config: ColumnConfig
}
impl DateColumn {
    pub fn defaults(config: &ColumnConfig) -> Self {
        let field_config = config.clone();
        let field_obj = Self{
            config: field_config
        };
        return field_obj
    }
}
impl StorageColumn for DateColumn {
    fn create_config(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let mut field_config_map = field_config_map.clone();
        let date_format = self.config.date_format.clone();
        let time_format = self.config.time_format.clone();
        if date_format.is_some() {
            let date_format = date_format.unwrap();
            match date_format {
                DateFormat::Friendly => {
                    field_config_map.insert(DATE_FORMAT.to_string(), DATE_FORMAT_FRIENDLY.to_string());
                },
                DateFormat::US => {
                    field_config_map.insert(DATE_FORMAT.to_string(), DATE_FORMAT_US.to_string());
                },
                DateFormat::European => {
                    field_config_map.insert(DATE_FORMAT.to_string(), DATE_FORMAT_EUROPEAN.to_string());
                },
                DateFormat::ISO => {
                    field_config_map.insert(DATE_FORMAT.to_string(), DATE_FORMAT_ISO.to_string());
                },
            }
        }
        if time_format.is_some() {
            let time_format = time_format.unwrap();
            let time_format_str = time_format.to_string();
            let time_format_str = time_format_str.as_str();
            if time_format_str != "24" && time_format_str != "12" {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!("Time format must be \"12\" or \"24\"")),
                    )
                );
            }
            field_config_map.insert(TIME_FORMAT.to_string(), time_format_str.to_string());
        }
        return Ok(field_config_map)
    }
    fn get_config(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError> {
        let field_config_map = field_config_map.clone();
        let date_format = field_config_map.get(DATE_FORMAT);
        let time_format = field_config_map.get(TIME_FORMAT);
        let mut config = self.config.clone();
        if date_format.is_some() {
            let date_format_str = date_format.unwrap().as_str();
            let date_format: DateFormat;
            match date_format_str {
                DATE_FORMAT_FRIENDLY => {
                    date_format = DateFormat::Friendly;
                },
                DATE_FORMAT_US => {
                    date_format = DateFormat::US;
                },
                DATE_FORMAT_EUROPEAN => {
                    date_format = DateFormat::European;
                },
                DATE_FORMAT_ISO => {
                    date_format = DateFormat::ISO;
                },
                _ => {
                    // Raise error if not found in enum
                    return Err(
                        PlanetError::new(
                            500, 
                            Some(tr!("Date format not supported: {}", date_format_str)),
                        )
                    );
                }
            }
            config.date_format = Some(date_format);
        }
        if time_format.is_some() {
            let time_format_str = time_format.unwrap().as_str().trim();
            if time_format_str != "24" && time_format_str != "12" {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!("Time format must be \"12\" or \"24\"")),
                    )
                );
            }
            let time_format: i8 = FromStr::from_str(time_format_str).unwrap();
            config.time_format = Some(time_format);
        }
        return Ok(config)
    }
    fn validate(&self, data: &Vec<String>) -> Result<Vec<String>, PlanetError> {
        let config = self.config.clone();
        let date_string = data.clone();
        let data = data.clone();
        let set_validate = validate_set(&config, &date_string);
        if set_validate.is_err() {
            let error = set_validate.unwrap_err();
            return Err(error)
        }
        // date and/or time with different formats
        let date_format = config.date_format;
        let time_format = config.time_format;
        let mut fmt: &str = "";
        let mut fmt_string: String;
        let mut is_iso: bool = false;
        let mut is_time: bool = false;
        let mut is_am = false;
        if date_format.is_some() {
            let date_format = date_format.unwrap();
            match date_format {
                DateFormat::Friendly => {
                    // Check format DD-MMM-YYYY (HH:MM:SSam|HH:MM:SS)
                    fmt = "%d-%b-%Y";
                },
                DateFormat::US => {
                    // MM/DD/YYYY (HH:MM:SSam|HH:MM:SS)
                    fmt = "%m/%d/%Y";
                },
                DateFormat::European => {
                    // DD/MM/YYYY (HH:MM:SSam|HH:MM:SS)
                    fmt = "%d/%m/%Y";
                },
                DateFormat::ISO => {
                    // YYYY-MM-DDTHH:MM:SSam
                    // YYYY-MM-DDTHH:MM:SS
                    fmt = "%Y-%m-%d";
                    is_iso = true;
                },
            }
        }
        let mut sep = " ";
        if is_iso {
            sep = "T";
        }
        if time_format.is_some() {
            is_time = true;
            let time_format = time_format.unwrap();
            if time_format == 12 {
                fmt_string = format!("{}{}%I:%M:%S%P%z", fmt, sep);
                fmt = fmt_string.as_str();
                is_am = true;
            } else {
                // date and time with 24 hours
                fmt_string = format!("{}{}%H:%M:%S%z", fmt, sep);
                fmt = fmt_string.as_str();
            }
        }
        let mut data_new: Vec<String> = Vec::new();
        for mut date_string in data {
            // Check date_string with fmt
            if !is_time {
                if is_am {
                    // TODO: Check hours is 0-12 format
                    date_string = format!("{}{}00:00:00am+0000", &date_string, sep);
                    fmt_string = format!("{}{}%I:%M:%S%P%z", fmt, sep);
                    fmt = fmt_string.as_str();
                } else {
                    date_string = format!("{}{}00:00:00+0000", &date_string, sep);
                    fmt_string = format!("{}{}%H:%M:%S%z", fmt, sep);
                    fmt = fmt_string.as_str();
                }
            }
            let date_obj = DateTime::parse_from_str(
                &date_string, 
                fmt
            );
            if date_obj.is_err() {
                // Raise validation error
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!("Validation error on date \"{}\"", date_string.clone())),
                    )
                );
            }
            data_new.push(date_string);
        }
        return Ok(data_new)
    }
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String {
        let field_config = self.config.clone();
        let field_name = field_config.name.unwrap();
        let mut yaml_string = yaml_string.clone();
        let field = &field_name.truecolor(
            YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
        );
        let value = format!("{}",
            value.truecolor(YAML_COLOR_ORANGE[0], YAML_COLOR_ORANGE[1], YAML_COLOR_ORANGE[2]), 
        );
        yaml_string.push_str(format!("  {field}: {value}\n", field=field, value=value).as_str());
        return yaml_string;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DurationColumn {
    pub config: ColumnConfig
}
impl DurationColumn {
    pub fn defaults(config: &ColumnConfig) -> Self {
        let field_config = config.clone();
        let field_obj = Self{
            config: field_config
        };
        return field_obj
    }
}
impl StorageColumn for DurationColumn {
    fn create_config(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let field_config_map = field_config_map.clone();
        return Ok(field_config_map)
    }
    fn get_config(
        &mut self, 
        _: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError> {
        let config = self.config.clone();
        return Ok(config)
    }
    fn validate(&self, data: &Vec<String>) -> Result<Vec<String>, PlanetError> {
        let data = data.clone();
        let config = self.config.clone();
        let set_validate = validate_set(&config, &data);
        if set_validate.is_err() {
            let error = set_validate.unwrap_err();
            return Err(error)
        }
        // validate HH:MM[:SS.SSS]
        let expr = &RE_DURATION;
        let mut data_new: Vec<String> = Vec::new();
        for data_item in data {
            let is_valid = expr.is_match(&data_item);
            if !is_valid {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!("Duration format is not valid: \"{}\"", &data_item)),
                    )
                );
            }
            let matches = expr.captures(&data_item).unwrap();
            let hour = matches.name("hour");
            let minute = matches.name("minute");
            let second = matches.name("second");
            let micro = matches.name("micro");
            let hour_str = hour.unwrap().as_str();
            let minute_str = minute.unwrap().as_str();
            let mut second_str: &str;
            let micro_str: &str;
            let minute: i8 = FromStr::from_str(minute_str).unwrap();
            if minute > 60 {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!("Minutes needs to be less than 60: \"{}\"", &data_item)),
                    )
                );
            }
            if second.is_some() {
                second_str = second.unwrap().as_str();
                let second: i8 = FromStr::from_str(second_str).unwrap();
                if second > 60 {
                    return Err(
                        PlanetError::new(
                            500, 
                            Some(tr!("Seconds needs to be less than 60: \"{}\"", &data_item)),
                        )
                    );
                }
            }
            let mut data_final = format!("{}:{}", hour_str, minute_str);
            if second.is_some() {
                second_str = second.unwrap().as_str();
                data_final = format!("{}:{}", &data_final, second_str);
            }
            if micro.is_some() {
                micro_str = micro.unwrap().as_str();
                data_final = format!("{}.{}", &data_final, micro_str);
            }
            data_new.push(data_final);
        }
        return Ok(data_new)
    }
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String {
        let field_config = self.config.clone();
        let field_name = field_config.name.unwrap();
        let mut yaml_string = yaml_string.clone();
        let field = &field_name.truecolor(
            YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
        );
        let value = format!("{}",
            value.truecolor(YAML_COLOR_ORANGE[0], YAML_COLOR_ORANGE[1], YAML_COLOR_ORANGE[2]), 
        );
        yaml_string.push_str(format!("  {field}: {value}\n", field=field, value=value).as_str());
        return yaml_string;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditDateColumn {
    pub config: ColumnConfig,
}
impl AuditDateColumn {
    pub fn defaults(config: &ColumnConfig) -> Self {
        let field_config = config.clone();
        let field_obj = Self{
            config: field_config,
        };
        return field_obj
    }
}
impl StorageColumn for AuditDateColumn {
    fn create_config(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let field_config_map = field_config_map.clone();
        return Ok(field_config_map)
    }
    fn get_config(
        &mut self, 
        _: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError> {
        let config = self.config.clone();
        return Ok(config)
    }
    fn validate(&self, _: &Vec<String>) -> Result<Vec<String>, PlanetError> {
        let now = Utc::now();
        // 2021-12-18T12:14:15.528276533+00:00
        let now_str = now.to_rfc3339();
        let mut list: Vec<String> = Vec::new();
        list.push(now_str);
        return Ok(list)
    }
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String {
        let field_config = self.config.clone();
        let field_name = field_config.name.unwrap();
        let mut yaml_string = yaml_string.clone();
        let field = &field_name.truecolor(
            YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
        );
        let value = format!("{}",
            value.truecolor(YAML_COLOR_ORANGE[0], YAML_COLOR_ORANGE[1], YAML_COLOR_ORANGE[2]), 
        );
        yaml_string.push_str(format!("  {field}: {value}\n", field=field, value=value).as_str());
        return yaml_string;
    }
}
