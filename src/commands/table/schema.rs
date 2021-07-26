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
            self.planet_context,
            self.context,
        );
        match result {
            Ok(_) => {
                //TODO: I need to grab this through Regex, the table name embedded into the command
                let table_name = String::from("My Table");
                let account_id = self.context.account_id.unwrap_or_default();
                let space_id = self.context.space_id.unwrap_or_default();
                let schema_data: SchemaData = SchemaData::defaults(
                    &table_name, 
                    &config,
                    account_id,
                    space_id,
                );
                let db_table: DbTable<'gb> = result.unwrap();

                let response = db_table.create(&schema_data)?;
                Ok(response)
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
                        println!("runner :: I could create table");
                    },
                    Err(error) => {
                        println!("runner :: Error: {:?}", error.message);
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

