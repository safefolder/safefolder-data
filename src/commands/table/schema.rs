extern crate tr;
extern crate colored;

use tr::tr;
use colored::*;

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
            &self.planet_context,
            &self.context,
        );
        match result {
            Ok(_) => {
                // I need to grab this through Regex, the table name embedded into the command
                let table_name = String::from("My Table");
                let schema_data: SchemaData = SchemaData::defaults(&table_name, &config);
                let db_table = result.unwrap();
                let result = db_table.create(&schema_data);
                match result {
                    Ok(_) => {
                        let response: SchemaData = result.unwrap();
                        Ok(response)
                    },
                    Err(_) => {
                        Err(
                            PlanetError::new(
                                500, 
                                Some(tr!("Could not create table")),
                            )
                        )
                    }
                }        
            },
            Err(_) => {
                Err(
                    PlanetError::new(
                        500, 
                        Some(tr!("Could not create table")),
                    )
                )
            }
        }
    }

    fn runner(runner: &CommandRunner, path_yaml: &String) -> () {
        let config_ = CreateTableConfig::defaults(&path_yaml, runner.planet_context);
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
                create_table.run();
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

