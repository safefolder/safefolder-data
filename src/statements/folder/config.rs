extern crate xid;

use std::collections::{BTreeMap};
use serde::{Deserialize, Serialize};
use serde_yaml;
use validator::{Validate, ValidationErrors};
use lazy_static::lazy_static;
use regex::Regex;

use crate::planet::validation::{CommandImportConfig, PlanetValidationError};
use crate::planet::{PlanetContext};

use crate::storage::constants::*;
use crate::statements::folder::schema::*;
// use crate::storage::columns::{
//     StorageColumn,
//     ObjectStorageColumn,
// };
use crate::planet::constants::*;

use super::fetch_yaml_config;

pub struct DbTableConfig02 {
    pub language: BTreeMap<String, String>,
    pub columns: Option<Vec<BTreeMap<String, String>>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbFolderConfig {
    pub language: Option<LanguageConfig>,
    pub columns: Option<Vec<ColumnConfig>>,
}

lazy_static! {
    static ref RE_COMMAND_CREATE_FOLDER: Regex = Regex::new(r#"(CREATE FOLDER) "([a-zA-Z0-9_ ]+)""#).unwrap();
    static ref RE_COMMAND_INSERT_INTO_FOLDER: Regex = Regex::new(r#"(INSERT INTO FOLDER) "([a-zA-Z0-9_ ]+)""#).unwrap();
    static ref RE_COMMAND_GET_FROM_FOLDER: Regex = Regex::new(r#"(GET FROM FOLDER) "([a-zA-Z0-9_ ]+)"#).unwrap();
    static ref RE_COMMAND_SELECT_FROM_FOLDER: Regex = Regex::new(r#"(SELECT FROM FOLDER) "([a-zA-Z0-9_ ]+)"#).unwrap();
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct SubFolderDataConfig {
    pub id: Option<String>,
    pub is_reference: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct DataId {
    pub id: Option<String>,
    pub columns: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct GetFromFolderConfig {
    #[validate(required, regex="RE_COMMAND_GET_FROM_FOLDER")]
    pub command: Option<String>,
    #[validate(required)]
    pub data: Option<DataId>,
}

impl GetFromFolderConfig {

    pub fn defaults(id: String) -> GetFromFolderConfig {
        let data_id: DataId = DataId{
            id: Some(id),
            columns: None,
        };
        let config: GetFromFolderConfig = GetFromFolderConfig{
            command: Some(String::from("GET FROM FOLDER")),
            data: Some(data_id),
        };
        return config
    }

    pub fn import(
        &self, 
        planet_context: &PlanetContext, 
        yaml_path: &String
    ) -> Result<GetFromFolderConfig, Vec<PlanetValidationError>> {
        let yaml_str: String = fetch_yaml_config(&yaml_path);
        // Deseralize the config entity
        let response: Result<GetFromFolderConfig, serde_yaml::Error> = serde_yaml::from_str(&yaml_str);
        let import_config: CommandImportConfig = CommandImportConfig{
            command: String::from(""),
            planet_context: planet_context,
        };
        match response {
            Ok(_) => {
                let config_model: GetFromFolderConfig = response.unwrap();
                let validate: Result<(), ValidationErrors> = config_model.validate();
                match validate {
                    Ok(_) => {
                        return Ok(config_model)
                    },
                    Err(errors) => {
                        let command = &config_model.command.unwrap();
                        let planet_errors: Vec<PlanetValidationError> = import_config.parse_validator(
                            command, errors);
                        return Err(planet_errors);
                    }
                }
            },
            Err(error) => {
                let mut planet_errors: Vec<PlanetValidationError> = Vec::new();
                planet_errors.push(import_config.parse_serde(&error));
                return Err(planet_errors);
            }
        }
    }

}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct SelectFromFolderConfig {
    #[validate(required, regex="RE_COMMAND_SELECT_FROM_FOLDER")]
    pub command: Option<String>,
    #[validate(required)]
    pub r#where: Option<String>,
    #[serde(default="SelectFromFolderConfig::page_default")]
    pub page: Option<String>,
    #[serde(default="SelectFromFolderConfig::number_items_default")]
    pub number_items: Option<String>,
    pub columns: Option<Vec<String>>,
}

impl SelectFromFolderConfig {

    pub fn defaults(r#where: Option<String>, page: Option<String>, number_items: Option<String>) -> Self {
        let config: SelectFromFolderConfig = Self{
            command: Some(String::from("SELECT FROM FOLDER")),
            r#where: r#where,
            page: page,
            number_items: number_items,
            columns: None,
        };
        return config
    }
    pub fn page_default() -> Option<String> {
        return Some(SELECT_DEFAULT_PAGE.to_string());
    }
    pub fn number_items_default() -> Option<String> {
        return Some(SELECT_DEFAULT_NUMBER_ITEMS.to_string());
    }
    pub fn import(
        &self, 
        planet_context: &PlanetContext, 
        yaml_path: &String
    ) -> Result<SelectFromFolderConfig, Vec<PlanetValidationError>> {
        let yaml_str: String = fetch_yaml_config(&yaml_path);
        // Deseralize the config entity
        let response: Result<SelectFromFolderConfig, serde_yaml::Error> = serde_yaml::from_str(&yaml_str);
        let import_config: CommandImportConfig = CommandImportConfig{
            command: String::from(""),
            planet_context: planet_context,
        };
        match response {
            Ok(_) => {
                let config_model: SelectFromFolderConfig = response.unwrap();
                let validate: Result<(), ValidationErrors> = config_model.validate();
                match validate {
                    Ok(_) => {
                        return Ok(config_model)
                    },
                    Err(errors) => {
                        let command = &config_model.command.unwrap();
                        let planet_errors: Vec<PlanetValidationError> = import_config.parse_validator(
                            command, errors);
                        return Err(planet_errors);
                    }
                }
            },
            Err(error) => {
                let mut planet_errors: Vec<PlanetValidationError> = Vec::new();
                planet_errors.push(import_config.parse_serde(&error));
                return Err(planet_errors);
            }
        }
    }
}

pub fn create_minimum_column_map(
    column_id: &String,
    column_name: &String,
    column_type: &String,
) -> BTreeMap<String, String> {
    let mut map: BTreeMap<String, String> = BTreeMap::new();
    map.insert(String::from(ID), column_id.clone());
    map.insert(String::from(NAME), column_name.clone());
    map.insert(String::from(COLUMN_TYPE), column_type.clone());
    return map
}
