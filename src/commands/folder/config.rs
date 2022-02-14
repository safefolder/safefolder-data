extern crate xid;

use std::collections::{BTreeMap,HashMap};
use serde::{Deserialize, Serialize};
use serde_yaml;
use validator::{Validate, ValidationErrors, ValidationError};
use lazy_static::lazy_static;
use regex::Regex;
use tr::tr;

use crate::planet::validation::{CommandImportConfig, PlanetValidationError};
use crate::planet::{PlanetContext, PlanetError, Context};

use crate::storage::constants::*;
use crate::storage::*;
use crate::storage::folder::{DbData, TreeFolder};
use crate::planet::make_bool_str;
use crate::storage::columns::{
    text::*,
    date::*, 
    number::*,
    formula::*,
    reference::*,
    structure::*,
    StorageColumn,
    ObjectStorageColumn,
};
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

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct LanguageConfig {
    #[validate(custom="validate_default_language")]
    pub default: String,
}

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct TextSearchConfig {
    #[validate(custom="validate_column_relevance")]
    pub column_relevance: BTreeMap<String, u8>,
}

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct SubFolderConfig {
    #[validate(length(equal=20))]
    #[serde(default="generate_id")]
    pub id: Option<String>,
    #[validate(required)]
    pub name: Option<String>,
    #[validate(required)]
    #[serde(default="SubFolderConfig::version")]
    pub version: Option<String>,
    pub parent_id: Option<String>,
    pub parent: Option<String>,
}
impl SubFolderConfig {
    pub fn version() -> Option<String> {
        return Some(String::from(SUB_FOLDER_VERSION));
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct CreateFolderConfig {
    #[validate(required, regex="RE_COMMAND_CREATE_FOLDER")]
    pub command: Option<String>,
    #[validate]
    pub language: Option<LanguageConfig>,
    #[validate]
    pub text_search: Option<TextSearchConfig>,
    #[validate(required)]
    pub name: Option<ColumnConfig>,
    #[validate]
    pub columns: Option<Vec<ColumnConfig>>,
    #[validate]
    pub sub_folders: Option<Vec<SubFolderConfig>>,
}

impl CreateFolderConfig {

    pub fn defaults(name: Option<ColumnConfig>) -> CreateFolderConfig {
        let config: CreateFolderConfig = CreateFolderConfig{
            command: None,
            language: None,
            text_search: None,
            name: name,
            columns: None,
            sub_folders: None,
        };
        return config
    }

    pub fn import(&self, planet_context: &PlanetContext, yaml_path: &String) -> Result<CreateFolderConfig, Vec<PlanetValidationError>> {
        let yaml_str: String = fetch_yaml_config(&yaml_path);
        // Deseralize the config entity
        let response: Result<CreateFolderConfig, serde_yaml::Error> = serde_yaml::from_str(&yaml_str);
        let import_config: CommandImportConfig = CommandImportConfig{
            command: String::from(""),
            planet_context: planet_context,
        };
        match response {
            Ok(_) => {
                eprintln!("CreateFolderConfig.import :: Imported fine");
                let mut config_model: CreateFolderConfig = response.unwrap();
                let columns = config_model.clone().columns.unwrap();
                let validate: Result<(), ValidationErrors> = config_model.validate();
                match validate {
                    Ok(_) => {
                        let mut name_field = config_model.name.clone().unwrap();
                        name_field.required = Some(true);
                        name_field.name = Some(String::from(NAME_CAMEL));
                        config_model.name = Some(name_field);
                        // eprintln!("CreateFolderConfig.import :: config_model: {:#?}", &config_model);
                        // Go through columns and check if any has name "Name", raising an error since
                        // is not allowed, reserved column.
                        for column in columns {
                            let column_name = column.name.clone().unwrap();
                            if column_name.to_lowercase() == NAME_CAMEL.to_lowercase() {
                                let mut planet_errors: Vec<PlanetValidationError> = Vec::new();
                                let error = PlanetValidationError{
                                    command: String::from("Create Folder"),
                                    column: String::from("Name"),
                                    error_code: String::from("401"),
                                    message: tr!("Name is reserved column name. You cannot add it into 
                                    your columns. Use \"name\"")
                                };
                                planet_errors.push(error);
                                return Err(planet_errors);
                            }
                        }
                        return Ok(config_model);
                    },
                    Err(errors) => {
                        eprintln!("CreateFolderConfig.import :: ValidationErrors: {:?}", &errors);
                        let command = &config_model.command.unwrap();
                        let planet_errors: Vec<PlanetValidationError> = import_config.parse_validator(
                            command, errors);
                        return Err(planet_errors);
                    }
                }
            },
            Err(error) => {
                eprintln!("CreateFolderConfig.import :: error: {:?}", &error);
                let mut planet_errors: Vec<PlanetValidationError> = Vec::new();
                planet_errors.push(import_config.parse_serde(&error));
                return Err(planet_errors);
            }
        }
    }

}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum DateFormat {
    Friendly,
    US,
    European,
    ISO,
}

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct ColumnConfig {
    #[validate(length(equal=20))]
    #[serde(default="generate_id")]
    pub id: Option<String>,
    #[validate(required)]
    pub name: Option<String>,
    #[validate(required)]
    pub column_type: Option<String>,
    pub default: Option<String>,
    #[validate(required)]
    #[serde(default="ColumnConfig::version")]
    pub version: Option<String>,
    pub required: Option<bool>,
    pub indexed: Option<bool>,
    pub many: Option<bool>,
    pub options: Option<Vec<String>>,
    pub formula: Option<String>,
    pub formula_format: Option<String>,
    pub formula_compiled: Option<String>,
    pub date_format: Option<DateFormat>,
    pub time_format: Option<i8>,
    pub currency_symbol: Option<String>,
    pub number_decimals: Option<i8>,
    pub linked_folder_id: Option<String>,
    pub delete_on_link_drop: Option<bool>,
    pub related_column: Option<String>,
    pub sequence: Option<String>,
    pub maximum: Option<String>,
    pub minimum: Option<String>,
    pub set_maximum: Option<String>,
    pub set_minimum: Option<String>,
    pub max_length: Option<String>,
    pub is_set: Option<String>,
    pub stats_function: Option<String>,
}

impl ConfigStorageColumn for ColumnConfig {
    fn defaults(
        options: Option<Vec<String>>,
    ) -> ColumnConfig {
        let mut object: ColumnConfig = ColumnConfig{
            id: None,
            name: None,
            column_type: None,
            default: Some(String::from("")),
            version: Some(String::from(FIELD_VERSION)),
            required: Some(false),
            indexed: Some(true),
            options: None,
            many: None,
            formula: None,
            formula_format: None,
            formula_compiled: None,
            date_format: None,
            time_format: None,
            currency_symbol: None,
            number_decimals: None,
            linked_folder_id: None,
            delete_on_link_drop: None,
            related_column: None,
            sequence: None,
            maximum: None,
            minimum: None,
            set_maximum: None,
            set_minimum: None,
            max_length: None,
            is_set: None,
            stats_function: None,
        };
        if options.is_some() {
            object.options = Some(options.unwrap());
        }
        return object;
    }
    fn version() -> Option<String> {
        return Some(String::from(FIELD_VERSION));
    }
    /// Checks that ColumnConfig passes validations
    fn is_valid(&self) -> Result<(), ValidationErrors> {
        match self.validate() {
            Ok(_) => return Ok(()),
            Err(errors) => {
                return Err(errors);
            },
        };
    }
    fn get_name_column(db_data: &DbData) -> Option<ColumnConfig> {
        let db_data = db_data.clone();
        let data_collections = db_data.data_collections;
        if data_collections.is_some() {
            let data_collections = data_collections.unwrap();
            let columns = data_collections.get(COLUMNS);
            if columns.is_some() {
                let column_config_map = columns.unwrap();
                let column_config_map = column_config_map.clone()[0].clone();
                let required = make_bool_str(column_config_map.get(REQUIRED).unwrap().clone());
                let indexed = make_bool_str(column_config_map.get(INDEXED).unwrap().clone());
                let many = make_bool_str(column_config_map.get(MANY).unwrap().clone());
                let column_id = column_config_map.get(ID).unwrap().clone();
                let column_config = ColumnConfig{
                    id: Some(column_id.clone()),
                    name: Some(column_config_map.get(NAME).unwrap().clone()),
                    column_type: Some(column_config_map.get(COLUMN_TYPE).unwrap().clone()),
                    default: Some(column_config_map.get(DEFAULT).unwrap().clone()),
                    version: Some(column_config_map.get(VERSION).unwrap().clone()),
                    required: Some(required),
                    indexed: Some(indexed),
                    many: Some(many),
                    options: None,
                    formula: None,
                    formula_format: None,
                    formula_compiled: None,
                    date_format: None,
                    time_format: None,
                    currency_symbol: None,
                    number_decimals: None,
                    linked_folder_id: None,
                    delete_on_link_drop: None,
                    related_column: None,
                    sequence: None,
                    maximum: None,
                    minimum: None,
                    set_maximum: None,
                    set_minimum: None,
                    max_length: None,
                    is_set: None,
                    stats_function: None,
                };
                return Some(column_config);
            }
        }
        return None
    }
    fn get_column_config_map(
        planet_context: &PlanetContext,
        context: &Context,
        table: &DbData
    ) -> Result<BTreeMap<String, ColumnConfig>, PlanetError> {
        let columns = ColumnConfig::get_config(
            planet_context,
            context,
            table
        );
        if columns.is_ok() {
            let columns = columns.unwrap().clone();
            let mut map: BTreeMap<String, ColumnConfig> = BTreeMap::new();
            for column in columns {
                let column_name = column.name.clone().unwrap_or_default();
                map.insert(column_name, column.clone());
            }
            return Ok(map)
        }
        return Err(
            PlanetError::new(
                500, 
                Some(tr!("Could not get column config map")),
            )
        )
    }
    fn get_config(
        planet_context: &PlanetContext,
        context: &Context,
        db_data: &DbData
    ) -> Result<Vec<ColumnConfig>, PlanetError> {
        // eprintln!("get_config...");
        // let select_data: Option<Vec<(String, String)>> = None;
        let db_data = db_data.clone();
        let mut columns: Vec<ColumnConfig> = Vec::new();
        // I use data_collections, where we store the columns
        let data_collections = db_data.data_collections.clone();
        // let data = db_data.data;
        // let data_objects = db_data.data_objects;
        // eprintln!("get_config :: data: {:#?}", &data);
        // eprintln!("get_config :: data_objects: {:#?}", &data_objects);
        // eprintln!("get_config :: data_collections: {:#?}", &data_collections);

        // 1. Go through data_objects and make map column names column_name -> ColumnConfig. Also
        //   vector for order in 
        let mut map_columns_by_id: BTreeMap<String, ColumnConfig> = BTreeMap::new();
        let mut map_columns_by_name: BTreeMap<String, ColumnConfig> = BTreeMap::new();
        if data_collections.is_some() {
            let data_collections = data_collections.unwrap();
            let column_list = &data_collections.get(COLUMNS);
            if column_list.is_some() {
                let column_list = column_list.unwrap();
                for column_config_map in column_list {
                    // let column_config_map = data_objects.get(column_id).unwrap();
                    let column_type = column_config_map.get(COLUMN_TYPE);
                    if !column_type.is_some() {
                        continue
                    }
                    // Populate ColumnConfig with attributes from map, which would do simple columns
                    // Add to map_columns_by_id, already having ColumnConfig map
                    let required_wrap = column_config_map.get(REQUIRED);
                    let mut required: bool = false;
                    if required_wrap.is_some() {
                        required = make_bool_str(required_wrap.unwrap().clone());
                    }
                    let indexed_wrap = column_config_map.get(INDEXED);
                    let mut indexed = false;
                    if indexed_wrap.is_some() {
                        indexed = make_bool_str(indexed_wrap.unwrap().clone());
                    }
                    let mut many = false;
                    let many_wrap = column_config_map.get(MANY);
                    if many_wrap.is_some() {
                        many = make_bool_str(many_wrap.unwrap().clone());
                    }
                    let mut column_config = ColumnConfig::defaults(None);
                    column_config.default = None;
                    let default_wrap = column_config_map.get(DEFAULT);
                    if default_wrap.is_some() {
                        column_config.default = Some(default_wrap.unwrap().clone());
                    }
                    column_config.version = None;
                    let version_wrap = column_config_map.get(VERSION);
                    if version_wrap.is_some() {
                        column_config.version = Some(version_wrap.unwrap().clone());
                    }
                    let column_id = column_config_map.get(ID).unwrap().clone();
                    let column_name = column_config_map.get(NAME).unwrap().clone();
                    let column_type_str = column_config_map.get(COLUMN_TYPE).unwrap().as_str();
                    
                    column_config.id = Some(column_id.clone());
                    column_config.name = Some(column_name.clone());
                    column_config.column_type = Some(column_config_map.get(COLUMN_TYPE).unwrap().clone());
                    column_config.required = Some(required);
                    column_config.indexed = Some(indexed);
                    column_config.many = Some(many);
                    // eprintln!("get_config :: column_type_str: {}", column_type_str);
    
                    let is_set = column_config_map.get(IS_SET);
                    if is_set.is_some() {
                        let is_set = is_set.unwrap().clone();
                        if is_set == String::from("true") || is_set == String::from("1") {
                            // Update with SetColumn properties / attributes, and later on the item column config
                            let mut obj = SetColumn::defaults(&column_config);
                            column_config = obj.get_config(column_config_map)?;
                        }
                    }
    
                    match column_type_str {
                        COLUMN_TYPE_SMALL_TEXT => {
                            let mut obj = SmallTextColumn::defaults(&column_config);
                            column_config = obj.get_config(column_config_map)?;
                        },
                        COLUMN_TYPE_LONG_TEXT => {
                            let mut obj = LongTextColumn::defaults(&column_config);
                            column_config = obj.get_config(column_config_map)?;
                        },
                        COLUMN_TYPE_NUMBER => {
                            let mut obj = NumberColumn::defaults(&column_config);
                            column_config = obj.get_config(column_config_map)?;
                        },
                        COLUMN_TYPE_CHECKBOX => {
                            let mut obj = CheckBoxColumn::defaults(&column_config);
                            column_config = obj.get_config(column_config_map)?;
                        },
                        COLUMN_TYPE_DATE => {
                            let mut obj = DateColumn::defaults(&column_config);
                            column_config = obj.get_config(column_config_map)?;
                        },
                        COLUMN_TYPE_FORMULA => {
                            let mut obj = FormulaColumn::defaults(&column_config);
                            column_config = obj.get_config(column_config_map)?;
                        },
                        COLUMN_TYPE_SELECT => {
                            let mut obj = SelectColumn::defaults(&column_config, None);
                            column_config = obj.get_config(column_config_map)?;
                        },
                        COLUMN_TYPE_DURATION => {
                            let mut obj = DurationColumn::defaults(&column_config);
                            column_config = obj.get_config(column_config_map)?;
                        },
                        COLUMN_TYPE_CREATED_TIME => {
                            let mut obj = AuditDateColumn::defaults(&column_config);
                            column_config = obj.get_config(column_config_map)?;
                        },
                        COLUMN_TYPE_LAST_MODIFIED_TIME => {
                            let mut obj = AuditDateColumn::defaults(&column_config);
                            column_config = obj.get_config(column_config_map)?;
                        },
                        COLUMN_TYPE_CURRENCY => {
                            let mut obj = CurrencyColumn::defaults(&column_config);
                            column_config = obj.get_config(column_config_map)?;
                        },
                        COLUMN_TYPE_PERCENTAGE => {
                            let mut obj = PercentageColumn::defaults(&column_config);
                            column_config = obj.get_config(column_config_map)?;
                        },
                        COLUMN_TYPE_LINK => {
                            let mut obj = LinkColumn::defaults(
                                planet_context,
                                context,
                                &column_config,
                                None,
                                None
                            );
                            column_config = obj.get_config(column_config_map)?;
                        },
                        COLUMN_TYPE_REFERENCE => {
                            let mut obj = ReferenceColumn::defaults(
                                planet_context,
                                context,
                                &column_config,
                                None,
                            );
                            column_config = obj.get_config(column_config_map)?;
                        },
                        COLUMN_TYPE_TEXT => {
                            let mut obj = TextColumn::defaults(
                                &column_config,
                                None
                            );
                            column_config = obj.get_config(column_config_map)?;
                        },
                        COLUMN_TYPE_GENERATE_ID => {
                            let mut obj = GenerateIdColumn::defaults(
                                &column_config,
                            );
                            column_config = obj.get_config(column_config_map)?;
                        },
                        COLUMN_TYPE_GENERATE_NUMBER => {
                            let mut obj = GenerateNumberColumn::defaults(
                                &column_config,
                                None,
                                None
                            );
                            column_config = obj.get_config(column_config_map)?;
                        },
                        COLUMN_TYPE_PHONE => {
                            let mut obj = PhoneColumn::defaults(
                                &column_config,
                            );
                            column_config = obj.get_config(column_config_map)?;
                        },
                        COLUMN_TYPE_EMAIL => {
                            let mut obj = EmailColumn::defaults(
                                &column_config,
                            );
                            column_config = obj.get_config(column_config_map)?;
                        },
                        COLUMN_TYPE_URL => {
                            let mut obj = UrlColumn::defaults(
                                &column_config,
                            );
                            column_config = obj.get_config(column_config_map)?;
                        },
                        COLUMN_TYPE_RATING => {
                            let mut obj = RatingColumn::defaults(
                                &column_config,
                            );
                            column_config = obj.get_config(column_config_map)?;
                        },
                        COLUMN_TYPE_OBJECT => {
                            let mut obj = ObjectColumn::defaults(
                                &column_config,
                            );
                            column_config = obj.get_config(column_config_map)?;
                        },
                        _ => {}
                    }
                    let _ = &map_columns_by_id.insert(column_id, column_config.clone());
                    let _ = &map_columns_by_name.insert(column_name.clone(), column_config.clone());
                    columns.push(column_config.clone());
                }
            }
            for data_collection_field in data_collections.keys() {
                let data_collection_field = data_collection_field.clone();
                // eprintln!("get_config :: data_collection_field: {:?}", &data_collection_field);
                let data_collection_field_str = &data_collection_field.as_str();
                let index = &data_collection_field_str.find("__");
                if index.is_some() {
                    // {column_name}__{attr}
                    let pieces = &data_collection_field.split("__");
                    let pieces: Vec<&str> = pieces.clone().collect();
                    let column_name = pieces[0];
                    let attr_name = pieces[1];
                    // eprintln!("get_config :: column_name: {:?} attr_name: {:?}", &column_name, &attr_name);
                    // select_options, and other structures
                    let field_list = 
                        data_collections.get(&data_collection_field).unwrap().clone();
                    // eprintln!("get_config :: field_list: {:?}", &field_list);
                    // I need to get the Status column config, get by name
                    // eprintln!("get_config :: data_collection_field: {}", &data_collection_field);
                    // data_collection_field: Status__select_options
                    if *&attr_name.to_lowercase() == SELECT_OPTIONS.to_lowercase() {
                        // eprintln!("get_config :: I get into the options process",);
                        let mut propertty_config_ = map_columns_by_name.get(column_name).unwrap().clone();
                        // let column_id = &propertty_config_.id.clone().unwrap();
                        // let column_id = column_id.clone();
                        let mut field_options: Vec<String> = Vec::new();
                        for field_item in field_list {
                            let field_value = field_item.get(VALUE).unwrap().clone();
                            field_options.push(field_value);
                        }
                        // eprintln!("get_config :: options: {:#?}", &field_options);
                        propertty_config_.options = Some(field_options);
                        // map_columns_by_id.insert(column_id, propertty_config_);
                        columns.push(propertty_config_);
                    }
                }
            }
        }
        // eprintln!("get_config :: !!!!!!!!!!!!!!! columns: {:#?}", &columns);
        return Ok(columns)
    }
    fn create_config(
        &self, 
        planet_context: &PlanetContext,
        context: &Context,
        properties_map: &HashMap<String, ColumnConfig>,
        db_folder: &TreeFolder,
        folder_name: &String,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        // I use this operation when creating folders
        let column_config = self.clone();
        let propertty_config_ = self.clone();
        let mut map: BTreeMap<String, String> = BTreeMap::new();
        let required = column_config.required.unwrap_or_default();
        let indexed = column_config.indexed.unwrap_or_default();
        let many = column_config.many.unwrap_or_default();
        let column_name = column_config.name.unwrap_or_default();
        let column_type = column_config.column_type.unwrap_or_default();
        let column_id = column_config.id.unwrap_or_default();
        let column_type_str = column_type.as_str();
        // eprintln!("map_object_db :: column_name: {}", &column_name);
        map.insert(String::from(ID), column_id.clone());
        map.insert(String::from(NAME), column_name.clone());
        map.insert(String::from(COLUMN_TYPE), column_type.clone());
        map.insert(String::from(DEFAULT), column_config.default.unwrap_or_default());
        map.insert(String::from(VERSION), column_config.version.unwrap_or_default());
        map.insert(String::from(REQUIRED), required.to_string());
        map.insert(String::from(INDEXED), indexed.to_string());
        map.insert(String::from(MANY), many.to_string());

        let is_set = column_config.is_set;
        if is_set.is_some() {
            let is_set = is_set.unwrap().clone();
            if is_set == String::from("true") || is_set == String::from("1") {
                // Update with SetColumn properties / attributes, and later on the item column config
                map = SetColumn::defaults(&propertty_config_).create_config(&map)?;
            }
        }

        match column_type_str {
            COLUMN_TYPE_SMALL_TEXT => {
                map = SmallTextColumn::defaults(&propertty_config_).create_config(&map)?;
            },
            COLUMN_TYPE_LONG_TEXT => {
                map = LongTextColumn::defaults(&propertty_config_).create_config(&map)?;
            },
            COLUMN_TYPE_SELECT => {
                map = SelectColumn::defaults(&propertty_config_, None).create_config(&map)?;
            },
            COLUMN_TYPE_DATE => {
                map = DateColumn::defaults(&propertty_config_).create_config(&map)?;
            },
            COLUMN_TYPE_FORMULA => {
                map = FormulaColumn::defaults(&propertty_config_).create_config(
                    &map,
                    &properties_map,
                    &db_folder,
                    &folder_name
                )?;
            },
            COLUMN_TYPE_DURATION => {
                map = DurationColumn::defaults(&propertty_config_).create_config(&map)?;
            },
            COLUMN_TYPE_CREATED_TIME => {
                map = AuditDateColumn::defaults(&propertty_config_).create_config(&map)?;
            },
            COLUMN_TYPE_LAST_MODIFIED_TIME => {
                map = AuditDateColumn::defaults(&propertty_config_).create_config(&map)?;
            },
            COLUMN_TYPE_CURRENCY => {
                map = CurrencyColumn::defaults(&propertty_config_).create_config(&map)?;
            },
            COLUMN_TYPE_PERCENTAGE => {
                map = PercentageColumn::defaults(&propertty_config_).create_config(&map)?;
            },
            COLUMN_TYPE_LINK => {
                map = LinkColumn::defaults(
                    planet_context,
                    context,
                    &propertty_config_,
                    Some(db_folder.clone()),
                    None
                ).create_config(
                    &map,
                    &properties_map,
                    &folder_name,
                )?;
            },
            COLUMN_TYPE_REFERENCE => {
                map = ReferenceColumn::defaults(
                    planet_context,
                    context,
                    &propertty_config_,
                    Some(db_folder.clone()),
                ).create_config(
                    &map,
                    &properties_map,
                    &folder_name
                )?;
            },
            COLUMN_TYPE_GENERATE_ID => {
                map = GenerateIdColumn::defaults(&propertty_config_).create_config(&map)?;
            },
            COLUMN_TYPE_GENERATE_NUMBER => {
                map = GenerateNumberColumn::defaults(
                    &propertty_config_,
                    None,
                    None
                ).create_config(&map)?;
            },
            COLUMN_TYPE_PHONE => {
                map = PhoneColumn::defaults(&propertty_config_).create_config(&map)?;
            },
            COLUMN_TYPE_EMAIL => {
                map = EmailColumn::defaults(&propertty_config_).create_config(&map)?;
            },
            COLUMN_TYPE_URL => {
                map = UrlColumn::defaults(&propertty_config_).create_config(&map)?;
            },
            COLUMN_TYPE_RATING => {
                map = RatingColumn::defaults(&propertty_config_).create_config(&map)?;
            },
            COLUMN_TYPE_OBJECT => {
                map = ObjectColumn::defaults(&propertty_config_).create_config(&map)?;
            },
            _ => {}
        }
        // eprintln!("map_object_db :: finished!!!");
        return Ok(map);
    }

    fn map_collections_db(&self) -> Result<BTreeMap<String, Vec<BTreeMap<String, String>>>, PlanetError> {
        // 08/11/2021 We remove options from here, since is many structure often swapped
        let column_config = self.clone();
        // let column_type = &column_config.column_type.unwrap();
        // let map: BTreeMap<String, Vec<BTreeMap<String, String>>> = BTreeMap::new();
        // select_options and multi_select_options
        let options = column_config.options.unwrap_or_default();
        let mut map: BTreeMap<String, Vec<BTreeMap<String, String>>> = BTreeMap::new();
        let mut select_options: Vec<BTreeMap<String, String>> = Vec::new();
        for select_value in options {
            let select_id = generate_id().unwrap();
            let mut map: BTreeMap<String, String> = BTreeMap::new();
            map.insert(String::from("key"), select_id);
            map.insert(String::from("value"), select_value);
            select_options.push(map);
        }
        if select_options.len() != 0 {
            let column_name = column_config.name.unwrap_or_default();
            let collection_field = format!("{}__select_options", column_name);
            map.insert(collection_field, select_options);    
        }
        return Ok(map)
    }
   
    fn map_objects_db(&self) -> Result<BTreeMap<String, Vec<BTreeMap<String, String>>>, PlanetError> {
        // let column = self.clone();
        let map: BTreeMap<String, Vec<BTreeMap<String, String>>> = BTreeMap::new();
        // Include here items where you need column -> object in column configuration
        return Ok(map)
    }

    fn get_column_id_map(columns: &Vec<ColumnConfig>) -> Result<BTreeMap<String, ColumnConfig>, PlanetError> {
        let mut map: BTreeMap<String, ColumnConfig> = BTreeMap::new();
        for column in columns {
            let column_ = column.clone();
            let column_id = column.id.clone().unwrap_or_default();
            map.insert(column_id, column_);
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

fn validate_column_relevance(column_relevance: &BTreeMap<String, u8>) -> Result<(), ValidationError> {
    let column_relevance = column_relevance.clone();
    for (_, relevance) in column_relevance {
        if relevance > 5 || relevance < 1 {
            return Err(ValidationError::new("Invalid Text Search Relevance"));
        }
    }
    return Ok(())    
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct SubFolderDataConfig {
    pub id: Option<String>,
    pub is_reference: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct InsertIntoFolderConfig {
    #[validate(required, regex="RE_COMMAND_INSERT_INTO_FOLDER")]
    pub command: Option<String>,
    #[validate(required)]
    pub name: Option<String>,
    pub sub_folders: Option<Vec<SubFolderDataConfig>>,
    #[validate(required)]
    pub data: Option<BTreeMap<String, String>>,
    pub data_collections: Option<BTreeMap<String, Vec<String>>>,
}

impl InsertIntoFolderConfig {

    pub fn defaults(name: Option<String>) -> InsertIntoFolderConfig {
        let config: InsertIntoFolderConfig = InsertIntoFolderConfig{
            command: None,
            name: name,
            data: Some(BTreeMap::new()),
            data_collections: None,
            sub_folders: None,
        };
        return config
    }

    pub fn import(
        &self, 
        planet_context: &PlanetContext, 
        yaml_path: &String
    ) -> Result<InsertIntoFolderConfig, Vec<PlanetValidationError>> {
        let yaml_str: String = fetch_yaml_config(&yaml_path);
        // Deseralize the config entity
        let response: Result<InsertIntoFolderConfig, serde_yaml::Error> = serde_yaml::from_str(&yaml_str);
        let import_config: CommandImportConfig = CommandImportConfig{
            command: String::from(""),
            planet_context: planet_context,
        };
        match response {
            Ok(_) => {
                let config_model: InsertIntoFolderConfig = response.unwrap();
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
