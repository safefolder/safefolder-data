extern crate tr;
extern crate colored;
extern crate slug;

use std::collections::HashMap;

use tr::tr;
use colored::*;
use regex::Regex;
use slug::slugify;


use crate::commands::table::config::{
    InsertIntoTableConfig, 
    GetFromTableConfig,
    FieldConfig
};
use crate::commands::table::{Command};
use crate::commands::{CommandRunner};
use crate::storage::table::{DbTable, DbRow, Row, Schema, DbData};
use crate::storage::table::*;
use crate::storage::ConfigStorageField;
use crate::planet::{
    PlanetContext, 
    PlanetError,
    Context,
    validation::PlanetValidationError,
};
use crate::storage::fields::*;

pub struct InsertIntoTable<'gb> {
    pub planet_context: &'gb PlanetContext<'gb>,
    pub context: &'gb Context<'gb>,
    pub config: InsertIntoTableConfig,
}

impl<'gb> Command<DbData> for InsertIntoTable<'gb> {

    fn run(&self) -> Result<DbData, PlanetError> {
        let command = self.config.command.clone().unwrap_or_default();
        let expr = Regex::new(r#"(INSERT INTO TABLE) "(?P<table_name>[a-zA-Z0-9_ ]+)"#).unwrap();
        let table_name_match = expr.captures(&command).unwrap();
        let table_name = &table_name_match["table_name"].to_string();
        let table_file = slugify(&table_name);
        let table_file = table_file.as_str().replace("-", "_");

        let result: Result<DbRow<'gb>, PlanetError> = DbRow::defaults(
            &table_file,
            self.planet_context,
            self.context,
        );
        match result {
            Ok(_) => {
                // let data_config = self.config.data.clone();
                let db_row: DbRow<'gb> = result.unwrap();
                // I need to get SchemaData and schema for the table
                // I go through fields in order to build RowData
                let db_table: DbTable = DbTable::defaults(
                    self.planet_context,
                    self.context,
                )?;
                let table = db_table.get_by_name(table_name)?;
                if *&table.is_none() {
                    return Err(
                        PlanetError::new(
                            500, 
                            Some(tr!("Could not find table {}", &table_name)),
                        )
                    );
                }

                // routing
                let account_id = Some(self.context.account_id.unwrap_or_default().to_string());
                let space_id = Some(self.context.space_id.unwrap_or_default().to_string());
                let routing_wrap = RoutingData::defaults(
                    account_id, 
                    space_id, 
                    None
                );
                let table = table.unwrap();
                eprintln!("InsertIntoTable.run :: table: {:#?}", &table);
                // I need a way to get list of instance FieldConfig (fields)
                let config_fields = FieldConfig::parse_from_db(&table);
                eprintln!("InsertIntoTable.run :: config_fields: {:#?}", &config_fields);
                
                let insert_data_map: HashMap<String, String> = self.config.data.clone().unwrap();
                eprintln!("InsertIntoTable.run :: insert_data_map: {:#?}", &insert_data_map);
                // let insert_data_collections_map = self.config.data_collections.clone().unwrap();
                // eprintln!("InsertIntoTable.run :: insert_data__collections_map: {:#?}", &insert_data_collections_map);
                // TODO: Change for the item name
                // We will use this when we have the Name field, which is required in all tables
                eprintln!("InsertIntoTable.run :: routing_wrap: {:#?}", &routing_wrap);
                let name: &String = &String::from("hello");
                let mut db_data = DbData::defaults(
                    name,
                    None,
                    None,
                    None,
                    None,
                    routing_wrap,
                    None,
                );
                for field in config_fields {
                    let field_config = field.clone();
                    let field_type = field.field_type.unwrap_or_default();
                    let field_type = field_type.as_str();
                    eprintln!("InsertIntoTable.run :: field_type: {}", &field_type);
                    match field_type {
                        "Small Text" => {
                            db_data = SmallTextField::init_do(&field_config, insert_data_map.clone(), db_data)?
                        },
                        "Long Text" => {
                            db_data = LongTextField::init_do(&field_config, insert_data_map.clone(), db_data)?
                        },
                        "Checkbox" => {
                            db_data = CheckBoxField::init_do(&field_config, insert_data_map.clone(), db_data)?
                        },
                        "Number" => {
                            db_data = NumberField::init_do(&field_config, insert_data_map.clone(), db_data)?
                        },
                        "Select" => {
                            db_data = SelectField::init_do(&field_config, &table, insert_data_map.clone(), db_data)?
                        },
                        _ => {
                            return Ok(db_data);
                        }
                    };
                }
                eprintln!("InsertIntoTable.run :: I will write: {:#?}", &db_data);
                let response: DbData = db_row.insert(&db_data)?;
                return Ok(response);
            },
            Err(error) => {
                return Err(error);
            }
        }
    }

    fn runner(runner: &CommandRunner, path_yaml: &String) -> () {
        let config_ = InsertIntoTableConfig::defaults();
        let config: Result<InsertIntoTableConfig, Vec<PlanetValidationError>> = config_.import(
            runner.planet_context,
            &path_yaml
        );
        match config {
            Ok(_) => {
                let insert_into_table: InsertIntoTable = InsertIntoTable{
                    planet_context: runner.planet_context,
                    context: runner.context,
                    config: config.unwrap(),
                };
                let result: Result<_, PlanetError> = insert_into_table.run();
                match result {
                    Ok(_) => {
                        println!();
                        println!("{}", String::from("[OK]").green());
                    },
                    Err(error) => {
                        let count = 1;
                        println!();
                        println!("{}", tr!("I found these errors").red().bold());
                        println!("{}", "--------------------".red());
                        println!();
                        println!(
                            "{}{} {}", 
                            count.to_string().blue(),
                            String::from('.').blue(),
                            error.message
                        );
                    }
                }
            },
            Err(errors) => {
                println!();
                println!("{}", tr!("I found these errors").red().bold());
                println!("{}", "--------------------".red());
                println!();
                let mut count = 1;
                for error in errors {
                    println!(
                        "{}{} {}", 
                        count.to_string().blue(), 
                        String::from('.').blue(), 
                        error.message
                    );
                    count += 1;
                }
            }
        }
    }
}

