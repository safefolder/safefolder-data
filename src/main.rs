extern crate argparse;

pub mod commands;

use argparse::{ArgumentParser, StoreTrue, Store};
use std::fs;

use crate::commands::data_tables::schema::Command;

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

    if op.to_lowercase() == "run" && &scope.to_lowercase() == "command" {
        run_command(&command, &account_id, &space_id, &path_yaml).unwrap();
    }

}

fn run_command(command: &str, account_id: &str, space_id: &str, path_yaml: &str) -> Result<String, String> {
    if Some(&path_yaml).is_some() {
        let yaml_config = fs::read_to_string(&path_yaml)
        .expect("Something went wrong reading the YAML file");
        // TODO: I will need soon a registry of commands in a table, and I grab from there in order to execute commands,
        // so commands con grow. Commands would be registered into the registry. I need to do after the permission
        // and security system implemented, since needs to be signed. So far we just map the commands.
        // Also, think about a folder where to grab commands, actions and journeys, so full path is not neccesary
        // "CREATE TABLE" => commands::data_tables::schema::create_table(&account_id, &space_id, &yaml_config),
        match command {
            "CREATE TABLE" => {
                let schema_name = "CREATE TABLE".to_lowercase().replace(" ", "_");
                let create_table = commands::data_tables::schema::CreateTable{
                    schema_name: &schema_name,
                    account_id: &account_id,
                    space_id: &space_id,
                    yaml_config: &yaml_config,
                };
                create_table.validate(&yaml_config, &schema_name);
                create_table.run();
            },
            _ => println!("default")
        }
        Ok("Command executed".to_string())
    } else {
        return Err("Path to YAML command not informed".to_string());
    }
}