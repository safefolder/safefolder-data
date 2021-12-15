use std::str::FromStr;
use std::collections::HashMap;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use tr::tr;

use crate::planet::{PlanetError};
use crate::storage::table::{DbData};
use crate::commands::table::config::FieldConfig;
use crate::storage::constants::*;
use crate::storage::fields::*;

pub trait DbDumpBool {
    fn get_yaml_out(&self, yaml_string: &String, value: bool) -> String;
}
pub trait DbDumpNumber {
    fn get_yaml_out(&self, yaml_string: &String, value: &i32) -> String;
}

pub trait NumberValueField {
    fn get_value(&self, value_db: Option<&String>) -> Option<i32>;
    fn get_value_db(&self, value: Option<&i32>) -> Option<String>;
}
pub trait BoolValueField {
    fn get_value(&self, value_db: Option<&String>) -> Option<bool>;
    fn get_value_db(&self, value: Option<bool>) -> Option<String>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CheckBoxField {
    pub field_config: FieldConfig
}

impl CheckBoxField {
    pub fn defaults(field_config: &FieldConfig) -> Self {
        let field_config = field_config.clone();
        let field_obj = Self{
            field_config: field_config
        };
        return field_obj
    }
    pub fn init_do(field_config: &FieldConfig, data_map: HashMap<String, String>, mut db_data: DbData) -> Result<DbData, PlanetError> {
        let field_object = Self::defaults(field_config);
        db_data = field_object.process(data_map.clone(), db_data)?;
        return Ok(db_data)
    }
    pub fn init_get(
        field_config: &FieldConfig, 
        data: Option<&HashMap<String, String>>, 
        yaml_out_str: &String
    ) -> Result<String, PlanetError> {
        let field_config_ = field_config.clone();
        let field_id = field_config_.id.unwrap();
        let data = data.unwrap().clone();
        let field_obj = Self::defaults(&field_config);
        let value_db = data.get(&field_id);
        if value_db.is_some() {
            let value_db = value_db.unwrap().clone();
            let value = field_obj.get_value(Some(&value_db)).unwrap();
            let yaml_out_str = field_obj.get_yaml_out(yaml_out_str, value);    
            return Ok(yaml_out_str)
        }
        let yaml_out_str = yaml_out_str.clone();
        return Ok(yaml_out_str)
    }
}

impl DbDumpBool for CheckBoxField {
    fn get_yaml_out(&self, yaml_string: &String, value: bool) -> String {
        let field_config = self.field_config.clone();
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

impl ValidateField for CheckBoxField {
    fn is_valid(&self, value: Option<&String>) -> Result<bool, PlanetError> {
        let field_config = self.field_config.clone();
        let required = field_config.required.unwrap();
        let name = field_config.name.unwrap();
        // eprintln!("CheckBoxField.is_valid :: value: {:?}", &value);
        if value.is_none() && required == true {
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
            let value_str = value.unwrap().as_str();
            // eprintln!("CheckBoxField.is_valid :: value_str: {:?}", &value_str);
            if value_str == "true" || value_str == "false" {
                return Ok(true);
            } else {
                return Ok(false)
            }
        }
    }
}
impl ProcessField for CheckBoxField {
    fn process(
        &self,
        insert_data_map: HashMap<String, String>,
        mut db_data: DbData
    ) -> Result<DbData, PlanetError> {
        let field_config = self.field_config.clone();
        let field_name = field_config.name.unwrap_or_default();
        let field_id = field_config.id.unwrap_or_default();
        let value_entry = insert_data_map.get(&field_id).unwrap().clone();
        let value_db = value_entry.clone();
        let field = Self{
            field_config: self.field_config.clone()
        };
        let mut data: HashMap<String, String> = HashMap::new();
        if db_data.data.is_some() {
            data = db_data.data.unwrap();
        }
        let is_valid = field.is_valid(Some(&value_db))?;
        if is_valid == true {
            &data.insert(field_id, value_db);
            db_data.data = Some(data);
            return Ok(db_data);
        } else {
            return Err(error_validate_process("CheckBox", &field_name))
        }
    }
}
impl BoolValueField for CheckBoxField {
    fn get_value(&self, value_db: Option<&String>) -> Option<bool> {
        if value_db.is_none() {
            return None
        } else {
            let value_str = value_db.unwrap().as_str();
            if value_str == "true" {
                return Some(true);
            } else {
                return Some(false);
            }
        }
    }
    fn get_value_db(&self, value: Option<bool>) -> Option<String> {
        if *&value.is_some() {
            let value = value.unwrap();
            if value == true {
                return Some(String::from("true"))
            } else {
                return Some(String::from("false"));
            }
        } else {
            return None
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NumberField {
    pub field_config: FieldConfig
}
impl NumberField {
    pub fn defaults(field_config: &FieldConfig) -> Self {
        let field_config = field_config.clone();
        let field_obj = Self{
            field_config: field_config
        };
        return field_obj
    }
    pub fn init_do(field_config: &FieldConfig, data_map: HashMap<String, String>, mut db_data: DbData) -> Result<DbData, PlanetError> {
        let field_object = Self::defaults(field_config);
        db_data = field_object.process(data_map.clone(), db_data)?;
        return Ok(db_data)
    }
    pub fn init_get(
        field_config: &FieldConfig, 
        data: Option<&HashMap<String, String>>, 
        yaml_out_str: &String
    ) -> Result<String, PlanetError> {
        let field_config_ = field_config.clone();
        let field_id = field_config_.id.unwrap();
        let data = data.unwrap().clone();
        let field_obj = Self::defaults(&field_config);
        let value_db = data.get(&field_id);
        if value_db.is_some() {
            let value_db = value_db.unwrap().clone();
            let value = field_obj.get_value(Some(&value_db)).unwrap();
            let yaml_out_str = field_obj.get_yaml_out(yaml_out_str, &value);    
            return Ok(yaml_out_str)
        }
        let yaml_out_str = yaml_out_str.clone();
        return Ok(yaml_out_str)
    }
}
impl DbDumpNumber for NumberField {
    fn get_yaml_out(&self, yaml_string: &String, value: &i32) -> String {
        let field_config = self.field_config.clone();
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
impl ValidateField for NumberField {
    fn is_valid(&self, value: Option<&String>) -> Result<bool, PlanetError> {
        let field_config = self.field_config.clone();
        let required = field_config.required.unwrap();
        let name = field_config.name.unwrap();
        if value.is_none() && required == true {
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
            let value_str = value.unwrap().as_str();
            let result = i32::from_str(value_str);
            match result {
                Ok(_) => {
                    // let value: i32 = result.unwrap();
                    return Ok(true);
                },
                Err(_) => {
                    return Ok(false)
                }
            }
        }
    }
}
impl ProcessField for NumberField {
    fn process(
        &self,
        insert_data_map: HashMap<String, String>,
        mut db_data: DbData
    ) -> Result<DbData, PlanetError> {
        let field_config = self.field_config.clone();
        let field_name = field_config.name.unwrap_or_default();
        let field_id = field_config.id.unwrap_or_default();
        let value_entry = insert_data_map.get(&field_id).unwrap().clone();
        let value_db = value_entry.clone();
        let field = Self{
            field_config: self.field_config.clone()
        };
        let mut data: HashMap<String, String> = HashMap::new();
        if db_data.data.is_some() {
            data = db_data.data.unwrap();
        }
        let is_valid = field.is_valid(Some(&value_entry))?;
        if is_valid == true {
            &data.insert(field_id, value_db);
            db_data.data = Some(data);
            return Ok(db_data);
        } else {
            return Err(error_validate_process("Number", &field_name))
        }
    }
}
impl NumberValueField for NumberField {
    fn get_value(&self, value_db: Option<&String>) -> Option<i32> {
        if value_db.is_none() {
            return None
        } else {
            let value_str = value_db.unwrap().as_str();
            let value: i32 = i32::from_str(value_str).unwrap();
            return Some(value)
        }
    }
    fn get_value_db(&self, value: Option<&i32>) -> Option<String> {
        if *&value.is_some() {
            let value = value.unwrap();
            let value_str = value.to_string();
            return Some(value_str);
        } else {
            return None
        }
    }
}
