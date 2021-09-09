extern crate tr;
extern crate colored;

use std::collections::HashMap;

use tr::tr;
use colored::*;
use regex::Regex;
use validator::Contains;

use crate::commands::table::config::{CreateTableConfig};
use crate::commands::table::{Command};
use crate::commands::{CommandRunner};
use crate::commands::table::constants::*;
use crate::planet::constants::ID;
use crate::storage::{ConfigStorageField};
use crate::storage::table::{DbTable, Schema, DbData, RoutingData};
use crate::planet::{
    PlanetContext, 
    PlanetError,
    Context, 
    validation::PlanetValidationError,
};

pub struct CreateTable<'gb> {
    pub planet_context: &'gb PlanetContext<'gb>,
    pub context: &'gb Context<'gb>,
    pub config: CreateTableConfig,
}

impl<'gb> Command<DbData> for CreateTable<'gb> {

    fn run(&self) -> Result<DbData, PlanetError> {
        let result: Result<DbTable, PlanetError> = DbTable::defaults(
            self.planet_context,
            self.context,
        );
        match result {
            Ok(_) => {
                let command = self.config.command.clone().unwrap_or_default();
                let expr = Regex::new(r#"(CREATE TABLE) "(?P<table_name>[a-zA-Z0-9_ ]+)"#).unwrap();
                let table_name_match = expr.captures(&command).unwrap();
                let table_name = &table_name_match["table_name"].to_string();
                let config = self.config.clone();

                // db table options with language data
                let mut data: HashMap<String, String> = HashMap::new();
                let language = config.language.unwrap();
                let language_codes_list = language.codes.unwrap();
                let language_codes_str = language_codes_list.join(",");
                let language_default = language.default;
                data.insert(String::from(LANGUAGE_CODES), language_codes_str);
                data.insert(String::from(LANGUAGE_DEFAULT), language_default);

                // config data
                let mut data_objects: HashMap<String, HashMap<String, String>> = HashMap::new();
                let mut data_collections: HashMap<String, Vec<HashMap<String, String>>> = HashMap::new();
                let mut fields = config.fields.unwrap().clone();
                let mut field_ids: Vec<HashMap<String, String>> = Vec::new();

                // name field
                let name_field_config = config.name.unwrap();
                fields.insert(0, name_field_config);
                let mut field_name_map: HashMap<String, bool> = HashMap::new();
                for field in fields.iter() {
                    // field simple attributes
                    let field_attrs = field.clone();
                    let mut field_id_map: HashMap<String, String> = HashMap::new();
                    field_id_map.insert(String::from(ID), field_attrs.id.unwrap());
                    &field_ids.push(field_id_map);
                    let field_name = field_attrs.name.unwrap_or_default().clone();
                    let field_name_str = &field_name.as_str();
                    if field_name_map.has_element(field_name_str) == false {
                        field_name_map.insert(field_name.clone(), true);
                    } else {
                        return Err(
                            PlanetError::new(
                                500, 
                                Some(tr!("There is already a field with name \"{}\"", &field_name)),
                            )
                        );                        
                    }
                    let map = &field.map_object_db()?;
                    data_objects.insert(String::from(field_name.clone()), map.clone());
                    // field complex attributes like select_data
                    let map_list = &field.map_collections_db();
                    let map_list = map_list.clone();
                    // data_collections = map_list.clone();
                    data_collections.extend(map_list);
                }
                data_collections.insert(String::from(FIELD_IDS), field_ids);
                // routing
                let account_id = Some(self.context.account_id.unwrap_or_default().to_string());
                let space_id = Some(self.context.space_id.unwrap_or_default().to_string());
                let routing_wrap = RoutingData::defaults(
                    account_id, 
                    space_id, 
                    None
                );
                let mut data_wrap: Option<HashMap<String, String>> = None;
                let mut data_collections_wrap: Option<HashMap<String, Vec<HashMap<String, String>>>> = None;
                let mut data_objects_wrap: Option<HashMap<String, HashMap<String, String>>> = None;
                if data.len() > 0 {
                    data_wrap = Some(data);
                }
                if data_collections.len() > 0 {
                    data_collections_wrap = Some(data_collections);
                }
                if data_objects.len() > 0 {
                    data_objects_wrap = Some(data_objects);
                }
                let db_data: DbData = DbData::defaults(
                    &table_name, 
                    data_wrap,
                    data_collections_wrap,
                    data_objects_wrap,
                    None,
                    routing_wrap,
                    None,
                )?;
                // Onl output TEMP the choices data to include in insert
                let mut mine = db_data.clone().data_collections.unwrap();
                mine.remove("field_ids");
                eprintln!("CreateTable.run :: db_data: {:#?}", mine);

                let db_table: DbTable = result.unwrap();

                let response: DbData = db_table.create(&db_data)?;
                let response_src = response.clone();
                // response.id
                let table_name = &response.name.unwrap_or_default();
                let table_id = &response.id.unwrap();

                println!();
                let quote_color = format!("{}", String::from("\""));
                println!("Created table {} :: {} => {}",
                    format!("{}{}{}", &quote_color.blue(), &table_name.blue(), &quote_color.blue()),
                    &table_id.magenta(),
                    format!("{}{}{}", &quote_color.green(), &table_name.green(), &quote_color.green()),
                );

                Ok(response_src)
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    fn runner(runner: &CommandRunner, path_yaml: &String) -> () {
        let config_ = CreateTableConfig::defaults(None);
        let config: Result<CreateTableConfig, Vec<PlanetValidationError>> = config_.import(
            runner.planet_context,
            &path_yaml
        );
        match config {
            Ok(_) => {
                let create_table: CreateTable = CreateTable{
                    planet_context: runner.planet_context,
                    context: runner.context,
                    config: config.unwrap(),
                };
                let result = create_table.run();
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