pub struct GetFromTable<'gb> {
    pub planet_context: &'gb PlanetContext<'gb>,
    pub context: &'gb Context<'gb>,
    pub config: GetFromTableConfig,
}

impl<'gb> Command<String> for GetFromTable<'gb> {

    fn run(&self) -> Result<String, PlanetError> {
        let command = self.config.command.clone().unwrap_or_default();
        let expr = Regex::new(r#"(GET FROM TABLE) "(?P<table_name>[a-zA-Z0-9_ ]+)""#).unwrap();
        let table_name_match = expr.captures(&command).unwrap();
        let table_name = &table_name_match["table_name"].to_string();
        let table_file = slugify(&table_name);
        let table_file = table_file.as_str().replace("-", "_");

        let result: Result<DbRow<'gb>, PlanetError> = DbRow::defaults(
            &table_file,
            self.planet_context,
            self.context,
        );
        match result {
            Ok(_) => {
                // let data_config = self.config.data.clone();
                let db_row: DbRow<'gb> = result.unwrap();
                // I need to get SchemaData and schema for the table
                // I go through fields in order to build RowData                
                let db_table: DbTable = DbTable::defaults(
                    self.planet_context,
                    self.context,
                )?;
                let table = db_table.get_by_name(table_name)?;
                if *&table.is_none() {
                    return Err(
                        PlanetError::new(
                            500, 
                            Some(tr!("Could not find table {}", &table_name)),
                        )
                    );
                }
                let table = table.unwrap();
                let config_fields = FieldConfig::parse_from_db(&table);
                let field_id_map: HashMap<String, FieldConfig> = FieldConfig::get_field_id_map(&config_fields);
                // routing
                let account_id = Some(self.context.account_id.unwrap_or_default().to_string());
                let space_id = Some(self.context.space_id.unwrap_or_default().to_string());
                let routing_wrap = RoutingData::defaults(
                    account_id, 
                    space_id, 
                    None
                );
                let item_id = self.config.data.clone().unwrap().id.unwrap();
                // Get item from database
                let db_data = db_row.get(&item_id)?;
                // data and basic fields
                let data = db_data.data;
                let mut yaml_out_str = String::from("---\n");
                if data.is_some() {
                    // field_id -> string value
                    let data = data.unwrap();
                    // I need to go through in same order as fields were registered in FieldConfig when creating schema
                    for field_id in data.keys() {
                        let field_config = field_id_map.get(field_id).unwrap().clone();
                        let field_config_ = field_config.clone();
                        let field_type = field_config.field_type.unwrap();
                        let field_type =field_type.as_str();
                        // Get will return YAML document for the data
                        match field_type {
                            "Small Text" => {
                                yaml_out_str = SmallTextField::init_get(&field_config_, Some(&data), &yaml_out_str)?;
                            },
                            "Long Text" => {
                                yaml_out_str = LongTextField::init_get(&field_config_, Some(&data), &yaml_out_str)?;
                            },
                            "Checkbox" => {
                                yaml_out_str = CheckBoxField::init_get(&field_config_, Some(&data), &yaml_out_str)?;
                            },
                            "Number" => {
                                yaml_out_str = NumberField::init_get(&field_config_, Some(&data), &yaml_out_str)?;
                            },
                            "Select" => {
                                yaml_out_str = SelectField::init_get(&field_config_, &table, Some(&data), &yaml_out_str)?
                            },
                            _ => {
                                yaml_out_str = yaml_out_str;
                            }
                        }
                    }
                }
                eprintln!("{}", yaml_out_str);
                return Ok(yaml_out_str);
                // return Err(
                //     PlanetError::new(
                //         500, 
                //         Some(tr!("Hey man!")),
                //     )
                // );
            },
            Err(error) => {
                return Err(error);
            }
        }
    }

    fn runner(runner: &CommandRunner, path_yaml: &String) -> () {
        let config_ = GetFromTableConfig::defaults(
            String::from("")
        );
        let config: Result<GetFromTableConfig, Vec<PlanetValidationError>> = config_.import(
            runner.planet_context,
            &path_yaml
        );
        match config {
            Ok(_) => {
                let insert_into_table: GetFromTable = GetFromTable{
                    planet_context: runner.planet_context,
                    context: runner.context,
                    config: config.unwrap(),
                };
                let result: Result<_, PlanetError> = insert_into_table.run();
                match result {
                    Ok(_) => {
                        println!();
                        println!("{}", String::from("[OK]").green());
                    },
                    Err(error) => {
                        let count = 1;
                        println!();
                        println!("{}", tr!("I found these errors").red().bold());
                        println!("{}", "--------------------".red());
                        println!();
                        println!(
                            "{}{} {}", 
                            count.to_string().blue(),
                            String::from('.').blue(),
                            error.message
                        );
                    }
                }
            },
            Err(errors) => {
                println!();
                println!("{}", tr!("I found these errors").red().bold());
                println!("{}", "--------------------".red());
                println!();
                let mut count = 1;
                for error in errors {
                    println!(
                        "{}{} {}", 
                        count.to_string().blue(), 
                        String::from('.').blue(), 
                        error.message
                    );
                    count += 1;
                }
            }
        }
    }
}