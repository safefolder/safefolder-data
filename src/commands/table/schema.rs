extern crate tr;
extern crate colored;

use tr::tr;
use colored::*;
use std::io;

use crate::commands::table::config::CreateTableConfig;
use crate::commands::table::{Command};
use crate::commands::CommandRunner;
use crate::storage::table::{DbTable, DbTableConfig};
use crate::planet::{
    PlanetContext, 
    Context, 
    validation::PlanetValidationError
};

pub struct CreateTable<'a> {
    pub planet_context: PlanetContext,
    pub context: &'a Context,
    pub config: CreateTableConfig,
    pub account_id: Option<&'a str>,
    pub space_id: Option<&'a str>,
}

impl<'a> Command<DbTable> for CreateTable<'a> {

    fn run(&self) -> Result<DbTable, io::Error> {
        // Insert into account "tables" the config
        let config: DbTableConfig = DbTableConfig{
            language: self.config.language.clone(),
            fields: self.config.fields.clone(),
        };
        let db_table: DbTable = DbTable::defaults(&String::from("My Table"), config);
        let result = db_table.create();
        match result {
            Ok(_) => {
                let response = result.unwrap();
                Ok(response)
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    fn runner(runner: CommandRunner, path_yaml: String) -> () {
        let planet_context: PlanetContext = runner.planet_context.clone();
        let config_ = CreateTableConfig::defaults(path_yaml, planet_context);
        let config: Result<CreateTableConfig, Vec<PlanetValidationError>> = config_.import();
        // let config: Result<CreateTableConfig, Vec<PlanetValidationError>> = 
        // CreateTableConfig::import(&runner.planet_context, &path_yaml);
        match config {
            Ok(_) => {
                let create_table: CreateTable = CreateTable{
                    planet_context: runner.planet_context.clone(),
                    context: runner.context,
                    config: config.unwrap(),
                    account_id: runner.account_id,
                    space_id: runner.space_id,
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

