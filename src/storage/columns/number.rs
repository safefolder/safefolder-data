use std::str::FromStr;
use std::collections::BTreeMap;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use tr::tr;
use lazy_static::lazy_static;
use regex::{Regex};
use rust_decimal::prelude::*;

use crate::planet::{PlanetError};
use crate::commands::folder::config::ColumnConfig;
use crate::storage::constants::*;
use crate::storage::columns::*;

lazy_static! {
    pub static ref RE_CURRENCY: Regex = Regex::new(r#"^(?P<symbol_pre>[^\d\.]*)*(?P<amount>\d+[\.\d+]*)(?P<symbol_post>[^\d\.]+)*$"#).unwrap();
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CheckBoxColumn {
    pub config: ColumnConfig
}
impl CheckBoxColumn {
    pub fn defaults(field_config: &ColumnConfig) -> Self {
        let field_config = field_config.clone();
        let field_obj = Self{
            config: field_config
        };
        return field_obj
    }
}
impl StorageColumn for CheckBoxColumn {
    fn update_config_map(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let field_config_map = field_config_map.clone();
        // No special attributes so far for small text field
        return Ok(field_config_map)
    }
    fn build_config(
        &mut self, 
        _: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError> {
        let config = self.config.clone();
        // No special attributes so far for small text field
        return Ok(config)
    }
    fn validate(&self, data: &String) -> Result<String, PlanetError> {
        let data = data.clone();

        let field_config = self.config.clone();
        let required = field_config.required.unwrap();
        let name = field_config.name.unwrap();
        // eprintln!("CheckBoxColumn.is_valid :: value: {:?}", &value);
        if data == String::from("") && required == true {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!(
                        "Field {}{}{} is required", 
                        String::from("\"").blue(), name.blue(), String::from("\"").blue()
                    )),
                )
            );
        } else {
            let value_str = data.as_str();
            // eprintln!("CheckBoxColumn.is_valid :: value_str: {:?}", &value_str);
            if value_str == "true" || value_str == "false" {
                return Ok(data);
            } else {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!("Checkbox value needs to be \"true\" or \"false\"")),
                    )
                );
            }
        }
    }
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String {
        let field_config = self.config.clone();
        let field_name = field_config.name.unwrap();
        let mut yaml_string = yaml_string.clone();
        let field = &field_name.truecolor(
            YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
        );
        let value = format!("{}", value.to_string().truecolor(
            YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
        ));
        let value = format!("{}", value.to_string().truecolor(
            YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
        ));
        yaml_string.push_str(format!("  {field}: {value}\n", field=field, value=value).as_str());
        return yaml_string;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NumberColumn {
    pub config: ColumnConfig
}
impl NumberColumn {
    pub fn defaults(field_config: &ColumnConfig) -> Self {
        let field_config = field_config.clone();
        let field_obj = Self{
            config: field_config
        };
        return field_obj
    }
}
impl StorageColumn for NumberColumn {
    fn update_config_map(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let field_config_map = field_config_map.clone();
        // No special attributes so far for small text field
        return Ok(field_config_map)
    }
    fn build_config(
        &mut self, 
        _: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError> {
        let config = self.config.clone();
        // No special attributes so far for small text field
        return Ok(config)
    }
    fn validate(&self, data: &String) -> Result<String, PlanetError> {
        let data = data.clone();
        let field_config = self.config.clone();
        let required = field_config.required.unwrap();
        let name = field_config.name.unwrap();
        if data == String::from("") && required == true {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!(
                        "Field {}{}{} is required", 
                        String::from("\"").blue(), name.blue(), String::from("\"").blue()
                    )),
                )
            );
        } else {
            let value_str = data.as_str();
            let result = i32::from_str(value_str);
            match result {
                Ok(_) => {
                    return Ok(data);
                },
                Err(_) => {
                    return Err(
                        PlanetError::new(
                            500, 
                            Some(tr!("I could not process as number: \"{}\"", data)),
                        )
                    );    
                }
            }
        }
    }
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String {
        let field_config = self.config.clone();
        let field_name = field_config.name.unwrap();
        let mut yaml_string = yaml_string.clone();
        let field = &field_name.truecolor(
            YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
        );
        let value = format!("{}", value.to_string().truecolor(
            YAML_COLOR_YELLOW[0], YAML_COLOR_YELLOW[1], YAML_COLOR_YELLOW[2]
        ));
        yaml_string.push_str(format!("  {field}: {value}\n", field=field, value=value).as_str());
        return yaml_string;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CurrencyColumn {
    pub config: ColumnConfig
}
impl CurrencyColumn {
    pub fn defaults(field_config: &ColumnConfig) -> Self {
        let field_config = field_config.clone();
        let field_obj = Self{
            config: field_config
        };
        return field_obj
    }
}
impl StorageColumn for CurrencyColumn {
    fn update_config_map(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let mut field_config_map = field_config_map.clone();
        let config = self.config.clone();
        let number_decimals = config.number_decimals;
        let currency_symbol = config.currency_symbol;
        if number_decimals.is_some() {
            let number_decimals = number_decimals.unwrap();
            let number_decimals = number_decimals.to_string();
            field_config_map.insert(NUMBER_DECIMALS.to_string(), number_decimals);
        } else {
            let number_decimals = String::from("2");
            field_config_map.insert(NUMBER_DECIMALS.to_string(), number_decimals);
        }
        if currency_symbol.is_some() {
            let currency_symbol = currency_symbol.unwrap();
            field_config_map.insert(CURRENCY_SYMBOL.to_string(), currency_symbol);
        }
        return Ok(field_config_map)
    }
    fn build_config(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError> {
        let mut config = self.config.clone();
        let number_decimals = field_config_map.get(NUMBER_DECIMALS);
        let currency_symbol = field_config_map.get(CURRENCY_SYMBOL);
        if number_decimals.is_some() {
            let number_decimals = number_decimals.unwrap().clone();
            let number_decimals: i8 = FromStr::from_str(number_decimals.as_str()).unwrap();
            config.number_decimals = Some(number_decimals);
        }
        if currency_symbol.is_some() {
            let currency_symbol = currency_symbol.unwrap().clone();
            config.currency_symbol = Some(currency_symbol);
        }
        return Ok(config)
    }
    fn validate(&self, data: &String) -> Result<String, PlanetError> {
        let mut data = data.clone();
        let config = self.config.clone();
        let number_decimals = config.number_decimals;
        let currency_symbol = config.currency_symbol;
        if number_decimals.is_none() {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("Field not configured for currency \"{}\"", data.clone())),
                )
            );
        }
        let number_decimals = number_decimals.unwrap();
        let number_decimals: u32 = number_decimals.to_u32().unwrap();
        let currency_symbol = currency_symbol.unwrap();
        let currency_symbol = currency_symbol.as_str();
        let expr = &RE_CURRENCY;
        let match_data = data.clone();
        let is_valid = expr.is_match(&match_data);
        if !is_valid {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("Validation error on currency \"{}\"", data.clone())),
                )
            );
        }
        let captures = expr.captures(&match_data).unwrap();
        let amount_wrap = captures.name("amount");
        let symbol_pre_wrap = captures.name("symbol_pre");
        let symbol_post_wrap = captures.name("symbol_post");
        let mut amount_string: String = String::from("");
        // eprintln!("CurrencyColumn.validate :: before symbol replace: {}", &data);
        if symbol_pre_wrap.is_some() || symbol_post_wrap.is_some() {
            // I have symbol on sent data
            if symbol_pre_wrap.is_some() {
                let symbol_pre = symbol_pre_wrap.unwrap().as_str();
                // eprintln!("CurrencyColumn.validate :: [regex] symbol_pre: {}", symbol_pre);
                data = data.clone().replace(symbol_pre, "");
            }
            if symbol_post_wrap.is_some() {
                let symbol_post = symbol_post_wrap.unwrap().as_str();
                // eprintln!("CurrencyColumn.validate :: [regex] symbol_post: {}", symbol_post);
                data = data.clone().replace(symbol_post, "");
            }
        }
        data = data.trim().to_string();
        // eprintln!("CurrencyColumn.validate :: after symbol replace: {}", &data);
        if amount_wrap.is_some() {
            // Might be 7658.45 or 7658 or $7658. Need to get number without the currency symbol
            let amount_str = amount_wrap.unwrap().as_str();
            // eprintln!("CurrencyColumn.validate :: [regex] amount_str: {}", amount_str);
            // 79876.45
            // format amount to have number decimals from config
            let amount = Decimal::from_str(amount_str);
            if amount.is_err() {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!("Validation error on currency \"{}\"", data.clone())),
                    )
                );
            }
            let amount = amount.unwrap().round_dp(number_decimals);
            // amount_string = amount.to_string();
            let number_decimals = number_decimals.to_usize().unwrap();
            amount_string = format!("{:.1$}", &amount, number_decimals);
        }
        // eprintln!("CurrencyColumn.validate :: amount_string: {}", &amount_string);
        // data needs to have right number of decimals and the currency symbol
        
        data = format!("{}{}", currency_symbol, &amount_string);
        return Ok(data)
    }
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String {
        let field_config = self.config.clone();
        let field_name = field_config.name.unwrap();
        let mut yaml_string = yaml_string.clone();
        let field = &field_name.truecolor(
            YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
        );
        let value = format!("{}", value.to_string().truecolor(
            YAML_COLOR_YELLOW[0], YAML_COLOR_YELLOW[1], YAML_COLOR_YELLOW[2]
        ));
        yaml_string.push_str(format!("  {field}: {value}\n", field=field, value=value).as_str());
        return yaml_string;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PercentageColumn {
    pub config: ColumnConfig
}
impl PercentageColumn {
    pub fn defaults(field_config: &ColumnConfig) -> Self {
        let field_config = field_config.clone();
        let field_obj = Self{
            config: field_config
        };
        return field_obj
    }
}
impl StorageColumn for PercentageColumn {
    fn update_config_map(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let mut field_config_map = field_config_map.clone();
        let config = self.config.clone();
        let number_decimals = config.number_decimals;
        let number_decimals_string: String;
        if number_decimals.is_some() {
            let number_decimals = number_decimals.unwrap();
            number_decimals_string = number_decimals.to_string();
        } else {
            number_decimals_string = String::from("2");
        }
        field_config_map.insert(NUMBER_DECIMALS.to_string(), number_decimals_string);
        return Ok(field_config_map)
    }
    fn build_config(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError> {
        let mut config = self.config.clone();
        let number_decimals = field_config_map.get(NUMBER_DECIMALS);
        if number_decimals.is_some() {
            let number_decimals = number_decimals.unwrap().clone();
            let number_decimals: i8 = FromStr::from_str(number_decimals.as_str()).unwrap();
            config.number_decimals = Some(number_decimals);
        }
        return Ok(config)
    }
    fn validate(&self, data: &String) -> Result<String, PlanetError> {
        let mut data = data.clone();
        let config = self.config.clone();
        let number_decimals = config.number_decimals;
        if number_decimals.is_none() {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("Field not configured for percentage \"{}\"", data.clone())),
                )
            );            
        }
        let number_decimals = number_decimals.unwrap();
        let number_decimals: u32 = number_decimals.to_u32().unwrap();
        let amount_str = data.as_str();
        // format amount to have number decimals from config
        let amount = Decimal::from_str(amount_str);
        if amount.is_err() {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("Validation error on percentage \"{}\"", data.clone())),
                )
            );
        }
        let amount = amount.unwrap().round_dp(number_decimals);
        let number_decimals = number_decimals.to_usize().unwrap();
        data = format!("{:.1$}", &amount, number_decimals);
        return Ok(data)
    }
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String {
        let field_config = self.config.clone();
        let field_name = field_config.name.unwrap();
        let mut yaml_string = yaml_string.clone();
        let field = &field_name.truecolor(
            YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
        );
        let value = format!("{}", value.to_string().truecolor(
            YAML_COLOR_YELLOW[0], YAML_COLOR_YELLOW[1], YAML_COLOR_YELLOW[2]
        ));
        yaml_string.push_str(format!("  {field}: {value}\n", field=field, value=value).as_str());
        return yaml_string;
    }
}
