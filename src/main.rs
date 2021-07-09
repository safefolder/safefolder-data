extern crate argparse;
extern crate xid;
extern crate serde_yaml;

pub mod commands;
pub mod storage;

use argparse::{ArgumentParser, StoreTrue, Store};
use std::fs;

use crate::commands::table::Command;

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

    // This is a test of only field config, having validations
    // let field_config = FieldConfig{
    //     id: "xxxxxxxxxxxxxxxxxxx",
    //     name: Some("Company Name"),
    //     field_type: Some("SmallText"),
    //     ..FieldConfig::defaults()
    // };
    // match field_config.is_valid() {
    //     Ok(_) => println!("Went fine"),
    //     Err(e) => println!("Had an error {}", e)
    // }

    // I take YAML file and convert into a CreateTableConfig
    // if Some(&path_yaml).is_some() {
    //     let yaml_config = fs::read_to_string(&path_yaml)
    //     .expect("Something went wrong reading the YAML file");
    //     println!("YAML config: {}", yaml_config);
    //     // Create struct from YAML file
    //     // let deserialized_map: BTreeMap<String, f64> = serde_yaml::from_str(&s)?;
    //     let create_table: CreateTableConfig = serde_yaml::from_str(&yaml_config).unwrap();
    //     println!("{:?}", create_table);
    //     match create_table.is_valid() {
    //         Ok(_) => println!("Went fine"),
    //         Err(e) => println!("Had an error {}", e)
    //     }
    // }
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
                let create_table = commands::table::schema::CreateTable{
                    config: serde_yaml::from_str(&yaml_config).unwrap(),
                    account_id: &account_id,
                    space_id: &space_id,
                };
                create_table.validate().unwrap();
                create_table.run();
            },
            _ => println!("default")
        }
        Ok("Command executed".to_string())
    } else {
        return Err("Path to YAML command not informed".to_string());
    }
}