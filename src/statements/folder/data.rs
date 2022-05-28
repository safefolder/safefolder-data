extern crate tr;
extern crate colored;
extern crate slug;

use std::collections::{BTreeMap, HashMap};
use std::time::Instant;
use std::cmp::Ordering;

use tr::tr;
use regex::Regex;
use lazy_static::lazy_static;
use colored::Colorize;

use serde_encrypt::{
    shared_key::SharedKey, traits::SerdeEncryptSharedKey,
    AsSharedKey, EncryptedMessage,
};

use crate::functions::Formula;
use crate::statements::folder::config::*;
use crate::storage::constants::*;
use crate::statements::folder::schema::*;
use crate::statements::*;
use crate::statements::constants::*;
use crate::planet::constants::{ID, NAME, VALUE, FALSE, COLUMNS};
// use crate::storage::folder::{TreeFolder, TreeFolderItem, FolderItem, FolderSchema, DbData, GetItemOption};
use crate::storage::folder::*;
use crate::storage::{ConfigStorageColumn, generate_id};
use crate::storage::space::{SpaceDatabase};
use crate::planet::{
    PlanetContext, 
    PlanetError,
    Context,
    Environment,
};
use crate::storage::columns::{text::*, StorageColumn, ObjectStorageColumn, EnvDbStorageColumn};
use crate::storage::columns::number::*;
use crate::storage::columns::date::*;
use crate::storage::columns::formula::*;
use crate::storage::columns::reference::*;
use crate::storage::columns::structure::*;
use crate::storage::columns::processing::*;
use crate::storage::columns::media::*;
// use crate::statements::constants::{COLUMN_ID};
use crate::functions::{RE_FORMULA_QUERY, RE_FORMULA_ASSIGN, execute_formula};

