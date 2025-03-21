extern crate rust_stemmers;

use std::collections::BTreeMap;
use std::str::FromStr;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use tr::tr;
use asciifolding::fold_to_ascii;
use lazy_static::lazy_static;
use regex::Regex;
use lingua::{Language, LanguageDetector, LanguageDetectorBuilder};
use rust_stemmers::{Algorithm, Stemmer};

use crate::planet::constants::ID;
use crate::planet::PlanetError;
use crate::storage::folder::{DbData, get_value_list};
use crate::statements::folder::schema::*;
use crate::storage::constants::*;
use crate::planet::constants::*;
use crate::storage::generate_id;
use crate::storage::columns::*;

lazy_static! {
    pub static ref RE_TEXT: Regex = Regex::new(r#"[a-zA-Z0-9]+"#).unwrap();
    pub static ref RE_PHONE: Regex = Regex::new(r#"^(\+\d{1,2}\s)?\(?\d{2,3}\)?[\s.-]\d{2,3}[\s.-]\d{2,4}([\s.-]\d{2,4})?$"#).unwrap();
    pub static ref RE_EMAIL: Regex = Regex::new(r#"^(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9]))\.){3}(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9])|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])$"#).unwrap();
    pub static ref RE_URL: Regex = Regex::new(r#"^https?:[/][/](www\.)?[-a-zA-Z0-9@:%._\+~#=]{1,256}\.[a-zA-Z0-9()]{1,6}\b([-a-zA-Z0-9()@:%_\+.~#?&//=]*)$"#).unwrap();
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SmallTextColumn {
    pub config: ColumnConfig
}
impl SmallTextColumn {
    pub fn defaults(column_config: &ColumnConfig) -> Self {
        let column_config = column_config.clone();
        let column_obj = Self{
            config: column_config
        };
        return column_obj
    }
}
impl StorageColumn for SmallTextColumn {
    fn create_config(
        &mut self, 
        column_config_map: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let mut column_config_map = column_config_map.clone();
        let config = self.config.clone();
        let max_length = config.max_length;
        if max_length.is_some() {
            let max_length = max_length.unwrap();
            column_config_map.insert(String::from(MAX_LENGTH), max_length);
        }
        return Ok(column_config_map)
    }
    fn get_config(
        &mut self, 
        column_config_map: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError> {
        let mut config = self.config.clone();
        let max_length = column_config_map.get(MAX_LENGTH);
        if max_length.is_some() {
            let max_length = max_length.unwrap();
            config.max_length = Some(max_length.clone());
        }
        return Ok(config)
    }
    fn validate(&self, data: &Vec<String>) -> Result<Vec<String>, Vec<PlanetError>> {
        let data = data.clone();
        let config = self.config.clone();
        let column_name = config.clone().name.unwrap_or_default();
        let set_validate = validate_set(&config, &data);
        if set_validate.is_err() {
            let error = set_validate.unwrap_err();
            let mut errors: Vec<PlanetError> = Vec::new();
            errors.push(error);
            return Err(errors)
        }
        let mut data_new: Vec<String> = Vec::new();
        let required = config.required.unwrap();
        let max_length = config.max_length.as_ref();
        for data_item in data {
            if data_item == String::from("") && required == true {
                let error = PlanetError::new(
                    500, 
                    Some(tr!(
                        "Field {}{}{} is required", 
                        String::from("\"").blue(), &column_name.blue(), String::from("\"").blue()
                    )),
                );
                let mut errors: Vec<PlanetError> = Vec::new();
                errors.push(error);
                return Err(errors);
            } else {
                if max_length.is_some() {
                    let max_length = max_length.unwrap();
                    let max_length: usize = FromStr::from_str(&max_length).unwrap();
                    let text_length = data_item.len();
                    if text_length > max_length {
                        let error = PlanetError::new(
                            500, 
                            Some(tr!(
                                "Length of column \"{}\" is bigger than maximum length, \"{}\".",
                                &column_name, &max_length
                            )),
                        );
                        let mut errors: Vec<PlanetError> = Vec::new();
                        errors.push(error);
                        return Err(errors);
                    }
                }
            }
            data_new.push(data_item);
        }
        return Ok(data_new)
    }
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String {
        let column_config = self.config.clone();
        let column_name = column_config.name.unwrap();
        let mut yaml_string = yaml_string.clone();
        let field = &column_name.truecolor(
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
    pub fn defaults(column_config: &ColumnConfig) -> Self {
        let column_config = column_config.clone();
        let column_obj = Self{
            config: column_config
        };
        return column_obj
    }
}
impl StorageColumn for LongTextColumn {
    fn create_config(
        &mut self, 
        column_config_map: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let column_config_map = column_config_map.clone();
        // No special attributes so far for small text field
        return Ok(column_config_map)
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
        let config = self.config.clone();
        let set_validate = validate_set(&config, &data);
        if set_validate.is_err() {
            let error = set_validate.unwrap_err();
            let mut errors: Vec<PlanetError> = Vec::new();
            errors.push(error);
            return Err(errors)
        }
        let required = config.required.unwrap();
        let name = config.name.unwrap();
        let mut data_new: Vec<String> = Vec::new();
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
            }
            data_new.push(data_item);
        }
        return Ok(data_new)
    }
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String {
        let column_config = self.config.clone();
        let column_name = column_config.name.unwrap();
        let mut yaml_string = yaml_string.clone();
        let field = &column_name.truecolor(
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
    pub fn defaults(column_config: &ColumnConfig, folder: Option<&DbData>) -> Self {
        let column_config = column_config.clone();
        let mut column_obj = Self{
            config: column_config,
            options_id_map: None,
            options_name_map: None,
        };
        if folder.is_some() {
            let folder = folder.unwrap();
            let mut options_id_map: BTreeMap<String, String> = BTreeMap::new();
            let mut options_name_map: BTreeMap<String, String> = BTreeMap::new();
            let column_config = &column_obj.config;
            let column_name = column_config.name.clone().unwrap();
            for data in folder.data.iter() {
                // key for ordering: field_ids
                for key in data.keys() {
                    let has_select_sep = key.find("__");
                    if has_select_sep.is_some() {
                        // key: Status__select_options
                        let key_items: Vec<&str> = key.split("__").collect();
                        let key_field_name = key_items[0];
                        let key_field_type = key_items[1];
                        if key_field_type == SELECT_OPTIONS && key_field_name.to_lowercase() == column_name.to_lowercase() {
                            // Process, since we have a simple select field
                            // "Status__select_options": [
                            //     {
                            //         "key": "c48kg78smpv5gct3hfqg",
                            //         "value": "Draft",
                            //     },
                            //     ...
                            // ],
                            let options: &Vec<BTreeMap<String, String>> = data.get(key).unwrap();
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
            column_obj.options_id_map = Some(options_id_map);
            column_obj.options_name_map = Some(options_name_map);
        }
        return column_obj;
    }
}
impl StorageColumn for SelectColumn {
    fn create_config(
        &mut self, 
        column_config_map: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let mut column_config_map = column_config_map.clone();
        let column_config = self.config.clone();
        let column_name = column_config.name.unwrap_or_default();
        let options = column_config.options;
        if options.is_some() {
            let options_yaml = serde_yaml::to_string(&options);
            if options_yaml.is_ok() {
                column_config_map.insert(String::from(OPTIONS), options_yaml.unwrap());
            } else {
                panic!("Could not parse options for field \"{}\"", &column_name);
            }
        }
        return Ok(column_config_map)
    }
    fn get_config(
        &mut self, 
        column_config_map: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError> {
        let mut config = self.config.clone();
        let options_str = column_config_map.get(OPTIONS);
        let options_wrap: Option<Vec<String>>;
        if options_str.is_some() {
            let options_str = column_config_map.get(OPTIONS).unwrap().clone();
            let options_str = options_str.as_str();
            let options: Vec<String> = serde_yaml::from_str(options_str).unwrap();
            options_wrap = Some(options);
            config.options = options_wrap;
        }
        return Ok(config)
    }
    fn validate(&self, data: &Vec<String>) -> Result<Vec<String>, Vec<PlanetError>> {
        let data = data.clone();
        // value represents the id for the option selected, like id->name
        let column_config = self.config.clone();
        let set_validate = validate_set(&column_config, &data);
        if set_validate.is_err() {
            let error = set_validate.unwrap_err();
            let mut errors: Vec<PlanetError> = Vec::new();
            errors.push(error);
            return Err(errors)
        }
        let required = column_config.required.unwrap();
        let name = column_config.name.unwrap();
        // let column_name = self.field.name.clone().unwrap_or_default();
        let mut data_new: Vec<String> = Vec::new();
        for data_item in data {
            if data_item == String::from("") && required == true {
                let error = PlanetError::new(
                    500, 
                    Some(tr!(
                        "Field {}{}{} is required", 
                        String::from("\"").blue(), &name.blue(), String::from("\"").blue()
                    )),
                );
                let mut errors: Vec<PlanetError> = Vec::new();
                errors.push(error);
                return Err(errors);
            } else {
                // In case no many, just string with id. In case many, list of ids separated by commas
                let value_id = data_item.clone();
                let id_list: Vec<&str> = value_id.split(",").collect();
                let column_config = self.config.clone();
                // Check that value appears on the config for choices id -> value
                // The option id is obtained from the folder config
                let options = column_config.options.unwrap();
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
                    // return Ok(data.clone())
                    data_new.push(data_item.clone());
                } else {
                    let error = PlanetError::new(
                        500, 
                        Some(tr!(
                            "Field {}{}{} is not configured with select id {}{}{}", 
                            String::from("\"").blue(), &name.blue(), String::from("\"").blue(),
                            String::from("\"").blue(), value_id, String::from("\"").blue(),
                        )),
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
        let column_config = self.config.clone();
        let column_name = column_config.name.unwrap();
        let many = column_config.many.unwrap();
        let mut yaml_string = yaml_string.clone();
        let field = &column_name.truecolor(
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
    pub fn defaults(column_config: &ColumnConfig) -> Self {
        let column_config = column_config.clone();
        let column_obj = Self{
            config: column_config
        };
        return column_obj
    }
}
impl StorageColumn for AuditByColumn {
    fn create_config(
        &mut self, 
        column_config_map: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let column_config_map = column_config_map.clone();
        return Ok(column_config_map)
    }
    fn get_config(
        &mut self, 
        _: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError> {
        let config = self.config.clone();
        return Ok(config)
    }
    fn validate(&self, data: &Vec<String>) -> Result<Vec<String>, Vec<PlanetError>> {
        // CreatedBy: I map into insert
        // LastModifiedBy: I map into insert and update
        // I save the user id
        // The user id should not come from the payload, but from the auth system
        // TODO: Check user_id exists on folder of users, or any other mean of storage
        let config = self.config.clone();
        let set_validate = validate_set(&config, &data);
        if set_validate.is_err() {
            let error = set_validate.unwrap_err();
            let mut errors: Vec<PlanetError> = Vec::new();
            errors.push(error);
            return Err(errors)
        }
        let mut data_new: Vec<String> = Vec::new();
        for data_item in data {
            let user_id = data_item.clone();
            data_new.push(user_id);
        }
        return Ok(data_new)
    }
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String {
        let column_config = self.config.clone();
        let column_name = column_config.name.unwrap();
        let mut yaml_string = yaml_string.clone();
        let field = &column_name.truecolor(
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
pub struct LanguageColumn {
    pub config: ColumnConfig,
}
impl LanguageColumn {
    pub fn defaults(
        column_config: &ColumnConfig,
    ) -> Self {
        let column_config = column_config.clone();
        let field_obj = Self{
            config: column_config,
        };
        return field_obj
    }
}
impl LanguageColumn {
    pub fn create_config(
        &mut self, 
        column_config_map: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let column_config_map = column_config_map.clone();
        return Ok(column_config_map)
    }
    pub fn get_config(
        &mut self, 
        _: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError> {
        let config = self.config.clone();
        return Ok(config)
    }
    pub fn validate(&self, text: &String) -> Result<String, PlanetError> {
        let detector: LanguageDetector = LanguageDetectorBuilder::from_languages(&LANGUAGES).with_preloaded_language_models().build();
        let detected_language: Option<Language> = detector.detect_language_of(text);
        let mut language_code = String::from("");
        if detected_language.is_some() {
            let detected_language = detected_language.unwrap();
            language_code = detected_language.iso_code_639_1().to_string();
            eprintln!("LanguageColumn.validate :: resolved language_code: {}", &language_code);
            return Ok(language_code)
        } else {
            // language is empty string. When indexing, we get default from folder to index in that language when
            // language not detected.
            return Ok(language_code)
        }        
    }
    pub fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String {
        let column_config = self.config.clone();
        let column_name = column_config.name.unwrap();
        let mut yaml_string = yaml_string.clone();
        let field = &column_name.truecolor(
            YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
        );
        let value = format!("{}", 
            value.truecolor(YAML_COLOR_ORANGE[0], YAML_COLOR_ORANGE[1], YAML_COLOR_ORANGE[2]), 
        );
        yaml_string.push_str(format!("  {field}: {value}\n", field=field, value=value).as_str());
        return yaml_string;
    }
}

pub fn get_default_language_code(folder: &DbData) -> Result<String, PlanetError> {
    let folder = folder.clone();
    let data = folder.data.unwrap();
    let language_default = data.get(&LANGUAGE_DEFAULT.to_string());
    if language_default.is_some() {
        let language_default = language_default.unwrap();
        let language_default = get_value_list(language_default);
        if language_default.is_some() {
            let language_default = language_default.unwrap();
            let language_default = language_default.as_str();
            match language_default {
                LANGUAGE_DANISH => {
                    return Ok(String::from(LANGUAGE_CODE_DANISH))
                },
                LANGUAGE_ENGLISH => {
                    return Ok(String::from(LANGUAGE_CODE_ENGLISH))
                },
                LANGUAGE_SPANISH => {
                    return Ok(String::from(LANGUAGE_CODE_SPANISH))
                },
                LANGUAGE_FRENCH => {
                    return Ok(String::from(LANGUAGE_CODE_FRENCH))
                },
                LANGUAGE_ITALIAN => {
                    return Ok(String::from(LANGUAGE_CODE_ITALIAN))
                },
                LANGUAGE_GERMAN => {
                    return Ok(String::from(LANGUAGE_CODE_GERMAN))
                },
                LANGUAGE_PORTUGUESE => {
                    return Ok(String::from(LANGUAGE_CODE_PORTUGUESE))
                },
                LANGUAGE_NORWEGIAN => {
                    return Ok(String::from(LANGUAGE_CODE_NORWEGIAN))
                },
                LANGUAGE_SWEDISH => {
                    return Ok(String::from(LANGUAGE_CODE_SWEDISH))
                },
                _ => {
                    let error = PlanetError::new(
                        500, 
                        Some(tr!("Language default \"{}\" not supported.", language_default)),
                    );
                    return Err(error);
                },
            }        
        } else {
            let error = PlanetError::new(
                500, 
                Some(tr!("Error defining default language")),
            );
            return Err(error);
        }
    } else {
        let error = PlanetError::new(
            500, 
            Some(tr!("Error defining default language, not found on folder configuration.")),
        );
        return Err(error);
    }
}

#[derive(Debug, Clone)]
pub struct TextColumn {
    pub config: ColumnConfig,
    pub column_config_map: Option<BTreeMap<String, ColumnConfig>>,
}
impl TextColumn {
    pub fn defaults(
        column_config: &ColumnConfig,
        column_config_map: Option<BTreeMap<String, ColumnConfig>>,
    ) -> Self {
        let column_config = column_config.clone();
        let field_obj = Self{
            config: column_config,
            column_config_map: column_config_map.clone(),
        };
        return field_obj
    }
    fn do_text_basic(
        &mut self, 
        data_map: &BTreeMap<String, Vec<BTreeMap<String, String>>>, 
        column_id: &String
    ) -> Option<Vec<String>> {
        let wrap = data_map.get(column_id);
        let mut values_wrap: Option<Vec<String>> = None;
        if wrap.is_some() {
            let items = wrap.unwrap();
            let mut my_list: Vec<String> = Vec::new();
            for item in items {
                let value = item.get(VALUE);
                if value.is_some() {
                    let value = value.unwrap();
                    my_list.push(value.clone());
                }
            }
            values_wrap = Some(my_list);
        }
        return values_wrap
    }
}
impl TextColumn {
    pub fn create_config(
        &mut self, 
        column_config_map: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let column_config_map = column_config_map.clone();
        return Ok(column_config_map)
    }
    pub fn get_config(
        &mut self, 
        _: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError> {
        let config = self.config.clone();
        return Ok(config)
    }
    pub fn validate(
        &mut self, 
        data_map: &BTreeMap<String, Vec<BTreeMap<String, String>>>, 
        folder: &DbData,
        text_column_id: &String,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let text_column_id = text_column_id.clone();
        let folder = folder.clone();
        let column_config_map = self.column_config_map.clone().unwrap();
        let mut map: BTreeMap<String, String> = BTreeMap::new();
        let mut text: String = String::from("");
        for (column_name, column_config) in column_config_map {
            let column_type = column_config.column_type.unwrap();
            let column_type = column_type.as_str();
            let column_id = column_config.id.unwrap();
            let mut values_wrap: Option<Vec<String>> = None;
            match column_type {
                // COLUMN_TYPE_SMALL_TEXT => {
                //     values_wrap = self.do_text_basic(data_map, &column_id);
                // },
                // COLUMN_TYPE_LONG_TEXT => {
                //     values_wrap = self.do_text_basic(data_map, &column_id);
                // },
                // COLUMN_TYPE_NUMBER => {
                //     values_wrap = self.do_text_basic(data_map, &column_id);
                // },
                COLUMN_TYPE_SELECT => {
                    // Need to check from data_map the values sent, which can be a list separated by commas
                    let wrap = data_map.get(&column_id);
                    if wrap.is_some() {
                        let value_list = wrap.unwrap();
                        let mut my_value_list: Vec<String> = Vec::new();
                        for values in value_list {
                            let values = values.get(VALUE);
                            if values.is_some() {
                                let values = values.unwrap();
                                let mut value_ids: Vec<&str> = Vec::new();
                                if values.find(",").is_some() {
                                    value_ids = values.split(",").collect();
                                } else {
                                    value_ids.push(values.as_str());
                                }
                                let mut value: String = String::from("");
                                for data in folder.data.iter() {
                                    for (key, collection_value) in data {
                                        let has_select_sep = &key.find("__").clone();
                                        if has_select_sep.is_some() {
                                            let key_items: Vec<&str> = key.split("__").collect();
                                            let key_column_name = key_items[0];
                                            let key_column_type = key_items[1];
                                            if key_column_type == SELECT_OPTIONS && 
                                               key_column_name.to_lowercase() == column_name.to_lowercase() {
                                                for option_ in collection_value {
                                                    let option_key = option_.get(KEY).unwrap().clone();
                                                    let option_value = option_.get(VALUE).unwrap().clone();
                                                    for value_id in value_ids.clone() {
                                                        if option_key.as_str() == value_id {
                                                            // data = format!("{} {}", &data, option_value);
                                                            // value_wrap = Some(&option_value);
                                                            value = format!("{} {}", &value, option_value);
                                                        }    
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                my_value_list.push(value);
                            }
                        }
                        values_wrap = Some(my_value_list);
                    }
                },
                // COLUMN_TYPE_DATE => {
                //     values_wrap = self.do_text_basic(data_map, &column_id);
                // },
                // COLUMN_TYPE_DURATION => {
                //     values_wrap = self.do_text_basic(data_map, &column_id);
                // },
                // COLUMN_TYPE_CURRENCY => {
                //     values_wrap = self.do_text_basic(data_map, &column_id);
                // },
                // COLUMN_TYPE_PERCENTAGE => {
                //     values_wrap = self.do_text_basic(data_map, &column_id);
                // },
                // COLUMN_TYPE_PHONE => {
                //     values_wrap = self.do_text_basic(data_map, &column_id);
                // },
                // COLUMN_TYPE_EMAIL => {
                //     values_wrap = self.do_text_basic(data_map, &column_id);
                // },
                // COLUMN_TYPE_FILE => {
                //     values_wrap = self.do_text_basic(data_map, &column_id);
                // },
                _ => {
                    values_wrap = self.do_text_basic(data_map, &column_id);
                },
            }
            if values_wrap.is_some() {
                let values = Some(values_wrap.unwrap().clone());
                if values.is_some() {
                    let values = values.unwrap();
                    let mut column_text: String = String::from("");
                    for mut value in values {
                        if value == String::from("") {
                            continue
                        }
                        // Do ascii folding
                        value = fold_to_ascii(value.as_str());
                        // Through regex, parse only words and numbers, remove duplicate whitespace
                        let expr = &RE_TEXT;
                        let mut value_new = String::from("");
                        let words = expr.captures_iter(value.as_str());
                        for word in words {
                            let word = word.get(0).unwrap().as_str();
                            value_new = format!("{} {}", value_new, word);
                        }
                        let value_new = value_new.trim().to_string();
                        value = value_new;
                        // Convert to lower
                        value = value.to_lowercase();
                        column_text = format!("{} {}", &column_text, &value).trim().to_string();
                        text = format!("{} {}", &text, &value).trim().to_string();
                    }
                    map.insert(column_id, column_text.clone());
                }
            } else {
                continue
            }
        }
        // add text to map
        // let text = text.trim().to_string();
        map.insert(text_column_id, text);
        return Ok(map.clone())
    }
    pub fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String {
        let column_config = self.config.clone();
        let column_name = column_config.name.unwrap();
        let mut yaml_string = yaml_string.clone();
        let field = &column_name.truecolor(
            YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
        );
        let value = format!("{}", 
            value.truecolor(YAML_COLOR_ORANGE[0], YAML_COLOR_ORANGE[1], YAML_COLOR_ORANGE[2]), 
        );
        yaml_string.push_str(format!("  {field}: {value}\n", field=field, value=value).as_str());
        return yaml_string;
    }
}

#[derive(Debug, Clone)]
pub struct GenerateIdColumn {
    pub config: ColumnConfig,
}
impl GenerateIdColumn {
    pub fn defaults(
        column_config: &ColumnConfig,
    ) -> Self {
        let column_config = column_config.clone();
        let field_obj = Self{
            config: column_config,
        };
        return field_obj
    }
}
impl StorageColumn for GenerateIdColumn {
    fn create_config(
        &mut self, 
        column_config_map: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let column_config_map = column_config_map.clone();
        return Ok(column_config_map)
    }
    fn get_config(
        &mut self, 
        _: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError> {
        let config = self.config.clone();
        return Ok(config)
    }
    fn validate(
        &self, 
        _value: &Vec<String>
    ) -> Result<Vec<String>, Vec<PlanetError>> {
        let id = generate_id();
        if id.is_some() {
            let id = id.unwrap();
            let mut ids: Vec<String> = Vec::new();
            ids.push(id);
            return Ok(ids)
        } else {
            let error = PlanetError::new(
                500, 
                Some(tr!(
                    "Error generating id value"
                )),
            );
            let mut errors: Vec<PlanetError> = Vec::new();
            errors.push(error);
            return Err(errors)
        }
    }
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String {
        let column_config = self.config.clone();
        let column_name = column_config.name.unwrap();
        let mut yaml_string = yaml_string.clone();
        let field = &column_name.truecolor(
            YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
        );
        let value = format!("{}", 
            value.truecolor(YAML_COLOR_ORANGE[0], YAML_COLOR_ORANGE[1], YAML_COLOR_ORANGE[2]), 
        );
        yaml_string.push_str(format!("  {field}: {value}\n", field=field, value=value).as_str());
        return yaml_string;
    }
}

#[derive(Debug, Clone)]
pub struct PhoneColumn {
    pub config: ColumnConfig,
}
impl PhoneColumn {
    pub fn defaults(
        column_config: &ColumnConfig,
    ) -> Self {
        let column_config = column_config.clone();
        let field_obj = Self{
            config: column_config,
        };
        return field_obj
    }
}
impl StorageColumn for PhoneColumn {
    fn create_config(
        &mut self, 
        column_config_map: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let column_config_map = column_config_map.clone();
        return Ok(column_config_map)
    }
    fn get_config(
        &mut self, 
        _: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError> {
        let config = self.config.clone();
        return Ok(config)
    }
    fn validate(
        &self, 
        value: &Vec<String>
    ) -> Result<Vec<String>, Vec<PlanetError>> {
        let config = self.config.clone();
        let data = value.clone();
        let set_validate = validate_set(&config, &data);
        if set_validate.is_err() {
            let error = set_validate.unwrap_err();
            let mut errors: Vec<PlanetError> = Vec::new();
            errors.push(error);
            return Err(errors)
        }
        let column_name = config.name.unwrap_or_default();
        let mut data_new: Vec<String> = Vec::new();
        for data_item in data {
            let expr = &RE_PHONE;
            let is_found = expr.is_match(&data_item);
            if is_found == true {
                data_new.push(data_item);
            } else {
                let error = PlanetError::new(
                    500, 
                    Some(tr!("Error validating phone column \"{}\" with value \"{}\".", 
                    &column_name, &data_item))
                );
                let mut errors: Vec<PlanetError> = Vec::new();
                errors.push(error);
                return Err(errors);
            }    
        }
        return Ok(data_new)
    }
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String {
        let column_config = self.config.clone();
        let column_name = column_config.name.unwrap();
        let mut yaml_string = yaml_string.clone();
        let field = &column_name.truecolor(
            YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
        );
        let value = format!("{}", 
            value.truecolor(YAML_COLOR_ORANGE[0], YAML_COLOR_ORANGE[1], YAML_COLOR_ORANGE[2]), 
        );
        yaml_string.push_str(format!("  {field}: {value}\n", field=field, value=value).as_str());
        return yaml_string;
    }
}

#[derive(Debug, Clone)]
pub struct EmailColumn {
    pub config: ColumnConfig,
}
impl EmailColumn {
    pub fn defaults(
        column_config: &ColumnConfig,
    ) -> Self {
        let column_config = column_config.clone();
        let field_obj = Self{
            config: column_config,
        };
        return field_obj
    }
}
impl StorageColumn for EmailColumn {
    fn create_config(
        &mut self, 
        column_config_map: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let column_config_map = column_config_map.clone();
        return Ok(column_config_map)
    }
    fn get_config(
        &mut self, 
        _: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError> {
        let config = self.config.clone();
        return Ok(config)
    }
    fn validate(
        &self, 
        value: &Vec<String>
    ) -> Result<Vec<String>, Vec<PlanetError>> {
        let config = self.config.clone();
        let data = value.clone();
        let set_validate = validate_set(&config, &data);
        if set_validate.is_err() {
            let error = set_validate.unwrap_err();
            let mut errors: Vec<PlanetError> = Vec::new();
            errors.push(error);
            return Err(errors)
        }
        let column_name = config.name.unwrap_or_default();
        let mut data_new: Vec<String> = Vec::new();
        for data_item in data {
            let expr = &RE_EMAIL;
            let is_found = expr.is_match(&data_item);
            if is_found == true {
                // return Ok(value.clone())
                data_new.push(data_item);
            } else {
                let error = PlanetError::new(
                    500, 
                    Some(tr!("Error validating email column \"{}\" with value \"{}\".", 
                    &column_name, &data_item))
                );
                let mut errors: Vec<PlanetError> = Vec::new();
                errors.push(error);
                return Err(errors);
            }    
        }
        return Ok(data_new)
    }
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String {
        let column_config = self.config.clone();
        let column_name = column_config.name.unwrap();
        let mut yaml_string = yaml_string.clone();
        let field = &column_name.truecolor(
            YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
        );
        let value = format!("{}", 
            value.truecolor(YAML_COLOR_ORANGE[0], YAML_COLOR_ORANGE[1], YAML_COLOR_ORANGE[2]), 
        );
        yaml_string.push_str(format!("  {field}: {value}\n", field=field, value=value).as_str());
        return yaml_string;
    }
}

#[derive(Debug, Clone)]
pub struct UrlColumn {
    pub config: ColumnConfig,
}
impl UrlColumn {
    pub fn defaults(
        column_config: &ColumnConfig,
    ) -> Self {
        let column_config = column_config.clone();
        let field_obj = Self{
            config: column_config,
        };
        return field_obj
    }
}
impl StorageColumn for UrlColumn {
    fn create_config(
        &mut self, 
        column_config_map: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let column_config_map = column_config_map.clone();
        return Ok(column_config_map)
    }
    fn get_config(
        &mut self, 
        _: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError> {
        let config = self.config.clone();
        return Ok(config)
    }
    fn validate(
        &self, 
        value: &Vec<String>
    ) -> Result<Vec<String>, Vec<PlanetError>> {
        let config = self.config.clone();
        let data = value.clone();
        let set_validate = validate_set(&config, &data);
        if set_validate.is_err() {
            let error = set_validate.unwrap_err();
            let mut errors: Vec<PlanetError> = Vec::new();
            errors.push(error);
            return Err(errors)
        }
        let mut data_new: Vec<String> = Vec::new();
        let column_name = config.name.unwrap_or_default();
        for data_item in data {
            let expr = &RE_URL;
            let is_found = expr.is_match(&data_item);
            if is_found == true {
                data_new.push(data_item);
            } else {
                let error = PlanetError::new(
                    500, 
                    Some(tr!("Error validating url column \"{}\" with value \"{}\".", 
                    &column_name, &data_item))
                );
                let mut errors: Vec<PlanetError> = Vec::new();
                errors.push(error);
                return Err(errors);
            }
        }
        return Ok(data_new)
    }
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String {
        let column_config = self.config.clone();
        let column_name = column_config.name.unwrap();
        let mut yaml_string = yaml_string.clone();
        let field = &column_name.truecolor(
            YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
        );
        let value = format!("{}", 
            value.truecolor(YAML_COLOR_ORANGE[0], YAML_COLOR_ORANGE[1], YAML_COLOR_ORANGE[2]), 
        );
        yaml_string.push_str(format!("  {field}: {value}\n", field=field, value=value).as_str());
        return yaml_string;
    }
}

pub fn get_stop_words_by_language(language_code: &str) -> Vec<String> {
    let mut stop_words: Vec<String> = Vec::new();
    match language_code {
        LANGUAGE_CODE_ENGLISH => {
            stop_words = stop_words::get(stop_words::LANGUAGE::English);
        },
        LANGUAGE_CODE_FRENCH => {
            stop_words = stop_words::get(stop_words::LANGUAGE::French);
        },
        LANGUAGE_CODE_ITALIAN => {
            stop_words = stop_words::get(stop_words::LANGUAGE::Italian);
        },
        LANGUAGE_CODE_GERMAN => {
            stop_words = stop_words::get(stop_words::LANGUAGE::German);
        },
        LANGUAGE_CODE_SPANISH => {
            stop_words = stop_words::get(stop_words::LANGUAGE::Spanish);
        },
        LANGUAGE_CODE_PORTUGUESE => {
            stop_words = stop_words::get(stop_words::LANGUAGE::Portuguese);
        },
        LANGUAGE_CODE_DANISH => {
            stop_words = stop_words::get(stop_words::LANGUAGE::Danish);
        },
        LANGUAGE_CODE_NORWEGIAN => {
            stop_words = stop_words::get(stop_words::LANGUAGE::Norwegian);
        },
        LANGUAGE_CODE_SWEDISH => {
            stop_words = stop_words::get(stop_words::LANGUAGE::Swedish);
        },
        _ => {}
    }
    return stop_words
}

pub fn get_stemmer_by_language(language_code: &str) -> Stemmer {
    let mut stemmer: Stemmer = Stemmer::create(Algorithm::English);
    match language_code {
        LANGUAGE_CODE_ENGLISH => {
            stemmer = Stemmer::create(Algorithm::English);
        },
        LANGUAGE_CODE_FRENCH => {
            stemmer = Stemmer::create(Algorithm::French);
        },
        LANGUAGE_CODE_ITALIAN => {
            stemmer = Stemmer::create(Algorithm::Italian);
        },
        LANGUAGE_CODE_GERMAN => {
            stemmer = Stemmer::create(Algorithm::German);
        },
        LANGUAGE_CODE_SPANISH => {
            stemmer = Stemmer::create(Algorithm::Spanish);
        },
        LANGUAGE_CODE_PORTUGUESE => {
            stemmer = Stemmer::create(Algorithm::Portuguese);
        },
        LANGUAGE_CODE_DANISH => {
            stemmer = Stemmer::create(Algorithm::Danish);
        },
        LANGUAGE_CODE_NORWEGIAN => {
            stemmer = Stemmer::create(Algorithm::Norwegian);
        },
        LANGUAGE_CODE_SWEDISH => {
            stemmer = Stemmer::create(Algorithm::Swedish);
        },
        _ => {}
    }
    return stemmer;
}
