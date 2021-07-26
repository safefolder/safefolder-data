extern crate tr;
extern crate colored;

use tr::tr;
use colored::*;
use regex::Regex;

use crate::commands::table::config::{CreateTableConfig, DbTableConfig};
use crate::commands::table::{Command};
use crate::commands::{CommandRunner};
use crate::storage::table::{DbTable, Schema, SchemaData};
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

impl<'gb> Command<SchemaData> for CreateTable<'gb> {

    fn run(&self) -> Result<SchemaData, PlanetError> {
        // Insert into account "tables" the config
        let config: DbTableConfig = DbTableConfig{
            language: self.config.language.clone(),
            fields: self.config.fields.clone(),
        };
        let result: Result<DbTable<'gb>, PlanetError> = DbTable::defaults(
            self.planet_context,
            self.context,
        );
        match result {
            Ok(_) => {
                let command = self.config.command.clone().unwrap_or_default();
                let expr = Regex::new(r#"(CREATE TABLE) "(?P<table_name>[a-zA-Z0-9_ ]+)"#).unwrap();
                let table_name_match = expr.captures(&command).unwrap();
                let table_name = &table_name_match["table_name"].to_string();
                let account_id = self.context.account_id.unwrap_or_default();
                let space_id = self.context.space_id.unwrap_or_default();
                let schema_data: SchemaData = SchemaData::defaults(
                    &table_name, 
                    &config,
                    account_id,
                    space_id,
                );
                let db_table: DbTable<'gb> = result.unwrap();

                let response: SchemaData = db_table.create(&schema_data)?;
                let response_src = response.clone();
                // response.id
                let table_name = &response.name;
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
        let config_ = CreateTableConfig::defaults();
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

