extern crate tr;
extern crate colored;

use std::collections::{BTreeMap, HashMap};
use std::time::Instant;
use yaml_rust;
use serde_yaml;
use validator::{Validate, ValidationError};
use lazy_static::lazy_static;

use tr::tr;
use colored::*;
use regex::{Regex, Captures};

use crate::statements::folder::config::{
    create_minimum_column_map,
};
use crate::statements::*;
use crate::statements::{Statement, StatementCallMode};
use crate::storage::folder::{TreeFolder, FolderSchema, DbData, RoutingData, build_value_list, DbDataMini, TreeFolderItem, FolderItem};
use crate::storage::space::{SpaceDatabase};
use crate::planet::{
    PlanetContext, 
    PlanetError,
    Context, 
    Environment
};
use crate::storage::ConfigStorageColumn;
use crate::storage::{generate_id, get_db_languages};
use crate::storage::constants::*;
use crate::planet::constants::*;
use crate::planet::make_bool_str;
use crate::storage::columns::{
    text::*,
    date::*, 
    number::*,
    formula::*,
    reference::*,
    media::*,
    structure::*,
    processing::*,
    StorageColumn,
    StorageColumnBasic,
    ObjectStorageColumn,
    EnvDbStorageColumn
};

lazy_static! {
    pub static ref RE_CREATE_FOLDER_MAIN: Regex = Regex::new(r#"CREATE[\s]+FOLDER[\s]+"*(?P<FolderName>[\w\s]+)"*\s+\([\n\t\s]*(?P<Config>.[^)]+),*\);"#).unwrap();
    pub static ref RE_CREATE_FOLDER_CONFIG: Regex = Regex::new(r#"([\s]*LANGUAGE (?P<Language>spanish|english|french|german|italian|portuguese|norwegian|swedish|danish),*)|([\s]*NAME COLUMN (?P<NameConfig>(SmallText|LongText|Number|Currency|Percentage|GenerateNumber|Phone|Email|Url|Rating)),*)|([\s]*("(?P<Column>[\w\s]+)")[\s]+(?P<ColumnType>SmallText|LongText|Checkbox|Number|Select|Currency|Percentage|GenerateNumber|Phone|Email|Url|Rating|Object|File|Date|Formula|Duration|CreatedTime|LastModifiedTime|CreatedBy|LastModifiedBy|Link|Reference|Language|GenerateId|Stats))([\s]*[WITH]*[\s]*(?P<Options>[\w\s"\$=\{\}\|]*)),|([\s]*SUB FOLDER (?P<SubFolderName>[\w\s]+)),|([\s]*SUB FOLDER (?P<SubFolderNameAlt>[\w\s]+) WITH (?P<SubFolderOptions>[\w\s"\$=\{\}\|]*)),|([\s]*SEARCH RELEVANCE WITH (?P<SearchRelevanceOptions>[\w\s"\$=\{\}\|]*)),"#).unwrap();
    pub static ref RE_LIST_FOLDERS: Regex = Regex::new(r#"LIST[\s]+FOLDERS;"#).unwrap();
    pub static ref RE_DESCRIBE_FOLDER: Regex = Regex::new(r#"DESCRIBE[\s]+FOLDER[\s]+(?P<FolderName>[\w\s]+);"#).unwrap();
    pub static ref RE_DROP_FOLDER: Regex = Regex::new(r#"DROP[\s]+FOLDER[\s]+(?P<FolderName>[\w\s]+);"#).unwrap();
    pub static ref RE_ADD_COLUMN: Regex = Regex::new(r#"ADD[\s]+COLUMN[\s]+INTO[\s]+"*(?P<FolderName>[\w\s]+)"*\([\n\t\s]*(?P<Config>.[^)]+),*\);"#).unwrap();
    pub static ref RE_ADD_COLUMN_CONFIG: Regex = Regex::new(r#"([\s]*("(?P<Column>[\w\s]+)")[\s]+(?P<ColumnType>SmallText|LongText|Checkbox|Number|Select|Currency|Percentage|GenerateNumber|Phone|Email|Url|Rating|Object|File|Date|Formula|Duration|CreatedTime|LastModifiedTime|CreatedBy|LastModifiedBy|Link|Reference|Language|GenerateId|Stats))([\s]*[WITH]*[\s]*(?P<Options>[\w\s"\$=\{\}\|]*))"#).unwrap();
}

pub const WITH_PARENT: &str = "Parent";
pub const WITH_REQUIRED: &str = "Required";
pub const WITH_OPTIONS: &str = "Options";
pub const WITH_NUMBER_DECIMALS: &str = "NumberDecimals";
pub const WITH_CURRENCY_SYMBOL: &str = "CurrencySymbol";
pub const WITH_MAXIMUM: &str = "Maximum";
pub const WITH_MINIMUM: &str = "Minimum";
pub const WITH_SET_MINIMUM: &str = "SetMinimum";
pub const WITH_SET_MAXIMUM: &str = "SetMaximum";
pub const WITH_IS_SET: &str = "IsSet";
pub const WITH_MANY: &str = "Many";
pub const WITH_DEFAULT: &str = "Default";
pub const WITH_FORMULA: &str = "Formula";
pub const WITH_FORMULA_FORMAT: &str = "FormulaFormat";
pub const WITH_DATE_FORMAT: &str = "DateFormat";
pub const WITH_TIME_FORMAT: &str = "TimeFormat";
pub const WITH_LINKED_FOLDER_ID: &str = "LinkedFolderId";
pub const WITH_DELETE_ON_LINK_DROP: &str = "DeleteOnLinkDrop";
pub const WITH_RELATED_COLUMN: &str = "RelatedColumn";
pub const WITH_SEQUENCE: &str = "Sequence";
pub const WITH_MAX_LENGTH: &str = "MaxLength";
pub const WITH_STATS_FUNCTION: &str = "StatsFunction";
pub const WITH_CONTENT_TYPES: &str = "ContentTypes";
pub const WITH_MODE: &str = "Mode";

pub const ALLOWED_WITH_OPTIONS: [&str; 24] = [
    WITH_PARENT, WITH_REQUIRED, WITH_OPTIONS, WITH_NUMBER_DECIMALS, WITH_CURRENCY_SYMBOL, 
    WITH_MAXIMUM, WITH_MINIMUM, WITH_SET_MINIMUM, WITH_SET_MAXIMUM, WITH_IS_SET, WITH_MANY, 
    WITH_DEFAULT, WITH_FORMULA, WITH_FORMULA_FORMAT, WITH_DATE_FORMAT, WITH_TIME_FORMAT, 
    WITH_LINKED_FOLDER_ID, WITH_DELETE_ON_LINK_DROP, WITH_RELATED_COLUMN, WITH_SEQUENCE, 
    WITH_MAX_LENGTH, WITH_STATS_FUNCTION, WITH_CONTENT_TYPES, WITH_MODE
];


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
    pub content_types: Option<Vec<String>>,
    pub mode: Option<String>,
    pub statements: Option<String>,
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
            content_types: None,
            mode: None,
            statements: None,
        };
        if options.is_some() {
            object.options = Some(options.unwrap());
        }
        return object;
    }
    fn version() -> Option<String> {
        return Some(String::from(FIELD_VERSION));
    }
    fn get_name_column(db_data: &DbData) -> Option<ColumnConfig> {
        let db_data = db_data.clone();
        let data = db_data.data;
        if data.is_some() {
            let data = data.unwrap();
            let columns = data.get(COLUMNS);
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
                    content_types: None,
                    mode: None,
                    statements: None,
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
        let data = db_data.data.clone();
        // let data = db_data.data;
        // let data_objects = db_data.data_objects;
        // eprintln!("get_config :: data: {:#?}", &data);
        // eprintln!("get_config :: data_objects: {:#?}", &data_objects);
        // eprintln!("get_config :: data_collections: {:#?}", &data_collections);

        // 1. Go through data_objects and make map column names column_name -> ColumnConfig. Also
        //   vector for order in 
        let mut map_columns_by_id: BTreeMap<String, ColumnConfig> = BTreeMap::new();
        let mut map_columns_by_name: BTreeMap<String, ColumnConfig> = BTreeMap::new();
        if data.is_some() {
            let data = data.unwrap();
            let column_list = &data.get(COLUMNS);
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
                        COLUMN_TYPE_FILE => {
                            let mut obj = FileColumn::defaults(
                                &column_config,
                                None,
                                None,
                            );
                            column_config = obj.get_config(column_config_map)?;
                        },
                        COLUMN_TYPE_STATEMENT => {
                            let mut obj = StatementColumn::defaults(
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
            for data_collection_column in data.keys() {
                let data_collection_column = data_collection_column.clone();
                // eprintln!("get_config :: data_collection_field: {:?}", &data_collection_field);
                let data_collection_column_str = &data_collection_column.as_str();
                let index = &data_collection_column_str.find("__");
                if index.is_some() {
                    // {column_name}__{attr}
                    let pieces = &data_collection_column.split("__");
                    let pieces: Vec<&str> = pieces.clone().collect();
                    let column_name = pieces[0];
                    let attr_name = pieces[1];
                    // eprintln!("get_config :: column_name: {:?} attr_name: {:?}", &column_name, &attr_name);
                    // select_options, and other structures
                    let column_list = data.get(&data_collection_column).unwrap().clone();
                    // eprintln!("get_config :: field_list: {:?}", &field_list);
                    // I need to get the Status column config, get by name
                    // eprintln!("get_config :: data_collection_field: {}", &data_collection_field);
                    // data_collection_field: Status__select_options
                    if *&attr_name.to_lowercase() == SELECT_OPTIONS.to_lowercase() {
                        // eprintln!("get_config :: I get into the options process",);
                        let mut propertty_config_ = map_columns_by_name.get(column_name).unwrap().clone();
                        // let column_id = &propertty_config_.id.clone().unwrap();
                        // let column_id = column_id.clone();
                        let mut column_options: Vec<String> = Vec::new();
                        for column_item in column_list {
                            let column_value = column_item.get(VALUE).unwrap().clone();
                            column_options.push(column_value);
                        }
                        // eprintln!("get_config :: options: {:#?}", &field_options);
                        propertty_config_.options = Some(column_options);
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
        space_database: &SpaceDatabase
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
        let env = Environment{
            context: context,
            planet_context: planet_context
        };
        let space_database = space_database.clone();
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
            COLUMN_TYPE_FILE => {
                map = FileColumn::defaults(
                    &propertty_config_,
                    None,
                    None
                ).create_config(&map)?;
            },
            COLUMN_TYPE_STATEMENT => {
                map = StatementColumn::defaults(&propertty_config_).create_config(
                    &map,
                    &env,
                    &space_database
                )?;
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateFolderCompiledStmt {
    pub folder_name: String,
    pub language: Option<LanguageConfig>,
    pub text_search: Option<TextSearchConfig>,
    pub name: ColumnConfig,
    pub columns: Option<Vec<ColumnConfig>>,
    pub sub_folders: Option<Vec<SubFolderConfig>>,
}

impl CreateFolderCompiledStmt {

    pub fn defaults(name: Option<ColumnConfig>) -> Self {
        let mut name_conf: ColumnConfig = ColumnConfig::defaults(None);
        if name.is_some() {
            name_conf = name.unwrap();
        }
        let config= Self {
            language: None,
            text_search: None,
            name: name_conf,
            columns: None,
            sub_folders: None,
            folder_name: String::from(""),
        };
        return config
    }

}

pub fn process_column(column_str: &str, column_type: &str, item: &Captures) -> Result<ColumnConfig, Vec<PlanetError>> {
    let mut column = ColumnConfig::defaults(None);
    let name = column_str.trim().to_string();
    column.column_type = Some(column_type.to_string());
    column.name = Some(name);
    column.id = generate_id();
    let options = item.name("Options");
    let mut errors: Vec<PlanetError> = Vec::new();
    if options.is_some() {
        let options = options.unwrap().as_str();
        // eprintln!("CreateFolder.compile :: options: {}", options);
        let result = WithOptions::defaults(
            &options.to_string()
        );
        if result.is_err() {
            let error = result.unwrap_err();
            errors.push(error);
        } else {
            let with_options_obj = result.unwrap();
            let with_options = &with_options_obj.options;
            // eprintln!("CreateFolder.compile :: with_options: {:#?}", with_options);
            // Validate I have allowed options
            let mut is_valid = true;
            for (k, _v) in with_options {
                let found = ALLOWED_WITH_OPTIONS.contains(&k.as_str());
                if !found {
                    errors.push(
                        PlanetError::new(
                            500, 
                            Some(
                                tr!("Statement compile error: Option \"{}\" not allowed.", &k)
                            ),
                        )
                    );
                    is_valid = false;
                }
            }
            if is_valid {
                // We process with options only in case all options sent are OK
                if *&with_options.contains_key(WITH_REQUIRED) {
                    let required = &with_options_obj.get_single_value(
                        WITH_REQUIRED
                    );
                    if *required == String::from("True") {
                        column.required = Some(true);
                    } else {
                        column.required = Some(false);
                    }
                }
                if *&with_options.contains_key(WITH_OPTIONS) {
                    let options = with_options.get(WITH_OPTIONS);
                    if options.is_some() {
                        let options = options.unwrap().clone();
                        let mut options_string: Vec<String> = Vec::new();
                        for option in options {
                            let option_value = option.value;
                            options_string.push(option_value);
                        }
                        column.options = Some(options_string);
                    }
                }
                if *&with_options.contains_key(WITH_NUMBER_DECIMALS) {
                    let number_decimals = &with_options_obj.get_single_value(
                        WITH_NUMBER_DECIMALS
                    );
                    let number_decimals: i8 = FromStr::from_str(number_decimals.as_str()).unwrap();
                    column.number_decimals = Some(number_decimals);
                }
                if *&with_options.contains_key(WITH_CURRENCY_SYMBOL) {
                    let currency_symbol = &with_options_obj.get_single_value(
                        WITH_CURRENCY_SYMBOL
                    );
                    column.currency_symbol = Some(currency_symbol.clone());
                }
                if *&with_options.contains_key(WITH_MAXIMUM) {
                    let maximum = &with_options_obj.get_single_value(
                        WITH_MAXIMUM
                    );
                    column.maximum = Some(maximum.clone());
                }
                if *&with_options.contains_key(WITH_MINIMUM) {
                    let minimum = &with_options_obj.get_single_value(
                        WITH_MINIMUM
                    );
                    column.minimum = Some(minimum.clone());
                }
                if *&with_options.contains_key(WITH_SET_MINIMUM) {
                    let set_minimum = &with_options_obj.get_single_value(
                        WITH_SET_MINIMUM
                    );
                    column.set_minimum = Some(set_minimum.clone());
                }
                if *&with_options.contains_key(WITH_SET_MAXIMUM) {
                    let set_maximum = &with_options_obj.get_single_value(
                        WITH_SET_MAXIMUM
                    );
                    column.set_maximum = Some(set_maximum.clone());
                }
                if *&with_options.contains_key(WITH_IS_SET) {
                    let is_set = &with_options_obj.get_single_value(
                        WITH_IS_SET
                    );
                    column.is_set = Some(is_set.clone().to_lowercase());
                }
                if *&with_options.contains_key(WITH_MANY) {
                    let many = &with_options_obj.get_single_value(
                        WITH_MANY
                    );
                    if *many == String::from("True") {
                        column.many = Some(true);
                    } else {
                        column.many = Some(false);
                    }
                }
                if *&with_options.contains_key(WITH_DEFAULT) {
                    let default = &with_options_obj.get_single_value(
                        WITH_DEFAULT
                    );
                    column.default = Some(default.clone());
                }
                if *&with_options.contains_key(WITH_FORMULA) {
                    let formula = &with_options_obj.get_single_value(
                        WITH_FORMULA
                    );
                    column.formula = Some(formula.clone());
                }
                if *&with_options.contains_key(WITH_FORMULA_FORMAT) {
                    let formula_format = &with_options_obj.get_single_value(
                        WITH_FORMULA_FORMAT
                    );
                    column.formula_format = Some(formula_format.clone());
                }
                if *&with_options.contains_key(WITH_DATE_FORMAT) {
                    let date_format = &with_options_obj.get_single_value(
                        WITH_DATE_FORMAT
                    );
                    let date_format = date_format.as_str();
                    let mut date_format_obj: DateFormat = DateFormat::Friendly;
                    match date_format {
                        DATE_FORMAT_FRIENDLY => {
                            date_format_obj = DateFormat::Friendly;
                        },
                        DATE_FORMAT_US => {
                            date_format_obj = DateFormat::US;
                        },
                        DATE_FORMAT_EUROPEAN => {
                            date_format_obj = DateFormat::European;
                        },
                        DATE_FORMAT_ISO => {
                            date_format_obj = DateFormat::ISO;
                        },
                        _ => {},
                    }
                    column.date_format = Some(date_format_obj);
                }
                if *&with_options.contains_key(WITH_TIME_FORMAT) {
                    let time_format = &with_options_obj.get_single_value(
                        WITH_TIME_FORMAT
                    );
                    let time_format: i8 = FromStr::from_str(time_format.as_str()).unwrap();
                    column.time_format = Some(time_format);
                }
                if *&with_options.contains_key(WITH_LINKED_FOLDER_ID) {
                    let linked_folder_id = &with_options_obj.get_single_value(
                        WITH_LINKED_FOLDER_ID
                    );
                    column.linked_folder_id = Some(linked_folder_id.clone());
                }
                if *&with_options.contains_key(WITH_DELETE_ON_LINK_DROP) {
                    let delete_on_link_drop = &with_options_obj.get_single_value(
                        WITH_DELETE_ON_LINK_DROP
                    );
                    if *delete_on_link_drop == String::from("True") {
                        column.delete_on_link_drop = Some(true);
                    } else {
                        column.delete_on_link_drop = Some(false);
                    }
                }
                if *&with_options.contains_key(WITH_RELATED_COLUMN) {
                    let related_column = &with_options_obj.get_single_value(
                        WITH_RELATED_COLUMN
                    );
                    column.related_column = Some(related_column.clone());
                }
                if *&with_options.contains_key(WITH_SEQUENCE) {
                    let sequence = &with_options_obj.get_single_value(
                        WITH_SEQUENCE
                    );
                    column.sequence = Some(sequence.clone());
                }
                if *&with_options.contains_key(WITH_MAX_LENGTH) {
                    let max_length = &with_options_obj.get_single_value(
                        WITH_MAX_LENGTH
                    );
                    column.max_length = Some(max_length.clone());
                }
                if *&with_options.contains_key(WITH_STATS_FUNCTION) {
                    let stats_function = &with_options_obj.get_single_value(
                        WITH_STATS_FUNCTION
                    );
                    column.stats_function = Some(stats_function.clone());
                }
                if *&with_options.contains_key(WITH_CONTENT_TYPES) {
                    let content_types = with_options.get(
                        WITH_CONTENT_TYPES
                    );
                    if content_types.is_some() {
                        let content_types = content_types.unwrap().clone();
                        let mut content_types_str: Vec<String> = Vec::new();
                        for content_type in content_types {
                            let content_type_str = content_type.value;
                            content_types_str.push(content_type_str);
                        }
                        column.content_types = Some(content_types_str);
                    }
                }
                if *&with_options.contains_key(WITH_MODE) {
                    let mode = &with_options_obj.get_single_value(
                        WITH_MODE
                    );
                    column.mode = Some(mode.clone());
                }
            }
        }
    }
    if errors.len() > 0 {
        return Err(errors)
    }
    return Ok(column)
}

#[derive(Debug, Clone)]
pub struct CreateFolderStatement {
}

impl<'gb> StatementCompiler<'gb, CreateFolderCompiledStmt> for CreateFolderStatement {

    fn compile(
        &self, 
        statement_text: &String
    ) -> Result<CreateFolderCompiledStmt, Vec<PlanetError>> {
        let mut compiled_statement = CreateFolderCompiledStmt::defaults(
            None
        );
        let expr = &RE_CREATE_FOLDER_MAIN;
        let check = expr.is_match(&statement_text);
        let mut errors: Vec<PlanetError> = Vec::new();
        if !check {
            let error = PlanetError::new(
                500, 
                Some(
                    tr!("Create folder syntax not valid.")
                ),
            );
            errors.push(error);
            return Err(errors)
        }
        let captures = expr.captures(&statement_text);
        if captures.is_some() {
            let captures = captures.unwrap();
            let folder_name = captures.name("FolderName").unwrap().as_str();
            compiled_statement.folder_name = folder_name.to_string();
        }
        let expr = &RE_CREATE_FOLDER_CONFIG;
        let items = expr.captures_iter(&statement_text.as_str());
        let mut sub_folders: Vec<SubFolderConfig> = Vec::new();
        let mut columns: Vec<ColumnConfig> = Vec::new();
        for item in items {
            let name_config = item.name("NameConfig");
            let sub_folder_name = item.name("SubFolderName");
            let sub_folder_name_alt = item.name("SubFolderNameAlt");
            let column = item.name("Column");
            let column_type = item.name("ColumnType");
            let language = item.name("Language");
            let search_options = item.name("SearchRelevanceOptions");
            if language.is_some() {
                let language_str = language.unwrap().as_str();
                let language = LanguageConfig{
                    default: language_str.to_string()
                };
                compiled_statement.language = Some(language);
            }
            if sub_folder_name.is_some() || sub_folder_name_alt.is_some() {
                // Sub folders
                let mut sub_folder_name_: &str = "";
                if sub_folder_name.is_some() {
                    sub_folder_name_ = sub_folder_name.unwrap().as_str();
                }
                if sub_folder_name_alt.is_some() {
                    sub_folder_name_ = sub_folder_name_alt.unwrap().as_str();
                }
                let sub_folder_options = item.name("SubFolderOptions");
                let mut sub_folder_obj: SubFolderConfig = SubFolderConfig{
                    id: generate_id(),
                    name: Some(sub_folder_name_.to_string()),
                    parent: None,
                    parent_id: None,
                    version: SubFolderConfig::version(),
                };
                if sub_folder_options.is_some() {
                    let sub_folder_options = sub_folder_options.unwrap().as_str();
                    let result = WithOptions::defaults(
                        &sub_folder_options.to_string()
                    );
                    if result.is_err() {
                        let error = result.unwrap_err();
                        errors.push(error);
                    } else {
                        let with_options_obj = result.unwrap();
                        let with_options = &with_options_obj.options;
                        if *&with_options.contains_key(WITH_PARENT) {
                            let parent = &with_options_obj.get_single_value(
                                WITH_PARENT
                            );
                            sub_folder_obj.parent = Some(parent.clone());
                        }
                    }
                }
                sub_folders.push(sub_folder_obj);
            } else if name_config.is_some() {
                // Name config, should be first item in columns
                let column_type = name_config.unwrap().as_str();
                let mut name = compiled_statement.name.clone();
                name.name = Some(String::from("Name"));
                name.id = generate_id();
                name.column_type = Some(column_type.to_string());
                compiled_statement.name = name;
            } else if search_options.is_some() {
                let search_options = search_options.unwrap().as_str();
                let result = WithOptions::defaults(
                    &search_options.to_string()
                );
                if result.is_err() {
                    let error = result.unwrap_err();
                    errors.push(error);
                } else {
                    let with_options_obj = result.unwrap();
                    let with_options = &with_options_obj.options;
                    let mut column_relevance: BTreeMap<String, u8> = BTreeMap::new();
                    for (k, v) in with_options {
                        let value = v.clone()[0].clone().value;
                        let value_int: u8 = FromStr::from_str(value.as_str()).unwrap();
                        let key = k.clone();
                        column_relevance.insert(key, value_int);
                    }
                    // Validate column relevance
                    let validate = validate_column_relevance(&column_relevance);
                    if validate.is_err() {
                        errors.push(
                            PlanetError::new(
                                500, 
                                Some(
                                    tr!("Validation error for column relevance: \"{}\".", &search_options)
                                ),
                            )
                        );
                    }
                    let text_search_config = TextSearchConfig{
                        column_relevance: column_relevance
                    };
                    compiled_statement.text_search = Some(text_search_config);
                }
            } else if column.is_some() && column_type.is_some() {
                // Column
                // Here I need compilation of options the same as in subfolders, being common software
                let column_str = column.unwrap().as_str();
                let column_type = column_type.unwrap().as_str();
                let result = process_column(column_str, column_type, &item);
                if result.is_ok() {
                    let column = result.unwrap();
                    columns.push(column);
                }
            }
        }
        compiled_statement.sub_folders = Some(sub_folders);
        compiled_statement.columns = Some(columns);
        if errors.len() > 0 {
            return Err(errors)
        }
        eprintln!("CreateFolder.compile :: compiled_statement: {:#?}", &compiled_statement);
        return Ok(compiled_statement.clone())
    }

}

impl<'gb> Statement<'gb> for CreateFolderStatement {

    fn run(
        &self,
        env: &'gb Environment<'gb>,
        space_database: &SpaceDatabase,
        statement_text: &String,
    ) -> Result<Vec<yaml_rust::Yaml>, Vec<PlanetError>> {
        // run would return a generic Yaml object which has the statement response
        // yaml_rust::Yaml
        let space_database = space_database.clone();
        let context = env.context;
        let planet_context = env.planet_context;
        let t_1 = Instant::now();
        let mut errors: Vec<PlanetError> = Vec::new();
        let statement = self.compile(statement_text);
        if statement.is_err() {
            let errors = statement.unwrap_err();
            return Err(errors)
        }
        let statement = statement.unwrap();
        let home_dir = planet_context.home_path.unwrap_or_default();
        let account_id = context.account_id.unwrap_or_default();
        let space_id = context.space_id.unwrap_or_default();
        let site_id = context.site_id;

        let result: Result<TreeFolder, PlanetError> = TreeFolder::defaults(
            space_database.connection_pool.clone(),
            Some(home_dir),
            Some(account_id),
            Some(space_id),
            site_id,
        );
        match result {
            Ok(_) => {
                // let command = self.config.command.clone().unwrap_or_default();
                // let expr = Regex::new(r#"(CREATE FOLDER) "(?P<folder_name>[a-zA-Z0-9_ ]+)"#).unwrap();
                // let table_name_match = expr.captures(&command).unwrap();
                // let folder_name = &table_name_match["folder_name"].to_string();
                let folder_name = statement.folder_name;

                // routing parameters
                let account_id = Some(context.account_id.unwrap_or_default().to_string());
                let site_id = context.site_id;
                let space_id = context.space_id;
                let box_id = context.box_id;

                // db folder options with language data
                let db_folder: TreeFolder = result.unwrap();
                let mut data: BTreeMap<String, Vec<BTreeMap<String, String>>> = BTreeMap::new();
                // config data
                let mut columns = statement.columns.unwrap().clone();
                let sub_folders = statement.sub_folders;
                let mut column_ids: Vec<BTreeMap<String, String>> = Vec::new();
                let mut columns_list: Vec<BTreeMap<String, String>> = Vec::new();

                // validate and clean defaults for language and text_search
                // TODO: Place this into a validation operation for the entity
                let language = statement.language;
                if language.is_some() {
                    let language = language.unwrap();
                    let language_default = language.default;
                    data.insert(LANGUAGE_DEFAULT.to_string(), build_value_list(&language_default));
                } else {
                    data.insert(LANGUAGE_DEFAULT.to_string(), build_value_list(&LANGUAGE_ENGLISH.to_string()));
                }
                let text_search = statement.text_search;
                if text_search.is_some() {
                    let text_search = text_search.unwrap();
                    let mut column_relevance = text_search.column_relevance;
                    let mut my_map: BTreeMap<String, String> = BTreeMap::new();
                    // Include Text rule if not found
                    let mut has_text = false;
                    for (column_relevance_item, _) in column_relevance.clone() {
                        if column_relevance_item == String::from(TEXT_COLUMN) {
                            has_text = true;
                        }
                    }
                    if !has_text {
                        let relevance: u8 = 1;
                        column_relevance.insert(TEXT_COLUMN.to_string(), relevance);
                    }
                    for (column_relevance_item, relevance) in column_relevance {
                        let mut has_column = false;
                        for column in columns.clone().iter() {
                            let column = column.clone();
                            let column_name = column.name.unwrap_or_default();
                            if column_name.to_lowercase() == column_relevance_item.to_lowercase() {
                                has_column = true;
                            }
                        }
                        if !has_column && column_relevance_item != TEXT_COLUMN.to_string() {
                            errors.push(
                                PlanetError::new(
                                    500, 
                                    Some(
                                        tr!("Configuration does not have field \"{}\"", &column_relevance_item)
                                    ),
                                )
                            );
                        } else {
                            let relevance_string= relevance.to_string();
                            my_map.insert(column_relevance_item.clone(), relevance_string);
                        }
                    }
                    let mut my_list: Vec<BTreeMap<String, String>> = Vec::new();
                    my_list.push(my_map);
                    data.insert(TEXT_SEARCH_COLUMN_RELEVANCE.to_string(), my_list);
                } else {
                    let mut my_map: BTreeMap<String, String> = BTreeMap::new();
                    my_map.insert(TEXT_COLUMN.to_string(), String::from("1"));
                    let mut my_list: Vec<BTreeMap<String, String>> = Vec::new();
                    my_list.push(my_map);
                    data.insert(TEXT_SEARCH_COLUMN_RELEVANCE.to_string(), my_list);
                }

                // name column
                let name_field_config = statement.name;
                columns.insert(0, name_field_config);
                let mut column_name_map: BTreeMap<String, String> = BTreeMap::new();
                // populate column_type_map and column_name_map
                let mut column_type_map: BTreeMap<String, String> = BTreeMap::new();
                let mut columns_map: HashMap<String, ColumnConfig> = HashMap::new();
                for column in columns.iter() {
                    let column_attrs = column.clone();
                    let column_name = column.name.clone().unwrap();
                    let column_type = column.column_type.clone();
                    let mut column_id_map: BTreeMap<String, String> = BTreeMap::new();
                    let column_id = column_attrs.id.unwrap_or_default();
                    column_id_map.insert(String::from(ID), column_id.clone());
                    columns_map.insert(column_name.clone(), column.clone());
                    let _ = &column_ids.push(column_id_map);
                    if column_type.is_some() {
                        let column_type = column_type.unwrap();
                        column_type_map.insert(column_name.clone(), column_type);
                    }
                    let column_name_str = column_name.as_str();
                    if column_name_map.get(column_name_str).is_some() == false {
                        // id => name
                        column_name_map.insert(column_name.clone(), column_id.clone());
                    } else {
                        errors.push(
                            PlanetError::new(
                                500, 
                                Some(tr!("There is already a column with name \"{}\"", &column_name)),
                            )
                        );
                    }
                }
                
                for column in columns.iter() {
                    // column simple attributes
                    let map = &column.create_config(
                        planet_context,
                        context,
                        &columns_map,
                        &db_folder,
                        &folder_name,
                        &space_database
                    );
                    if map.is_err() {
                        let error = map.clone().unwrap_err();
                        errors.push(error);
                    }
                    let map = map.clone().unwrap();
                    let map_list = &column.map_collections_db();
                    if map_list.is_err() {
                        let error = map_list.clone().unwrap_err();
                        errors.push(error);
                    }
                    let map_list = map_list.clone().unwrap();
                    let map_list = map_list.clone();
                    data.extend(map_list);
                    columns_list.push(map);
                }
                // Sub folders
                if sub_folders.is_some() {
                    let sub_folders = sub_folders.unwrap();
                    let mut my_map: HashMap<String, String> = HashMap::new();
                    for sub_folder in sub_folders.clone() {
                        let sub_folder_name = sub_folder.name.unwrap_or_default();
                        let sub_folder_id = sub_folder.id.unwrap_or_default();
                        my_map.insert(sub_folder_name, sub_folder_id);
                    }
                    // resolve parent id by parent name
                    let mut list: Vec<BTreeMap<String, String>> = Vec::new();
                    for sub_folder in sub_folders.clone() {
                        let parent = sub_folder.parent.clone().unwrap_or_default();
                        let parent_id = my_map.get(&parent);
                        let mut sub_folder_db_map: BTreeMap<String, String> = BTreeMap::new();
                        sub_folder_db_map.insert(NAME.to_string(), sub_folder.name.unwrap_or_default());
                        sub_folder_db_map.insert(ID.to_string(), sub_folder.id.unwrap_or_default());
                        sub_folder_db_map.insert(
                            VERSION.to_string(), 
                            sub_folder.version.unwrap_or_default()
                        );
                        if parent_id.is_some() {
                            let parent_id = parent_id.unwrap();
                            sub_folder_db_map.insert(
                                PARENT_ID.to_string(), 
                                parent_id.clone()
                            );
                        }
                        list.push(sub_folder_db_map);
                    }
                    data.insert(
                        SUB_FOLDERS.to_string(),
                        list
                    );
                }
                // eprintln!("CreateFolder.run :: data_objects_new: {:#?}", &data_objects_new);
                // text column
                let column_id = &generate_id().unwrap_or_default();
                columns_list.push(
                    create_minimum_column_map(
                        column_id,
                        &TEXT_COLUMN.to_string(),
                        &COLUMN_TYPE_TEXT.to_string(),
                    )
                );
                let mut column_id_map: BTreeMap<String, String> = BTreeMap::new();
                column_id_map.insert(String::from(ID), column_id.clone());
                column_ids.push(column_id_map);
                // language column
                let column_id = &generate_id().unwrap_or_default();
                columns_list.push(
                    create_minimum_column_map(
                        column_id,
                        &LANGUAGE_COLUMN.to_string(),
                        &COLUMN_TYPE_LANGUAGE.to_string(),
                    )
                );
                data.insert(COLUMNS.to_string(), columns_list);
                // routing
                let routing_wrap = RoutingData::defaults(
                    account_id, 
                    site_id, 
                    space_id,
                    box_id,
                    None,
                );
                let mut data_wrap: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>> = None;
                if data.len() > 0 {
                    data_wrap = Some(data);
                }
                let db_data = DbData::defaults(
                    &folder_name, 
                    data_wrap,
                    None,
                    routing_wrap,
                    None,
                    None,
                );
                if db_data.is_err() {
                    let error = db_data.clone().unwrap_err();
                    errors.push(error);
                }
                let db_data = db_data.unwrap();
                // Only output TEMP the choices data to include in insert
                eprintln!("CreateFolder.run :: db_data: {:#?}", db_data.clone());

                let response = db_folder.create(&db_data);
                if response.is_err() {
                    let error = response.clone().unwrap_err();
                    errors.push(error);
                    // Since we have data error on creating folder, we return all errors
                    return Err(errors);
                }
                let response = response.unwrap();
                let response_src = response.clone();
                // response.id
                let folder_name = &response.name.unwrap_or_default();
                let folder_id = &response.id.unwrap();

                //
                // Related folders, I need to update their config, like Links
                //
                let columns = response.data.unwrap();
                let mut linked_folder_ids: Vec<String> = Vec::new();
                let mut map_column_names: BTreeMap<String, String> = BTreeMap::new();
                let mut map_column_ids: BTreeMap<String, String> = BTreeMap::new();
                for (_, v) in columns {
                    if v.len() == 1 {
                        let v = &v[0];
                        let column_type = v.get(COLUMN_TYPE);
                        let column_name = v.get(NAME);
                        let column_id = v.get(ID);
                        if column_type.is_some() {
                            let column_type = column_type.unwrap();
                            let column_name = column_name.unwrap();
                            let column_id = column_id.unwrap();
                            if column_type == COLUMN_TYPE_LINK {
                                let linked_folder_id = v.get(LINKED_FOLDER_ID);
                                if linked_folder_id.is_some() {
                                    let linked_folder_id = linked_folder_id.unwrap();
                                    let has_id = linked_folder_ids.contains(linked_folder_id);
                                    if !has_id {
                                        linked_folder_ids.push(linked_folder_id.clone());
                                        map_column_names.insert(
                                            linked_folder_id.clone(), column_name.clone()
                                        );
                                        map_column_ids.insert(
                                            linked_folder_id.clone(), column_id.clone()
                                        );
                                    }
                                }
                            }
                        }    
                    }
                }
                // Get each folder from db_folder instance and update with link to this created table
                let local_column_map = db_data.data.unwrap();
                for link_folder_id in linked_folder_ids {
                    let linked_folder = db_folder.get(&link_folder_id);
                    if linked_folder.is_ok() {
                        let mut linked_folder = linked_folder.unwrap();
                        let data = linked_folder.clone().data;
                        let mut map = linked_folder.clone().data.unwrap();
                        let column_name = map_column_names.get(&link_folder_id).unwrap();
                        let remote_column_map = local_column_map.get(
                            column_name
                        ).unwrap().clone();
                        // Update Column map with columns for local column, Link with link_folder_id being 
                        // this local Column
                        if remote_column_map.len() == 1 {
                            let mut remote_column_map = remote_column_map[0].clone();
                            remote_column_map.insert(String::from(LINKED_FOLDER_ID), folder_id.clone());
                            remote_column_map.insert(String::from(NAME), folder_name.clone());
                            remote_column_map.insert(String::from(MANY), String::from(TRUE));
                            let mut my_list: Vec<BTreeMap<String, String>> = Vec::new();
                            my_list.push(remote_column_map.clone());
                            map.insert(folder_name.clone(), my_list);
                            let mut column_ids_map = linked_folder.data.unwrap();
                            let mut column_ids = column_ids_map.get(
                                COLUMN_IDS
                            ).unwrap().clone();
                            let column_id = map_column_ids.get(&link_folder_id.clone()).unwrap();
                            let mut element: BTreeMap<String, String> = BTreeMap::new();
                            element.insert(String::from(ID),column_id.clone());
                            column_ids.push(element);
                            column_ids_map.insert(String::from(COLUMN_IDS), column_ids);
                            let mut new_data = data.unwrap();
                            new_data.extend(column_ids_map);
                            linked_folder.data = Some(new_data);
                            eprintln!("CreateFolder.run :: linked_folder: {:#?}", &linked_folder);
                            let _ = db_folder.update(&linked_folder);
                        }
                    }
                }

                println!();
                let quote_color = format!("{}", String::from("\""));
                if errors.len() > 0 {
                    return Err(errors)
                }
                // TODO: I will have this into an object that I return
                println!("Created folder {} :: {} => {}",
                    format!("{}{}{}", &quote_color.blue(), &folder_name.blue(), &quote_color.blue()),
                    &folder_id.magenta(),
                    format!("{}{}{}", &quote_color.green(), &folder_name.green(), &quote_color.green()),
                );
                eprintln!("CreateFolder.run :: time: {} µs", &t_1.elapsed().as_micros());

                let _mine = db_folder.get_by_name(folder_name);
                let response_coded = serde_yaml::to_string(&response_src);
                if response_coded.is_err() {
                    let error = PlanetError::new(
                        500, 
                        Some(tr!("Error encoding statement response.")),
                    );
                    errors.push(error);
                    return Err(errors)
                }
                let response = response_coded.unwrap();
                let yaml_response = yaml_rust::YamlLoader::load_from_str(
                    response.as_str()
                ).unwrap();
                let yaml_response = yaml_response.clone();
                return Ok(yaml_response)
                },
            Err(error) => {
                eprintln!("CreateFolder.run :: schema error...");
                errors.push(error);
                Err(errors)
            }
        }
    }

}

#[derive(Debug, Clone)]
pub struct ListFoldersStatement {
}

impl<'gb> StatementCompiler<'gb, ()> for ListFoldersStatement {

    fn compile(
        &self, 
        statement_text: &String
    ) -> Result<(), Vec<PlanetError>> {
        let expr = &RE_LIST_FOLDERS;
        let check = expr.is_match(&statement_text);
        let mut errors: Vec<PlanetError> = Vec::new();
        // This match is already executed on resolution operation. Here we include for consistency, would really
        // apply if statement is more complex than LIST FOLDERS; with more parameters.
        if !check {
            let error = PlanetError::new(
                500, 
                Some(
                    tr!("List folder syntax not valid.")
                ),
            );
            errors.push(error);
            return Err(errors)
        }
        return Ok(())
    }
}

impl<'gb> Statement<'gb> for ListFoldersStatement {

    fn run(
        &self,
        env: &'gb Environment<'gb>,
        space_database: &SpaceDatabase,
        statement_text: &String,
    ) -> Result<Vec<yaml_rust::Yaml>, Vec<PlanetError>> {
        let space_database = space_database.clone();
        let context = env.context;
        let planet_context = env.planet_context;
        let mut errors: Vec<PlanetError> = Vec::new();
        let statement = self.compile(statement_text);
        if statement.is_err() {
            let errors = statement.unwrap_err();
            return Err(errors)
        }
        let _statement = statement.unwrap();
        let home_dir = planet_context.home_path.unwrap_or_default();
        let account_id = context.account_id.unwrap_or_default();
        let space_id = context.space_id.unwrap_or_default();
        let site_id = context.site_id;
        let result: Result<TreeFolder, PlanetError> = TreeFolder::defaults(
            space_database.connection_pool.clone(),
            Some(home_dir),
            Some(account_id),
            Some(space_id),
            site_id,
        );
        match result {
            Ok(_) => {
                let result = result.unwrap();
                let items = result.list();
                if items.is_err() {
                    let error = items.unwrap_err();
                    errors.push(error);
                    return Err(errors)
                }
                let items = items.unwrap();
                let mut items_mini: Vec<DbDataMini> = Vec::new();
                for item in items {
                    let item_mini = DbDataMini{
                        id: item.id,
                        slug: item.slug,
                        name: item.name,
                        routing: item.routing
                    };
                    items_mini.push(item_mini);
                }
                let response_coded = serde_yaml::to_string(&items_mini);
                if response_coded.is_err() {
                    let error = PlanetError::new(
                        500, 
                        Some(tr!("Error encoding statement response.")),
                    );
                    errors.push(error);
                    return Err(errors)
                }
                let response = response_coded.unwrap();
                let yaml_response = yaml_rust::YamlLoader::load_from_str(
                    response.as_str()
                ).unwrap();
                let yaml_response = yaml_response.clone();
                return Ok(yaml_response)
            }
            Err(error) => {
                errors.push(error);
                Err(errors)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct DescribeFolderStatement {
}

impl<'gb> StatementCompiler<'gb, String> for DescribeFolderStatement {

    fn compile(
        &self, 
        statement_text: &String
    ) -> Result<String, Vec<PlanetError>> {
        let expr = &RE_DESCRIBE_FOLDER;
        let check = expr.is_match(&statement_text);
        let mut errors: Vec<PlanetError> = Vec::new();
        if !check {
            let error = PlanetError::new(
                500, 
                Some(
                    tr!("Describe folder syntax not valid.")
                ),
            );
            errors.push(error);
            return Err(errors)
        }
        let captures = expr.captures(&statement_text);
        let folder_name: &str;
        if captures.is_some() {
            let captures = captures.unwrap();
            folder_name = captures.name("FolderName").unwrap().as_str();
            return Ok(folder_name.to_string().clone())
        } else {
            let error = PlanetError::new(
                500, 
                Some(
                    tr!("Could not parse folder name from describe folder statement.")
                ),
            );
            errors.push(error);
            return Err(errors)
        }
    }
}

impl<'gb> Statement<'gb> for DescribeFolderStatement {

    fn run(
        &self,
        env: &'gb Environment<'gb>,
        space_database: &SpaceDatabase,
        statement_text: &String,
    ) -> Result<Vec<yaml_rust::Yaml>, Vec<PlanetError>> {
        let space_database = space_database.clone();
        let context = env.context;
        let planet_context = env.planet_context;
        let mut errors: Vec<PlanetError> = Vec::new();
        let statement = self.compile(statement_text);
        if statement.is_err() {
            let errors = statement.unwrap_err();
            return Err(errors)
        }
        let folder_name = statement.unwrap();
        // get folder from data layer
        let home_dir = planet_context.home_path.unwrap_or_default();
        let account_id = context.account_id.unwrap_or_default();
        let space_id = context.space_id.unwrap_or_default();
        let site_id = context.site_id;
        let result: Result<TreeFolder, PlanetError> = TreeFolder::defaults(
            space_database.connection_pool.clone(),
            Some(home_dir),
            Some(account_id),
            Some(space_id),
            site_id,
        );
        match result {
            Ok(_) => {
                let result = result.unwrap();
                let folder = result.get_by_name(folder_name.as_str());
                if folder.is_ok() {
                    let folder = folder.unwrap();
                    if folder.is_some() {
                        let folder = folder.unwrap();
                        let response_coded = serde_yaml::to_string(&folder);
                        if response_coded.is_err() {
                            let error = PlanetError::new(
                                500, 
                                Some(tr!("Error encoding statement response.")),
                            );
                            errors.push(error);
                            return Err(errors)
                        }
                        let response = response_coded.unwrap();
                        let yaml_response = yaml_rust::YamlLoader::load_from_str(
                            response.as_str()
                        ).unwrap();
                        let yaml_response = yaml_response.clone();
                        return Ok(yaml_response)
                    } else {
                        // Folder could not be found
                        let error = PlanetError::new(
                            500, 
                            Some(
                                tr!("Folder \"{}\" not found.", &folder_name)
                            ),
                        );
                        errors.push(error);
                        return Err(errors)
                    }
                } else {
                    let error = folder.unwrap_err();
                    let mut errors: Vec<PlanetError> = Vec::new();
                    errors.push(error);
                    return Err(errors)
                }
            },
            Err(error) => {
                errors.push(error);
                Err(errors)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct DropFolderStatement {
}

impl<'gb> StatementCompiler<'gb, String> for DropFolderStatement {

    fn compile(
        &self, 
        statement_text: &String
    ) -> Result<String, Vec<PlanetError>> {
        let expr = &RE_DROP_FOLDER;
        let check = expr.is_match(&statement_text);
        let mut errors: Vec<PlanetError> = Vec::new();
        if !check {
            let error = PlanetError::new(
                500, 
                Some(
                    tr!("Drop folder syntax not valid.")
                ),
            );
            errors.push(error);
            return Err(errors)
        }
        let captures = expr.captures(&statement_text);
        let folder_name: &str;
        if captures.is_some() {
            let captures = captures.unwrap();
            folder_name = captures.name("FolderName").unwrap().as_str();
            return Ok(folder_name.to_string().clone())
        } else {
            let error = PlanetError::new(
                500, 
                Some(
                    tr!("Could not parse folder name from drop folder statement.")
                ),
            );
            errors.push(error);
            return Err(errors)
        }
    }
}

impl<'gb> Statement<'gb> for DropFolderStatement {

    fn run(
        &self,
        env: &'gb Environment<'gb>,
        space_database: &SpaceDatabase,
        statement_text: &String,
    ) -> Result<Vec<yaml_rust::Yaml>, Vec<PlanetError>> {
        let space_database = space_database.clone();
        let context = env.context;
        let planet_context = env.planet_context;
        let mut errors: Vec<PlanetError> = Vec::new();
        let statement = self.compile(statement_text);
        if statement.is_err() {
            let errors = statement.unwrap_err();
            return Err(errors)
        }
        let folder_name = statement.unwrap();
        // get folder from data layer
        let home_dir = planet_context.home_path.unwrap_or_default();
        let account_id = context.account_id.unwrap_or_default();
        let space_id = context.space_id.unwrap_or_default();
        let box_id = context.box_id.unwrap_or_default();
        let site_id = context.site_id;
        let mut site_id_alt: Option<String> = None;
        if site_id.is_some() {
            let site_id = site_id.unwrap();
            site_id_alt = Some(site_id.to_string());
        }
        let result: Result<TreeFolder, PlanetError> = TreeFolder::defaults(
            space_database.connection_pool.clone(),
            Some(home_dir),
            Some(account_id),
            Some(space_id),
            site_id,
        );
        match result {
            Ok(_) => {
                let db_folder = result.unwrap();
                let folder = db_folder.get_by_name(folder_name.as_str());
                if folder.is_ok() {
                    let folder = folder.unwrap();
                    if folder.is_some() {
                        let folder = folder.unwrap();
                        // Delete folder items
                        let folder_id = folder.id.clone().unwrap();
                        let result: Result<TreeFolderItem, PlanetError> = TreeFolderItem::defaults(
                            space_database.connection_pool.clone(),
                            home_dir,
                            account_id,
                            space_id,
                            site_id_alt,
                            box_id,
                            folder_id.as_str(),
                            &db_folder,
                        );
                        if result.is_err() {
                            let error = result.unwrap_err();
                            errors.push(error);
                            return Err(errors)
                        }
                        let mut db_folder_item = result.unwrap();
                        let result = db_folder_item.drop_trees();
                        if result.is_err() {
                            let error = result.unwrap_err();
                            errors.push(error);
                            return Err(errors)
                        }

                        // Delete from folders.db
                        let result = db_folder.delete(&folder_id);
                        if result.is_err() {
                            let error = result.unwrap_err();
                            errors.push(error);
                            return Err(errors)
                        }

                        // Delete files from OS dir "files"
                        let result = db_folder_item.drop_files();
                        if result.is_err() {
                            let error = result.unwrap_err();
                            errors.push(error);
                            return Err(errors)
                        }

                        // Build Output
                        let response_coded = serde_yaml::to_string(&folder);
                        if response_coded.is_err() {
                            let error = PlanetError::new(
                                500, 
                                Some(tr!("Error encoding statement response.")),
                            );
                            errors.push(error);
                            return Err(errors)
                        }
                        let response = response_coded.unwrap();
                        let yaml_response = yaml_rust::YamlLoader::load_from_str(
                            response.as_str()
                        ).unwrap();
                        let yaml_response = yaml_response.clone();
                        return Ok(yaml_response)
                    } else {
                        let error = PlanetError::new(
                            500, 
                            Some(
                                tr!("Folder \"{}\" not found.", &folder_name)
                            ),
                        );
                        errors.push(error);
                        return Err(errors)
                    }
                } else {
                    let error = folder.unwrap_err();
                    let mut errors: Vec<PlanetError> = Vec::new();
                    errors.push(error);
                    return Err(errors)
                }
            },
            Err(error) => {
                errors.push(error);
                Err(errors)
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ColumnCompiledStmt {
    pub folder_name: String,
    pub language: Option<LanguageConfig>,
    pub text_search: Option<TextSearchConfig>,
    pub name: Option<ColumnConfig>,
    pub columns: Option<Vec<ColumnConfig>>,
    pub sub_folders: Option<Vec<SubFolderConfig>>,
}
impl ColumnCompiledStmt {
    pub fn defaults(folder_name: &String) -> Self {
        let obj = Self{
            folder_name: folder_name.clone(),
            language: None,
            text_search: None,
            name: None,
            columns: None,
            sub_folders: None
        };
        return obj
    }
}

#[derive(Debug, Clone)]
pub struct AddColumnStatement {
}

impl<'gb> StatementCompiler<'gb, ColumnCompiledStmt> for AddColumnStatement {

    fn compile(
        &self, 
        statement_text: &String
    ) -> Result<ColumnCompiledStmt, Vec<PlanetError>> {
        let expr = &RE_ADD_COLUMN;
        let check = expr.is_match(&statement_text);
        let mut errors: Vec<PlanetError> = Vec::new();
        if !check {
            let error = PlanetError::new(
                500, 
                Some(
                    tr!("Add column syntax not valid.")
                ),
            );
            errors.push(error);
            return Err(errors)
        }
        let captures = expr.captures(&statement_text);
        let config: &str;
        let folder_name: &str;
        if captures.is_none() {
            let error = PlanetError::new(
                500, 
                Some(
                    tr!("Could not parse add column statement.")
                ),
            );
            errors.push(error);
            return Err(errors)
        }
        let captures = captures.unwrap();
        folder_name = captures.name("FolderName").unwrap().as_str();
        config = captures.name("Config").unwrap().as_str();
        let expr = &RE_ADD_COLUMN_CONFIG;
        let item = expr.captures(&config);
        if item.is_some() {
            let item = item.unwrap();
            let column = item.name("Column");
            let column_type = item.name("ColumnType");
            let column_str = column.unwrap().as_str();
            let column_type = column_type.unwrap().as_str();
            let result = process_column(column_str, column_type, &item);
            if result.is_ok() {
                let column = result.unwrap();
                let mut compiled = ColumnCompiledStmt::defaults(
                    &folder_name.to_string()
                );
                let mut columns: Vec<ColumnConfig> = Vec::new();
                columns.push(column);
                compiled.columns = Some(columns);
                return Ok(compiled)
            } else {
                let error = PlanetError::new(
                    500, 
                    Some(
                        tr!("Could not parse config attributes for add column statement.")
                    ),
                );
                errors.push(error);
                return Err(errors)
            }
        } else {
            let error = PlanetError::new(
                500, 
                Some(
                    tr!("Could not parse config detail for add column statement.")
                ),
            );
            errors.push(error);
            return Err(errors)
        }
    }
}

impl<'gb> Statement<'gb> for AddColumnStatement {

    fn run(
        &self,
        env: &'gb Environment<'gb>,
        space_database: &SpaceDatabase,
        statement_text: &String,
    ) -> Result<Vec<yaml_rust::Yaml>, Vec<PlanetError>> {
        let space_database = space_database.clone();
        let context = env.context;
        let planet_context = env.planet_context;
        let statement = self.compile(statement_text);
        if statement.is_err() {
            let errors = statement.unwrap_err();
            return Err(errors)
        }
        let mut errors: Vec<PlanetError> = Vec::new();
        let column_compiled = statement.unwrap();
        let home_dir = planet_context.home_path.unwrap_or_default();
        let account_id = context.account_id.unwrap_or_default();
        let space_id = context.space_id.unwrap_or_default();
        let site_id = context.site_id;
        let result: Result<TreeFolder, PlanetError> = TreeFolder::defaults(
            space_database.connection_pool.clone(),
            Some(home_dir),
            Some(account_id),
            Some(space_id),
            site_id,
        );
        if result.is_ok() {
            let db_folder = result.unwrap();
            let folder_name = column_compiled.folder_name;
            let folder = db_folder.get_by_name(folder_name.as_str());
            if folder.is_ok() {
                let folder = folder.unwrap();
                if folder.is_some() {
                    let mut folder = folder.unwrap();
                    let data = folder.data;
                    if data.is_some() {
                        let mut data = data.unwrap();
                        let column_list = data.get(COLUMNS);
                        if column_list.is_some() {
                            let mut column_list = column_list.unwrap().clone();
                            let column = column_compiled.columns.unwrap()[0].clone();
                            let mut columns_map: HashMap<String, ColumnConfig> = HashMap::new();
                            let column_name = column.clone().name.unwrap_or_default();
                            columns_map.insert(column_name, column.clone());
                            let map = &column.create_config(
                                planet_context,
                                context,
                                &columns_map,
                                &db_folder,
                                &folder_name,
                                &space_database
                            );
                            if map.is_err() {
                                let error = map.clone().unwrap_err();
                                errors.push(error);
                            }
                            let map = map.clone().unwrap();
                            column_list.push(map);
                            let map_list = &column.map_collections_db();
                            if map_list.is_err() {
                                let error = map_list.clone().unwrap_err();
                                errors.push(error);
                            }
                            let map_list = map_list.clone().unwrap();
                            let map_list = map_list.clone();
                            data.extend(map_list);
                            data.insert(COLUMNS.to_string(), column_list);
                            // Update folder config
                            folder.data = Some(data);
                            let result = db_folder.update(&folder);
                            // Build output
                            if result.is_ok() {
                                let folder = result.unwrap();
                                let response_coded = serde_yaml::to_string(&folder);
                                if response_coded.is_err() {
                                    let error = PlanetError::new(
                                        500, 
                                        Some(tr!("Error encoding statement response.")),
                                    );
                                    errors.push(error);
                                }
                                let response = response_coded.unwrap();
                                let yaml_response = yaml_rust::YamlLoader::load_from_str(
                                    response.as_str()
                                ).unwrap();
                                let yaml_response = yaml_response.clone();
                                return Ok(yaml_response)
                            } else {
                                let error = PlanetError::new(
                                    500, 
                                    Some(
                                        tr!("Could not update folder on database.")
                                    ),
                                );
                                errors.push(error);
                            }    
                        }
                    } else {
                        let error = PlanetError::new(
                            500, 
                            Some(
                                tr!("Folder has no data.")
                            ),
                        );
                        errors.push(error);
                    }
                } else {
                    let error = PlanetError::new(
                        500, 
                        Some(
                            tr!("Folder not found.")
                        ),
                    );
                    errors.push(error);
                    return Err(errors)
                }
            } else {
                let error = PlanetError::new(
                    500, 
                    Some(
                        tr!("Error fetching folder by name.")
                    ),
                );
                errors.push(error);
            }
        } else {
            let error = PlanetError::new(
                500, 
                Some(
                    tr!("Could not parse add column statement.")
                ),
            );
            errors.push(error);
        }
        return Err(errors)
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

pub fn resolve_schema_statement(
    env: &Environment,
    space_data: &SpaceDatabase,
    statement_text: &String, 
    response_wrap: Option<Result<Vec<yaml_rust::Yaml>, Vec<PlanetError>>>,
    column_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
    mode: &StatementCallMode
) -> Option<Result<Vec<yaml_rust::Yaml>, Vec<PlanetError>>> {
    let mode = mode.clone();
    let response_wrap = response_wrap.clone();
    if response_wrap.is_some() {
        let response = response_wrap.unwrap();
        return Some(response)
    }
    let column_map = column_map.clone();
    let env = env.clone();
    let statement_text = substitute_variables(statement_text, &env, column_map.clone());
    // CREATE FOLDER
    let expr = &RE_CREATE_FOLDER_MAIN;
    let check = expr.is_match(&statement_text);
    if check {
        let stmt = CreateFolderStatement{};
        match mode {
            StatementCallMode::Run => {
                let response = stmt.run(
                    &env, 
                    &space_data, 
                    &statement_text,
                );
                return Some(response);
            },
            StatementCallMode::Compile => {
                let response = stmt.compile(&statement_text);
                if response.is_err() {
                    let errors = response.unwrap_err();
                    return Some(Err(errors))
                }
                let result = yaml_rust::YamlLoader::load_from_str("---\nstatus: ok");
                return Some(Ok(result.unwrap()))
            }
        }
    }
    // LIST FOLDERS
    let expr = &RE_LIST_FOLDERS;
    let check = expr.is_match(&statement_text);
    if check {
        let stmt = ListFoldersStatement{};
        match mode {
            StatementCallMode::Run => {
                let response = stmt.run(
                    &env, 
                    &space_data, 
                    &statement_text,
                );
                return Some(response);
            },
            StatementCallMode::Compile => {
                let response = stmt.compile(&statement_text);
                if response.is_err() {
                    let errors = response.unwrap_err();
                    return Some(Err(errors))
                }
                let result = yaml_rust::YamlLoader::load_from_str("---\nstatus: ok");
                return Some(Ok(result.unwrap()))
            }
        }
    }
    // DESCRIBE FOLDER
    let expr = &RE_DESCRIBE_FOLDER;
    let check = expr.is_match(&statement_text);
    if check {
        let stmt = DescribeFolderStatement{};
        match mode {
            StatementCallMode::Run => {
                let response = stmt.run(
                    &env, 
                    &space_data, 
                    &statement_text,
                );
                return Some(response);
            },
            StatementCallMode::Compile => {
                let response = stmt.compile(&statement_text);
                if response.is_err() {
                    let errors = response.unwrap_err();
                    return Some(Err(errors))
                }
                let result = yaml_rust::YamlLoader::load_from_str("---\nstatus: ok");
                return Some(Ok(result.unwrap()))
            }
        }
    }
    // DROP FOLDER
    let expr = &RE_DROP_FOLDER;
    let check = expr.is_match(&statement_text);
    if check {
        let stmt = DropFolderStatement{};
        match mode {
            StatementCallMode::Run => {
                let response = stmt.run(
                    &env, 
                    &space_data, 
                    &statement_text,
                );
                return Some(response);
            },
            StatementCallMode::Compile => {
                let response = stmt.compile(&statement_text);
                if response.is_err() {
                    let errors = response.unwrap_err();
                    return Some(Err(errors))
                }
                let result = yaml_rust::YamlLoader::load_from_str("---\nstatus: ok");
                return Some(Ok(result.unwrap()))
            }
        }
    }
    // ADD COLUMN
    let expr = &RE_ADD_COLUMN;
    let check = expr.is_match(&statement_text);
    if check {
        let stmt = AddColumnStatement{};
        match mode {
            StatementCallMode::Run => {
                let response = stmt.run(
                    &env, 
                    &space_data, 
                    &statement_text,
                );
                return Some(response);
            },
            StatementCallMode::Compile => {
                let response = stmt.compile(&statement_text);
                if response.is_err() {
                    let errors = response.unwrap_err();
                    return Some(Err(errors))
                }
                let result = yaml_rust::YamlLoader::load_from_str("---\nstatus: ok");
                return Some(Ok(result.unwrap()))
            }
        }
    }
    // MODIFY COLUMN
    // MODIFY LANGUAGE
    // ADD SUBFOLDER
    // REMOVE SUBFOLDER
    // MODIFY SUBFOLDER
    // MODIFY SEARCH RELEVANCE
    // DROP COLUMN
    return None
}
