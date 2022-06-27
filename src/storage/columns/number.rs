use std::str::FromStr;
use std::collections::BTreeMap;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use tr::tr;
use lazy_static::lazy_static;
use regex::{Regex};
use rust_decimal::prelude::*;

use crate::planet::{PlanetError};
use crate::statements::folder::schema::*;
use crate::storage::constants::*;
use crate::storage::columns::*;
use crate::storage::folder::FolderSchema;
use crate::functions::{execute_formula, Formula};

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
    fn create_config(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let field_config_map = field_config_map.clone();
        // No special attributes so far for small text field
        return Ok(field_config_map)
    }
    fn get_config(
        &mut self, 
        _: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError> {
        let config = self.config.clone();
        // No special attributes so far for small text field
        return Ok(config)
    }
    fn validate(&self, data: &Vec<String>) -> Result<Vec<String>, Vec<PlanetError>> {
        let data = data.clone();
        let field_config = self.config.clone();
        let set_validate = validate_set(&field_config, &data);
        if set_validate.is_err() {
            let error = set_validate.unwrap_err();
            let mut errors: Vec<PlanetError> = Vec::new();
            errors.push(error);
            return Err(errors)
        }
        let required = field_config.required.unwrap();
        let name = field_config.name.unwrap();
        let mut data_new: Vec<String> = Vec::new();
        for data_item in data {
            // eprintln!("CheckBoxColumn.is_valid :: value: {:?}", &value);
            if data_item == String::from("") && required == true {
                let error = PlanetError::new(
                    500, 
                    Some(tr!(
                        "Field {}{}{} is required", 
                        String::from("\"").blue(), name.blue(), String::from("\"").blue()
                    )),
                );
                let mut errors: Vec<PlanetError> = Vec::new();
                errors.push(error);
                return Err(errors);
            } else {
                let value_str = data_item.as_str();
                // eprintln!("CheckBoxColumn.is_valid :: value_str: {:?}", &value_str);
                if value_str == "true" || value_str == "false" {
                    // return Ok(data);
                    data_new.push(data_item);
                } else {
                    let error = PlanetError::new(
                        500, 
                        Some(tr!("Checkbox value needs to be \"true\" or \"false\"")),
                    );
                    let mut errors: Vec<PlanetError> = Vec::new();
                    errors.push(error);
                    return Err(errors);
                }
            }
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
    fn create_config(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let field_config_map = field_config_map.clone();
        // No special attributes so far for small text field
        return Ok(field_config_map)
    }
    fn get_config(
        &mut self, 
        _: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError> {
        let config = self.config.clone();
        // No special attributes so far for small text field
        return Ok(config)
    }
    fn validate(&self, data: &Vec<String>) -> Result<Vec<String>, Vec<PlanetError>> {
        let data = data.clone();
        let field_config = self.config.clone();
        let set_validate = validate_set(&field_config, &data);
        if set_validate.is_err() {
            let error = set_validate.unwrap_err();
            let mut errors: Vec<PlanetError> = Vec::new();
            errors.push(error);
            return Err(errors)
        }
        let required = *&field_config.required.unwrap();
        let name = &field_config.clone().name.unwrap().clone();
        let mut data_new: Vec<String> = Vec::new();
        let minimum = field_config.minimum.as_ref();
        let maximum = field_config.maximum.as_ref();
        for data_item in data {
            if data_item == String::from("") && required == true {
                let error = PlanetError::new(
                    500, 
                    Some(tr!(
                        "Field {}{}{} is required", 
                        String::from("\"").blue(), name.blue(), String::from("\"").blue()
                    )),
                );
                let mut errors: Vec<PlanetError> = Vec::new();
                errors.push(error);
                return Err(errors);
            } else {
                let value_str = data_item.as_str();
                let result = i32::from_str(value_str);
                match result {
                    Ok(_) => {
                        let value_int = result.unwrap().to_usize().unwrap();
                        if minimum.is_some() {
                            let minimum = minimum.unwrap();
                            let minimum: usize = FromStr::from_str(&minimum).unwrap();
                            if value_int < minimum {
                                let error = PlanetError::new(
                                    500, 
                                    Some(tr!(
                                        "Number value \"{}\" is smaller than minimum, \"{}\"", 
                                        &value_int, &minimum
                                    )),
                                );
                                let mut errors: Vec<PlanetError> = Vec::new();
                                errors.push(error);
                                return Err(errors);
                            }
                        }
                        if maximum.is_some() {
                            let maximum = maximum.unwrap();
                            let maximum: usize = FromStr::from_str(&maximum).unwrap();
                            if value_int > maximum {
                                let error = PlanetError::new(
                                    500, 
                                    Some(tr!(
                                        "Number value \"{}\" is bigger than maximum, \"{}\"", 
                                        &value_int, &maximum
                                    )),
                                );
                                let mut errors: Vec<PlanetError> = Vec::new();
                                errors.push(error);
                                return Err(errors);
                            }
                        }
                        data_new.push(data_item);
                    },
                    Err(_) => {
                        let error = PlanetError::new(
                            500, 
                            Some(tr!("I could not process as number: \"{}\"", &data_item)),
                        );
                        let mut errors: Vec<PlanetError> = Vec::new();
                        errors.push(error);
                        return Err(errors);
                    }
                }
            }
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
    fn create_config(
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
        } else {
            field_config_map.insert(CURRENCY_SYMBOL.to_string(), CURRENCY_SYMBOL_DOLLAR.to_string());
        }
        return Ok(field_config_map)
    }
    fn get_config(
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
    fn validate(&self, data: &Vec<String>) -> Result<Vec<String>, Vec<PlanetError>> {
        let data = data.clone();
        let config = self.config.clone();
        let set_validate = validate_set(&config, &data);
        if set_validate.is_err() {
            let error = set_validate.unwrap_err();
            let mut errors: Vec<PlanetError> = Vec::new();
            errors.push(error);
            return Err(errors)
        }
        let column_name = config.name.unwrap_or_default();
        let number_decimals = config.number_decimals;
        let currency_symbol = config.currency_symbol;
        if number_decimals.is_none() {
            let error = PlanetError::new(
                500, 
                Some(
                    tr!(
                        "Column \"{}\" not configured for currency, number decimals not configured.", 
                        &column_name
                    )
                ),
            );
            let mut errors: Vec<PlanetError> = Vec::new();
            errors.push(error);
            return Err(errors);
        }
        let number_decimals = number_decimals.unwrap();
        let number_decimals: u32 = number_decimals.to_u32().unwrap();
        let currency_symbol = currency_symbol.unwrap();
        let currency_symbol = currency_symbol.as_str();
        let mut data_new: Vec<String> = Vec::new();
        let expr = &RE_CURRENCY;
        for mut data_item in data {
            let match_data = data_item.clone();
            let is_valid = expr.is_match(&match_data);
            if !is_valid {
                let error = PlanetError::new(
                    500, 
                    Some(tr!("Validation error on currency \"{}\"", data_item.clone())),
                );
                let mut errors: Vec<PlanetError> = Vec::new();
                errors.push(error);
                return Err(errors);
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
                    data_item = data_item.clone().replace(symbol_pre, "");
                }
                if symbol_post_wrap.is_some() {
                    let symbol_post = symbol_post_wrap.unwrap().as_str();
                    // eprintln!("CurrencyColumn.validate :: [regex] symbol_post: {}", symbol_post);
                    data_item = data_item.clone().replace(symbol_post, "");
                }
            }
            data_item = data_item.trim().to_string();
            // eprintln!("CurrencyColumn.validate :: after symbol replace: {}", &data);
            if amount_wrap.is_some() {
                // Might be 7658.45 or 7658 or $7658. Need to get number without the currency symbol
                let amount_str = amount_wrap.unwrap().as_str();
                // eprintln!("CurrencyColumn.validate :: [regex] amount_str: {}", amount_str);
                // 79876.45
                // format amount to have number decimals from config
                let amount = Decimal::from_str(amount_str);
                if amount.is_err() {
                    let error = PlanetError::new(
                        500, 
                        Some(tr!("Validation error on currency \"{}\"", data_item.clone())),
                    );
                    let mut errors: Vec<PlanetError> = Vec::new();
                    errors.push(error);
                    return Err(errors);
                }
                let amount = amount.unwrap().round_dp(number_decimals);
                // amount_string = amount.to_string();
                let number_decimals = number_decimals.to_usize().unwrap();
                amount_string = format!("{:.1$}", &amount, number_decimals);
            }
            // eprintln!("CurrencyColumn.validate :: amount_string: {}", &amount_string);
            // data needs to have right number of decimals and the currency symbol
            data_new.push(format!("{}{}", currency_symbol, &amount_string));
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
    fn create_config(
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
    fn get_config(
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
    fn validate(&self, data: &Vec<String>) -> Result<Vec<String>, Vec<PlanetError>> {
        let data = data.clone();
        let config = self.config.clone();
        let set_validate = validate_set(&config, &data);
        if set_validate.is_err() {
            let error = set_validate.unwrap_err();
            let mut errors: Vec<PlanetError> = Vec::new();
            errors.push(error);
            return Err(errors)
        }
        let column_name = config.name.unwrap_or_default();
        let number_decimals = config.number_decimals;
        if number_decimals.is_none() {
            let error = PlanetError::new(
                500, 
                Some(tr!("Column not configured for percentage \"{}\"", &column_name)),
            );
            let mut errors: Vec<PlanetError> = Vec::new();
            errors.push(error);
            return Err(errors);
        }
        let number_decimals = number_decimals.unwrap();
        let number_decimals: u32 = number_decimals.to_u32().unwrap();
        let mut data_new: Vec<String> = Vec::new();
        for data_item in data {
            let amount_str = data_item.as_str();
            // format amount to have number decimals from config
            let amount = Decimal::from_str(amount_str);
            if amount.is_err() {
                let error = PlanetError::new(
                    500, 
                    Some(tr!("Validation error on percentage \"{}\"", data_item.clone())),
                );
                let mut errors: Vec<PlanetError> = Vec::new();
                errors.push(error);
                return Err(errors);
            }
            let amount = amount.unwrap().round_dp(number_decimals);
            let number_decimals = number_decimals.to_usize().unwrap();
            data_new.push(format!("{:.1$}", &amount, number_decimals));
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
        let value = format!("{}", value.to_string().truecolor(
            YAML_COLOR_YELLOW[0], YAML_COLOR_YELLOW[1], YAML_COLOR_YELLOW[2]
        ));
        yaml_string.push_str(format!("  {field}: {value}\n", field=field, value=value).as_str());
        return yaml_string;
    }
}


#[derive(Debug, Clone)]
pub struct GenerateNumberColumn {
    pub config: ColumnConfig,
    pub folder: Option<DbData>,
    pub db_folder: Option<TreeFolder>,
}
impl GenerateNumberColumn {
    pub fn defaults(
        field_config: &ColumnConfig,
        folder: Option<DbData>,
        db_folder: Option<TreeFolder>
    ) -> Self {
        let field_config = field_config.clone();
        let field_obj = Self{
            config: field_config,
            folder: folder,
            db_folder: db_folder
        };
        return field_obj
    }
}
impl StorageColumn for GenerateNumberColumn {
    fn create_config(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let mut field_config_map = field_config_map.clone();
        field_config_map.insert(SEQUENCE.to_string(), String::from("0"));
        return Ok(field_config_map)
    }
    fn get_config(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError> {
        let mut config = self.config.clone();
        let sequence = field_config_map.get(SEQUENCE);
        if sequence.is_some() {
            let sequence = sequence.unwrap().clone();
            config.sequence = Some(sequence);
        }
        return Ok(config)
    }
    fn validate(&self, _data: &Vec<String>) -> Result<Vec<String>, Vec<PlanetError>> {
        let config = self.config.clone();
        let mut folder = self.folder.clone().unwrap();
        let db_folder = self.db_folder.clone().unwrap();
        let mut data = folder.data.unwrap();
        let column_id = config.id.unwrap();
        let column_name = config.name.unwrap();
        let columns = data.get(COLUMNS);
        let mut columns_new: Vec<BTreeMap<String, String>> = Vec::new();
        let mut sequence_list: Vec<String> = Vec::new();
        if columns.is_some() {
            let columns = columns.unwrap().clone();
            for mut column in columns {
                let column_target_id = column.get(ID);
                if column_target_id.is_some() {
                    let column_target_id = column_target_id.unwrap().clone();
                    if column_id == column_target_id {
                        let sequence = column.get(SEQUENCE);
                        if sequence.is_some() {
                            let sequence = sequence.unwrap();
                            let mut sequence: usize = FromStr::from_str(sequence).unwrap();
                            sequence += 1;
                            column.insert(SEQUENCE.to_string(), sequence.to_string());
                            columns_new.push(column);
                            sequence_list.push(sequence.to_string());
                        }            
                    } else {
                        columns_new.push(column);
                    }
                }
            }
            data.insert(COLUMNS.to_string(), columns_new.clone());
            folder.data = Some(data.clone());
            let result = db_folder.update(&folder);
            if result.is_ok() {
                return Ok(sequence_list)
            }
        }
        let error = PlanetError::new(
            500, 
            Some(tr!("Error generating new number for column \"{}\".", &column_name)),
        );
        let mut errors: Vec<PlanetError> = Vec::new();
        errors.push(error);
        return Err(errors)
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


#[derive(Debug, Clone)]
pub struct RatingColumn {
    pub config: ColumnConfig,
}
impl RatingColumn {
    pub fn defaults(
        field_config: &ColumnConfig,
    ) -> Self {
        let field_config = field_config.clone();
        let field_obj = Self{
            config: field_config,
        };
        return field_obj
    }
}
impl StorageColumn for RatingColumn {
    fn create_config(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let mut field_config_map = field_config_map.clone();
        let config = self.config.clone();
        let maximum = config.maximum;
        let minimum = config.minimum;
        if maximum.is_some() {
            let maximum = maximum.unwrap();
            field_config_map.insert(MAXIMUM.to_string(), maximum);
        } else {
            let maximum: String = String::from("5");
            field_config_map.insert(MAXIMUM.to_string(), maximum);
        }
        if minimum.is_some() {
            let minimum = minimum.unwrap();
            field_config_map.insert(MINIMUM.to_string(), minimum);
        }
        return Ok(field_config_map)
    }
    fn get_config(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError> {
        let mut config = self.config.clone();
        let minimum = field_config_map.get(MINIMUM);
        let maximum = field_config_map.get(MAXIMUM);
        if minimum.is_some() {
            let minimum = minimum.unwrap();
            config.minimum = Some(minimum.clone());
        }
        if maximum.is_some() {
            let maximum = maximum.unwrap();
            config.maximum = Some(maximum.clone());
        }
        return Ok(config)
    }
    fn validate(&self, data: &Vec<String>) -> Result<Vec<String>, Vec<PlanetError>> {
        let config = self.config.clone();
        let data = data.clone();
        let set_validate = validate_set(&config, &data);
        if set_validate.is_err() {
            let error = set_validate.unwrap_err();
            let mut errors: Vec<PlanetError> = Vec::new();
            errors.push(error);
            return Err(errors)
        }
        let column_name = config.name.unwrap_or_default();
        let minimum = config.minimum.as_ref();
        let maximum = config.maximum.as_ref();
        let mut data_new: Vec<String> = Vec::new();
        for data_item in data {
            let test = &data_item.parse::<f64>();
            if test.is_err() {
                let error = PlanetError::new(
                    500, 
                    Some(tr!("Column value {} for column \"{}\" is not a number.", 
                    &data_item, &column_name)),
                );
                let mut errors: Vec<PlanetError> = Vec::new();
                errors.push(error);
                return Err(errors)
            }
            let data_int: usize = FromStr::from_str(&data_item).unwrap();
            if minimum.is_some() {
                let minimum = minimum.unwrap();
                let minimum: usize = FromStr::from_str(&minimum).unwrap();
                if data_int < minimum {
                    let error = PlanetError::new(
                        500, 
                        Some(tr!(
                            "Rating for column \"{}\" is lower than minimum, {}.", 
                            &column_name, &minimum)),
                    );
                    let mut errors: Vec<PlanetError> = Vec::new();
                    errors.push(error);
                    return Err(errors)
                }
            }
            if maximum.is_some() {
                let maximum = maximum.unwrap();
                let maximum: usize = FromStr::from_str(&maximum).unwrap();
                if data_int > maximum {
                    let error = PlanetError::new(
                        500, 
                        Some(tr!(
                            "Rating for column \"{}\" is higher than maximum, {}.", 
                            &column_name, &maximum)),
                    );
                    let mut errors: Vec<PlanetError> = Vec::new();
                    errors.push(error);
                    return Err(errors)
                }
            }
            data_new.push(data_item);
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
        let value = format!("{}", value.to_string().truecolor(
            YAML_COLOR_YELLOW[0], YAML_COLOR_YELLOW[1], YAML_COLOR_YELLOW[2]
        ));
        yaml_string.push_str(format!("  {field}: {value}\n", field=field, value=value).as_str());
        return yaml_string;
    }
}

#[derive(Debug, Clone)]
pub struct StatsColumn {
    pub config: ColumnConfig,
    pub allowed_functions: [String; 10],
    pub column_config_map: Option<BTreeMap<String, ColumnConfig>>,
    pub folder_name: Option<String>,
    pub db_folder: Option<TreeFolder>,
    pub properties_map: Option<HashMap<String, ColumnConfig>>,
    pub data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
}
impl StatsColumn {
    pub fn defaults(
        field_config: &ColumnConfig,
        column_config_map: Option<BTreeMap<String, ColumnConfig>>,
        folder_name: Option<String>,
        db_folder: Option<TreeFolder>,
        properties_map: Option<HashMap<String, ColumnConfig>>,
        data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
    ) -> Self {
        let field_config = field_config.clone();
        let field_obj = Self{
            config: field_config,
            allowed_functions: [
                STATS_FUNCTION_COUNT.to_string(),
                STATS_FUNCTION_COUNTA.to_string(),
                STATS_FUNCTION_COUNTALL.to_string(),
                STATS_FUNCTION_MAX.to_string(),
                STATS_FUNCTION_MIN.to_string(),
                STATS_FUNCTION_AVG.to_string(),
                STATS_FUNCTION_SUM.to_string(),
                STATS_FUNCTION_AND.to_string(),
                STATS_FUNCTION_OR.to_string(),
                STATS_FUNCTION_XOR.to_string(),
            ],
            column_config_map: column_config_map,
            folder_name: folder_name,
            db_folder: db_folder,
            properties_map: properties_map,
            data_map: data_map,
        };
        return field_obj
    }
}
impl StorageColumn for StatsColumn {
    fn create_config(
        &mut self, 
        column_config_map: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let mut column_config_map = column_config_map.clone();
        let config = self.config.clone();
        let stats_function = config.stats_function;
        if stats_function.is_some() {
            let stats_function = stats_function.unwrap();
            let check = self.allowed_functions.contains(&stats_function);
            if check {
                column_config_map.insert(STATS_FUNCTION.to_string(), stats_function);
            } else {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!(
                            "Stats function \"{}\" not allowed", &stats_function
                        )),
                    )
                )
            }
        }
        let related_column = config.related_column;
        if related_column.is_some() {
            let related_column = related_column.unwrap();
            column_config_map.insert(RELATED_COLUMN.to_string(), related_column);
        } else {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!(
                        "You need to define a related column where to apply the statistics."
                    )),
                )
            )
        }
        return Ok(column_config_map)
    }
    fn get_config(
        &mut self, 
        column_config_map: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError> {
        let mut config = self.config.clone();
        let stats_function = column_config_map.get(STATS_FUNCTION);
        if stats_function.is_some() {
            let stats_function = stats_function.unwrap();
            let check = self.allowed_functions.contains(stats_function);
            if check {
                config.stats_function = Some(stats_function.clone());
            } else {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!(
                            "Stats function \"{}\" not allowed", &stats_function
                        )),
                    )
                )
            }
        }
        let related_column = column_config_map.get(RELATED_COLUMN);
        if related_column.is_some() {
            let related_column = related_column.unwrap();
            config.related_column = Some(related_column.clone());
        } else {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!(
                        "You need to define a related column where to apply the statistics."
                    )),
                )
            )
        }
        return Ok(config)
    }
    fn validate(&self, _data: &Vec<String>) -> Result<Vec<String>, Vec<PlanetError>> {
        let config = self.config.clone();
        // eprintln!("StatsColumn.validate :: config: {:#?}", &config);
        // eprintln!("StatsColumn.validate :: data: {:#?}", &data);
        let stats_function = config.stats_function;
        let related_column = self.config.related_column.clone().unwrap();
        let data_map = self.data_map.clone().unwrap();
        // eprintln!("StatsColumn.validate :: data_map: {:#?}", &data_map);
        let column_config_map = self.column_config_map.clone();
        if column_config_map.is_none() {
            let error = PlanetError::new(
                500, 
                Some(tr!(
                    "Init error for \"column_config_map\"."
                )),
            );
            let mut errors: Vec<PlanetError> = Vec::new();
            errors.push(error);
            return Err(errors)
        }
        let column_config_map = column_config_map.unwrap();
        let related_column_config = column_config_map.get(&related_column);
        if related_column_config.is_some() {
            let related_column_config = related_column_config.unwrap();
            let related_column_id = related_column_config.id.clone().unwrap();
            let has_column = data_map.get(&related_column_id).is_some();
            if !has_column {
                let error = PlanetError::new(
                    500, 
                    Some(tr!(
                        "Column object data has no column values for \"{}\".", &related_column
                    )),
                );
                let mut errors: Vec<PlanetError> = Vec::new();
                errors.push(error);
                return Err(errors)
            }    
        }
        
        let folder_name = self.folder_name.clone().unwrap();
        let db_folder = self.db_folder.clone().unwrap();
        let properties_map = self.properties_map.clone().unwrap();
        
        let mut data_new: Vec<String> = Vec::new();
        if stats_function.is_some() {
            let stats_function = stats_function.unwrap();
            // eprintln!("StatsColumn.validate :: stats_function: {}", &stats_function);
            let stats_function = stats_function.as_str();
            let formula: String;
            let formula_format = String::from(FORMULA_FORMAT_NUMBER);
            match stats_function {
                STATS_FUNCTION_COUNT => {
                    formula = format!("COUNT({{{}}})", &related_column);
                },
                STATS_FUNCTION_COUNTA => {
                    formula = format!("COUNTA({{{}}})", &related_column);
                },
                STATS_FUNCTION_COUNTALL => {
                    formula = format!("COUNTALL({{{}}})", &related_column);
                },
                STATS_FUNCTION_MAX => {
                    formula = format!("MAX({{{}}})", &related_column);
                },
                STATS_FUNCTION_MIN => {
                    formula = format!("MIN({{{}}})", &related_column);
                },
                STATS_FUNCTION_AVG => {
                    formula = format!("AVG({{{}}})", &related_column);
                },
                STATS_FUNCTION_SUM => {
                    formula = format!("SUM({{{}}})", &related_column);
                },
                _ => {
                    let error = PlanetError::new(
                        500, 
                        Some(tr!(
                            "Stats function \"{}\" not allowed", &stats_function
                        )),
                    );
                    let mut errors: Vec<PlanetError> = Vec::new();
                    errors.push(error);
                    return Err(errors)
                }
            }
            // eprintln!("StatsColumn.validate :: formula: {:#?}", &formula);
            let formula_compiled = Formula::defaults(
                &formula,
                &formula_format,
                None,
                Some(properties_map.clone()),
                Some(db_folder.clone()),
                Some(folder_name.clone()),
                false,
                None
            );
            if formula_compiled.is_err() {
                let error = formula_compiled.unwrap_err();
                let mut errors: Vec<PlanetError> = Vec::new();
                errors.push(error);
                return Err(errors)
            }
            let formula_compiled = formula_compiled.unwrap();
            let formula_result = execute_formula(
                &formula_compiled, 
                &data_map, 
                None,
                &column_config_map
            );
            if formula_result.is_err() {
                let error = formula_result.unwrap_err();
                let mut errors: Vec<PlanetError> = Vec::new();
                errors.push(error);
                return Err(errors)
            }
            let formula_result = formula_result.unwrap();
            data_new.push(formula_result.result);
        }
        // eprintln!("StatsColumn.validate :: data_new: {:#?}", &data_new);
        return Ok(data_new)
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
