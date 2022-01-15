use std::collections::BTreeMap;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use tr::tr;

use crate::planet::constants::ID;
use crate::planet::{PlanetError};
use crate::storage::folder::{DbData};
use crate::commands::folder::config::ColumnConfig;
use crate::storage::constants::*;
use crate::planet::constants::*;
use crate::storage::columns::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SmallTextColumn {
    pub config: ColumnConfig
}
impl SmallTextColumn {
    pub fn defaults(field_config: &ColumnConfig) -> Self {
        let field_config = field_config.clone();
        let field_obj = Self{
            config: field_config
        };
        return field_obj
    }
}
impl StorageColumn for SmallTextColumn {
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
        let config = self.config.clone();
        let required = config.required.unwrap();
        let name = config.name.unwrap();
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
        }
        return Ok(data)
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
pub struct LongTextColumn {
    pub config: ColumnConfig
}
impl LongTextColumn {
    pub fn defaults(field_config: &ColumnConfig) -> Self {
        let field_config = field_config.clone();
        let field_obj = Self{
            config: field_config
        };
        return field_obj
    }
}
impl StorageColumn for LongTextColumn {
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
        let config = self.config.clone();
        let required = config.required.unwrap();
        let name = config.name.unwrap();
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
        }
        return Ok(data)
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
pub struct SelectColumn {
    pub config: ColumnConfig,
    pub options_id_map: Option<BTreeMap<String, String>>,
    pub options_name_map: Option<BTreeMap<String, String>>,
}
impl SelectColumn {
    pub fn defaults(field_config: &ColumnConfig, table: Option<&DbData>) -> Self {
        let field_config = field_config.clone();
        let mut field_obj = Self{
            config: field_config,
            options_id_map: None,
            options_name_map: None,
        };
        if table.is_some() {
            let table = table.unwrap();
            let mut options_id_map: BTreeMap<String, String> = BTreeMap::new();
            let mut options_name_map: BTreeMap<String, String> = BTreeMap::new();
            let field_config = &field_obj.config;
            let field_name = field_config.name.clone().unwrap();
            for data_collection in table.data_collections.clone() {
                // key for ordering: field_ids
                for key in data_collection.keys() {
                    if key.to_lowercase() != String::from(COLUMN_IDS) {
                        // key: Status__select_options
                        let key_items: Vec<&str> = key.split("__").collect();
                        let key_field_name = key_items[0];
                        let key_field_type = key_items[1];
                        if key_field_type == SELECT_OPTIONS && key_field_name.to_lowercase() == field_name.to_lowercase() {
                            // Process, since we have a simple select field
                            // "Status__select_options": [
                            //     {
                            //         "key": "c48kg78smpv5gct3hfqg",
                            //         "value": "Draft",
                            //     },
                            //     ...
                            // ],
                            let options: &Vec<BTreeMap<String, String>> = data_collection.get(key).unwrap();
                            for option in options {
                                options_id_map.insert(
                                    option.get(KEY).unwrap().clone(), 
                                    option.get(VALUE).unwrap().clone()
                                );
                                options_name_map.insert(
                                    option.get(VALUE).unwrap().clone(), 
                                    option.get(KEY).unwrap().clone()
                                );
                            }
                        }
                    }
                }
            }
            field_obj.options_id_map = Some(options_id_map);
            field_obj.options_name_map = Some(options_name_map);
        }
        return field_obj;
    }
}
impl StorageColumn for SelectColumn {
    fn update_config_map(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let mut field_config_map = field_config_map.clone();
        let field_config = self.config.clone();
        let field_name = field_config.name.unwrap_or_default();
        let options = field_config.options;
        if options.is_some() {
            let options_yaml = serde_yaml::to_string(&options);
            if options_yaml.is_ok() {
                field_config_map.insert(String::from(OPTIONS), options_yaml.unwrap());
            } else {
                panic!("Could not parse options for field \"{}\"", &field_name);
            }
        }
        return Ok(field_config_map)
    }
    fn build_config(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError> {
        let mut config = self.config.clone();
        let options_str = field_config_map.get(OPTIONS);
        let options_wrap: Option<Vec<String>>;
        if options_str.is_some() {
            let options_str = field_config_map.get(OPTIONS).unwrap().clone();
            let options_str = options_str.as_str();
            let options: Vec<String> = serde_yaml::from_str(options_str).unwrap();
            options_wrap = Some(options);
            config.options = options_wrap;
        }
        return Ok(config)
    }
    fn validate(&self, data: &String) -> Result<String, PlanetError> {
        let data = data.clone();
        // value represents the id for the option selected, like id->name
        let field_config = self.config.clone();
        let required = field_config.required.unwrap();
        let name = field_config.name.unwrap();
        // let field_name = self.field.name.clone().unwrap_or_default();
        if data == String::from("") && required == true {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!(
                        "Field {}{}{} is required", 
                        String::from("\"").blue(), &name.blue(), String::from("\"").blue()
                    )),
                )
            );
        } else {
            // In case no many, just string with id. In case many, list of ids separated by commas
            let value_id = data.clone();
            let id_list: Vec<&str> = value_id.split(",").collect();
            let field_config = self.config.clone();
            // Check that value appears on the config for choices id -> value
            // The option id is obtained from the table config
            let options = field_config.options.unwrap();
            let options_name_map = &self.options_name_map.clone().unwrap();
            let options_id_map = &self.options_id_map.clone().unwrap();
            let mut verified = false;
            if *&id_list.len() == 1 {
                for select_option in options {
                    let select_id = options_name_map.get(&select_option);
                    if select_id.is_some() {
                        let select_id = options_name_map.get(&select_option).unwrap().clone();
                        if select_id == value_id {
                            verified = true;
                            break;
                        }
                    }
                }
            } else {
                verified = true;
                for item_select_id in id_list {
                    let item_select_id = &item_select_id.to_string();
                    let select_option = options_id_map.get(item_select_id);
                    if select_option.is_none() {
                        verified = false;
                        break
                    }
                }
            }
            if verified == true {
                return Ok(data.clone())
            } else {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!(
                            "Field {}{}{} is not configured with select id {}{}{}", 
                            String::from("\"").blue(), &name.blue(), String::from("\"").blue(),
                            String::from("\"").blue(), value_id, String::from("\"").blue(),
                        )),
                    )
                );
            }            
        }
    }
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String {
        let field_config = self.config.clone();
        let field_name = field_config.name.unwrap();
        let many = field_config.many.unwrap();
        let mut yaml_string = yaml_string.clone();
        let field = &field_name.truecolor(
            YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
        );
        let select_id = value;
        // let options_name_map = &self.options_name_map.clone().unwrap();
        let options_id_map = &self.options_id_map.clone().unwrap();
        if many == false {
            let select_name = options_id_map.get(select_id).unwrap();
            let select_id = format!("{}", 
                value.truecolor(YAML_COLOR_ORANGE[0], YAML_COLOR_ORANGE[1], YAML_COLOR_ORANGE[2]), 
            );
            let select_name = format!("{}", 
                select_name.truecolor(YAML_COLOR_ORANGE[0], YAML_COLOR_ORANGE[1], YAML_COLOR_ORANGE[2]), 
            );
            yaml_string.push_str(format!("  {field}:\n    {id}: {select_id}\n    {value}: {select_name}\n", 
                field=&field, 
                select_id=select_id,
                select_name=select_name,
                id=String::from(ID).truecolor(YAML_COLOR_ORANGE[0], YAML_COLOR_ORANGE[1], YAML_COLOR_ORANGE[2]),
                value=String::from(VALUE).truecolor(
                    YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
                ),
            ).as_str());
        } else {
            // Check we have fields
            let select_ids: Vec<&str> = value.split(",").collect();
            if select_ids.len() > 1 {
                yaml_string.push_str(format!("  {field}:\n", 
                    field=&field, 
                ).as_str());
                for select_id in select_ids {
                    let select_name = options_id_map.get(select_id).unwrap();
                    let select_id = format!("{}", 
                        select_id.truecolor(YAML_COLOR_ORANGE[0], YAML_COLOR_ORANGE[1], YAML_COLOR_ORANGE[2]), 
                    );
                    let select_name = format!("{}", 
                        select_name.truecolor(YAML_COLOR_ORANGE[0], YAML_COLOR_ORANGE[1], YAML_COLOR_ORANGE[2]), 
                    );
                    yaml_string.push_str(format!("    - {id}: {select_id}\n    {value}: {select_name}\n", 
                        select_id=select_id,
                        select_name=select_name,
                        id=String::from(ID).truecolor(
                            YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
                        ),
                        value=String::from(VALUE).truecolor(
                            YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
                        ),
                    ).as_str());
                }
            }
        }
        return yaml_string;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditByColumn {
    pub config: ColumnConfig
}
impl AuditByColumn {
    pub fn defaults(field_config: &ColumnConfig) -> Self {
        let field_config = field_config.clone();
        let field_obj = Self{
            config: field_config
        };
        return field_obj
    }
}
impl StorageColumn for AuditByColumn {
    fn update_config_map(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let field_config_map = field_config_map.clone();
        return Ok(field_config_map)
    }
    fn build_config(
        &mut self, 
        _: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError> {
        let config = self.config.clone();
        return Ok(config)
    }
    fn validate(&self, data: &String) -> Result<String, PlanetError> {
        // CreatedBy: I map into insert
        // LastModifiedBy: I map into insert and update
        // I save the user id
        // The user id should not come from the payload, but from the auth system
        // TODO: Check user_id exists on table of users, or any other mean of storage
        let user_id = data.clone();
        return Ok(user_id)
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
