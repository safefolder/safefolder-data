extern crate xid;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationErrors, ValidationError};
use lazy_static::lazy_static;
use regex::Regex;

use crate::commands::table::constants::FIELD_IDS;
use crate::commands::table::data;
use crate::planet::validation::{CommandImportConfig, PlanetValidationError};
use crate::planet::PlanetContext;

use crate::storage::constants::{FIELD_VERSION, FIELD_API_VERSION};
use crate::storage::*;
use crate::storage::table::DbData;
use crate::planet::constants::*;
use crate::planet::make_bool_str;

use super::constants::FIELDS;
use super::fetch_yaml_config;

pub struct DbTableConfig02 {
    pub language: HashMap<String, String>,
    pub fields: Option<Vec<HashMap<String, String>>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbTableConfig {
    pub language: Option<LanguageConfig>,
    pub fields: Option<Vec<FieldConfig>>,
}
impl DbTableConfig {
    pub fn to_hashmap(&self) -> HashMap<String, String> {
        // separate fields by "__" in objects, so I can have it plain.
        // field1__field2_opt
        let mut map: HashMap<String, String>  =HashMap::new();
        let language = self.language.clone().unwrap();
        let fields = self.fields.clone().unwrap();
        return map
    }
}

lazy_static! {
    static ref RE_COMMAND_CREATE_TABLE: Regex = Regex::new(r#"(CREATE TABLE) "([a-zA-Z0-9_ ]+)"#).unwrap();
    static ref RE_COMMAND_INSERT_INTO_TABLE: Regex = Regex::new(r#"(INSERT INTO TABLE) "([a-zA-Z0-9_ ]+)"#).unwrap();
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct CreateTableConfig {
    #[validate(required, regex="RE_COMMAND_CREATE_TABLE")]
    pub command: Option<String>,
    #[validate]
    pub language: Option<LanguageConfig>,
    #[validate]
    pub fields: Option<Vec<FieldConfig>>,
}

impl CreateTableConfig {

    pub fn defaults() -> CreateTableConfig {
        let config: CreateTableConfig = CreateTableConfig{
            command: None,
            language: None,
            fields: None,
        };
        return config
    }

    pub fn import(&self, planet_context: &PlanetContext, yaml_path: &String) -> Result<CreateTableConfig, Vec<PlanetValidationError>> {
        let yaml_str: String = fetch_yaml_config(&yaml_path);
        // Deseralize the config entity
        let response: Result<CreateTableConfig, serde_yaml::Error> = serde_yaml::from_str(&yaml_str);
        let import_config: CommandImportConfig = CommandImportConfig{
            command: String::from(""),
            planet_context: planet_context,
        };
        match response {
            Ok(_) => {
                let config_model: CreateTableConfig = response.unwrap();
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

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct LanguageConfig {
    #[validate(required, custom="validate_language_codes")]
    pub codes: Option<Vec<String>>,
    #[validate(custom="validate_default_language")]
    pub default: String,
}

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct FieldConfig {
    #[validate(length(equal=20))]
    #[serde(default="generate_id")]
    pub id: Option<String>,
    #[validate(required)]
    pub name: Option<String>,
    #[validate(required)]
    pub field_type: Option<String>,
    pub default: Option<String>,
    #[validate(required)]
    #[serde(default="FieldConfig::version")]
    pub version: Option<String>,
    pub required: Option<bool>,
    #[validate(required)]
    #[serde(default="FieldConfig::api_version")]
    pub api_version: Option<String>,
    pub indexed: Option<bool>,
    pub many: Option<bool>,
    pub select_data: Option<Vec<(String, String)>>,
}

impl ConfigStorageField for FieldConfig {
    fn defaults(
        select_data: Option<Vec<(String, String)>>,
    ) -> FieldConfig {
        let mut object: FieldConfig = FieldConfig{
            id: None,
            name: None,
            field_type: None,
            default: Some(String::from("")),
            version: Some(String::from(FIELD_VERSION)),
            required: Some(false),
            api_version: Some(String::from(FIELD_API_VERSION)),
            indexed: Some(true),
            select_data: None,
            many: None,
        };
        if select_data.is_some() {
            object.select_data = Some(select_data.unwrap());
        }
        return object;
    }
    fn version() -> Option<String> {
        return Some(String::from(FIELD_VERSION));
    }
    fn api_version() -> Option<String> {
        return Some(String::from(FIELD_API_VERSION));
    }
    /// Checks that FieldConfig passes validations
    fn is_valid(&self) -> Result<(), ValidationErrors> {
        match self.validate() {
            Ok(_) => return Ok(()),
            Err(errors) => {
                return Err(errors);
            },
        };
    }
    fn parse_from_db(db_data: DbData) -> Vec<FieldConfig> {
        // let select_data: Option<Vec<(String, String)>> = None;
        let mut fields: Vec<FieldConfig> = Vec::new();
        // I use data_collections, where we store the fields
        let data_collections = db_data.data_collections;
        let data = db_data.data;
        let data_objects = db_data.data_objects;
        eprintln!("parse_from_db :: data: {:#?}", &data);
        eprintln!("parse_from_db :: data_objects: {:#?}", &data_objects);
        eprintln!("parse_from_db :: data_collections: {:#?}", &data_collections);

        // 1. Go through data_objects and make map field names field_name -> FieldConfig. Also
        //   vector for order in 
        let mut map_fields_by_id: HashMap<String, FieldConfig> = HashMap::new();
        let mut map_fields_by_name: HashMap<String, FieldConfig> = HashMap::new();
        if data_objects.is_some() {
            let data_objects = data_objects.unwrap();
            for field_name in data_objects.keys() {
                let field_config_map = data_objects.get(field_name).unwrap();
                // Populate FieldConfig with attributes from map, which would do simple fields
                // Add to map_fields_by_id, already having FieldConfig map
                let required = make_bool_str(field_config_map.get("required").unwrap().clone());
                let indexed = make_bool_str(field_config_map.get("indexed").unwrap().clone());
                let many = make_bool_str(field_config_map.get("many").unwrap().clone());
                let field_id = field_config_map.get(ID).unwrap().clone();
                let field_config: FieldConfig = FieldConfig{
                    id: Some(field_id.clone()),
                    name: Some(field_config_map.get("name").unwrap().clone()),
                    field_type: Some(field_config_map.get("field_type").unwrap().clone()),
                    default: Some(field_config_map.get("default").unwrap().clone()),
                    version: Some(field_config_map.get("version").unwrap().clone()),
                    required: Some(required),
                    api_version: Some(field_config_map.get("api_version").unwrap().clone()),
                    indexed: Some(indexed),
                    many: Some(many),
                    select_data: None,
                };
                &map_fields_by_id.insert(field_id, field_config.clone());
                &map_fields_by_name.insert(field_name.clone(), field_config.clone());
            }
        }

        // 2. Go through data_collections for select_data and other complex structures. Add fields fo
        //      fields at map_fields_by_id
        let data_collections_1 = data_collections.clone();
        let data_collections_2 = data_collections.clone();
        if data_collections_1.is_some() {
            let data_collections = data_collections.unwrap();
            for data_collection_field in data_collections.keys() {
                let data_collection_field = data_collection_field.clone();
                eprintln!("parse_from_db :: data_collection_field: {:?}", &data_collection_field);
                let data_collection_field_str = &data_collection_field.as_str();
                let index = &data_collection_field_str.find("__");
                if index.is_none() {
                    continue;
                }
                // {field_name}__{attr}
                let pieces = &data_collection_field.split("__");
                let pieces: Vec<&str> = pieces.clone().collect();
                let field_name = pieces[0];
                let attr_name = pieces[1];
                eprintln!("parse_from_db :: field_name: {:?} attr_name: {:?}", &field_name, &attr_name);
                // if &data_collection_field != FIELD_IDS {
                //     // select_data, and other structures
                //     let field_list = 
                //         data_collections.get(&data_collection_field).unwrap().clone();
                //     if &data_collection_field == "select_data" {
                //         let field_config_ = &map_fields_by_id.get()
                //     }
                // }
            }
        }

        // 3. Go through fields_ids (data_collections) having list of ids and add to Vec fields and return
        if data_collections_2.is_some() {
            let data_collections_2 = data_collections_2.unwrap().clone();
            let field_ids = &data_collections_2.get(FIELD_IDS).unwrap();
            for field_id_data in field_ids.iter() {
                let field_id = &field_id_data.get(ID).unwrap().clone();
                let field_config = map_fields_by_id.get(field_id).unwrap().clone();
                &fields.push(field_config);
            }
        }

        // if data_collections.is_some() {
        //     let data_collections = data_collections.unwrap();
        //     let db_fields = data_collections.get(FIELDS).unwrap();
        //     for db_field in db_fields {
        //         let required = make_bool_str(db_field.get("required").unwrap().clone());
        //         let indexed = make_bool_str(db_field.get("indexed").unwrap().clone());
        //         let many = make_bool_str(db_field.get("many").unwrap().clone());
        //         // select_data is (id,option)::(id,option)::(id,option)
        //         let select_data_str = db_field.get("select_data").unwrap().clone();
        //         let select_data: Option<Vec<(String, String)>> = None;
        //         let mut select_data_list: Vec<(String, String)> = Vec::new();
        //         if select_data_str != String::from("") {
        //             let select_data_items = select_data_str.split("::");
        //         }
        //         let field: FieldConfig = FieldConfig{
        //             id: Some(db_field.get("id").unwrap().clone()),
        //             name: Some(db_field.get("name").unwrap().clone()),
        //             field_type: Some(db_field.get("field_type").unwrap().clone()),
        //             default: Some(db_field.get("default").unwrap().clone()),
        //             version: Some(db_field.get("version").unwrap().clone()),
        //             required: Some(required),
        //             api_version: Some(db_field.get("api_version").unwrap().clone()),
        //             indexed: Some(indexed),
        //             many: Some(many),
        //             select_data: select_data,
        //         };
        //     }
        // }
        // if data.is_some() {

        // }
        return fields
    }
    fn map_object_db(&self) -> HashMap<String, String> {
        let field = self.clone();
        let mut map: HashMap<String, String> = HashMap::new();
        let required = field.required.unwrap_or_default();
        let indexed = field.indexed.unwrap_or_default();
        let many = field.many.unwrap_or_default();
        map.insert(String::from("id"), field.id.unwrap_or_default());
        map.insert(String::from("name"), field.name.unwrap_or_default());
        map.insert(String::from("field_type"), field.field_type.unwrap_or_default());
        map.insert(String::from("default"), field.default.unwrap_or_default());
        map.insert(String::from("version"), field.version.unwrap_or_default());
        map.insert(String::from("required"), required.to_string());
        map.insert(String::from("api_version"), field.api_version.unwrap_or_default());
        map.insert(String::from("indexed"), indexed.to_string());
        map.insert(String::from("many"), many.to_string());
        return map;
    }

    fn map_collections_db(&self) -> HashMap<String, Vec<HashMap<String, String>>> {
        let field = self.clone();
        // select_data
        let select_data = field.select_data.unwrap_or_default();
        let mut map: HashMap<String, Vec<HashMap<String, String>>> = HashMap::new();
        let mut select_options: Vec<HashMap<String, String>> = Vec::new();
        for (select_id, select_value) in select_data {
            let mut map: HashMap<String, String> = HashMap::new();
            map.insert(String::from("key"), select_id);
            map.insert(String::from("value"), select_value);
            select_options.push(map);
        }
        if select_options.len() != 0 {
            let field_name = field.name.unwrap_or_default();
            map.insert(format!("{}__select_data", field_name), select_options);    
        }
        return map
    }
   
    fn map_objects_db(&self) -> HashMap<String, Vec<HashMap<String, String>>> {
        let field = self.clone();
        let mut map: HashMap<String, Vec<HashMap<String, String>>> = HashMap::new();
        // Include here items where you need field -> object in field configuration
        return map
    }
}

fn validate_default_language(language: &String) -> Result<(), ValidationError> {
    let language = &**language;
    let db_languages = get_db_languages();
    if db_languages.contains(&language) {
        return Ok(())
    } else {
        return Err(ValidationError::new("Invalid Default Language"));
    }
    
}

fn validate_language_codes(languages: &Vec<String>) -> Result<(), ValidationError> {
    let db_languages = get_db_languages();
    let mut check: bool = true;
    for language in languages.into_iter() {
        let language = &**language;
        if !db_languages.contains(&language) {
            check = false;
        }
    }
    if check {
        return Ok(())
    } else {
        return Err(ValidationError::new("Invalid Language"));
    }

}


#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct InsertIntoTableConfig {
    #[validate(required, regex="RE_COMMAND_INSERT_INTO_TABLE")]
    pub command: Option<String>,
    #[validate(required)]
    pub data: Option<HashMap<String, String>>,
}

impl InsertIntoTableConfig {

    pub fn defaults() -> InsertIntoTableConfig {
        let config: InsertIntoTableConfig = InsertIntoTableConfig{
            command: None,
            data: Some(HashMap::new()),
        };
        return config
    }

    pub fn import(
        &self, 
        planet_context: &PlanetContext, 
        yaml_path: &String
    ) -> Result<InsertIntoTableConfig, Vec<PlanetValidationError>> {
        let yaml_str: String = fetch_yaml_config(&yaml_path);
        // Deseralize the config entity
        let response: Result<InsertIntoTableConfig, serde_yaml::Error> = serde_yaml::from_str(&yaml_str);
        let import_config: CommandImportConfig = CommandImportConfig{
            command: String::from(""),
            planet_context: planet_context,
        };
        match response {
            Ok(_) => {
                let config_model: InsertIntoTableConfig = response.unwrap();
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