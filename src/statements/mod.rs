pub mod folder;
pub mod constants;

use std::collections::{BTreeMap};
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;
use regex::Regex;
use tr::tr;

use crate::planet::{PlanetError, Environment};
use crate::storage::space::*;
use crate::statements::folder::schema::CreateFolderStatement;
use crate::functions::date::*;

lazy_static! {
    pub static ref RE_WITH_OPTIONS: Regex = Regex::new(r#"(?P<Name>\w+)=(?P<Value>(\d)|(true|false|True|False)|([a-zA-Z0-9{}|]+)|("[\w\s]+)")"#).unwrap();
    pub static ref RE_OPTION_LIST_ITEMS: Regex = Regex::new(r#"(?P<Item>((\d+)|([a-zA-Z0-9]+)|(true|false|True|False)))"#).unwrap();
}

pub struct StatementResponse {
    pub status: String,
    pub status_code: usize,
    pub data: BTreeMap<String, Vec<BTreeMap<String, String>>>
}

// Run needs entry statement_text, and response YAML generic object, or JsonValue
pub trait Statement<'gb, T> {
    fn compile(
        &self, 
        statement_text: &String
    ) -> Result<T, Vec<PlanetError>>;
    fn run(
        &self,
        env: &'gb Environment<'gb>,
        space_database: &SpaceDatabase,
        statement_text: &String,
    ) -> Result<yaml_rust::Yaml, Vec<PlanetError>>;
}

pub trait StatementErrors<T> {
    fn run(&self) -> Result<T, Vec<PlanetError>>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StatementType {
    CreateFolder,
    InsertIntoFolder
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatementConfig {
    pub statement_type: StatementType,
    pub data: BTreeMap<String, Vec<BTreeMap<String, String>>>
}

impl StatementConfig {

    pub fn defaults(statement_type: &StatementType, data: &BTreeMap<String, Vec<BTreeMap<String, String>>>) -> Self {
        let config= Self{
            statement_type: statement_type.clone(),
            data: data.clone()
        };
        return config
    }
}

#[derive(Debug, PartialEq)]
struct MyBullshitError;

pub enum StatementResponseFormat {
    YAML,
    JSON,
    XML
}

pub struct StatementRunner {
    pub response_format: StatementResponseFormat
}

impl StatementRunner {

    pub fn run(
        &self,
        env: &Environment,
        space_database: Option<SpaceDatabase>,
        statement_text: &String, 
    ) -> Result<String, Vec<PlanetError>> {
        let space_data: SpaceDatabase;
        let context = env.context;
        let planet_context = env.planet_context;

        let create_folder = CreateFolderStatement{};
        let mut response_str = String::from("");

        if space_database.is_none() {
            let site_id = context.site_id;
            let space_id = context.space_id;
            let space_id = space_id.unwrap();
            let home_dir = planet_context.home_path.unwrap_or_default();
            let result = SpaceDatabase::defaults(
                site_id, 
                space_id, 
                Some(home_dir),
                Some(true)
            );
            if result.is_err() {
    
            }
            space_data = result.unwrap();
        } else {
            space_data = space_database.unwrap();
        }
        let response = create_folder.run(
            env, 
            &space_data,
            statement_text
        );
        if response.is_ok() {
            // TODO: Implement for JSON and XML?????
            let response = response.unwrap();
            let mut emitter = yaml_rust::YamlEmitter::new(&mut response_str);
            emitter.dump(&response).unwrap();
            // eprintln!("StatementRunner.run :: response encoded: {}", &response_str);
        } else {
            // I don't abort db transactions, since I do not own the db connection, only return errors
            let errors = response.unwrap_err();
            return Err(errors);
        }
        // let errors: Vec<PlanetError>;
        
        // I need to map transaction and link transaction objects to space database....
        // Since this is temp, until I have the statement registry, just map the private space

        // I need the workspace db, space db and optionally

        // I might have transactions already in the space database. This is the case when the connection
        // is owned by the journeys. In this case we don't wrap the transaction
        
        // let mine = planet_context.get_statement(statement_text);
        // I execute mine.run(environment, statement_text)
        // if statement_text.find(STATEMENT_CREATE_FOLDER).is_some() {
        //     // This response needs to be generic one, YAML based as object
        //     let mine = CreateFolderStatement{
        //     }.run();
        //     let my_object = CreateFolderStatement{
        //     };
        //     let mut my_map: HashMap<String, dyn Statement> = HashMap::new();
        //     my_map.insert(String::from("key"), my_object);
        //     // Since it is working I can have the statement registry mapped into the planet context
        //     // 1. I can register statements when system starts and place into planet context. I might
        //     //      also serialize the planet context, but better be just in time on real time JIT.
        //     // 2. I need to attach data to statements when registering, like:
        //     //      * title
        //     //      * key: This is like id, for example, CREATE FOLDER, INSERT INTO FOLDER, etc...
        //     //      * description
        //     //      * keywords
        //     //      * category
        //     // 3. Have a rust function that returns statement key based on statement text.
        //     // 4. Fetch statement object from registry at planet context.
        // } else if statement_text.find(STATEMENT_INSERT_INTO_FOLDER).is_some() {
        //     // statements::folder::data::InsertIntoFolder::runner(&runner)
        // }else {
        //     let errors: Vec<PlanetError> = Vec::new();
        //     return Err(
        //         errors
        //     )
        // }
        // I handle data from generic structure in case I need to return errors
        // Of Ok, simply return some string saying was executed, or some data from generic structure implemented in
        // statement.
        // deal with errrors and return error as object encoded
        Ok(response_str)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum WithOptionValueItemType {
    String,
    Number,
    Bool,
    Date,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WithOptionValueItem {
    pub value: String,
    pub item_type: WithOptionValueItemType,
}
impl WithOptionValueItem {
    pub fn defaults(value: &String) -> Result<Self, PlanetError> {
        // Check value and define item_type
        let obj: WithOptionValueItem;
        let mut value = value.clone();
        // Number
        let result = i32::from_str(&value);
        if result.is_ok() {
            let result = result.unwrap().to_string();
            obj = Self{
                value: result,
                item_type: WithOptionValueItemType::Number
            };            
        } else {
            let mut is_date: bool = false;
            let date_01_obj_wrap = get_date_object_iso(&value.to_string());
            let date_02_obj_wrap = get_date_object_human_time(&value.to_string());
            let date_03_obj_wrap = get_date_object_only_date(&value.to_string());
            if date_01_obj_wrap.is_ok() || date_02_obj_wrap.is_ok() || date_03_obj_wrap.is_ok() {
                is_date = true;
            }
            if value.to_lowercase() == "true" {
                // True
                obj = Self{
                    value: String::from("True"),
                    item_type: WithOptionValueItemType::Bool
                };
            } else if value.to_lowercase() == "false" {
                // False
                obj = Self{
                    value: String::from("False"),
                    item_type: WithOptionValueItemType::Bool
                };
            } else if is_date {
                // Date
                obj = Self{
                    value: value.clone(),
                    item_type: WithOptionValueItemType::Date
                };
            } else {
                // String
                if value.find("\"").is_some() {
                    value = value.replace("\"", "");
                }
                obj = Self{
                    value: value.clone(),
                    item_type: WithOptionValueItemType::String
                };
            }    
        }
        return Ok(obj)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WithOptions {
    pub options: BTreeMap<String, Vec<WithOptionValueItem>>,
}
impl WithOptions {
    pub fn defaults(options_str: &String) -> Result<Self, PlanetError> {
        // I have options without WITH, key1=value1 key2=value2
        let expr = &RE_WITH_OPTIONS;
        let expr_items = &RE_OPTION_LIST_ITEMS;
        let mut options: BTreeMap<String, Vec<WithOptionValueItem>> = BTreeMap::new();
        // Process options string
        let items = expr.captures_iter(options_str);
        for item in items {
            let name = item.name("Name").unwrap().as_str();
            let value = item.name("Value").unwrap().as_str();
            let value_string = value.replace("\"", "");
            let value = value_string.as_str();
            if value.find("{").is_some() {
                // list
                let items = expr_items.captures_iter(&value);
                let mut list: Vec<WithOptionValueItem> = Vec::new();
                for item in items {
                    let value = item.name("Item").unwrap().as_str();
                    let option_value = WithOptionValueItem::defaults(&value.to_string());
                    if option_value.is_ok() {
                        let option_value = option_value.unwrap();
                        list.push(option_value);
                    } else {
                        return Err(
                            PlanetError::new(
                                500, 
                                Some(tr!("Error parsing statement: With list options \"{}\"", options_str)),
                            )
                        )
                    }
                }
                options.insert(name.to_string(), list);
            } else {
                // String, Number or Boolean
                let option_value = WithOptionValueItem::defaults(&value.to_string());
                let mut list: Vec<WithOptionValueItem> = Vec::new();
                if option_value.is_ok() {
                    let option_value = option_value.unwrap();
                    list.push(option_value);
                    options.insert(name.to_string(), list);
                } else {
                    return Err(
                        PlanetError::new(
                            500, 
                            Some(tr!("Error parsing statement: With option \"{}\"", options_str)),
                        )
                    )
                }
            }
        }
        let obj = Self{
            options: options,
        };
        return Ok(obj)
    }
    pub fn get_single_value(&self, key: &str) -> String {
        let with_options = self.options.clone();
        let item = *&with_options.get(key);
        if item.is_some() {
            let item = item.unwrap();
            let item = item.clone();
            if item.len() == 1 {
                let item_value = &item[0].value;
                return item_value.clone()
            }
        }
        return String::from("");
    }
}