lazy_static! {
    pub static ref RE_INSERT_INTO_FOLDER_MAIN: Regex = Regex::new(r#"INSERT INTO FOLDER (?P<FolderName>[\w\s]+)[\s\t\n]*(?P<Items>\([\s\S]+\));"#).unwrap();
    pub static ref RE_INSERT_INTO_FOLDER_ITEMS: Regex = Regex::new(r#"(?P<Item>\([\s\S][^)]+\)),*"#).unwrap();
    pub static ref RE_INSERT_INTO_FOLDER_ITEM_KEYS: Regex = Regex::new(r#"([\s\t]*(?P<Key>[\w\s]+)=[\s\t]*(?P<Value>[\s\S][^,\n)]*),*)"#).unwrap();
    pub static ref RE_INSERT_INTO_FOLDER_SUBFOLDERS: Regex = Regex::new(r#"(SUB FOLDER (?P<SubFolderId>[\w]+)([\s]*WITH[\s]*(?P<SubFolderIsReference>IsReference[\s]*=[\s]*(true|false)))*,*)"#).unwrap();
    pub static ref RE_SELECT: Regex = Regex::new(r#"SELECT[\s]*[\s\S]*[\s]*FROM[\s]*[\s\S]*;"#).unwrap();
    pub static ref RE_SELECT_COUNT: Regex = Regex::new(r#"SELECT[\s]*((?P<CountAll>COUNT\(\*\))|(COUNT\(DISTINCT[\s]+(?P<CountColumnDis>[\w\s]+)\))|(COUNT\((?P<CountColumn>[\w\s]+)\)))[\s]*FROM[\s]*(?P<FolderName>[\w\s]+);"#).unwrap();
    pub static ref RE_SELECT_PAGING: Regex = Regex::new(r#"([\s]*PAGE[\s]*(?P<Page>[\d]+))*([\s]*NUMBER ITEMS[\s]*(?P<NumberItems>[\d]+))*"#).unwrap();
    pub static ref RE_SELECT_FROM: Regex = Regex::new(r#"FROM[\s]*"(?P<FolderName>[\w\s]*)"[\s]*(WHERE|SORT BY|GROUP BY|SEARCH)*"#).unwrap();
    pub static ref RE_SELECT_COLUMNS: Regex = Regex::new(r#"SELECT[\s]*((?P<AllColumns>\*)|(?P<Columns>[\w\s,]+))[\s]*FROM"#).unwrap();
    pub static ref RE_SELECT_SORTING: Regex = Regex::new(r#"(SORT[\s]*BY[\s]*\{(?P<SortBy>[|\w\s]+)\})"#).unwrap();
    pub static ref RE_SELECT_SORT_FIELDS: Regex = Regex::new(r#"(?P<Column>[\w\s]+)(?P<Mode>ASC|DESC)*"#).unwrap();
    pub static ref RE_SELECT_GROUP_BY: Regex = Regex::new(r#"(GROUP[\s]*BY[\s]*"(?P<GroupByColumns>[\w\s,]+)")"#).unwrap();
    pub static ref RE_SELECT_GROUP_COLUMNS: Regex = Regex::new(r#"(?P<Column>[\w\s]+)"#).unwrap();
    pub static ref RE_SELECT_SEARCH: Regex = Regex::new(r#"(SEARCH[\s]*"(?P<Search>[\w\s]+)")"#).unwrap();
    pub static ref RE_SELECT_WHERE: Regex = Regex::new(r#"WHERE[\s]*(?P<Where>[\s\S]+);+"#).unwrap();
}

pub const WITH_IS_REFERENCE: &str = "IsReference";

pub const ALLOWED_WITH_OPTIONS_WRITE: [&str; 1] = [
    WITH_IS_REFERENCE
];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InsertIntoFolderCompiledStmt {
    pub folder_name: String,
    pub name: Option<String>,
    pub sub_folders: Option<Vec<SubFolderDataConfig>>,
    pub data: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
}

impl InsertIntoFolderCompiledStmt {

    pub fn defaults(name: Option<String>) -> InsertIntoFolderCompiledStmt {
        let config: InsertIntoFolderCompiledStmt = InsertIntoFolderCompiledStmt{
            folder_name: String::from(""),
            name: name,
            data: Some(BTreeMap::new()),
            sub_folders: None,
        };
        return config
    }

}


#[derive(Debug, Clone)]
pub struct InsertIntoFolderStatement {
}

impl<'gb> StatementCompilerBulk<'gb, InsertIntoFolderCompiledStmt> for InsertIntoFolderStatement {

    fn compile(
        &self, 
        statement_text: &String
    ) -> Result<Vec<InsertIntoFolderCompiledStmt>, Vec<PlanetError>> {
        let mut statements: Vec<InsertIntoFolderCompiledStmt> = Vec::new();
        let expr = &RE_INSERT_INTO_FOLDER_MAIN;
        let check = expr.is_match(&statement_text);
        let mut errors: Vec<PlanetError> = Vec::new();
        if !check {
            let error = PlanetError::new(
                500, 
                Some(
                    tr!("Insert into folder syntax not valid.")
                ),
            );
            errors.push(error);
            return Err(errors)
        }
        let captures = expr.captures(&statement_text);
        if captures.is_some() {
            let captures = captures.unwrap();
            let folder_name = captures.name("FolderName").unwrap().as_str();
            let folder_name = folder_name.replace("\n", "");
            let folder_name = folder_name.trim();
            let items = captures.name("Items");
            if items.is_some() {
                let items = items.unwrap().as_str();
                // I parse for long texts and possible text characters like commas, quotes, etc...
                let long_text = DataValueLongText::defaults(
                    &items.to_string()
                );
                if long_text.is_ok() {
                    let long_text = long_text.unwrap();
                    let items = long_text.parsed_text.as_str();
                    eprintln!("InsertIntoFolder.compile :: items: {}", items);
                    let expr = &RE_INSERT_INTO_FOLDER_ITEMS;
                    let item_list = expr.captures_iter(items);
                    for item_ in item_list {
                        let mut compiled_statement = InsertIntoFolderCompiledStmt::defaults(None);
                        compiled_statement.folder_name = folder_name.to_string();
                        let expr_item = item_.name("Item");
                        if expr_item.is_some() {
                            let expr_item = expr_item.unwrap().as_str();
                            eprintln!("InsertIntoFolder.compile :: item: {}", expr_item);
                            let expr = &RE_INSERT_INTO_FOLDER_ITEM_KEYS;
                            let list = expr.captures_iter(expr_item);
                            let mut map: BTreeMap<String, Vec<BTreeMap<String, String>>> = BTreeMap::new();
                            for list_item in list {
                                let key = list_item.name("Key");
                                let value = list_item.name("Value");
                                // eprintln!("InsertIntoFolderStatement.compile :: value: {:?}", &value);
                                if key.is_some() && value.is_some() {
                                    let key = key.unwrap().as_str().trim();
                                    let mut value = value.unwrap().as_str().to_string();
                                    // eprintln!("InsertIntoFolderStatement.compile :: [str] value: {}", &value);
                                    if key == NAME_CAMEL {
                                        // eprintln!("InsertIntoFolderStatement.compile :: name: {}", &value);
                                        compiled_statement.name = Some(value);
                                        continue;
                                    }
                                    if DataValueLongText::has_placeholder(&value) {
                                        let value_ = value.clone();
                                        let value_ = value_.replace("\n", "");
                                        // eprintln!("InsertIntoFolderStatement.compile :: value_: {}", &value_);
                                        let long_text_src = long_text.map.get(&value_);
                                        if long_text_src.is_some() {
                                            let long_text_src = long_text_src.unwrap();
                                            // eprintln!("InsertIntoFolderStatement.compile :: long text: {}", 
                                            //     long_text_src
                                            // );
                                            value = long_text_src.clone();
                                        }
                                    }
                                    // Here value is text with no placeholders in case of long text """..."""
                                    // eprintln!("InsertIntoFolderStatement.compile :: pre DataValue: {}", &value);
                                    let data_value = DataValue::defaults(
                                        &value
                                    );
                                    if data_value.is_ok() {
                                        let data_value = data_value.unwrap();
                                        // eprintln!("InsertIntoFolderStatement.compile :: DataValue: {:?}", &data_value.value);
                                        map.insert(key.to_string(), data_value.value);
                                    }
                                }
                            }
                            compiled_statement.data = Some(map);
                            let expr = &RE_INSERT_INTO_FOLDER_SUBFOLDERS;
                            let sub_folders = expr.captures_iter(expr_item);
                            let mut sub_folder_list: Vec<SubFolderDataConfig> = Vec::new();
                            for sub_folder in sub_folders {
                                let sub_folder_id = sub_folder.name("SubFolderId");
                                let is_reference = sub_folder.name("SubFolderIsReference");
                                let mut obj = SubFolderDataConfig{
                                    id: None,
                                    is_reference: None
                                };
                                if sub_folder_id.is_some() {
                                    let sub_folder_id = sub_folder_id.unwrap().as_str();
                                    obj.id = Some(sub_folder_id.to_string());
                                } else {
                                    continue
                                }
                                if is_reference.is_some() {
                                    let is_reference = is_reference.unwrap().as_str();
                                    let is_reference = is_reference.to_lowercase();
                                    let is_reference = is_reference.as_str();
                                    match is_reference {
                                        TRUE => {
                                            obj.is_reference = Some(true);
                                        },
                                        FALSE => {
                                            obj.is_reference = Some(false);
                                        },
                                        _ => {}
                                    }
                                }
                                sub_folder_list.push(obj);
                            }
                            if sub_folder_list.len() > 0 {
                                compiled_statement.sub_folders = Some(sub_folder_list);
                            }
                            statements.push(compiled_statement)
                        }
                    }
                }
            }
        }
        eprintln!("InsertIntoFolderStatement.compile :: statements: {:#?}", &statements);
        return Ok(statements)
    }
}

impl<'gb> InsertIntoFolderStatement {

    fn get_insert_id_data_map(
        &self,
        insert_data_map: &BTreeMap<String, Vec<BTreeMap<String, String>>>,
        folder_data: &BTreeMap<String, Vec<BTreeMap<String, String>>>,
    ) -> BTreeMap<String, Vec<BTreeMap<String, String>>> {
        let mut insert_id_data_map: BTreeMap<String, Vec<BTreeMap<String, String>>> = BTreeMap::new();
        let mut folder_map_by_name: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
        let columns = folder_data.get(COLUMNS);
        if columns.is_some() {
            let columns = columns.unwrap();
            for column in columns {
                let column_name = column.get(NAME).unwrap();
                folder_map_by_name.insert(column_name.clone(), column.clone());
            }
        }
        // eprintln!("InsertIntoFolder.get_insert_id_data_map :: folder_map_by_name: {:#?}", &folder_map_by_name);
        for (name, value) in insert_data_map.clone() {
            let map = folder_map_by_name.get(&name);
            if map.is_some() {
                let map = map.unwrap();
                let id = map.get(ID).unwrap().clone();
                // let column_type = map.get(COLUMN_TYPE);
                // let column_type = column_type.unwrap().clone();
                insert_id_data_map.insert(id, value);
            } else {
                // Append error column name is not configured in the configuration.
            }
        }
        // eprintln!("InsertIntoFolder.get_insert_id_data_map :: insert_id_data_map: {:#?}", &insert_id_data_map);
        return insert_id_data_map
    }

    pub fn check_name_exists(&self, folder_name: &String, name: &String, db_row: &mut TreeFolderItem) -> bool {
        let check: bool;
        let name = name.clone();
        let result = db_row.get(&folder_name, GetItemOption::ByName(name), None);
        eprintln!("InsertIntoFolder.check_name_exists :: get response: {:#?}", &result);
        match result {
            Ok(_) => {
                check = true
            },
            Err(_) => {
                check = false
            }
        }
        return check
    }

}

impl<'gb> Statement<'gb> for InsertIntoFolderStatement {

    fn run(
        &self,
        env: &'gb Environment<'gb>,
        space_database: &SpaceDatabase,
        statement_text: &String,
    ) -> Result<Vec<yaml_rust::Yaml>, Vec<PlanetError>> {
        let space_database = space_database.clone();
        let context = env.context;
        let planet_context = env.planet_context;
        let env = Environment{
            context: context,
            planet_context: planet_context
        };
        let t_1 = Instant::now();
        let statements = self.compile(statement_text);
        if statements.is_err() {
            let errors = statements.unwrap_err();
            return Err(errors)
        }
        let statements = statements.unwrap();
        let folder_name = &statements[0].folder_name;
        eprintln!("InsertIntoFolderStatement.run :: folder_name: {}", folder_name);

        let mut errors: Vec<PlanetError> = Vec::new();
        // folder
        let home_dir = planet_context.home_path.clone();
        let account_id = context.account_id.clone().unwrap_or_default();
        let space_id = context.space_id;
        let site_id = context.site_id.clone();
        let space_database = space_database.clone();
        let db_folder= TreeFolder::defaults(
            space_database.connection_pool.clone(),
            Some(home_dir.clone().unwrap_or_default().as_str()),
            Some(&account_id),
            Some(space_id),
            site_id.clone(),
        ).unwrap();
        let folder = db_folder.get_by_name(folder_name);
        if folder.is_err() {
            let error = folder.unwrap_err();
            errors.push(error);
            return Err(errors);
        }
        let folder = folder.unwrap();
        if *&folder.is_none() {
            errors.push(
                PlanetError::new(
                    500, 
                    Some(tr!("Could not find folder {}", &folder_name)),
                )
            );
            return Err(errors);
        }

        let folder = folder.unwrap();
        let folder_name = &folder.clone().name.unwrap();
        eprintln!("InsertIntoFolder.run :: folder: {:#?}", &folder);
        // eprintln!("InsertIntoFolder.run :: Got folder! folder_name: {}", folder_name);
        let folder_id = folder.clone().id.unwrap();
        let mut site_id_alt: Option<String> = None;
        if site_id.is_some() {
            let site_id = site_id.clone().unwrap();
            site_id_alt = Some(site_id.clone().to_string());
        }

        let result: Result<TreeFolderItem, PlanetError> = TreeFolderItem::defaults(
            space_database.connection_pool.clone(),
            home_dir.clone().unwrap_or_default().as_str(),
            &account_id,
            space_id,
            site_id_alt,
            folder_id.as_str(),
            &db_folder,
        );
        match result {
            Ok(_) => {
                let mut db_row: TreeFolderItem = result.unwrap();

                // routing
                let routing_wrap = RoutingData::defaults(
                    Some(account_id.to_string()),
                    site_id.clone(), 
                    &space_id, 
                    None
                );
                
                // eprintln!("InsertIntoFolder.run :: folder: {:#?}", &folder);

                // I need a way to get list of instance ColumnConfig (columns)
                let config_columns = ColumnConfig::get_config(
                    planet_context,
                    context,
                    &folder
                );
                if config_columns.is_err() {
                    let error = config_columns.unwrap_err();
                    errors.push(error);
                    return Err(errors);
                }
                let config_columns = config_columns.unwrap();
                // eprintln!("InsertIntoFolder.run :: config_columns: {:#?}", &config_columns);

                let mut db_data_list: Vec<DbData> = Vec::new();
                let mut links_map_map: HashMap<String, HashMap<String, Vec<ColumnConfig>>> = HashMap::new();
                let mut links_data_map_map: HashMap<String, HashMap<String, HashMap<String, Vec<String>>>> = HashMap::new();
                for statement in statements {
                    // I need to create the list of DbData
                    let insert_data_map= statement.data.clone().unwrap();
                    // I need to have {id} -> Value
                    let folder_data = folder.clone().data.unwrap();
    
                    // Validate sub_folder id exists in config for the folder and attach to DbData
                    let sub_folders_config = statement.clone().sub_folders;
                    let mut sub_folders_wrap: Option<Vec<SubFolderItem>> = None;
                    if sub_folders_config.is_some() {
                        let sub_folders_config = sub_folders_config.unwrap();
                        let mut sub_folders: Vec<SubFolderItem> = Vec::new();
                        for item in sub_folders_config {
                            let item_id = item.id.unwrap();
                            let check = TreeFolder::has_sub_folder_id(
                                &folder.clone(), 
                                &item_id
                            );
                            eprintln!("InsertIntoFolder.run :: item_id: {} check: {}", &item_id, &check);
                            if check {
                                let sub_folder = SubFolderItem{
                                    id: Some(item_id),
                                    name: None,
                                    is_reference: item.is_reference,
                                    data: None,
                                };
                                sub_folders.push(sub_folder);
                            } else {
                                errors.push(
                                    PlanetError::new(
                                        500, 
                                        Some(tr!(
                                            "Sub folder id \"{}\" does not exist in folder.", &item_id
                                        )),
                                    )
                                );
                            }
                        }
                        if sub_folders.len() > 0 {
                            sub_folders_wrap = Some(sub_folders);
                        }
                    }
    
                    // get id => value for data, data_objects and data_collections
                    let insert_id_data_map = self.get_insert_id_data_map(
                        &insert_data_map, &folder_data
                    );
                    
                    // process insert config data_collections
                    // User authentication
                    // TODO: Complete when implement the permission system exchange token by user_id
                    let user_id_string = generate_id().unwrap();
                    let mut user_id: Vec<String> = Vec::new();
                    user_id.push(user_id_string);
                    
                    // let insert_data_collections_map = self.config.data_collections.clone().unwrap();
                    // eprintln!("InsertIntoFolder.run :: insert_data__collections_map: {:#?}", &insert_data_collections_map);
                    // TODO: Change for the item name
                    // We will use this when we have the Name column, which is required in all tables
                    // eprintln!("InsertIntoFolder.run :: routing_wrap: {:#?}", &routing_wrap);
    
                    // Keep in mind on name attribute for DbData
                    // 1. Can be small text or any other column, so we need to do validation and generation of data...
                    // 2. Becaouse if formula is generated from other columns, is generated number or id is also generated
                    // I also need a set of columns allowed to be name (Small Text, Formula), but this in future....
                    // name on YAML not required, since can be generated
                    // Check column type and attribute to validate
                    // So far only take Small Text
                    let name_column: ColumnConfig = ColumnConfig::get_name_column(&folder).unwrap();
                    let name_column_type = name_column.column_type.unwrap().clone();
                    let insert_name = statement.name.clone();
                    // Only support so far Small Text and needs to be informed in YAML with name
                    if name_column_type != COLUMN_TYPE_SMALL_TEXT.to_string() || insert_name.is_none() {
                        errors.push(
                            PlanetError::new(
                                500, 
                                Some(tr!("You need to include name column when inserting data into database.
                                 Only \"Small Text\" supported so far")),
                            )
                        );
                    }
                    let name = insert_name.unwrap();
                    // Check name does not exist
                    // eprintln!("InsertIntoFolder.run :: name: {}", &name);
                    let name_exists = self.check_name_exists(&folder_name, &name, &mut db_row);
                    // eprintln!("InsertIntoFolder.run :: record name_exists: {}", &name_exists);
                    if name_exists {
                        errors.push(
                            PlanetError::new(
                                500, 
                                Some(tr!("A record with name \"{}\" already exists in database", &name)),
                            )
                        );
                    }
    
                    // Instantiate DbData and validate
                    let mut db_context: BTreeMap<String, String> = BTreeMap::new();
                    db_context.insert(FOLDER_NAME.to_string(), folder_name.clone());
                    let db_data = DbData::defaults(
                        &name,
                        None,
                        None,
                        routing_wrap.clone(),
                        None,
                        sub_folders_wrap,
                    );
                    if db_data.is_err() {
                        let error = db_data.unwrap_err();
                        errors.push(error);
                        return Err(errors)
                    }
                    let mut db_data = db_data.unwrap();
                    let item_id = db_data.id.clone().unwrap_or_default();
                    let mut data: BTreeMap<String, Vec<BTreeMap<String, String>>> = BTreeMap::new();
                    let mut column_config_map: BTreeMap<String, ColumnConfig> = BTreeMap::new();
                    for column in config_columns.clone() {
                        let column_name = column.name.clone().unwrap();
                        column_config_map.insert(column_name, column.clone());
                    }
                    let mut links_map: HashMap<String, Vec<ColumnConfig>> = HashMap::new();
                    let mut links_data_map: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();
                    // eprintln!("InsertIntoFolder.run :: insert_id_data_map: {:#?}", &insert_id_data_map);
                    for column in config_columns.clone() {
                        let mut column_data: Option<Vec<String>> = None;
                        let column_config = column.clone();
                        let column_type = column.column_type.unwrap_or_default();
                        let column_type = column_type.as_str();
                        let mut is_set: String = FALSE.to_string();
                        let is_set_wrap = column_config.clone().is_set;
                        if is_set_wrap.is_some() {
                            is_set = is_set_wrap.unwrap();
                        }                    
                        let column_name = column.name.unwrap();
                        eprintln!("InsertIntoFolder.run :: [{}] column_type: {} is_set: {}", &column_name, column_type, &is_set);
                        // I always have a column id
                        let column_id = column.id.unwrap_or_default();
                        
                        let data_item = insert_id_data_map.get(&column_id);
                        if data_item.is_some() {
                            let data_item = data_item.unwrap().clone();
                            let data_item = &data_item;
                            let mut my_list: Vec<String> = Vec::new();
                            for item in data_item {
                                let value = item.get(VALUE);
                                if value.is_some() {
                                    let value = value.unwrap();
                                    my_list.push(value.clone());
                                }
                            }
                            column_data = Some(my_list);
                        }
                        // eprintln!("InsertIntoFolder.run :: column_data: {:?}", &column_data);
                        // In case we don't have any value and is system generated we skip
                        let required_wrap = &column_config.required;
                        let required: bool;
                        if required_wrap.is_some() {
                            required = required_wrap.unwrap();
                        } else {
                            required = false;
                        }
                        // eprintln!("InsertIntoFolder.run :: column_data: {:?} required: {}", &column_data, &required);
                        if required && column_data.is_none() {
                            let error = PlanetError::new(
                                500, 
                                Some(tr!(
                                    "Field {}{}{} is required", 
                                    String::from("\"").blue(), &column_name.blue(), String::from("\"").blue()
                                )),
                            );
                            errors.push(error);
                            continue
                        }
                        if column_data.is_none() &&
                            (
                                column_type != COLUMN_TYPE_FORMULA && 
                                column_type != COLUMN_TYPE_CREATED_TIME && 
                                column_type != COLUMN_TYPE_LAST_MODIFIED_TIME && 
                                column_type != COLUMN_TYPE_GENERATE_ID && 
                                column_type != COLUMN_TYPE_GENERATE_NUMBER
                            ) {
                            continue
                        }
                        let column_data_: Vec<String>;
                        if column_data.is_some() {
                            column_data_ = column_data.clone().unwrap().clone();
                        } else {
                            let mut items = Vec::new();
                            items.push(String::from(""));
                            column_data_ = items;
                        }
                        let column_data = column_data_;
                        let mut column_data_wrap: Result<Vec<String>, Vec<PlanetError>> = Ok(Vec::new());
                        let mut skip_data_assign = false;
                        match column_type {
                            COLUMN_TYPE_SMALL_TEXT => {
                                let obj = SmallTextColumn::defaults(&column_config);
                                column_data_wrap = obj.validate(&column_data);
                            },
                            COLUMN_TYPE_LONG_TEXT => {
                                let obj = LongTextColumn::defaults(&column_config);
                                column_data_wrap = obj.validate(&column_data);
                            },
                            COLUMN_TYPE_CHECKBOX => {
                                let obj = CheckBoxColumn::defaults(&column_config);
                                column_data_wrap = obj.validate(&column_data);
                            },
                            COLUMN_TYPE_NUMBER => {
                                let obj = NumberColumn::defaults(&column_config);
                                column_data_wrap = obj.validate(&column_data);
                            },
                            COLUMN_TYPE_SELECT => {
                                let obj = SelectColumn::defaults(
                                    &column_config, 
                                    Some(&folder)
                                );
                                column_data_wrap = obj.validate(&column_data);
                            },
                            COLUMN_TYPE_FORMULA => {
                                let obj = FormulaColumn::defaults(&column_config);
                                column_data_wrap = obj.validate(&data, &column_config_map);
                            },
                            COLUMN_TYPE_DATE => {
                                let obj = DateColumn::defaults(&column_config);
                                column_data_wrap = obj.validate(&column_data);
                            },
                            COLUMN_TYPE_DURATION => {
                                let obj = DurationColumn::defaults(&column_config);
                                column_data_wrap = obj.validate(&column_data);
                            },
                            COLUMN_TYPE_CREATED_TIME => {
                                let obj = AuditDateColumn::defaults(&column_config);
                                column_data_wrap = obj.validate(&column_data);
                            },
                            COLUMN_TYPE_LAST_MODIFIED_TIME => {
                                let obj = AuditDateColumn::defaults(&column_config);
                                column_data_wrap = obj.validate(&column_data);
                            },
                            COLUMN_TYPE_CREATED_BY => {
                                let obj = AuditByColumn::defaults(&column_config);
                                column_data_wrap = obj.validate(&user_id);
                            },
                            COLUMN_TYPE_LAST_MODIFIED_BY => {
                                let obj = AuditByColumn::defaults(&column_config);
                                column_data_wrap = obj.validate(&user_id);
                            },
                            COLUMN_TYPE_CURRENCY => {
                                let obj = CurrencyColumn::defaults(&column_config);
                                column_data_wrap = obj.validate(&column_data);
                            },
                            COLUMN_TYPE_PERCENTAGE => {
                                let obj = PercentageColumn::defaults(&column_config);
                                column_data_wrap = obj.validate(&column_data);
                            },
                            COLUMN_TYPE_LINK => {
                                let obj = LinkColumn::defaults(
                                    planet_context,
                                    context,
                                    &column_config,
                                    folder_name,
                                    Some(db_folder.clone()),
                                    Some(space_database.clone())
                                );
                                let result = obj.validate(&column_data);
                                if result.is_err() {
                                    let errors_ = result.clone().err().unwrap();
                                    for error in errors_ {
                                        errors.push(error);
                                    }
                                } else {
                                    let id_list = result.unwrap();
                                    let many = column_config.many.unwrap();
                                    if many {
                                        let mut items: Vec<BTreeMap<String, String>> = Vec::new();
                                        for item_id in id_list.clone() {
                                            let mut map: BTreeMap<String, String> = BTreeMap::new();
                                            map.insert(ID.to_string(), item_id);
                                            items.push(map);
                                        }
                                        data.insert(column_id.clone(), items);
                                    } else {
                                        let mut map: BTreeMap<String, String> = BTreeMap::new();
                                        let value = id_list[0].clone();
                                        map.insert(ID.to_string(), value);
                                        let mut my_list: Vec<BTreeMap<String, String>> = Vec::new();
                                        my_list.push(map);
                                        data.insert(column_id.clone(), my_list);
                                    }
                                    skip_data_assign = true;
                                    // links_map
                                    let linked_folder = column_config.clone().linked_folder.unwrap();
                                    let map_item = links_map.get(
                                        &linked_folder
                                    );
                                    if map_item.is_some() {
                                        let mut array = map_item.unwrap().clone();
                                        array.push(column_config);
                                        links_map.insert(column_id.clone(), array.clone());
                                    } else {
                                        let mut array: Vec<ColumnConfig> = Vec::new();
                                        array.push(column_config);
                                        links_map.insert(column_id.clone(), array);
                                    }
                                    links_map_map.insert(item_id.clone(), links_map.clone());
                                    // links_data_map
                                    // address folder id => {"Home Addresses" => [jdskdsj], "Work Addresses": [djdks8dsjk]}
                                    let map_item_data = links_data_map.get(&linked_folder);
                                    if map_item_data.is_some() {
                                        let mut my_map = map_item_data.unwrap().clone();
                                        let my_list_wrap = my_map.get(&column_name.clone());
                                        let mut my_list: Vec<String>;
                                        if my_list_wrap.is_some() {
                                            my_list = my_list_wrap.unwrap().clone();
                                        } else {
                                            my_list = Vec::new();
                                        }
                                        for item_id in id_list.clone() {
                                            my_list.push(item_id);
                                        }
                                        my_map.insert(column_name.clone(), my_list);
                                        links_data_map.insert(column_id.clone(), my_map);
                                    } else {
                                        let mut my_map: HashMap<String, Vec<String>> = HashMap::new();
                                        let mut my_list: Vec<String> = Vec::new();
                                        for item_id in id_list.clone() {
                                            my_list.push(item_id);
                                        }
                                        my_map.insert(column_name.clone(), my_list);
                                        links_data_map.insert(column_id.clone(), my_map);
                                    }
                                    links_data_map_map.insert(item_id.clone(), links_data_map.clone());    
                                }
                            },
                            COLUMN_TYPE_GENERATE_ID => {
                                let obj = GenerateIdColumn::defaults(&column_config);
                                column_data_wrap = obj.validate(&column_data);
                            },
                            COLUMN_TYPE_GENERATE_NUMBER => {
                                let obj = GenerateNumberColumn::defaults(
                                    &column_config,
                                    Some(folder.clone()),
                                    Some(db_folder.clone()),
                                );
                                column_data_wrap = obj.validate(&column_data);
                            },
                            COLUMN_TYPE_PHONE => {
                                let obj = PhoneColumn::defaults(&column_config);
                                column_data_wrap = obj.validate(&column_data);
                            },
                            COLUMN_TYPE_EMAIL => {
                                let obj = EmailColumn::defaults(&column_config);
                                column_data_wrap = obj.validate(&column_data);
                            },
                            COLUMN_TYPE_URL => {
                                let obj = UrlColumn::defaults(&column_config);
                                column_data_wrap = obj.validate(&column_data);
                            },
                            COLUMN_TYPE_RATING => {
                                let obj = RatingColumn::defaults(&column_config);
                                column_data_wrap = obj.validate(&column_data);
                            },
                            COLUMN_TYPE_OBJECT => {
                                let obj = ObjectColumn::defaults(&column_config);
                                column_data_wrap = obj.validate(&column_data);
                            },
                            COLUMN_TYPE_FILE => {
                                let obj = FileColumn::defaults(
                                    &column_config,
                                    Some(db_row.clone()),
                                    Some(space_database.clone())
                                );
                                let fields = obj.validate(
                                    &column_data,
                                    &data,
                                    routing_wrap.clone(),
                                    &home_dir.clone().unwrap_or_default()
                                );
                                if fields.is_ok() {
                                    let fields = fields.unwrap();
                                    column_data_wrap = Ok(fields.0);
                                    data = fields.2;
                                }
                                // skip_data_assign = true;
                            },
                            _ => {
                                errors.push(
                                    PlanetError::new(
                                        500, 
                                        Some(tr!("Field \"{}\" not supported.", &column_type)),
                                    )
                                );
                            }
                        };
                        // eprintln!("InsertIntoFolder.run :: \"{}\" skip_data_assign: {} data: {} objects: {} collections: {}", 
                        //     &column_name,
                        //     &skip_data_assign,
                        //     &column_data_wrap.is_ok(),
                        //     &data_objects.len(),
                        //     &data_collections.len(),
                        // );
                        if skip_data_assign == false {
                            let tuple = handle_field_response(
                                &column_data_wrap, &errors, &column_id, &data, &is_set
                            );
                            data = tuple.0;
                            errors = tuple.1;
                        }
                    }
                    // text and language
                    let mut text_map: BTreeMap<String, String> = BTreeMap::new();
                    let mut text_column_id: String = String::from("");
                    for column_config in config_columns.clone() {
                        let column_config_ = column_config.clone();
                        let column_type = &column_config.column_type.unwrap();
                        let column_type = column_type.as_str();
                        let column_id = &column_config.id.unwrap();
                        if column_type == COLUMN_TYPE_TEXT {
                            let mut obj = TextColumn::defaults(
                                &column_config_,
                                Some(column_config_map.clone()),
                            );
                            text_column_id = column_id.clone();
                            let result_text = obj.validate(
                                &data, 
                                &folder,
                                &text_column_id
                            );
                            if result_text.is_err() {
                                let error_message = result_text.clone().unwrap_err().message;
                                errors.push(
                                    PlanetError::new(
                                        500, 
                                        Some(tr!("Error capturing text for folder item: {}", &error_message)),
                                    )
                                );
                            }
                            text_map = result_text.unwrap();
                            let mut my_list: Vec<BTreeMap<String, String>> = Vec::new();
                            my_list.push(text_map.clone());
                            data.insert(TEXT.to_string(), my_list);
                        } else if column_type == COLUMN_TYPE_LANGUAGE {
                            let obj = LanguageColumn::defaults(
                                &column_config_,
                            );
                            let text_map = text_map.clone();
                            let text = text_map.get(&text_column_id).unwrap();
                            let result_lang = obj.validate(text);
                            if result_lang.is_err() {
                                let error_message = result_lang.clone().unwrap_err().message;
                                errors.push(
                                    PlanetError::new(
                                        500, 
                                        Some(tr!("Error capturing language for folder item: {}", &error_message)),
                                    )
                                );
                            }
                            let language_code = result_lang.unwrap();
                            data.insert(column_id.clone(), build_value_list(&language_code));
                        } else if column_type == COLUMN_TYPE_STATEMENT {
                            let obj = StatementColumn::defaults(&column_config_);
                            let result_stmt = obj.validate(
                                &env, 
                                &space_database, 
                                &data
                            );
                            if result_stmt.is_err() {
                                let errors_ = result_stmt.clone().unwrap_err();
                                errors.extend(errors_);
                            }
                            let result_stmt = result_stmt.unwrap();
                            let mut list_value: Vec<BTreeMap<String, String>> = Vec::new();
                            for item in result_stmt {
                                let mut map: BTreeMap<String, String> = BTreeMap::new();
                                map.insert(VALUE.to_string(), item);
                                list_value.push(map);
                            }
                            data.insert(column_id.clone(), list_value);
                        }
                    }
                    if errors.len() > 0 {
                        return Err(errors)
                    }
                    db_data.data = Some(data);
                    db_data_list.push(db_data);
                }

                // Up to here the loop, final task is to add db_data to vector, list_db_data

                // eprintln!("InsertIntoFolder.run :: I will write: {:#?}", &db_data);
                let response_list= db_row.insert(
                    &folder_name, 
                    &db_data_list
                );
                if response_list.is_err() {
                    let errors_response = response_list.unwrap_err();
                    for error in errors_response {
                        errors.push(error);
                    }
                    return Err(errors)
                }
                let response_list = response_list.unwrap();
                // let id_record = response.clone().id.unwrap();

                // response would have a list of items instead of one
                
                // links
                // links_map_list and links_data_map_list, response_list, all same length for list
                // I can make a map of id -> links maps in a tuple, having then the record id from that map
                // links_map_map
                // links_data_map_map
                // I go through the response_list, and get links data from the id, process it.

                let mut yaml_response: Vec<yaml_rust::Yaml> = Vec::new();
                for response in response_list {
                    let id_record = response.id.clone().unwrap_or_default();
                    let links_map = links_map_map.get(&id_record);
                    let links_data_map = links_data_map_map.get(&id_record);
                    if links_map.is_some() && links_data_map.is_some() {
                        let links_map = links_map.unwrap();
                        let links_data_map = links_data_map.unwrap();
                        for (column_id, config_column_list) in links_map {
                            // Get db item for this link
                            let column_id = column_id.clone();
                            for config in config_column_list {
                                let config = config.clone();
                                let many = config.many.unwrap();
                                let remote_folder_name = config.linked_folder;
                                if remote_folder_name.is_none() {
                                    continue
                                }
                                let remote_folder_name = remote_folder_name.unwrap();
                                let folder = db_folder.get_by_name(&remote_folder_name).unwrap().unwrap();
                                // let folder = db_folder.get(&remote_folder_name).unwrap();
                                let remote_folder_id = folder.id.unwrap_or_default();
                                let folder_name = folder.name.unwrap();
                                let main_data_map = links_data_map.get(&column_id);
                                if main_data_map.is_some() {
                                    let main_data_map = main_data_map.unwrap();
                                    for (_column_name, id_list) in main_data_map {
                                        for item_id in id_list {
                                            let result: Result<TreeFolderItem, PlanetError> = TreeFolderItem::defaults(
                                                space_database.connection_pool.clone(),
                                                home_dir.clone().unwrap_or_default().as_str(),
                                                &account_id,
                                                space_id,
                                                Some(site_id.clone().unwrap().to_string()),
                                                remote_folder_id.as_str(),
                                                &db_folder,
                                            );
                                            if result.is_err() {
                                                // Return error about database problem
                                            }
                                            let mut db_row = result.unwrap();
                                            let linked_item = db_row.get(
                                                &folder_name, 
                                                GetItemOption::ById(item_id.clone()), 
                                                None
                                            );
                                            if linked_item.is_ok() {
                                                let mut linked_item = linked_item.unwrap();
                                                eprintln!("InsertIntoFolder.run :: linked_item: {:#?}", &linked_item);
                                                // I may need to update to data_objects or data_collections
                                                if many {
                                                    let data_wrap = linked_item.data;
                                                    let mut data: BTreeMap<String, Vec<BTreeMap<String, String>>>;
                                                    if data_wrap.is_some() {
                                                        data = data_wrap.unwrap();
                                                    } else {
                                                        data = BTreeMap::new();
                                                    }
                                                    let list_wrap = data.get(&column_id);
                                                    let mut list: Vec<BTreeMap<String, String>>;
                                                    if list_wrap.is_some() {
                                                        list = list_wrap.unwrap().clone();
                                                        let mut item_object: BTreeMap<String, String> = BTreeMap::new();
                                                        item_object.insert(ID.to_string(), id_record.clone());
                                                        list.push(item_object);
                                                    } else {
                                                        list = Vec::new();
                                                        let mut item_object: BTreeMap<String, String> = BTreeMap::new();
                                                        item_object.insert(ID.to_string(), id_record.clone());
                                                        list.push(item_object);
                                                    }
                                                    data.insert(column_id.clone(), list);
                                                    linked_item.data = Some(data);
                                                    let _linked_item = db_row.update(&linked_item);
                                                } else {
                                                    let data_wrap = linked_item.data;
                                                    let mut data: BTreeMap<String, Vec<BTreeMap<String, String>>>;
                                                    if data_wrap.is_some() {
                                                        data = data_wrap.unwrap();
                                                    } else {
                                                        data = BTreeMap::new();
                                                    }
                                                    let mut item_object: BTreeMap<String, String> = BTreeMap::new();
                                                    item_object.insert(ID.to_string(), id_record.clone());
                                                    let mut my_list: Vec<BTreeMap<String, String>> = Vec::new();
                                                    my_list.push(item_object);
                                                    data.insert(column_id.clone(), my_list);
                                                    linked_item.data = Some(data);
                                                    let _linked_item = db_row.update(&linked_item);
                                                }
                                            } else {
                                                let error = linked_item.unwrap_err();
                                                eprintln!("InsertIntoFolder.run :: I have error on get linked_item: {}", &error.message);
                                            }
                                        }
                                    }
                                }
                            }
                        }        
                    }
                    let response_coded = serde_yaml::to_string(&response);
                    if response_coded.is_err() {
                        let error = PlanetError::new(
                            500, 
                            Some(tr!("Error encoding statement response.")),
                        );
                        errors.push(error);
                        return Err(errors)
                    }
                    let response = response_coded.unwrap();
                    let yaml_item = yaml_rust::YamlLoader::load_from_str(
                        response.as_str()
                    ).unwrap();
                    yaml_response.push(yaml_item[0].clone());
                }
                eprintln!("InsertIntoFolder.run :: time: {} ms", &t_1.elapsed().as_millis());
                // let yaml_response = yaml_response.clone();
                // Response would need to be list of Yaml documents
                return Ok(yaml_response)
            },
            Err(error) => {
                errors.push(error);
                return Err(errors);
            }
        }
    }
}

pub struct GetFromFolder<'gb> {
    pub planet_context: &'gb PlanetContext<'gb>,
    pub context: &'gb Context<'gb>,
    pub db_folder: TreeFolder,
    pub space_database: SpaceDatabase,
    pub config: GetFromFolderConfig,
}

// impl<'gb> Statement<'gb> for GetFromFolder<'gb> {

//     fn run(&self) -> Result<String, PlanetError> {
//         let command = self.config.command.clone().unwrap_or_default();
//         let expr = Regex::new(r#"(GET FROM TABLE) "(?P<folder_name>[a-zA-Z0-9_ ]+)""#).unwrap();
//         let table_name_match = expr.captures(&command).unwrap();
//         let folder_name = &table_name_match["folder_name"].to_string();
//         let folder_file = slugify(&folder_name);
//         let folder_file = folder_file.as_str().replace("-", "_");

//         let home_dir = self.planet_context.home_path.unwrap_or_default();
//         let account_id = self.context.account_id.unwrap_or_default();
//         let space_id = self.context.space_id.unwrap_or_default();
//         let site_id = self.context.site_id.unwrap_or_default();
//         let box_id = self.context.box_id.unwrap_or_default();
//         let space_database = self.space_database.clone();
//         let result: Result<TreeFolderItem, PlanetError> = TreeFolderItem::defaults(
//             space_database.connection_pool,
//             home_dir,
//             account_id,
//             space_id,
//             site_id,
//             box_id,
//             folder_file.as_str(),
//             &self.db_folder,
//         );
//         match result {
//             Ok(_) => {
//                 // let data_config = self.config.data.clone();
//                 let mut db_row: TreeFolderItem = result.unwrap();
//                 // I need to get SchemaData and schema for the folder
//                 // I go through columns in order to build RowData                
//                 let folder = self.db_folder.get_by_name(folder_name)?;
//                 if *&folder.is_none() {
//                     return Err(
//                         PlanetError::new(
//                             500, 
//                             Some(tr!("Could not find folder {}", &folder_name)),
//                         )
//                     );
//                 }
//                 let folder = folder.unwrap();
//                 let data = folder.clone().data;
//                 let field_ids = data.unwrap().get(COLUMN_IDS).unwrap().clone();
//                 let config_columns = ColumnConfig::get_config(
//                     self.planet_context,
//                     self.context,
//                     &folder
//                 )?;
//                 let field_id_map: BTreeMap<String, ColumnConfig> = ColumnConfig::get_column_id_map(&config_columns)?;
//                 let columns = self.config.data.clone().unwrap().columns;
//                 eprintln!("GetFromFolder.run :: columns: {:?}", &columns);
//                 let item_id = self.config.data.clone().unwrap().id.unwrap();
//                 // Get item from database
//                 let db_data = db_row.get(&folder_name, GetItemOption::ById(item_id), columns)?;
//                 // data and basic columns
//                 let data = db_data.data;
//                 let mut yaml_out_str = String::from("---\n");
//                 // id
//                 let id_yaml_value = self.config.data.clone().unwrap().id.unwrap().truecolor(
//                     YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
//                 );
//                 let id_yaml = format!("{}", 
//                     id_yaml_value.truecolor(YAML_COLOR_ORANGE[0], YAML_COLOR_ORANGE[1], YAML_COLOR_ORANGE[2]), 
//                 );
//                 yaml_out_str.push_str(format!("{column}: {value}\n", 
//                     column=String::from(ID).truecolor(
//                         YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
//                     ), 
//                     value=&id_yaml
//                 ).as_str());
//                 // name
//                 let name_yaml_value = &db_data.name.unwrap().clone();
//                 let name_yaml = format!("{}", 
//                     name_yaml_value.truecolor(YAML_COLOR_ORANGE[0], YAML_COLOR_ORANGE[1], YAML_COLOR_ORANGE[2]), 
//                 );
//                 yaml_out_str.push_str(format!("{column}: {value}\n", 
//                     column=String::from(NAME).truecolor(
//                         YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
//                     ), 
//                     value=&name_yaml
//                 ).as_str());
//                 yaml_out_str.push_str(format!("{}\n", 
//                     String::from("data:").truecolor(YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]),
//                 ).as_str());
//                 if data.is_some() {
//                     // column_id -> string value
//                     let data = data.unwrap();
//                     // I need to go through in same order as columns were registered in ColumnConfig when creating schema
//                     for field_id_data in field_ids {
//                         let column_id = field_id_data.get(ID).unwrap();
//                         let column_config = field_id_map.get(column_id).unwrap().clone();
//                         let field_config_ = column_config.clone();
//                         let column_type = column_config.column_type.unwrap();
//                         let column_type = column_type.as_str();
//                         let value = data.get(column_id);
//                         if value.is_none() {
//                             continue
//                         }
//                         let value = value.unwrap();
//                         let value = get_value_list(value);
//                         if value.is_none() {
//                             continue
//                         }
//                         let value = &value.unwrap();
//                         // Get will return YAML document for the data
//                         match column_type {
//                             COLUMN_TYPE_SMALL_TEXT => {
//                                 let obj = SmallTextColumn::defaults(&field_config_);
//                                 yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
//                             },
//                             COLUMN_TYPE_LONG_TEXT => {
//                                 let obj = LongTextColumn::defaults(&field_config_);
//                                 yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
//                             },
//                             COLUMN_TYPE_CHECKBOX => {
//                                 let obj = CheckBoxColumn::defaults(&field_config_);
//                                 yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
//                             },
//                             COLUMN_TYPE_NUMBER => {
//                                 let obj = NumberColumn::defaults(&field_config_);
//                                 yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
//                             },
//                             COLUMN_TYPE_SELECT => {
//                                 let obj = SelectColumn::defaults(&field_config_, Some(&folder));
//                                 yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
//                             },
//                             COLUMN_TYPE_FORMULA => {
//                                 let obj = FormulaColumn::defaults(&field_config_);
//                                 yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
//                             },
//                             COLUMN_TYPE_DATE => {
//                                 let obj = DateColumn::defaults(&field_config_);
//                                 yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
//                             },
//                             COLUMN_TYPE_DURATION => {
//                                 let obj = DurationColumn::defaults(&field_config_);
//                                 yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
//                             },                            
//                             COLUMN_TYPE_CREATED_TIME => {
//                                 let obj = AuditDateColumn::defaults(&field_config_);
//                                 yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
//                             },
//                             COLUMN_TYPE_LAST_MODIFIED_TIME => {
//                                 let obj = AuditDateColumn::defaults(&field_config_);
//                                 yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
//                             },
//                             COLUMN_TYPE_CREATED_BY => {
//                                 let obj = AuditByColumn::defaults(&field_config_);
//                                 yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
//                             },
//                             COLUMN_TYPE_LAST_MODIFIED_BY => {
//                                 let obj = AuditByColumn::defaults(&field_config_);
//                                 yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
//                             },
//                             COLUMN_TYPE_CURRENCY => {
//                                 let obj = CurrencyColumn::defaults(&field_config_);
//                                 yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
//                             },
//                             COLUMN_TYPE_PERCENTAGE => {
//                                 let obj = PercentageColumn::defaults(&field_config_);
//                                 yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
//                             },
//                             COLUMN_TYPE_GENERATE_ID => {
//                                 let obj = GenerateIdColumn::defaults(&field_config_);
//                                 yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
//                             },
//                             COLUMN_TYPE_GENERATE_NUMBER => {
//                                 let obj = GenerateNumberColumn::defaults(
//                                     &field_config_,
//                                     Some(folder.clone()),
//                                     Some(self.db_folder.clone()),
//                                 );
//                                 yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
//                             },
//                             COLUMN_TYPE_PHONE => {
//                                 let obj = PhoneColumn::defaults(&field_config_);
//                                 yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
//                             },
//                             COLUMN_TYPE_EMAIL => {
//                                 let obj = EmailColumn::defaults(&field_config_);
//                                 yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
//                             },
//                             COLUMN_TYPE_URL => {
//                                 let obj = UrlColumn::defaults(&field_config_);
//                                 yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
//                             },
//                             COLUMN_TYPE_RATING => {
//                                 let obj = RatingColumn::defaults(&field_config_);
//                                 yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
//                             },
//                             COLUMN_TYPE_OBJECT => {
//                                 let obj = ObjectColumn::defaults(&field_config_);
//                                 yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
//                             },
//                             _ => {
//                                 yaml_out_str = yaml_out_str;
//                             }
//                         }
//                     }
//                 }
//                 eprintln!("{}", yaml_out_str);
//                 return Ok(yaml_out_str);
//             },
//             Err(error) => {
//                 return Err(error);
//             }
//         }
//     }

    // fn runner(runner: &CommandRunner, path_yaml: &String) -> () {
    //     let config_ = GetFromFolderConfig::defaults(
    //         String::from("")
    //     );
    //     let config: Result<GetFromFolderConfig, Vec<PlanetValidationError>> = config_.import(
    //         runner.planet_context,
    //         &path_yaml
    //     );
    //     match config {
    //         Ok(_) => {
    //             let home_dir = runner.planet_context.home_path.unwrap_or_default();
    //             let account_id = runner.context.account_id.unwrap_or_default();
    //             let space_id = runner.context.space_id.unwrap_or_default();
    //             let site_id = runner.context.site_id.unwrap_or_default();
    //             let result = SpaceDatabase::defaults(
    //                 Some(site_id), 
    //                 space_id, 
    //                 Some(home_dir),
    //                 Some(false)
    //             );
    //             if result.is_err() {
    //                 let error = result.clone().unwrap_err();
    //                 println!();
    //                 println!("{}", tr!("I found these errors").red().bold());
    //                 println!("{}", "--------------------".red());
    //                 println!();
    //                 println!(
    //                     "{} {}", 
    //                     String::from('.').blue(),
    //                     error.message
    //                 );
    //             }
    //             let space_database = result.unwrap();
    //             let db_folder= TreeFolder::defaults(
    //                 space_database.connection_pool.clone(),
    //                 Some(home_dir),
    //                 Some(account_id),
    //                 Some(space_id),
    //                 Some(site_id),
    //             ).unwrap();

    //             let insert_into_table: GetFromFolder = GetFromFolder{
    //                 planet_context: runner.planet_context,
    //                 context: runner.context,
    //                 config: config.unwrap(),
    //                 space_database: space_database.clone(),
    //                 db_folder: db_folder.clone(),
    //             };
    //             let result: Result<_, PlanetError> = insert_into_table.run();
    //             match result {
    //                 Ok(_) => {
    //                     println!();
    //                     println!("{}", String::from("[OK]").green());
    //                 },
    //                 Err(error) => {
    //                     let count = 1;
    //                     println!();
    //                     println!("{}", tr!("I found these errors").red().bold());
    //                     println!("{}", "--------------------".red());
    //                     println!();
    //                     println!(
    //                         "{}{} {}", 
    //                         count.to_string().blue(),
    //                         String::from('.').blue(),
    //                         error.message
    //                     );
    //                 }
    //             }
    //         },
    //         Err(errors) => {
    //             println!();
    //             println!("{}", tr!("I found these errors").red().bold());
    //             println!("{}", "--------------------".red());
    //             println!();
    //             let mut count = 1;
    //             for error in errors {
    //                 println!(
    //                     "{}{} {}", 
    //                     count.to_string().blue(), 
    //                     String::from('.').blue(), 
    //                     error.message
    //                 );
    //                 count += 1;
    //             }
    //         }
    //     }
    // }
// }

// Sort By


#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SelectSortMode {
    Ascending,
    Descending,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SelectSortBy {
    pub column: String,
    pub mode: SelectSortMode,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SortedtBy {
    pub sorted_item: String,
    pub mode: SelectSortMode,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SelectCount {
    pub column: Option<String>,
    pub all: bool,
    pub distinct: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SelectFromFolderCompiledStmt {
    pub folder_name: String,
    pub columns: Option<Vec<String>>,
    pub page: u32,
    pub number_items: u32,
    pub search: Option<String>,
    pub where_source: Option<String>,
    pub where_compiled: Option<Formula>,
    pub group_by: Option<Vec<String>>,
    pub sort_by: Option<Vec<SelectSortBy>>,
    pub count: Option<SelectCount>,
}

impl SelectFromFolderCompiledStmt {

    pub fn defaults(
        folder_name: String,
        page: Option<u32>, 
        number_items: Option<u32>,
    ) -> SelectFromFolderCompiledStmt {
        let page_int: u32;
        let number_items_int: u32;
        if page.is_some() {
            page_int = page.unwrap();
        } else {
            page_int = 1;
        }
        if number_items.is_some() {
            number_items_int = number_items.unwrap();
        } else {
            number_items_int = 20;
        }
        let statement = SelectFromFolderCompiledStmt{
            folder_name: folder_name,
            page: page_int,
            number_items: number_items_int,
            columns: None,
            search: None,
            where_source: None,
            where_compiled: None,
            group_by: None,
            sort_by: None,
            count: None,
        };
        return statement
    }

}

#[derive(Debug, Clone)]
pub struct SearchCompiler<'gb>{
    pub statement_text: String,
    pub env: &'gb Environment<'gb>,
    pub space_database: SpaceDatabase,
}

impl<'gb> SearchCompiler<'gb> {

    pub fn compile(
        &self, 
    ) -> Result<SelectFromFolderCompiledStmt, Vec<PlanetError>> {
        let expr = &RE_SELECT_COUNT;
        let statement_text = self.statement_text.clone();
        let is_count = expr.is_match(&statement_text);
        let mut page: Option<u32> = None;
        let mut number_items: Option<u32> = None;
        let mut folder_name = String::from("");
        let mut errors: Vec<PlanetError> = Vec::new();
        if is_count {
            let captures = expr.captures(&statement_text);
            if captures.is_some() {
                let captures = captures.unwrap();
                let folder_name_ = captures.name("FolderName");
                let count_all = captures.name("CountAll");
                let count_column = captures.name("CountColumn");
                let count_column_distinct = captures.name("CountColumnDis");
                if folder_name_.is_some() {
                    let folder_name_ = folder_name_.unwrap().as_str();
                    folder_name = folder_name_.to_string();
                } else {
                    // Raise error since folder name is required
                    errors.push(
                        PlanetError::new(
                            500, 
                            Some(tr!("Folder name not detected and is required.")),
                        )
                    )
                }
                let mut statement = SelectFromFolderCompiledStmt::defaults(
                    folder_name, 
                    page, 
                    number_items
                );
                if count_all.is_some() {
                    let count = SelectCount{
                        column: None,
                        all: true,
                        distinct: false,
                    };
                    statement.count = Some(count);
                    return Ok(statement)
                } else {
                    let count_column_: &str;
                    let mut distinct = false;
                    if count_column.is_some() {
                        count_column_ = count_column.unwrap().as_str();
                    } else {
                        count_column_ = count_column_distinct.unwrap().as_str();
                        distinct = true;
                    }
                    let count = SelectCount{
                        column: Some(count_column_.to_string()),
                        all: false,
                        distinct: distinct,
                    };
                    statement.count = Some(count);
                    return Ok(statement)
                }
            }
        } else {
            let statement_text = statement_text.clone();
            let statement_text = statement_text.replace("\n", "").clone();
            let expr = &RE_SELECT;
            let is_match = expr.is_match(&statement_text);
            if !is_match {
                errors.push(
                    PlanetError::new(
                        500, 
                        Some(tr!("Bad syntax for SELECT statement.")),
                    )
                )
            } else {
                // 1 - Paging
                let expr = &RE_SELECT_PAGING;
                let captures_iter = expr.captures_iter(&statement_text);
                let mut statement_text_new = statement_text.clone();
                for captures in captures_iter {
                    let page_regex = captures.name("Page");
                    let number_items_regex = captures.name("NumberItems");
                    if page_regex.is_some() || number_items_regex.is_some() {
                        if page_regex.is_some() {
                            let page_regex = page_regex.unwrap().as_str();
                            let page_int: u32 = FromStr::from_str(page_regex).unwrap();
                            let check_text = format!("PAGE {}", &page_int);
                            statement_text_new = statement_text_new.replace(&check_text, "");
                            page = Some(page_int);
                        }
                        if number_items_regex.is_some() {
                            let number_items_regex = number_items_regex.unwrap().as_str();
                            let number_items_int: u32 = FromStr::from_str(number_items_regex).unwrap();
                            let check_text = format!("NUMBER ITEMS {}", &number_items_int);
                            statement_text_new = statement_text_new.replace(&check_text, "");
                            number_items = Some(number_items_int);
                        }
                    }
                }
                let mut statement_text = statement_text_new;
                // 2 - From
                let expr = &RE_SELECT_FROM;
                let captures = expr.captures(&statement_text);
                if captures.is_some() {
                    let captures = captures.unwrap();
                    let folder_name_ = captures.name("FolderName");
                    if folder_name_.is_some() {
                        let folder_name_ = folder_name_.unwrap().as_str();
                        folder_name = folder_name_.to_string();
                        folder_name = folder_name.trim().to_string();
                        let check_text = format!("FROM \"{}\"", &folder_name);
                        statement_text = statement_text.replace(
                            &check_text, 
                            "FROM {from}"
                        );
                    } else {
                        errors.push(
                            PlanetError::new(
                                500, 
                                Some(tr!("Folder name is required.")),
                            )
                        )
                    }
                }
                let mut statement = SelectFromFolderCompiledStmt::defaults(
                    folder_name, 
                    page, 
                    number_items
                );
                // 3 - Columns
                let expr = &RE_SELECT_COLUMNS;
                let captures = expr.captures(&statement_text);
                if captures.is_some() {
                    let captures = captures.unwrap();
                    let columns = captures.name("Columns");
                    if columns.is_some() {
                        let columns_str = columns.unwrap().as_str();
                        let columns_: Vec<&str> = columns_str.split(",").collect();
                        let columns: Vec<String> = columns_.iter().map(|&s|s.trim().into()).collect();
                        statement.columns = Some(columns);
                        let check_text = format!("SELECT {}", columns_str);
                        statement_text = statement_text.replace(
                            &check_text, 
                            ""
                        );
                    }
                }
                // 4 - Sort By
                let expr_1 = &RE_SELECT_SORTING;
                let captures = expr_1.captures(&statement_text);
                if captures.is_some() {
                    let captures = captures.unwrap();
                    let sort_by = captures.name("SortBy");
                    if sort_by.is_some() {
                        let sort_by = sort_by.unwrap().as_str();
                        // {COLUMN A ASC|COLUMN B DESC}
                        let expr = &RE_SELECT_SORT_FIELDS;
                        let items = expr.captures_iter(sort_by);
                        let mut sort_items: Vec<SelectSortBy> = Vec::new();
                        for item in items {
                            let column = item.name("Column");
                            let mode = item.name("Mode");
                            let mut sort_obj = SelectSortBy{
                                column: String::from(""),
                                mode: SelectSortMode::Ascending
                            };
                            if column.is_some() {
                                let column = column.unwrap().as_str();
                                sort_obj.column = column.to_string().trim().to_string();
                            }
                            if mode.is_some() {
                                let mode = mode.unwrap().as_str();
                                match mode {
                                    SORT_MODE_ASC => {
                                        sort_obj.mode = SelectSortMode::Ascending;
                                    },
                                    SORT_MODE_DESC => {
                                        sort_obj.mode = SelectSortMode::Descending;
                                    },
                                    _ => {}
                                }
                            } else {
                                sort_obj.mode = SelectSortMode::Ascending;
                            }
                            sort_items.push(sort_obj);
                        }
                        if sort_items.len() > 0 {
                            statement.sort_by = Some(sort_items);
                            statement_text = expr_1.replace(&statement_text, "").to_string();
                        }
                    }
                }
                // 5 - Group By
                let expr_1 = &RE_SELECT_GROUP_BY;
                let captures = expr_1.captures(&statement_text);
                if captures.is_some() {
                    let captures = captures.unwrap();
                    let group_by_columns = captures.name("GroupByColumns");
                    if group_by_columns.is_some() {
                        let group_by_columns = group_by_columns.unwrap().as_str();
                        // Column A, Column B
                        let expr = &RE_SELECT_GROUP_COLUMNS;
                        let columns = expr.captures_iter(group_by_columns);
                        let mut columns_string: Vec<String> = Vec::new();
                        for column in columns {
                            let column_str = column.name("Column").unwrap().as_str();
                            columns_string.push(column_str.to_string().trim().to_string());
                        }
                        if columns_string.len() > 0 {
                            statement.group_by = Some(columns_string);
                            statement_text = expr_1.replace(&statement_text, "").to_string();
                        }
                    }
                }
                // 6 - Search
                let expr = &RE_SELECT_SEARCH;
                let captures = expr.captures(&statement_text);
                if captures.is_some() {
                    let captures = captures.unwrap();
                    let search = captures.name("Search");
                    if search.is_some() {
                        let search = search.unwrap().as_str();
                        statement.search = Some(search.to_string());
                        statement_text = expr.replace(&statement_text, "").to_string();
                    }
                }
                // 7 - Where
                let expr_1 = &RE_SELECT_WHERE;
                let captures = expr_1.captures(&statement_text);
                if captures.is_some() {
                    let captures = captures.unwrap();
                    let where_formula = captures.name("Where");
                    if where_formula.is_some() {
                        let where_formula_str = where_formula.unwrap().as_str();
                        let expr = &RE_FORMULA_QUERY;
                        let is_valid = expr.is_match(where_formula_str);
                        statement.where_source = Some(where_formula_str.to_string());
                        if !is_valid {
                            errors.push(
                                PlanetError::new(
                                    500, 
                                    Some(tr!("WHERE formula is not valid.")),
                                )
                            )
                        }
                    }
                }
                return Ok(statement)
            }
        }
        return Err(errors)
    }

    pub fn validate(
        &self,
        mut compiled_statement: SelectFromFolderCompiledStmt,
    ) -> Result<SelectFromFolderCompiledStmt, Vec<PlanetError>> {
        let env = self.env.clone();
        let space_database = self.space_database.clone();
        let context = env.context;
        let planet_context = env.planet_context;
        let mut errors: Vec<PlanetError> = Vec::new();
        
        // - Get folder and other data
        let folder_name = compiled_statement.folder_name.clone();
        let home_dir = planet_context.home_path.clone();
        let account_id = context.account_id.clone().unwrap_or_default();
        let space_id = context.space_id;
        let site_id = context.site_id.clone();
        let space_database = space_database.clone();
        let db_folder= TreeFolder::defaults(
            space_database.connection_pool.clone(),
            Some(home_dir.clone().unwrap_or_default().as_str()),
            Some(&account_id),
            Some(space_id),
            site_id.clone(),
        ).unwrap();
        let folder = db_folder.get_by_name(&folder_name);
        if folder.is_err() {
            let error = folder.unwrap_err();
            errors.push(error);
            return Err(errors)
        }
        let folder = folder.unwrap();
        if *&folder.is_none() {
            errors.push(
                PlanetError::new(
                    500, 
                    Some(tr!("Could not find folder {}", &folder_name)),
                )
            );
            return Err(errors)
        }
        let folder = folder.unwrap();
        // - Validate columns: columns, group_by, sort
        let mut column_list: Vec<String> = Vec::new();
        if compiled_statement.columns.is_some() {
            let columns = compiled_statement.columns.clone().unwrap();
            column_list.extend(columns);
        }
        if compiled_statement.sort_by.is_some() {
            let sort_by = compiled_statement.sort_by.clone().unwrap();
            for sort_by_item in sort_by {
                let item_column = sort_by_item.column;
                if !column_list.contains(&item_column) {
                    column_list.push(item_column);
                }
            }
        }
        if compiled_statement.group_by.is_some() {
            let group_by = compiled_statement.group_by.clone().unwrap();
            column_list.extend(group_by);
        }
        let mut column_raised: HashMap<String, String> = HashMap::new();
        for column in column_list {
            let has_column = db_folder.has_column(&folder_name, &column);
            let error_raised = column_raised.get(&column).is_some();
            if !has_column && !error_raised {
                errors.push(
                    PlanetError::new(
                        500, 
                        Some(tr!(
                            "Column \"{}\" does not exist in folder \"{}\".", 
                            &column, &folder_name
                        )),
                    )
                );
                column_raised.insert(column.clone(), String::from(""));
            }
        }
        // - Compile Where formula
        let where_source = compiled_statement.where_source.clone();
        if where_source.is_some() {
            let mut where_source = where_source.unwrap();
            let expr = &RE_FORMULA_ASSIGN;
            let is_assign_function = expr.is_match(&where_source);
            eprintln!("SelectFromFolderStatement.validate :: is_assign_function: {}", &is_assign_function);
            let field_config_map = ColumnConfig::get_column_config_map(
                planet_context,
                context,
                &folder
            ).unwrap();
            let field_config_map_wrap = Some(field_config_map.clone());
            // Modify formula in case we have general SEARCH in the statement
            let stmt_search = compiled_statement.search.clone();
            if stmt_search.is_some() {
                let stmt_search = stmt_search.unwrap();
                let search_func = format!("SEARCH(\"Text\", \"{}\")", stmt_search);
                where_source = format!(
                    "AND({}, {})", &search_func, where_source
                );
            }
            let formula_query = Formula::defaults(
                &where_source, 
                &String::from("bool"), 
                Some(folder), 
                None, 
                Some(db_folder), 
                Some(folder_name.clone()), 
                is_assign_function,
                field_config_map_wrap.clone()
            );
            if formula_query.is_err() {
                let error = formula_query.unwrap_err();
                errors.push(error);
            } else {
                let formula_query = formula_query.unwrap();
                compiled_statement.where_compiled = Some(formula_query);
            }
        }
        if errors.len() > 0 {
            return Err(errors)
        }
        return Ok(compiled_statement.clone())
    }

    pub fn do_compile(
        &self,
    ) -> Result<SelectFromFolderCompiledStmt, Vec<PlanetError>> {
        // let mut errors: Vec<PlanetError> = Vec::new();
        // 1 - Compile SELECT statement into SelectFromFolderCompiledStmt
        let statement = self.compile();
        if statement.is_err() {
            let errors = statement.unwrap_err();
            return Err(errors)
        }
        let statement = statement.unwrap();
        eprintln!("SelectFromFolderStatement.run :: [compiled] select: {:#?}", &statement);
        // 2 - Compile Where formula and validate query for existing columns.
        let validation = self.validate(
            statement
        );
        if validation.is_err() {
            let errors = validation.unwrap_err();
            return Err(errors)
        }
        let statement = validation.unwrap();
        return Ok(statement)
    }

}

#[derive(Debug, Clone)]
pub struct SearchWhereBooster{}

#[derive(Debug, Clone)]
pub struct SortValueMode {
    str: Option<String>,
    number: Option<i64>
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Clone)]
pub struct SearchSorter{
    pub partition: u16,
    pub id: String,
    pub column_id: Option<String>,
    pub column_1_str: Option<String>,
    pub column_1_number: Option<i64>,
    pub column_2_str: Option<String>,
    pub column_2_number: Option<i64>,
    pub column_3_str: Option<String>,
    pub column_3_number: Option<i64>,
    pub column_4_str: Option<String>,
    pub column_4_number: Option<i64>,
    pub column_5_str: Option<String>,
    pub column_5_number: Option<i64>,
    pub column_6_str: Option<String>,
    pub column_6_number: Option<i64>,
    pub column_7_str: Option<String>,
    pub column_7_number: Option<i64>,
    pub column_8_str: Option<String>,
    pub column_8_number: Option<i64>,
    pub column_9_str: Option<String>,
    pub column_9_number: Option<i64>,
    pub column_10_str: Option<String>,
    pub column_10_number: Option<i64>,
}

impl SearchSorter {
    pub fn defaults(partition: &u16, id: &String) -> Self {
        let obj = Self{
            partition: partition.clone(),
            id: id.clone(),
            column_id: None,
            column_1_str: None,
            column_2_str: None,
            column_3_str: None,
            column_4_str: None,
            column_5_str: None,
            column_6_str: None,
            column_7_str: None,
            column_8_str: None,
            column_9_str: None,
            column_10_str: None,
            column_1_number: None,
            column_2_number: None,
            column_3_number: None,
            column_4_number: None,
            column_5_number: None,
            column_6_number: None,
            column_7_number: None,
            column_8_number: None,
            column_9_number: None,
            column_10_number: None,
        };
        return obj
    }
}

#[derive(Debug, Clone)]
pub struct SearchResultItem {
    pub id: Option<String>,
    pub partition: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct SearchIterator<'gb>{
    pub query: SelectFromFolderCompiledStmt,
    pub env: &'gb Environment<'gb>,
    pub space_database: SpaceDatabase,
}

impl<'gb> SearchIterator<'gb>{

    fn get_column_sort_type(
        &self,
        column: &BTreeMap<String, String>, 
    ) -> String {
        let column = column.clone();
        let column_type = column.get(COLUMN_TYPE);
        let mut sort_column_type = String::from(SORT_TYPE_STR);
        if column_type.is_some() {
            let column_type = column_type.unwrap().clone();
            let column_type = column_type.as_str();
            match column_type {
                COLUMN_TYPE_DURATION => {
                    sort_column_type = String::from(SORT_TYPE_NUMBER);
                },
                COLUMN_TYPE_CHECKBOX => {
                    sort_column_type = String::from(SORT_TYPE_NUMBER);
                },
                COLUMN_TYPE_NUMBER => {
                    sort_column_type = String::from(SORT_TYPE_NUMBER);
                },
                COLUMN_TYPE_GENERATE_NUMBER => {
                    sort_column_type = String::from(SORT_TYPE_NUMBER);
                },
                COLUMN_TYPE_CURRENCY => {
                    sort_column_type = String::from(SORT_TYPE_NUMBER);
                },
                COLUMN_TYPE_PERCENTAGE => {
                    sort_column_type = String::from(SORT_TYPE_NUMBER);
                },
                __ => {}
            }
        }
        return sort_column_type
    }

    fn get_sort_value(
        &self, 
        item: &DbData,
        column_id: &String,
        column_type: &String,
    ) -> Result<SortValueMode, PlanetError> {
        let column_id = column_id.clone();
        let item = item.clone();
        let item_data = item.data.clone();
        let column_type = column_type.as_str();
        if item_data.is_some() {
            let item_data = item_data.unwrap();
            let values = item_data.get(&column_id);
            if values.is_some() {
                let values = values.unwrap();
                let value = get_value_list(values);
                if value.is_some() {
                    let value = value.unwrap();
                    match column_type {
                        COLUMN_TYPE_DURATION => {
                            let number: i64 = FromStr::from_str(value.as_str()).unwrap();
                            let number = number*1000;
                            return Ok(SortValueMode{str: None, number: Some(number)})
                        },
                        COLUMN_TYPE_CHECKBOX => {
                            let number: i64 = FromStr::from_str(value.as_str()).unwrap();
                            let number = number*1000;
                            return Ok(SortValueMode{str: None, number: Some(number)})
                        },
                        COLUMN_TYPE_NUMBER => {
                            let number: i64 = FromStr::from_str(value.as_str()).unwrap();
                            let number = number*1000;
                            return Ok(SortValueMode{str: None, number: Some(number)})
                        },
                        COLUMN_TYPE_GENERATE_NUMBER => {
                            let number: i64 = FromStr::from_str(value.as_str()).unwrap();
                            let number = number*1000;
                            return Ok(SortValueMode{str: None, number: Some(number)})
                        },
                        COLUMN_TYPE_CURRENCY => {
                            let number: i64 = FromStr::from_str(value.as_str()).unwrap();
                            let number = number*1000;
                            return Ok(SortValueMode{str: None, number: Some(number)})
                        },
                        COLUMN_TYPE_PERCENTAGE => {
                            let number: i64 = FromStr::from_str(value.as_str()).unwrap();
                            let number = number*1000;
                            return Ok(SortValueMode{str: None, number: Some(number)})
                        },
                        __ => {
                            return Ok(SortValueMode{str: Some(value), number: None})
                        }
                    }
                }
            }
        }
        return Err(
            PlanetError::new(
                500, 
                Some(tr!("Error sorting query data.")),
            )
        )

    }

    fn add_to_sorter(
        &self,
        partition: &u16,
        item: &DbData,
        column_type_map: &HashMap<String, String>,
        sorter_map: &HashMap<String, SortedtBy>,
        sorter_list: &Vec<SearchSorter>
    ) -> Result<Vec<SearchSorter>, Vec<PlanetError>> {
        let partition = partition.clone();
        let column_type_map = column_type_map.clone();
        let mut sorter_list = sorter_list.clone();
        let item_id = item.id.clone().unwrap();
        for (sorter_column_id, sorter_column_item) in sorter_map {
            let sorter_column_item = sorter_column_item.sorted_item.clone();
            let column_type = column_type_map.get(sorter_column_id);
            if column_type.is_none() {
                continue
            }
            let column_type = column_type.unwrap();
            let result = self.get_sort_value(
                item, 
                sorter_column_id, 
                column_type
            );
            if result.is_err() {
                let mut errors: Vec<PlanetError> = Vec::new();
                errors.push(
                    PlanetError::new(
                        500, 
                        Some(tr!("Error sorting query data.")),
                    )
                );
            }
            let sort_value = result.unwrap();
            let sorter_column_item = sorter_column_item.as_str();
            let mut sorter = SearchSorter::defaults(&partition, &item_id);
            let mut column_value = String::from("");
            let mut column_value_number: i64 = 0;
            if sort_value.str.is_some() {
                column_value = sort_value.str.unwrap();
                // Cap sorting strings to 100 bytes to reduce size
                if column_value.len() > SORT_MAX_STRING_LENGTH {
                    let slice = &column_value[0..SORT_MAX_STRING_LENGTH];
                    column_value = format!("{}...", slice);
                }
            } else {
                column_value_number = sort_value.number.unwrap();
            }
            // if sorter_column_item.find("number").is_some() {
            //     column_value_number = FromStr::from_str(column_value.as_str()).unwrap();
            // }
            match sorter_column_item {
                "column_id" => {
                    sorter.column_id = Some(column_value);
                },
                "column_1_str" => {
                    sorter.column_1_str = Some(column_value);
                },
                "column_2_str" => {
                    sorter.column_2_str = Some(column_value);
                },
                "column_3_str" => {
                    sorter.column_3_str = Some(column_value);
                },
                "column_4_str" => {
                    sorter.column_4_str = Some(column_value);
                },
                "column_5_str" => {
                    sorter.column_5_str = Some(column_value);
                },
                "column_6_str" => {
                    sorter.column_6_str = Some(column_value);
                },
                "column_7_str" => {
                    sorter.column_7_str = Some(column_value);
                },
                "column_8_str" => {
                    sorter.column_8_str = Some(column_value);
                },
                "column_9_str" => {
                    sorter.column_9_str = Some(column_value);
                },
                "column_10_str" => {
                    sorter.column_10_str = Some(column_value);
                },
                "column_1_number" => {
                    sorter.column_1_number = Some(column_value_number);
                },
                "column_2_number" => {
                    sorter.column_2_number = Some(column_value_number);
                },
                "column_3_number" => {
                    sorter.column_3_number = Some(column_value_number);
                },
                "column_4_number" => {
                    sorter.column_4_number = Some(column_value_number);
                },
                "column_5_number" => {
                    sorter.column_5_number = Some(column_value_number);
                },
                "column_6_number" => {
                    sorter.column_6_number = Some(column_value_number);
                },
                "column_7_number" => {
                    sorter.column_7_number = Some(column_value_number);
                },
                "column_8_number" => {
                    sorter.column_8_number = Some(column_value_number);
                },
                "column_9_number" => {
                    sorter.column_9_number = Some(column_value_number);
                },
                "column_10_number" => {
                    sorter.column_10_number = Some(column_value_number);
                },
                __ => {}
            }
            sorter_list.push(sorter);
        }
        return Ok(sorter_list.clone())
    }

    fn compare_strings(
        &self,
        mode: &SelectSortMode,
        column_a: &String,
        column_b: &String
    ) -> Ordering {
        let mode = mode.clone();
        let column_order: Ordering;
        match mode {
            SelectSortMode::Ascending => {
                column_order = column_a.cmp(&column_b);
            },
            SelectSortMode::Descending => {
                column_order = column_b.cmp(&column_a);
            },
        }
        return column_order
    }

    fn compare_numbers(
        &self,
        mode: &SelectSortMode,
        column_a: &i64,
        column_b: &i64
    ) -> Ordering {
        let mode = mode.clone();
        let column_order: Ordering;
        match mode {
            SelectSortMode::Ascending => {
                column_order = column_a.cmp(&column_b);
            },
            SelectSortMode::Descending => {
                column_order = column_b.cmp(&column_a);
            },
        }
        return column_order
    }

    fn sort(
        &self,
        sorter_list: &Vec<SearchSorter>,
        sorter_map: &HashMap<String, SortedtBy>,
    ) -> Vec<SearchSorter> {
        let mut sorter_list = sorter_list.clone();
        let only_id = sorter_map.len() == 1;
        // Case I only sort on ids
        if only_id {
            sorter_list.sort();
        } else {
            // I sort each column independently
            sorter_list.sort_by(|a, b| {
                let mut match_order = Ordering::Greater;
                for (_column_id, sorted_by) in sorter_map.clone() {
                    let mode = sorted_by.mode;
                    let sorter_item = sorted_by.sorted_item;
                    let sorter_item = sorter_item.as_str();
                    let mut column_order: Ordering = Ordering::Greater;
                    match sorter_item {
                        "column_1_str" => {
                            let column_a_1_str = a.column_1_str.clone().unwrap();
                            let column_b_1_str = b.column_1_str.clone().unwrap();
                            column_order = self.compare_strings(&mode, &column_a_1_str, &column_b_1_str);
                        },
                        "column_1_number" => {
                            let column_a_1_number = a.column_1_number.clone().unwrap();
                            let column_b_1_number = b.column_1_number.clone().unwrap();
                            column_order = self.compare_numbers(&mode, &column_a_1_number, &column_b_1_number);
                        },
                        "column_2_str" => {
                            let column_a_2_str = a.column_2_str.clone().unwrap();
                            let column_b_2_str = b.column_2_str.clone().unwrap();
                            column_order = self.compare_strings(&mode, &column_a_2_str, &column_b_2_str);
                        },
                        "column_2_number" => {
                            let column_a_2_number = a.column_2_number.clone().unwrap();
                            let column_b_2_number = b.column_2_number.clone().unwrap();
                            column_order = self.compare_numbers(&mode, &column_a_2_number, &column_b_2_number);
                        },
                        "column_3_str" => {
                            let column_a_3_str = a.column_3_str.clone().unwrap();
                            let column_b_3_str = b.column_3_str.clone().unwrap();
                            column_order = self.compare_strings(&mode, &column_a_3_str, &column_b_3_str);
                        },
                        "column_3_number" => {
                            let column_a_3_number = a.column_3_number.clone().unwrap();
                            let column_b_3_number = b.column_3_number.clone().unwrap();
                            column_order = self.compare_numbers(&mode, &column_a_3_number, &column_b_3_number);
                        },
                        "column_4_str" => {
                            let column_a_4_str = a.column_4_str.clone().unwrap();
                            let column_b_4_str = b.column_4_str.clone().unwrap();
                            column_order = self.compare_strings(&mode, &column_a_4_str, &column_b_4_str);
                        },
                        "column_4_number" => {
                            let column_a_4_number = a.column_4_number.clone().unwrap();
                            let column_b_4_number = b.column_4_number.clone().unwrap();
                            column_order = self.compare_numbers(&mode, &column_a_4_number, &column_b_4_number);
                        },
                        "column_5_str" => {
                            let column_a_5_str = a.column_5_str.clone().unwrap();
                            let column_b_5_str = b.column_5_str.clone().unwrap();
                            column_order = self.compare_strings(&mode, &column_a_5_str, &column_b_5_str);
                        },
                        "column_5_number" => {
                            let column_a_5_number = a.column_5_number.clone().unwrap();
                            let column_b_5_number = b.column_5_number.clone().unwrap();
                            column_order = self.compare_numbers(&mode, &column_a_5_number, &column_b_5_number);
                        },
                        "column_6_str" => {
                            let column_a_6_str = a.column_6_str.clone().unwrap();
                            let column_b_6_str = b.column_6_str.clone().unwrap();
                            column_order = self.compare_strings(&mode, &column_a_6_str, &column_b_6_str);
                        },
                        "column_6_number" => {
                            let column_a_6_number = a.column_6_number.clone().unwrap();
                            let column_b_6_number = b.column_6_number.clone().unwrap();
                            column_order = self.compare_numbers(&mode, &column_a_6_number, &column_b_6_number);
                        },
                        "column_7_str" => {
                            let column_a_7_str = a.column_7_str.clone().unwrap();
                            let column_b_7_str = b.column_7_str.clone().unwrap();
                            column_order = self.compare_strings(&mode, &column_a_7_str, &column_b_7_str);
                        },
                        "column_7_number" => {
                            let column_a_7_number = a.column_7_number.clone().unwrap();
                            let column_b_7_number = b.column_7_number.clone().unwrap();
                            column_order = self.compare_numbers(&mode, &column_a_7_number, &column_b_7_number);
                        },
                        "column_8_str" => {
                            let column_a_8_str = a.column_8_str.clone().unwrap();
                            let column_b_8_str = b.column_8_str.clone().unwrap();
                            column_order = self.compare_strings(&mode, &column_a_8_str, &column_b_8_str);
                        },
                        "column_8_number" => {
                            let column_a_8_number = a.column_8_number.clone().unwrap();
                            let column_b_8_number = b.column_8_number.clone().unwrap();
                            column_order = self.compare_numbers(&mode, &column_a_8_number, &column_b_8_number);
                        },
                        "column_9_str" => {
                            let column_a_9_str = a.column_9_str.clone().unwrap();
                            let column_b_9_str = b.column_9_str.clone().unwrap();
                            column_order = self.compare_strings(&mode, &column_a_9_str, &column_b_9_str);
                        },
                        "column_9_number" => {
                            let column_a_9_number = a.column_9_number.clone().unwrap();
                            let column_b_9_number = b.column_9_number.clone().unwrap();
                            column_order = self.compare_numbers(&mode, &column_a_9_number, &column_b_9_number);
                        },
                        "column_10_str" => {
                            let column_a_10_str = a.column_10_str.clone().unwrap();
                            let column_b_10_str = b.column_10_str.clone().unwrap();
                            column_order = self.compare_strings(&mode, &column_a_10_str, &column_b_10_str);
                        },
                        "column_10_number" => {
                            let column_a_10_number = a.column_10_number.clone().unwrap();
                            let column_b_10_number = b.column_10_number.clone().unwrap();
                            column_order = self.compare_numbers(&mode, &column_a_10_number, &column_b_10_number);
                        },
                        __ => {
                            // I might have column_id but I ignore since we only process SORT BY from query
                        }
                    }
                    match column_order {
                        Ordering::Greater => {},
                        Ordering::Less => {
                            match_order = Ordering::Less;
                            return match_order
                        },
                        Ordering::Equal => {
                            match_order = Ordering::Equal;
                            return match_order
                        },
                    }
                }
                return match_order
            });
        }
        return sorter_list
    }

    pub fn do_search(
        &self
    ) -> Result<Vec<SearchResultItem>, Vec<PlanetError>> {
        // 1 - Open all partitions
        // 2 - Execute Formula in all partitions using threads
        // 3 - Sorting in same thread formula was executed
        let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        let mut errors: Vec<PlanetError> = Vec::new();
        let query = self.query.where_compiled.clone();
        let folder_name = self.query.folder_name.clone();
        let space_database = self.space_database.clone();
        let planet_context = self.env.planet_context.clone();
        let context = self.env.context.clone();
        let home_dir = planet_context.home_path.clone();
        let account_id = context.account_id.clone().unwrap_or_default();
        let space_id = context.space_id;
        let site_id = context.site_id.clone();
        // TreeFolder
        let db_folder= TreeFolder::defaults(
            space_database.connection_pool.clone(),
            Some(home_dir.clone().unwrap_or_default().as_str()),
            Some(&account_id),
            Some(space_id),
            site_id.clone(),
        ).unwrap();
        let folder = db_folder.get_by_name(&folder_name);
        if folder.is_err() {
            let error = folder.unwrap_err();
            errors.push(error);
            return Err(errors)
        }
        let folder = folder.unwrap();
        if *&folder.is_none() {
            errors.push(
                PlanetError::new(
                    500, 
                    Some(tr!("Could not find folder {}", &folder_name)),
                )
            );
            return Err(errors)
        }
        let folder = folder.unwrap();
        let folder_id = folder.id.clone().unwrap_or_default();
        // Sorter
        let sort_by = self.query.sort_by.clone();
        let mut sorter_map: HashMap<String, SortedtBy> = HashMap::new();
        // Default sort by id, used in case no SORT BY defined
        let column = TreeFolder::get_column_by_name(
            &String::from(ID), 
            &folder
        ).unwrap();
        let column_id = column.get(ID).unwrap();
        let sorter_item = format!("column_{}", ID);
        let id_sorted = SortedtBy{
            sorted_item: sorter_item,
            mode: SelectSortMode::Ascending
        };
        sorter_map.insert(column_id.clone(), id_sorted);
        let mut sorter_list: Vec<SearchSorter> = Vec::new();
        let mut column_type_map: HashMap<String, String> = HashMap::new();
        if sort_by.is_some() {
            let sort_by = sort_by.unwrap();
            let mut column_sort_id = 1;
            for sort_by_item in sort_by {
                let column_name = sort_by_item.column;
                let sort_mode = sort_by_item.mode;
                if column_name.to_lowercase() == String::from("id") {
                    continue
                }
                let column = TreeFolder::get_column_by_name(
                    &column_name, 
                    &folder
                );
                if column.is_ok() {
                    let column = column.unwrap();
                    let column_id = column.get(ID).unwrap();
                    let column_sort_type = self.get_column_sort_type(&column);
                    column_type_map.insert(column_id.clone(), column_sort_type.clone());
                    let column_sort_type = column_sort_type.as_str();
                    let sorter_item = format!("column_{}_{}", &column_sort_id, column_sort_type);
                    let sorted_item = SortedtBy{
                        sorted_item: sorter_item,
                        mode: sort_mode
                    };
                    sorter_map.insert(column_id.clone(), sorted_item);
                    column_sort_id += 1;
                }
            }
        }
        // TreeFolderItem
        let mut site_id_alt: Option<String> = None;
        if site_id.is_some() {
            let site_id = site_id.clone().unwrap();
            site_id_alt = Some(site_id.clone().to_string());
        }
        let result: Result<TreeFolderItem, PlanetError> = TreeFolderItem::defaults(
            space_database.connection_pool.clone(),
            home_dir.clone().unwrap_or_default().as_str(),
            &account_id,
            space_id,
            site_id_alt,
            folder_id.as_str(),
            &db_folder,
        );
        if result.is_ok() {
            let mut folder_item = result.unwrap();
            let partitions = folder_item.get_partitions();
            if partitions.is_ok() {
                let partitions = partitions.unwrap();
                for partition in partitions {
                    let (db_tree, _index_tree) = folder_item.open_partition(&partition).unwrap();
                    // I may need botth db and index to execute formula
                    let iter = db_tree.iter();
                    for db_result in iter {
                        if db_result.is_err() {
                            let mut errors: Vec<PlanetError> = Vec::new();
                            errors.push(
                                PlanetError::new(
                                    500, 
                                    Some(tr!("Could not fetch item from database"))
                                )
                            );
                            return Err(errors)
                        }
                        let item_tuple = db_result.unwrap();
                        // let item_id = item_tuple.0;
                        let item = item_tuple.1;
                        let item_db = item.to_vec();
                        let item_ = EncryptedMessage::deserialize(
                            item_db
                        ).unwrap();
                        let item_ = DbData::decrypt_owned(
                            &item_, 
                            &shared_key);
                        let field_config_map = ColumnConfig::get_column_config_map(
                            &planet_context,
                            &context,
                            &folder
                        ).unwrap();
                        match item_ {
                            Ok(_) => {
                                let item = item_.unwrap();
                                // execute formula
                                if query.is_some() {
                                    let query = query.clone().unwrap();
                                    let data_map = item.clone().data.unwrap();
                                    // This will be used by SEARCH function, implemented when SEARCH is done
                                    // index_data_map
                                    let formula_result = execute_formula(
                                        &query, 
                                        &data_map, 
                                        &field_config_map
                                    );
                                    if formula_result.is_err() {
                                        let error = formula_result.unwrap_err();
                                        let mut errors: Vec<PlanetError> = Vec::new();
                                        errors.push(error);
                                        return Err(errors)
                                    }
                                    let formula_result = formula_result.unwrap();
                                    let formula_matches: bool;
                                    if formula_result == String::from("1") {
                                        formula_matches = true;
                                    } else {
                                        formula_matches = false;
                                    }
                                    eprintln!("SearchIterator.do_search :: formula_matches: {}", 
                                        &formula_matches
                                    );
                                    if formula_matches {                                
                                        let result = self.add_to_sorter(
                                            &partition,
                                            &item,
                                            &column_type_map,
                                            &sorter_map,
                                            &sorter_list
                                        );
                                        if result.is_ok() {
                                            let sorter_list_ = result.unwrap();
                                            sorter_list = sorter_list_;
                                        } else {
                                            let mut errors: Vec<PlanetError> = Vec::new();
                                            errors.push(
                                                PlanetError::new(500, Some(tr!(
                                                    "Could not sort values from query."
                                                )))
                                            );
                                            return Err(errors)
                                        }
                                    }
                                } else {
                                    // Add to sorting, since no where formula, we add all items
                                    let result = self.add_to_sorter(
                                        &partition,
                                        &item,
                                        &column_type_map,
                                        &sorter_map,
                                        &sorter_list
                                    );
                                    if result.is_ok() {
                                        let sorter_list_ = result.unwrap();
                                        sorter_list = sorter_list_;
                                    } else {
                                        let mut errors: Vec<PlanetError> = Vec::new();
                                        errors.push(
                                            PlanetError::new(500, Some(tr!(
                                                "Could not sort values from query."
                                            )))
                                        );
                                        return Err(errors)
                                    }
                                }
                            },
                            Err(_) => {
                                let mut errors: Vec<PlanetError> = Vec::new();
                                errors.push(
                                    PlanetError::new(500, Some(tr!(
                                        "Could not fetch item from database"
                                    )))
                                );
                                return Err(errors)
                            }
                        }
                    }
                }
            }
        }
        eprintln!("SearchIterator.do_search :: sorter_list: {:#?}", &sorter_list);
        sorter_list = self.sort(&sorter_list, &sorter_map);
        eprintln!("SearchIterator.do_search :: [sorted] sorter_list: {:#?}", &sorter_list);
        let mut result_list: Vec<SearchResultItem> = Vec::new();
        for sorter in sorter_list {
            let item = SearchResultItem{
                id: Some(sorter.id),
                partition: Some(sorter.partition),
            };
            result_list.push(item);
        }
        return Ok(result_list)
    }
}

#[derive(Debug, Clone)]
pub struct SearchPaging{
    pub number_items: u32,
    pub page: u32,
}
impl SearchPaging {

    pub fn do_paging(
        &self,
        results: &Vec<SearchResultItem>
    ) -> Result<Vec<SearchResultItem>, Vec<PlanetError>> {
        let results = results.clone();
        let mut paged_results: Vec<SearchResultItem> = Vec::new();
        let page = self.page.clone();
        let number_items = self.number_items;
        let start = (page-1)*number_items;
        let end = page*number_items;
        let mut count = 0;
        for item in results {
            if count >= end {
                break
            }
            if count >= start {
                paged_results.push(item);
            }
            count += 1;
        }
        return Ok(paged_results)
    }

}

#[derive(Debug, Clone)]
pub struct SelectFromFolderStatement {
}

impl<'gb> Statement<'gb> for SelectFromFolderStatement {

    fn run(
        &self,
        env: &'gb Environment<'gb>,
        space_database: &SpaceDatabase,
        statement_text: &String,
    ) -> Result<Vec<yaml_rust::Yaml>, Vec<PlanetError>> {
        let space_database = space_database.clone();
        // let t_1 = Instant::now();
        // 1 - Compile SELECT statement into SelectFromFolderCompiledStmt
        let search_compiler = SearchCompiler{
            statement_text: statement_text.clone(),
            env: env,
            space_database: space_database.clone()
        };
        let result = search_compiler.do_compile();
        if result.is_err() {
            let errors = result.unwrap_err();
            return Err(errors)
        }
        let statement = result.unwrap();
        // 2 - Execute search iterator that performs query, filtering and sorting
        let search_iterator = SearchIterator{
            env: env,
            space_database: space_database.clone(),
            query: statement.clone()
        };
        let results = search_iterator.do_search();
        if results.is_err() {
            let errors = results.unwrap_err();
            return Err(errors)
        }
        let results = results.unwrap();
        // 3 - Paging
        let paging = SearchPaging{
            number_items: statement.number_items,
            page: statement.page
        };
        let results = paging.do_paging(&results);
        if results.is_err() {
            let errors = results.unwrap_err();
            return Err(errors)
        }
        let results = results.unwrap();
        // 4 - Generate Final Data
        // Generate Output
        let mut errors: Vec<PlanetError> = Vec::new();
        let mut yaml_response: Vec<yaml_rust::Yaml> = Vec::new();
        let response_coded = serde_yaml::to_string("");
        if response_coded.is_err() {
            let error = PlanetError::new(
                500, 
                Some(tr!("Error encoding statement response.")),
            );
            errors.push(error);
            return Err(errors)
        }
        let response = response_coded.unwrap();
        let yaml_item = yaml_rust::YamlLoader::load_from_str(
            response.as_str()
        ).unwrap();
        yaml_response.push(yaml_item[0].clone());
        return Ok(yaml_response)
    }
}


pub struct SelectFromFolder<'gb> {
    pub planet_context: &'gb PlanetContext<'gb>,
    pub context: &'gb Context<'gb>,
    pub db_folder: TreeFolder,
    pub space_database: SpaceDatabase,
    pub config: SelectFromFolderConfig,
}

// impl<'gb> Statement<'gb> for SelectFromFolder<'gb> {

//     fn run(&self) -> Result<String, PlanetError> {
//         let command = self.config.command.clone().unwrap_or_default();
//         let expr = Regex::new(r#"(SELECT FROM TABLE) "(?P<folder_name>[a-zA-Z0-9_ ]+)""#).unwrap();
//         let table_name_match = expr.captures(&command).unwrap();
//         let folder_name = &table_name_match["folder_name"].to_string();
//         let folder_file = slugify(&folder_name);
//         let folder_file = folder_file.as_str().replace("-", "_");
//         eprintln!("SelectFromFolder.run :: folder_file: {}", &folder_file);

//         let home_dir = self.planet_context.home_path.unwrap_or_default();
//         let account_id = self.context.account_id.unwrap_or_default();
//         let space_id = self.context.space_id.unwrap_or_default();
//         let site_id = self.context.site_id.unwrap_or_default();
//         let box_id = self.context.box_id.unwrap_or_default();
//         let space_database = self.space_database.clone();
//         let result: Result<TreeFolderItem, PlanetError> = TreeFolderItem::defaults(
//             space_database.connection_pool,
//             home_dir,
//             account_id,
//             space_id,
//             site_id,
//             box_id,
//             folder_file.as_str(),
//             &self.db_folder,
//         );
//         match result {
//             Ok(_) => {
//                 let mut db_row: TreeFolderItem = result.unwrap();
//                 let config = self.config.clone();
//                 let r#where = config.r#where;
//                 let page = config.page;
//                 let number_items = config.number_items;
//                 let columns = config.columns;
//                 let mut page_wrap: Option<usize> = None;
//                 let mut number_items_wrap: Option<usize> = None;
//                 if page.is_some() {
//                     let page_string = page.unwrap();
//                     let page_number: usize = FromStr::from_str(page_string.as_str()).unwrap();
//                     page_wrap = Some(page_number)
//                 }
//                 if number_items.is_some() {
//                     let number_items_string = number_items.unwrap();
//                     let number_items: usize = FromStr::from_str(number_items_string.as_str()).unwrap();
//                     number_items_wrap = Some(number_items)
//                 }
//                 let result = db_row.select(
//                     folder_name, 
//                     r#where, 
//                     page_wrap, 
//                     number_items_wrap, 
//                     columns,
//                 )?;
//                 eprintln!("SelectFromFolder :: result: {:#?}", &result);
//                 // Later on, I do pretty print
//             },
//             Err(error) => {
//                 return Err(error);
//             }
//         }

//         return Ok(String::from(""));
//     }
//     // fn runner(runner: &CommandRunner, path_yaml: &String) -> () {
//     //     let config_ = SelectFromFolderConfig::defaults(
//     //         None,
//     //         None,
//     //         None
//     //     );
//     //     let config: Result<SelectFromFolderConfig, Vec<PlanetValidationError>> = config_.import(
//     //         runner.planet_context,
//     //         &path_yaml
//     //     );
//     //     match config {
//     //         Ok(_) => {
//     //             let home_dir = runner.planet_context.home_path.unwrap_or_default();
//     //             let account_id = runner.context.account_id.unwrap_or_default();
//     //             let space_id = runner.context.space_id.unwrap_or_default();
//     //             let site_id = runner.context.site_id.unwrap_or_default();
//     //             let result = SpaceDatabase::defaults(
//     //                 Some(site_id), 
//     //                 space_id, 
//     //                 Some(home_dir),
//     //                 Some(false)
//     //             );
//     //             if result.is_err() {
//     //                 let error = result.clone().unwrap_err();
//     //                 println!();
//     //                 println!("{}", tr!("I found these errors").red().bold());
//     //                 println!("{}", "--------------------".red());
//     //                 println!();
//     //                 println!(
//     //                     "{} {}", 
//     //                     String::from('.').blue(),
//     //                     error.message
//     //                 );
//     //             }
//     //             let space_database = result.unwrap();
//     //             let db_folder= TreeFolder::defaults(
//     //                 space_database.connection_pool.clone(),
//     //                 Some(home_dir),
//     //                 Some(account_id),
//     //                 Some(space_id),
//     //                 Some(site_id),
//     //             ).unwrap();

//     //             let select_from_table: SelectFromFolder = SelectFromFolder{
//     //                 planet_context: runner.planet_context,
//     //                 context: runner.context,
//     //                 config: config.unwrap(),
//     //                 db_folder: db_folder.clone(),
//     //                 space_database: space_database.clone()
//     //             };
//     //             let result: Result<_, PlanetError> = select_from_table.run();
//     //             match result {
//     //                 Ok(_) => {
//     //                     println!();
//     //                     println!("{}", String::from("[OK]").green());
//     //                 },
//     //                 Err(error) => {
//     //                     let count = 1;
//     //                     println!();
//     //                     println!("{}", tr!("I found these errors").red().bold());
//     //                     println!("{}", "--------------------".red());
//     //                     println!();
//     //                     println!(
//     //                         "{}{} {}", 
//     //                         count.to_string().blue(),
//     //                         String::from('.').blue(),
//     //                         error.message
//     //                     );
//     //                 }
//     //             }
//     //         },
//     //         Err(errors) => {
//     //             println!();
//     //             println!("{}", tr!("I found these errors").red().bold());
//     //             println!("{}", "--------------------".red());
//     //             println!();
//     //             let mut count = 1;
//     //             for error in errors {
//     //                 println!(
//     //                     "{}{} {}", 
//     //                     count.to_string().blue(), 
//     //                     String::from('.').blue(), 
//     //                     error.message
//     //                 );
//     //                 count += 1;
//     //             }
//     //         }
//     //     }
//     // }
// }

fn handle_field_response(
    column_data: &Result<Vec<String>, Vec<PlanetError>>, 
    errors: &Vec<PlanetError>, 
    column_id: &String,
    data: &BTreeMap<String, Vec<BTreeMap<String, String>>>,
    is_set: &String
) -> (
    BTreeMap<String, Vec<BTreeMap<String, String>>>,
    Vec<PlanetError>
) {
    let column_data = column_data.clone();
    eprintln!("handle_field_response :: column_id: {} column_data: {:?}", column_id, column_data);
    let mut errors = errors.clone();
    let mut data = data.clone();
    let column_id = column_id.clone();
    let is_set = is_set.clone();
    if column_data.is_err() {
        let err = column_data.unwrap_err();
        for error in err {
            errors.push(error);
        }
    } else {
        let column_data = column_data.unwrap().clone();
        if is_set == FALSE.to_string() {
            // into data
            if column_data.clone().len() == 0 {
                let error = PlanetError::new(
                    500, 
                    Some(tr!("Content is empty for column id \"{}\", no data.", column_id))
                );
                errors.push(error);
            } else {
                let column_value = column_data[0].clone();
                data.insert(column_id.clone(), build_value_list(&column_value));
            }
        } else {
            // into data_collections, I have a set
            let mut list: Vec<BTreeMap<String, String>> = Vec::new();
            for item in column_data {
                let mut map: BTreeMap<String, String> = BTreeMap::new();
                map.insert(VALUE.to_string(), item);
                list.push(map);
            }
            data.insert(column_id.clone(), list);
        }
    }
    return (data, errors)
}

pub fn resolve_data_statement(
    env: &Environment,
    space_data: &SpaceDatabase,
    statement_text: &String, 
    response_wrap: Option<Result<Vec<yaml_rust::Yaml>, Vec<PlanetError>>>,
    column_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
    mode: &StatementCallMode
) -> Option<Result<Vec<yaml_rust::Yaml>, Vec<PlanetError>>> {
    let response_wrap = response_wrap.clone();
    if response_wrap.is_some() {
        let response = response_wrap.unwrap();
        return Some(response)
    }
    let statement_text = substitute_variables(statement_text, &env, column_map.clone());
    // INSERT INTO FOLDER
    let expr = &RE_INSERT_INTO_FOLDER_MAIN;
    let check = expr.is_match(&statement_text);
    if check {
        let stmt = InsertIntoFolderStatement{};
        match mode {
            StatementCallMode::Run => {
                let response = stmt.run(
                    env, 
                    &space_data, 
                    &statement_text,
                );
                return Some(response);
            }
            StatementCallMode::Compile => {
                let response = stmt.compile(&statement_text);
                if response.is_err() {
                    let errors = response.unwrap_err();
                    return Some(Err(errors))
                }
            }
        }
    }
    // SELECT FROM FOLDER
    let expr = &RE_SELECT;
    let check = expr.is_match(&statement_text);
    if check {
        let stmt = SelectFromFolderStatement{};
        match mode {
            StatementCallMode::Run => {
                let response = stmt.run(
                    env, 
                    &space_data, 
                    &statement_text,
                );
                return Some(response);
            }
            StatementCallMode::Compile => {
                let compiler = SearchCompiler{
                    statement_text: statement_text.clone(),
                    env: env,
                    space_database: space_data.clone()
                };
                let response = compiler.do_compile();
                if response.is_err() {
                    let errors = response.unwrap_err();
                    return Some(Err(errors))
                }
            }
        }
    }
    return None
}