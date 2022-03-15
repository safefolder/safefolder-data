pub mod validation;
pub mod constants;

extern crate serde_yaml;
extern crate colored;
extern crate dirs;
extern crate xid;

use serde::{Deserialize, Serialize};
use validator::{Validate};

use tr::tr;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
// use colored::*;

// use crate::statements::Statement;

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct PlanetContextSource {
    pub mission: String,
    pub home_path: Option<String>,
}

// pub trait StatementRegistry {
// }

// #[derive(Debug)]
// pub struct StatementRegistryItem<'gb> {
//     pub title: Option<String>,
//     pub description: Option<String>,
//     pub key: String,
//     pub keywords: Option<Vec<String>>,
//     pub category: Option<String>,
//     pub statement: dyn Statement<'gb>,
// }

#[derive(Debug, Clone)]
pub struct PlanetContext<'gb> {
    pub mission: &'gb str,
    pub home_path: Option<&'gb str>,
    // pub statement_registry: HashMap<String, dyn StatementRegistry>
}

impl<'gb> PlanetContext<'gb> {
    pub fn import_context() -> Result<PlanetContextSource, io::Error> {
        let app_path = env::current_dir();
        if app_path.is_err() {
            return Err(app_path.unwrap_err())
        }
        let app_path_str = app_path.unwrap();
        let app_path_str = app_path_str.to_str().unwrap();
        let path_planet_context: &str = &*format!("{}/planet_context.yaml", app_path_str);
        if Some(path_planet_context).is_some() {
            let planet_context_str = fs::read_to_string(&path_planet_context)
            .expect("Something went wrong reading the YAML file");
            let mut planet_context_source: PlanetContextSource = serde_yaml::from_str(&planet_context_str).unwrap();
            let sys_home_dir = dirs::home_dir().unwrap();
            let sys_home_dir_str = sys_home_dir.as_os_str().to_str().unwrap();
            planet_context_source.home_path = Some(format!("{home_dir}/.achiever-planet", home_dir=sys_home_dir_str));
            // println!("PlanetContext.import_context :: planet_context: {:#?}", planet_context);
            // let mine = planet_context_source.as_ref();
            return Ok(planet_context_source)
        } else {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, 
                &*tr!("Could not import planet context")))
        }
    }

    // pub fn register_statement(
    //     &self,
    //     statement: &dyn Statement,
    //     title: Option<String>,
    //     description: Option<String>,
    //     key: &String,
    //     keywords: Option<String>,
    //     category: Option<String>,
    // ) {
    //     let item: dyn StatementRegistry = StatementRegistryItem {
    //         title: title,
    //         description: description,
    //         key: key.clone(),
    //         statement: statement,
    //         keywords: keywords,
    //         category: category,
    //     };
    //     self.statement_registry.insert(key.clone(), item);
    // }

    // fn resolve_statement_key(&self, statement_text: &String) -> Result<String, PlanetError> {
    //     // I get all obhects from registry and check their keys and try to find in statement_text
    //     // Check I only have one match, return error in case we have multiple items
    //     return Ok(String::from(""))
    // }

    // pub fn get_statement(
    //     &self,
    //     statement_text: &String,
    // ) -> Result<dyn Statement, PlanetError> {
    //     let statement_key = self.resolve_statement_key(statement_text);
    //     if statement_key.is_err() {
    //         let error = statement_key.unwrap_err();
    //         return Err(error);
    //     }
    //     let statement_key = statement_key.unwrap();
    //     let statement_registry = self.statement_registry;
    //     let statement = statement_registry.get(&statement_key);
    //     if statement.is_some() {
    //         let statement_item: StatementRegistryItem = statement.unwrap();
    //         let statement = statement_item.statement;
    //         return Ok(statement)
    //     } else {
    //         let snippet = &statement_text[..100];
    //         return Err(
    //             PlanetError::new(
    //                 500, 
    //                 Some(
    //                     tr!("Statement not found in registry: \"{}\"...", snippet)
    //                 ),
    //             )
    //         );
    //     }
    // }

}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContextSource {
    pub id: Option<String>,
    pub data: Option<HashMap<String, String>>,
    pub account_id: Option<String>,
    pub space_id: Option<String>,
    pub box_id: Option<String>,
    pub site_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Context<'gb> {
    pub id: Option<&'gb str>,
    pub data: Option<&'gb HashMap<String, String>>,
    pub account_id: Option<&'gb str>,
    pub space_id: Option<&'gb str>,
    pub box_id: Option<&'gb str>,
    pub site_id: Option<&'gb str>,
}

#[derive(Debug, Clone)]
pub struct Environment<'gb> {
    pub context: &'gb Context<'gb>,
    pub planet_context: &'gb PlanetContext<'gb>,
}

#[derive(Debug, Clone)]
pub struct PlanetError {
    pub error_code: u16,
    pub reason: String,
    pub message: String,
}

pub const REASON_INTERNAL_ERROR: &str= "INTERNAL_ERROR";
pub const REASON_NOT_FOUND: &str= "NOT_FOUND";
pub const REASON_OK: &str= "OK";

impl PlanetError {
    pub fn new(error_code: u16, message: Option<String>) -> PlanetError {
        let mut error_reasons: HashMap<u16, (&str, String)> = HashMap::new();
        error_reasons.insert(200, (REASON_OK, tr!("Ok")));
        error_reasons.insert(500, (REASON_INTERNAL_ERROR, tr!("Internal Error")));
        error_reasons.insert(404, (REASON_NOT_FOUND, tr!("Not Found")));
        let reason_tuple = error_reasons.get(&error_code);
        let reason: &str = reason_tuple.unwrap().0;
        let reason_message: &String = &reason_tuple.unwrap().1;
        let my_message: Option<String>;
        if message.is_none() {
            my_message = Some(reason_message.to_string());
        } else {
            my_message = message;
        };
        let error = PlanetError{
            error_code: error_code,
            reason: reason.to_string(),
            message: my_message.unwrap()
        };
        return error
    }
}

pub fn make_bool_str(literal: String) -> bool {
    let mut check: bool = true;
    if literal.to_lowercase() == "false" {
        check = false
    }
    return check
}
