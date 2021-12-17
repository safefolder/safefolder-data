use std::collections::HashMap;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use tr::tr;

use crate::planet::constants::ID;
use crate::planet::{PlanetError};
use crate::storage::table::{DbData};
use crate::commands::table::config::FieldConfig;
use crate::storage::constants::*;
use crate::planet::constants::*;
use crate::storage::fields::*;

pub trait DbDumpString {
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String;
}

/* pub trait DbDumpSingleSelect {
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String;
}
 */
pub trait StringValueField {
    fn get_value(&self, value_db: Option<&String>) -> Option<String>;
    fn get_value_db(&self, value: Option<&String>) -> Option<String>;
}
pub trait StringVectorValueField {
    fn get_value(&self, values_db: Option<Vec<HashMap<String, String>>>) -> Option<Vec<HashMap<String, String>>>;
    fn get_value_db(&self, value: Option<Vec<String>>) -> Option<Vec<HashMap<String, String>>>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SmallTextField {
    pub config: FieldConfig
}
impl SmallTextField {
    pub fn defaults(field_config: &FieldConfig) -> Self {
        let field_config = field_config.clone();
        let field_obj = Self{
            config: field_config
        };
        return field_obj
    }
}
impl StorageField for SmallTextField {
    fn update_config_map(
        &mut self, 
        field_config_map: &HashMap<String, String>,
    ) -> Result<HashMap<String, String>, PlanetError> {
        let field_config_map = field_config_map.clone();
        // No special attributes so far for small text field
        return Ok(field_config_map)
    }
    fn build_config(
        &mut self, 
        _: &HashMap<String, String>,
    ) -> Result<FieldConfig, PlanetError> {
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
pub struct LongTextField {
    pub config: FieldConfig
}
impl LongTextField {
    pub fn defaults(field_config: &FieldConfig) -> Self {
        let field_config = field_config.clone();
        let field_obj = Self{
            config: field_config
        };
        return field_obj
    }
}
impl StorageField for LongTextField {
    fn update_config_map(
        &mut self, 
        field_config_map: &HashMap<String, String>,
    ) -> Result<HashMap<String, String>, PlanetError> {
        let field_config_map = field_config_map.clone();
        // No special attributes so far for small text field
        return Ok(field_config_map)
    }
    fn build_config(
        &mut self, 
        _: &HashMap<String, String>,
    ) -> Result<FieldConfig, PlanetError> {
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
pub struct SelectField {
    pub config: FieldConfig,
    pub options_id_map: Option<HashMap<String, String>>,
    pub options_name_map: Option<HashMap<String, String>>,
}
impl SelectField {
    pub fn defaults(field_config: &FieldConfig, table: Option<&DbData>) -> Self {
        let field_config = field_config.clone();
        let mut field_obj = Self{
            config: field_config,
            options_id_map: None,
            options_name_map: None,
        };
        if table.is_some() {
            let table = table.unwrap();
            let mut options_id_map: HashMap<String, String> = HashMap::new();
            let mut options_name_map: HashMap<String, String> = HashMap::new();
            let field_config = &field_obj.config;
            let field_name = field_config.name.clone().unwrap();
            for data_collection in table.data_collections.clone() {
                // key for ordering: field_ids
                for key in data_collection.keys() {
                    if key.to_lowercase() != String::from(FIELD_IDS) {
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
                            let options: &Vec<HashMap<String, String>> = data_collection.get(key).unwrap();
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
impl StorageField for SelectField {
    fn update_config_map(
        &mut self, 
        field_config_map: &HashMap<String, String>,
    ) -> Result<HashMap<String, String>, PlanetError> {
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
        field_config_map: &HashMap<String, String>,
    ) -> Result<FieldConfig, PlanetError> {
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

/* #[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SelectField {
    pub field_config: FieldConfig,
    pub options_id_map: Option<HashMap<String, String>>,
    pub options_name_map: Option<HashMap<String, String>>,
}
impl SelectField {
    pub fn defaults(field_config: &FieldConfig, table: Option<&DbData>) -> Self {
        let field_config = field_config.clone();
        let mut field_obj = Self{
            field_config: field_config,
            options_id_map: None,
            options_name_map: None,
        };
        if table.is_some() {
            let table = table.unwrap();
            let mut options_id_map: HashMap<String, String> = HashMap::new();
            let mut options_name_map: HashMap<String, String> = HashMap::new();
            let field_config = &field_obj.field_config;
            let field_name = field_config.name.clone().unwrap();
            for data_collection in table.data_collections.clone() {
                // key for ordering: field_ids
                for key in data_collection.keys() {
                    if key.to_lowercase() != String::from(FIELD_IDS) {
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
                            let options: &Vec<HashMap<String, String>> = data_collection.get(key).unwrap();
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
    pub fn init_do(
        field_config: &FieldConfig, 
        table: &DbData,
        data_map: HashMap<String, String>, 
        mut db_data: DbData
    ) -> Result<DbData, PlanetError> {
        let field_object = Self::defaults(field_config, Some(table));
        db_data = field_object.process(data_map.clone(), db_data)?;
        return Ok(db_data)
    }
    pub fn init_get(
        field_config: &FieldConfig, 
        table: &DbData,
        data: Option<&HashMap<String, String>>, 
        yaml_out_str: &String
    ) -> Result<String, PlanetError> {
        let field_config_ = field_config.clone();
        let field_id = field_config_.id.unwrap();
        let data = data.unwrap().clone();
        let field_obj = Self::defaults(&field_config, Some(table));
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
impl DbDumpSingleSelect for SelectField {
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String {
        let field_config = self.field_config.clone();
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
impl ValidateField for SelectField {
    fn is_valid(&self, value: Option<&String>) -> Result<bool, PlanetError> {
        // value represents the id for the option selected, like id->name
        let field_config = self.field_config.clone();
        eprintln!("SelectField.is_valid :: field_config: {:#?}", &field_config);
        eprintln!("SelectField.is_valid :: value: {:?}", &value);
        let required = field_config.required.unwrap();
        let name = field_config.name.unwrap();
        // let field_name = self.field.name.clone().unwrap_or_default();
        if value.is_none() && required == true {
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
            let value_id = value.unwrap();
            let id_list: Vec<&str> = value_id.split(",").collect();
            let field_config = self.field_config.clone();
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
                        let select_id = options_name_map.get(&select_option).unwrap();
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
                return Ok(true)
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
}
impl ProcessField for SelectField {
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
        let mut data: HashMap<String, String> = HashMap::new();
        if db_data.data.is_some() {
            data = db_data.data.unwrap();
        }
        let value_string_ = insert_data_map.get(&field_id).unwrap().clone();
        let is_valid = self.is_valid(Some(&value_string_))?;
        if is_valid == true {
            &data.insert(field_id, value_db);
            db_data.data = Some(data);
            return Ok(db_data);
        } else {
            return Err(error_validate_process("Single Select", &field_name))
        }
    }
}
impl StringValueField for SelectField {
    fn get_value(&self, value_db: Option<&String>) -> Option<String> {
        // value_db can be either single or multiple separated by commas
        let mut resolved_id: Option<String> = None;
        let options = self.field_config.options.clone().unwrap();
        let options_map = self.options_name_map.clone().unwrap();
        let options_id_map = self.options_id_map.clone().unwrap();
        if value_db.is_none() {
            return None
        } else {
            let value = value_db.unwrap();
            let value_items: Vec<&str> = value.split(",").collect();
            if *&value_items.len() == 1 {                
                for option_value in options.iter() {
                    let select_id = options_map.get(option_value).unwrap().clone();
                    if select_id.to_lowercase() == value.to_lowercase() {
                        resolved_id = Some(select_id);
                    }
                }
            } else {
                let mut resolved_ids: Vec<String> = Vec::new();
                for item_select_id in value_items {
                    // let select_option = options_id_map.get(value_item_id).unwrap().clone();
                    let item_select_id = &item_select_id.to_string();
                    let select_option = options_id_map.get(item_select_id);
                    if select_option.is_some() {
                        // let select_option = select_option.unwrap().clone();
                        resolved_ids.push(item_select_id.clone());
                    }
                    resolved_id = Some(resolved_ids.join(","));
                }
            }
            return resolved_id;
        }
    }
    fn get_value_db(&self, value: Option<&String>) -> Option<String> {
        // value is the literal, the option string literal
        // I return the option id
        if *&value.is_some() {
            let value = value.unwrap();
            let options = self.field_config.options.clone().unwrap();
            let options_map = self.options_name_map.clone().unwrap();
            let mut select_id: Option<String> = None;
            for option_value in options.iter() {
                if option_value.to_lowercase() == value.to_lowercase() {
                    select_id = Some(options_map.get(option_value).unwrap().clone());
                    break
                }
            }
            return select_id;
        } else {
            return None
        }
    }
}
 */