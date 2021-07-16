extern crate tr;
extern crate colored;

use tr::tr;
use colored::*;

use crate::commands::table::config::CreateTableConfig;
use crate::commands::table::{Command};
use crate::commands::CommandRunner;
use crate::planet::{
    PlanetContext, 
    Context, 
    validation::PlanetValidationError
};

pub struct CreateTable<'a> {
    pub planet_context: &'a PlanetContext,
    pub context: &'a Context,
    pub config: CreateTableConfig,
    pub account_id: Option<&'a str>,
    pub space_id: Option<&'a str>,
}

impl<'a> Command for CreateTable<'a> {

    fn run(&self) -> () {
        // Insert into account "tables" the config
        println!("I run create table....");
    }

    fn runner(runner: &CommandRunner, path_yaml: &String) -> () {
        let config: Result<CreateTableConfig, Vec<PlanetValidationError>> = 
        CreateTableConfig::import(&runner.planet_context, &path_yaml);
        match config {
            Ok(_) => {
                let create_table: CreateTable = CreateTable{
                    planet_context: runner.planet_context,
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

