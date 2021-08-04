extern crate tr;
extern crate colored;
extern crate slug;

use std::collections::HashMap;

use tr::tr;
use colored::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use slug::slugify;


use crate::commands::table::config::{InsertIntoTableConfig, FieldConfig};
use crate::commands::table::{Command};
use crate::commands::{CommandRunner};
use crate::storage::table::{DbTable, DbRow, Row, RowData, RoutingData, Schema, RowItem, DbData};
use crate::storage::table::*;
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

        // Insert into account "tables" the config
        // let config: DbTableConfig = DbTableConfig{
        //     language: self.config.language.clone(),
        //     fields: self.config.fields.clone(),
        // };
        
        let result: Result<DbRow<'gb>, PlanetError> = DbRow::defaults(
            &table_file,
            self.planet_context,
            self.context,
        );
        match result {
            Ok(_) => {
                let account_id = self.context.account_id.unwrap_or_default().to_string();
                let space_id = self.context.space_id.unwrap_or_default().to_string();
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
                eprintln!("InsertIntoTable.run :: table: {:#?}", table);
                // let mut row_data: RowData = RowData::defaults(&account_id, &space_id);
                // I need a way to get list of instance FieldConfig (fields)
                // FieldConfig.parse_db(db_data: DbData) -> Vec<FieldConfig>
                // let fields: Vec<FieldConfig> = table.config.fields.unwrap();
                // let mut insert_data: HashMap<String, RowItem> = HashMap::new();
                // let data_map = self.config.data.clone().unwrap();
                // for field in fields {
                //     let field_ = field.clone();
                //     let field_type = field.field_type.unwrap_or_default();
                //     let field_type = field_type.as_str();
                //     match field_type {
                //         "Small Text" => {
                //             insert_data = SmallTextField::process(&data_map, &field_, insert_data)?;
                //         },
                //         _ => {
                //         }
                //     }
                // }
                // row_data.data = Some(insert_data);
                // // Insert RowData into database
                // let response: RowData = db_row.insert(&row_data)?;
                // // let response_src = response.clone();
                // return Ok(response);

                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!("This is a dumb message")),
                    )
                );
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