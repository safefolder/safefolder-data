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

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct PlanetContextSource {
    pub mission: String,
    pub home_path: Option<String>,
}

// impl<'gb> PlanetContextSource {
//     fn get_ref(&self) -> &'gb PlanetContext<'gb> {
//         let planet_context: &'gb PlanetContext<'gb> = PlanetContext{
//             mission: "",
//             home_path: Some(""),
//         };
//         return planet_context
//     }
// }

impl<'gb> PlanetContextSource {
    pub fn get_ref(&self, mission: &'gb String, home_path: &'gb String) -> PlanetContext<'gb> {
        let planet_context: PlanetContext<'gb> = PlanetContext{
            mission: mission,
            home_path: Some(home_path.as_str()),
        };
        return planet_context
    }
}

#[derive(Debug, Clone)]
pub struct PlanetContext<'gb> {
    pub mission: &'gb str,
    pub home_path: Option<&'gb str>,
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

}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContextSource {
    pub id: Option<String>,
    pub data: Option<HashMap<String, String>>,
    pub account_id: Option<String>,
    pub space_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Context<'gb> {
    pub id: Option<&'gb String>,
    pub data: Option<&'gb HashMap<String, String>>,
    pub account_id: Option<&'gb String>,
    pub space_id: Option<&'gb String>,
}

impl<'gb> ContextSource {
    fn get_ref(&self) -> Context<'gb> {
        let context: Context<'gb> = Context{
            id: None,
            data: None,
            account_id: None,
            space_id: None,
        };
        return context
    }
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
        let mut my_message: Option<String> = Some(String::from(""));
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
