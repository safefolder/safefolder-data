extern crate xid;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_yaml;
use validator::{Validate, ValidationErrors, ValidationError};
use lazy_static::lazy_static;
use regex::Regex;
use tr::tr;

use crate::planet::validation::{CommandImportConfig, PlanetValidationError};
use crate::planet::{PlanetContext, PlanetError};

use crate::storage::constants::*;
use crate::storage::*;
use crate::storage::table::{DbData, DbTable};
use crate::planet::make_bool_str;
// use crate::functions::{Formula};
use crate::storage::fields::{
    text::*,
    date::*, 
    formula::*,
    StorageField
};
use crate::planet::constants::*;

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

lazy_static! {
    static ref RE_COMMAND_CREATE_TABLE: Regex = Regex::new(r#"(CREATE TABLE) "([a-zA-Z0-9_ ]+)""#).unwrap();
    static ref RE_COMMAND_INSERT_INTO_TABLE: Regex = Regex::new(r#"(INSERT INTO TABLE) "([a-zA-Z0-9_ ]+)""#).unwrap();
    static ref RE_COMMAND_GET_FROM_TABLE: Regex = Regex::new(r#"(GET FROM TABLE) "([a-zA-Z0-9_ ]+)"#).unwrap();
    static ref RE_COMMAND_SELECT_FROM_TABLE: Regex = Regex::new(r#"(SELECT FROM TABLE) "([a-zA-Z0-9_ ]+)"#).unwrap();
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct CreateTableConfig {
    #[validate(required, regex="RE_COMMAND_CREATE_TABLE")]
    pub command: Option<String>,
    #[validate]
    pub language: Option<LanguageConfig>,
    #[validate(required)]
    pub name: Option<FieldConfig>,
    #[validate]
    pub fields: Option<Vec<FieldConfig>>,
}

impl CreateTableConfig {

    pub fn defaults(name: Option<FieldConfig>) -> CreateTableConfig {
        let config: CreateTableConfig = CreateTableConfig{
            command: None,
            language: None,
            name: name,
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
                let mut config_model: CreateTableConfig = response.unwrap();
                let fields = config_model.clone().fields.unwrap();
                let validate: Result<(), ValidationErrors> = config_model.validate();
                match validate {
                    Ok(_) => {
                        let mut name_field = config_model.name.clone().unwrap();
                        name_field.required = Some(true);
                        name_field.name = Some(String::from(NAME_CAMEL));
                        config_model.name = Some(name_field);
                        // eprintln!("CreateTableConfig.import :: config_model: {:#?}", &config_model);
                        // Go through fields and check if any has name "Name", raising an error since
                        // is not allowed, reserved field.
                        for field in fields {
                            let field_name = field.name.clone().unwrap();
                            if field_name.to_lowercase() == NAME_CAMEL.to_lowercase() {
                                let mut planet_errors: Vec<PlanetValidationError> = Vec::new();
                                let error = PlanetValidationError{
                                    command: String::from("Create Table"),
                                    field: String::from("Name"),
                                    error_code: String::from("401"),
                                    message: tr!("Name is reserved field name. You cannot add it into 
                                    your fields. Use \"name\"")
                                };
                                planet_errors.push(error);
                                return Err(planet_errors);
                            }
                        }
                        return Ok(config_model);
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum DateFormat {
    Friendly,
    US,
    European,
    ISO,
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
    pub options: Option<Vec<String>>,
    pub formula: Option<String>,
    pub formula_format: Option<String>,
    pub formula_compiled: Option<String>,
    pub date_format: Option<DateFormat>,
    pub time_format: Option<i8>,
}

impl ConfigStorageField for FieldConfig {
    fn defaults(
        options: Option<Vec<String>>,
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
            options: None,
            many: None,
            formula: None,
            formula_format: None,
            formula_compiled: None,
            date_format: None,
            time_format: None,
        };
        if options.is_some() {
            object.options = Some(options.unwrap());
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
    fn get_name_field(db_data: &DbData) -> Option<FieldConfig> {
        let db_data = db_data.clone();
        let data_objects = db_data.data_objects;
        if data_objects.is_some() {
            let data_objects = data_objects.unwrap();
            for field_name in data_objects.keys() {
                if field_name == NAME_CAMEL {
                    let field_config_map = data_objects.get(field_name).unwrap();
                    let required = make_bool_str(field_config_map.get(REQUIRED).unwrap().clone());
                    let indexed = make_bool_str(field_config_map.get(INDEXED).unwrap().clone());
                    let many = make_bool_str(field_config_map.get(MANY).unwrap().clone());
                    let field_id = field_config_map.get(ID).unwrap().clone();
                    let field_config = FieldConfig{
                        id: Some(field_id.clone()),
                        name: Some(field_config_map.get(NAME).unwrap().clone()),
                        field_type: Some(field_config_map.get(FIELD_TYPE).unwrap().clone()),
                        default: Some(field_config_map.get(DEFAULT).unwrap().clone()),
                        version: Some(field_config_map.get(VERSION).unwrap().clone()),
                        required: Some(required),
                        api_version: Some(field_config_map.get(API_VERSION).unwrap().clone()),
                        indexed: Some(indexed),
                        many: Some(many),
                        options: None,
                        formula: None,
                        formula_format: None,
                        formula_compiled: None,
                        date_format: None,
                        time_format: None,
                    };
                    return Some(field_config);
                }
            }
        }
        return None
    }
    fn parse_from_db(db_data: &DbData) -> Result<Vec<FieldConfig>, PlanetError> {
        // let select_data: Option<Vec<(String, String)>> = None;
        let db_data = db_data.clone();
        let mut fields: Vec<FieldConfig> = Vec::new();
        // I use data_collections, where we store the fields
        let data_collections = db_data.data_collections;
        // let data = db_data.data;
        let data_objects = db_data.data_objects;
        // eprintln!("parse_from_db :: data: {:#?}", &data);
        // eprintln!("parse_from_db :: data_objects: {:#?}", &data_objects);
        // eprintln!("parse_from_db :: data_collections: {:#?}", &data_collections);

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
                let required = make_bool_str(field_config_map.get(REQUIRED).unwrap().clone());
                let indexed = make_bool_str(field_config_map.get(INDEXED).unwrap().clone());
                let many = make_bool_str(field_config_map.get(MANY).unwrap().clone());
                let field_id = field_config_map.get(ID).unwrap().clone();
                let field_name = field_config_map.get(NAME).unwrap().clone();
                let field_type_str = field_config_map.get(FIELD_TYPE).unwrap().as_str();
                let mut field_config = FieldConfig::defaults(None);
                field_config.id = Some(field_id.clone());
                field_config.name = Some(field_name.clone());
                field_config.field_type = Some(field_config_map.get(FIELD_TYPE).unwrap().clone());
                field_config.default = Some(field_config_map.get(DEFAULT).unwrap().clone());
                field_config.version = Some(field_config_map.get(VERSION).unwrap().clone());
                field_config.required = Some(required);
                field_config.api_version = Some(field_config_map.get(API_VERSION).unwrap().clone());
                field_config.indexed = Some(indexed);
                field_config.many = Some(many);
                // eprintln!("parse_from_db :: field_type_str: {}", field_type_str);
                
                match field_type_str {
                    FIELD_TYPE_SMALL_TEXT => {
                        let mut obj = SmallTextField::defaults(&field_config);
                        field_config = obj.build_config(field_config_map)?;
                    },
                    FIELD_TYPE_DATE => {
                        let mut obj = DateField::defaults(&field_config);
                        field_config = obj.build_config(field_config_map)?;
                    },
                    FIELD_TYPE_FORMULA => {
                        let mut obj = FormulaField::defaults(&field_config);
                        field_config = obj.build_config(field_config_map)?;
                        eprintln!("parse_from_db :: I did formula");
                    },
                    FIELD_TYPE_SELECT => {
                        let mut obj = SelectField::defaults(&field_config, None);
                        field_config = obj.build_config(field_config_map)?;
                    },
                    FIELD_TYPE_DURATION => {
                        let mut obj = DurationField::defaults(&field_config);
                        field_config = obj.build_config(field_config_map)?;
                    },
                    _ => {}
                }

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
                // eprintln!("parse_from_db :: data_collection_field: {:?}", &data_collection_field);
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
                // eprintln!("parse_from_db :: field_name: {:?} attr_name: {:?}", &field_name, &attr_name);
                if &data_collection_field != FIELD_IDS {
                    // select_options, and other structures
                    let field_list = 
                        data_collections.get(&data_collection_field).unwrap().clone();
                    // eprintln!("parse_from_db :: field_list: {:?}", &field_list);
                    // I need to get the Status field config, get by name
                    // eprintln!("parse_from_db :: data_collection_field: {}", &data_collection_field);
                    // data_collection_field: Status__select_options
                    if *&attr_name.to_lowercase() == SELECT_OPTIONS.to_lowercase() {
                        // eprintln!("parse_from_db :: I get into the options process",);
                        let mut field_config_ = map_fields_by_name.get(field_name).unwrap().clone();
                        let field_id = &field_config_.id.clone().unwrap();
                        let field_id = field_id.clone();
                        let mut field_options: Vec<String> = Vec::new();
                        for field_item in field_list {
                            let field_value = field_item.get(VALUE).unwrap().clone();
                            field_options.push(field_value);
                        }
                        // eprintln!("parse_from_db :: options: {:#?}", &field_options);
                        field_config_.options = Some(field_options);
                        map_fields_by_id.insert(field_id, field_config_);
                    }
                }
            }
        }

        // 3. Go through fields_ids (data_collections) having list of ids and add to Vec fields and return
        if data_collections_2.is_some() {
            let data_collections_2 = data_collections_2.unwrap().clone();
            let field_ids = data_collections_2.get(FIELD_IDS).unwrap();
            for field_id_data in field_ids.iter() {
                // eprintln!("parse_from_db :: field_id_data: {:#?}", &field_id_data);
                let field_id = &field_id_data.get(ID).unwrap().clone();
                let field_config = map_fields_by_id.get(field_id).unwrap().clone();
                &fields.push(field_config);
            }
        }

        // }
        // eprintln!("parse_from_db :: !!!!!!!!!!!!!!! fields: {:#?}", &fields);
        return Ok(fields)
    }
    fn map_object_db(
        &self, 
        field_type_map: &HashMap<String, String>,
        field_name_map: &HashMap<String, String>,
        db_table: &DbTable,
        table_name: &String
    ) -> Result<HashMap<String, String>, PlanetError> {
        let field_config = self.clone();
        let field_config_ = self.clone();
        let mut map: HashMap<String, String> = HashMap::new();
        let required = field_config.required.unwrap_or_default();
        let indexed = field_config.indexed.unwrap_or_default();
        let many = field_config.many.unwrap_or_default();
        let field_name = field_config.name.unwrap_or_default();
        let field_type = field_config.field_type.unwrap_or_default();
        let field_id = field_config.id.unwrap_or_default();
        let field_type_str = field_type.as_str();
        eprintln!("map_object_db :: field_name: {}", &field_name);
        map.insert(String::from(ID), field_id.clone());
        map.insert(String::from(NAME), field_name.clone());
        map.insert(String::from(FIELD_TYPE), field_type.clone());
        map.insert(String::from(DEFAULT), field_config.default.unwrap_or_default());
        map.insert(String::from(VERSION), field_config.version.unwrap_or_default());
        map.insert(String::from(REQUIRED), required.to_string());
        map.insert(String::from(API_VERSION), field_config.api_version.unwrap_or_default());
        map.insert(String::from(INDEXED), indexed.to_string());
        map.insert(String::from(MANY), many.to_string());

        match field_type_str {
            FIELD_TYPE_SMALL_TEXT => {
                map = SmallTextField::defaults(&field_config_).update_config_map(&map)?;
            },
            FIELD_TYPE_LONG_TEXT => {
                map = LongTextField::defaults(&field_config_).update_config_map(&map)?;
            },
            FIELD_TYPE_SELECT => {
                map = SelectField::defaults(&field_config_, None).update_config_map(&map)?;
            },
            FIELD_TYPE_DATE => {
                map = DateField::defaults(&field_config_).update_config_map(&map)?;
            },
            FIELD_TYPE_FORMULA => {
                map = FormulaField::defaults(&field_config_).update_config_map(
                    &map,
                    &field_name_map,
                    &field_type_map,
                    &db_table,
                    &table_name
                )?;
            },
            FIELD_TYPE_DURATION => {
                map = DurationField::defaults(&field_config_).update_config_map(&map)?;
            },
            _ => {}
        }
        // formula and functions
        // eprintln!("map_object_db :: formula...");
        // let formula = field_config.formula;
        // if formula.is_some() {
        //     let formula = formula.unwrap();
        //     let formula_format = field_config.formula_format.unwrap();
        //     let field_type_map = field_type_map.clone();
        //     let field_name_map = field_name_map.clone();
        //     let db_table = db_table.clone();
        //     let table_name = table_name.clone();
        //     let formula_compiled = Formula::defaults(
        //         &formula,
        //         &formula_format,
        //         None,
        //         Some(field_type_map),
        //         Some(field_name_map),
        //         Some(db_table),
        //         Some(table_name),
        //         false,
        //     )?;
        //     map.insert(String::from(FORMULA), formula);
        //     map.insert(String::from(FORMULA_FORMAT), formula_format);
        //     let formula_serialized = serde_yaml::to_string(&formula_compiled).unwrap();
        //     map.insert(String::from(FORMULA_COMPILED), formula_serialized);
        // }
        eprintln!("map_object_db :: finished!!!");
        return Ok(map);
    }

    fn map_collections_db(&self) -> Result<HashMap<String, Vec<HashMap<String, String>>>, PlanetError> {
        // 08/11/2021 We remove options from here, since is many structure often swapped
        let field_config = self.clone();
        // let field_type = &field_config.field_type.unwrap();
        // let map: HashMap<String, Vec<HashMap<String, String>>> = HashMap::new();
        // select_options and multi_select_options
        let options = field_config.options.unwrap_or_default();
        let mut map: HashMap<String, Vec<HashMap<String, String>>> = HashMap::new();
        let mut select_options: Vec<HashMap<String, String>> = Vec::new();
        for select_value in options {
            let select_id = generate_id().unwrap();
            let mut map: HashMap<String, String> = HashMap::new();
            map.insert(String::from("key"), select_id);
            map.insert(String::from("value"), select_value);
            select_options.push(map);
        }
        if select_options.len() != 0 {
            let field_name = field_config.name.unwrap_or_default();
            let collection_field = format!("{}__select_options", field_name);
            map.insert(collection_field, select_options);    
        }
        return Ok(map)
    }
   
    fn map_objects_db(&self) -> Result<HashMap<String, Vec<HashMap<String, String>>>, PlanetError> {
        // let field = self.clone();
        let map: HashMap<String, Vec<HashMap<String, String>>> = HashMap::new();
        // Include here items where you need field -> object in field configuration
        return Ok(map)
    }

    fn get_field_id_map(fields: &Vec<FieldConfig>) -> Result<HashMap<String, FieldConfig>, PlanetError> {
        let mut map: HashMap<String, FieldConfig> = HashMap::new();
        for field in fields {
            let field_ = field.clone();
            let field_id = field.id.clone().unwrap_or_default();
            map.insert(field_id, field_);
        }
        return Ok(map)
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
    pub name: Option<String>,
    #[validate(required)]
    pub data: Option<HashMap<String, String>>,
    pub data_collections: Option<HashMap<String, Vec<String>>>,
}

impl InsertIntoTableConfig {

    pub fn defaults(name: Option<String>) -> InsertIntoTableConfig {
        let config: InsertIntoTableConfig = InsertIntoTableConfig{
            command: None,
            name: name,
            data: Some(HashMap::new()),
            data_collections: None
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

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct DataId {
    pub id: Option<String>,
    pub fields: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct GetFromTableConfig {
    #[validate(required, regex="RE_COMMAND_GET_FROM_TABLE")]
    pub command: Option<String>,
    #[validate(required)]
    pub data: Option<DataId>,
}

impl GetFromTableConfig {

    pub fn defaults(id: String) -> GetFromTableConfig {
        let data_id: DataId = DataId{
            id: Some(id),
            fields: None,
        };
        let config: GetFromTableConfig = GetFromTableConfig{
            command: Some(String::from("GET FROM TABLE")),
            data: Some(data_id),
        };
        return config
    }

    pub fn import(
        &self, 
        planet_context: &PlanetContext, 
        yaml_path: &String
    ) -> Result<GetFromTableConfig, Vec<PlanetValidationError>> {
        let yaml_str: String = fetch_yaml_config(&yaml_path);
        // Deseralize the config entity
        let response: Result<GetFromTableConfig, serde_yaml::Error> = serde_yaml::from_str(&yaml_str);
        let import_config: CommandImportConfig = CommandImportConfig{
            command: String::from(""),
            planet_context: planet_context,
        };
        match response {
            Ok(_) => {
                let config_model: GetFromTableConfig = response.unwrap();
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
pub struct SelectFromTableConfig {
    #[validate(required, regex="RE_COMMAND_SELECT_FROM_TABLE")]
    pub command: Option<String>,
    #[validate(required)]
    pub r#where: Option<String>,
    #[serde(default="SelectFromTableConfig::page_default")]
    pub page: Option<String>,
    #[serde(default="SelectFromTableConfig::number_items_default")]
    pub number_items: Option<String>,
    pub fields: Option<Vec<String>>,
}

impl SelectFromTableConfig {

    pub fn defaults(r#where: Option<String>, page: Option<String>, number_items: Option<String>) -> Self {
        let config: SelectFromTableConfig = Self{
            command: Some(String::from("SELECT FROM TABLE")),
            r#where: r#where,
            page: page,
            number_items: number_items,
            fields: None,
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
    ) -> Result<SelectFromTableConfig, Vec<PlanetValidationError>> {
        let yaml_str: String = fetch_yaml_config(&yaml_path);
        // Deseralize the config entity
        let response: Result<SelectFromTableConfig, serde_yaml::Error> = serde_yaml::from_str(&yaml_str);
        let import_config: CommandImportConfig = CommandImportConfig{
            command: String::from(""),
            planet_context: planet_context,
        };
        match response {
            Ok(_) => {
                let config_model: SelectFromTableConfig = response.unwrap();
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
