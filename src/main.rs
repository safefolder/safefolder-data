extern crate argparse;
extern crate xid;
extern crate serde_yaml;
extern crate colored;

pub mod commands;
pub mod storage;
pub mod planet;

use colored::*;
use crate::commands::table::config::CreateTableConfig;
use crate::commands::CommandRunner;
use argparse::{ArgumentParser, StoreTrue, Store};
use tr::tr;
use std::collections::HashMap;

use crate::commands::table::Command;
use crate::planet::{PlanetContext, Context, validation::PlanetValidationError};

fn main() {

    // achiever run command ...
    // achiever run action ...
    // achiever run journey ...

    let mut verbose = false;
    let mut account_id = String::from("");
    let mut space_id= String::from("");
    let mut path_yaml = String::from("");
    let mut command = String::from("");
    let mut op = String::from("run");
    let mut scope = String::from("");

    { // this block limits scope of borrows by ap.refer() method
        let mut ap = ArgumentParser::new();
        ap.set_description("Achiever Client Tool");
        ap.refer(&mut verbose).add_option(
            &["-v", "--verbose"], 
            StoreTrue,
            "Be verbose");
        ap.refer(&mut account_id).add_option(
            &["-a", "--accountid"], Store,
            "Account Id");
        ap.refer(&mut space_id).add_option(
            &["-s", "--spaceid"], Store,
            "Space Id");
        ap.refer(&mut path_yaml).add_option(
            &["-c", "--path_yaml"], Store,
            "Path for YAML config file");
        ap.refer(&mut op).add_argument("op", Store, "Operation: run, etc...");
        ap.refer(&mut scope).add_argument("scope", Store, "Scope: command, action, journey");
        ap.refer(&mut command).add_argument("name", Store, "Command name, action name or journey name");
        ap.parse_args_or_exit();
    }

    // Get planet context. I embed planet context into components
    let planet_context: PlanetContext = PlanetContext::import_context().unwrap();

    // Context: This is TEMP, simply context struct, but in future will come from shell, or we create a new one
    let context: Context = Context{
        id: None,
        data: HashMap::new(),
    };

    if op.to_lowercase() == "run" && &scope.to_lowercase() == "command" {
        let command_runner: CommandRunner = CommandRunner{
            planet_context: &planet_context,
            context: &context,
            command: &command,
            space_id: Some(&space_id),
            account_id: Some(&account_id),
            path_yaml: Some(&path_yaml)
        };
        run_command(&command_runner).unwrap();
    }
}

fn run_command(runner: &CommandRunner) -> Result<String, String> {
    // CommandRunner: command, account_id, space_id, path_yaml, possible command_file (when get from dir), planet context
    // I also need to create a context if not informed.
    if Some(&runner.path_yaml).is_some() {
        let path_yaml: String;
        if runner.path_yaml.is_none() {
            // In future case that we don't send path through shell
            path_yaml = String::from("");
        } else {
            path_yaml = runner.path_yaml.unwrap().to_string();
        }
        match runner.command {
            "CREATE TABLE" => {
                let config: Result<CreateTableConfig, Vec<PlanetValidationError>> = 
                    CreateTableConfig::import(&runner.planet_context, &path_yaml);
                // println!("main :: config: {:#?}", config);
                match config {
                    Ok(_) => {
                        let create_table = commands::table::schema::CreateTable{
                            planet_context: runner.planet_context,
                            context: runner.context,
                            config: config.unwrap(),
                            account_id: runner.account_id,
                            space_id: runner.space_id,
                        };
                        // println!("{:?}", create_table_config);
                        // create_table.validate().unwrap();
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
            },
            _ => println!("default")
        }
        Ok("Command executed".to_string())
    } else {
        return Err("Path to YAML command not informed".to_string());
    }
}