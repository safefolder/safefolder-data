pub mod folder;
pub mod constants;

use yaml_rust;
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;
use regex::Regex;
use tr::tr;

use crate::planet::constants::*;
use crate::planet::{PlanetError, Environment};
use crate::storage::space::*;
use crate::functions::date::*;

use crate::statements::folder::schema::resolve_schema_statement;
use crate::statements::folder::data::resolve_data_statement;

lazy_static! {
    pub static ref RE_WITH_OPTIONS: Regex = Regex::new(r#"(?P<Name>\w+)=(?P<Value>(\d)|(true|false|True|False)|([a-zA-Z0-9{}|]+)|([\s\S]+)|("[\w\s]+)")"#).unwrap();
    pub static ref RE_OPTION_LIST_ITEMS: Regex = Regex::new(r#"(?P<Item>((\d+)|([a-zA-Z0-9]+)|(true|false|True|False)|(---\\n[\S\s]+)|(null)))"#).unwrap();
    pub static ref RE_DATA_LONG_TEXT: Regex = Regex::new(r#"(?P<Text>"""[\s\S\n\t][^"""]+""")"#).unwrap();
    pub static ref RE_STMT_VARIABLES: Regex = Regex::new(r#"(?P<Var>{[\w\s.]+})"#).unwrap();
}

pub enum StatementCallMode {
    Run,
    Compile
}

pub struct StatementResponse {
    pub status: String,
    pub status_code: usize,
    pub data: BTreeMap<String, Vec<BTreeMap<String, String>>>
}

// Run needs entry statement_text, and response YAML generic object, or JsonValue
pub trait Statement<'gb> {
    fn run(
        &self,
        env: &'gb Environment<'gb>,
        space_database: &SpaceDatabase,
        statement_text: &String,
    ) -> Result<Vec<yaml_rust::Yaml>, Vec<PlanetError>>;
}

pub trait StatementCompiler<'gb, T> {
    fn compile(
        &self, 
        statement_text: &String
    ) -> Result<T, Vec<PlanetError>>;
}

pub trait StatementCompilerBulk<'gb, T> {
    fn compile(
        &self, 
        statement_text: &String
    ) -> Result<Vec<T>, Vec<PlanetError>>;
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

    pub fn call(
        &self,
        env: &Environment,
        space_database: Option<SpaceDatabase>,
        statement_text: &String, 
        column_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
        mode: &StatementCallMode,
    ) -> Result<String, Vec<PlanetError>> {
        let space_data: SpaceDatabase;
        let context = env.context;
        let planet_context = env.planet_context;
        let column_map = column_map.clone();

        if space_database.is_none() {
            let site_id = context.site_id;
            let space_id = context.space_id;
            let space_id = space_id.unwrap();
            let home_dir = planet_context.home_path.unwrap_or_default();
            let result = SpaceDatabase::defaults(
                site_id, 
                space_id, 
                Some(home_dir),
            );
            if result.is_err() {
                let error = result.unwrap_err();
                let mut errors: Vec<PlanetError> = Vec::new();
                errors.push(error);
                return Err(errors)
            }
            space_data = result.unwrap();
        } else {
            space_data = space_database.unwrap();
        }
        let mut response_str = String::from("");
        let mut response_wrap: Option<Result<Vec<yaml_rust::Yaml>, Vec<PlanetError>>> = None;
        // Process all statements from all modules
        response_wrap = resolve_schema_statement(
            env, &space_data, statement_text, response_wrap, column_map.clone(), mode
        );
        response_wrap = resolve_data_statement(
            env, &space_data, statement_text, response_wrap, column_map.clone(), mode
        );
        if response_wrap.is_none() {
            let error = PlanetError::new(
                500, 
                Some(tr!("Statement not supported."))
            );
            let mut errors: Vec<PlanetError> = Vec::new();
            errors.push(error);
            return Err(errors)
        }
        let response = response_wrap.unwrap();
        if response.is_ok() {
            // TODO: Implement for JSON and XML
            let response_list = response.unwrap();
            for response in response_list {
                let mut emitter = yaml_rust::YamlEmitter::new(&mut response_str);
                emitter.dump(&response).unwrap();
                response_str = format!("{}\n", response_str);
            }
            // eprintln!("StatementRunner.run :: response encoded: {}", &response_str);
        } else {
            // I don't abort db transactions, since I do not own the db connection, only return errors
            let errors = response.unwrap_err();
            return Err(errors);
        }
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
                // eprintln!("WithOptionValueItem.defaults :: value: {}", &value);
                obj = Self{
                    value: value.clone(),
                    item_type: WithOptionValueItemType::String
                };
            }    
        }
        // eprintln!("WithOptionValueItem.defaults :: obj: {:#?}", &obj);
        return Ok(obj)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DataValue {
    pub value: Vec<BTreeMap<String, String>>,
}
impl DataValue {
    pub fn defaults(value_str: &String) -> Result<Self, PlanetError> {
        let value = value_str.as_str();
        let expr_items = &RE_OPTION_LIST_ITEMS;
        let value_string = value.replace("\"", "");
        let value = value_string.as_str();
        let mut map_list: Vec<BTreeMap<String, String>> = Vec::new();
        if value.find("{").is_some() {
            let items = expr_items.captures_iter(&value);
            for item in items {
                let value = item.name("Item").unwrap().as_str();
                let option_value = WithOptionValueItem::defaults(&value.to_string());
                if option_value.is_ok() {
                    let option_value = option_value.unwrap();
                    let mut my_map: BTreeMap<String, String> = BTreeMap::new();
                    
                    my_map.insert(VALUE.to_string(), option_value.value);
                    map_list.push(my_map);
                } else {
                    return Err(
                        PlanetError::new(
                            500, 
                            Some(tr!("Error parsing data value \"{}\".", value_str)),
                        )
                    )
                }
            }
        } else {
            let check_yaml = value.find("---\\n");
            if check_yaml.is_some() {
                let fields: Vec<&str> = value.split("---\\n").collect();
                let my_fields = &fields[1..];
                for field in my_fields {
                    let mut my_map: BTreeMap<String, String> = BTreeMap::new();
                    let field_ = format!("---\\n{}", field);
                    my_map.insert(VALUE.to_string(), field_.clone());
                    map_list.push(my_map);
                }
            } else {
                let mut my_map: BTreeMap<String, String> = BTreeMap::new();
                my_map.insert(VALUE.to_string(), value.to_string());
                map_list.push(my_map);
            }
        }
        let obj = Self{
            value: map_list
        };
        return Ok(obj)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DataValueLongText {
    pub parsed_text: String,
    pub map: HashMap<String, String>,
}
impl DataValueLongText {
    pub fn defaults(text: &String) -> Result<Self, PlanetError> {
        // Parse and replace """....""" as placeholder and register into map
        let expr = &RE_DATA_LONG_TEXT;
        let text_items = expr.captures_iter(text);
        let mut count = 1;
        let mut parsed_text: String = text.clone();
        let mut map: HashMap<String, String> = HashMap::new();
        for text_item in text_items {
            let text = text_item.name("Text");
            if text.is_some() {
                let text_source = text.unwrap().as_str();
                let text = text_source.replace("\"\"\"", "");
                let placeholder = format!("$AL__text_{}", &count);
                parsed_text = parsed_text.replace(text_source, placeholder.as_str());
                map.insert(placeholder.clone(), text);
            }
            count += 1;
        }
        let obj = Self{
            parsed_text: parsed_text,
            map: map,
        };
        return Ok(obj)
    }
    pub fn has_placeholder(text: &String) -> bool {
        // text might be a list of placeholders, like {$AL__text_1|$AL__text_2}
        let text = text.clone();
        let check = text.find("$AL__text_");
        eprintln!("DataValueLongText.has_placeholder :: text: {} check: {}", &text, &check.is_some());
        return check.is_some() 
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

pub fn substitute_variables(
    statement_text: &String, 
    env: &Environment, 
    column_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>
) -> String {
    let env = env.clone();
    let context = env.context;
    let column_map_wrap = column_map.clone();
    let column_map: BTreeMap<String, Vec<BTreeMap<String, String>>>;
    if column_map_wrap.is_some() {
        column_map = column_map_wrap.unwrap();
    } else {
        column_map = BTreeMap::new();
    }
    let mut statement_text = statement_text.clone();
    let statement_str = statement_text.clone();
    let statement_str = statement_str.as_str();
    let expr = &RE_STMT_VARIABLES;
    let variables = expr.captures_iter(statement_str);
    for variable in variables {
        // Check variable is included in column map or contexts and replace by value
        let var = variable.name("Var");
        if var.is_some() {
            let var = var.unwrap().as_str();
            let to_replace = format!("{{{}}}", var);
            let to_replace = to_replace.as_str();
            // Column map in case of statements used in statement column
            if column_map.len() != 0 {
                let map_var = column_map.get(var);
                if map_var.is_some() {
                    let map_var = map_var.unwrap();
                    for map in map_var {
                        let value = map.get(VALUE);
                        if value.is_some() {
                            let value = value.unwrap();
                            statement_text = statement_text.replace(to_replace, value.as_str());
                        }
                    }
                }
            }
            // Context
            let ctx_data = context.data.clone();
            if ctx_data.is_some() {
                let ctx_data = ctx_data.unwrap().clone();
                let ctx_item = ctx_data.get(var);
                if ctx_item.is_some() {
                    let ctx_item = ctx_item.unwrap().clone();
                    statement_text = statement_text.replace(to_replace, ctx_item.as_str());
                }
            }
        }
    }
    return statement_text
}
