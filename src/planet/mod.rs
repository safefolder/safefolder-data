pub mod validation;
pub mod constants;

extern crate serde_yaml;
extern crate colored;
extern crate dirs;
extern crate xid;

use serde::{Deserialize, Serialize};
use validator::Validate;

use tr::tr;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;

use crate::storage::constants::PRIVATE;


#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct StatementRegistryItem {
    pub title: Option<String>,
    pub description: Option<String>,
    pub key: String,
    pub keywords: Option<Vec<String>>,
    pub category: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct PlanetContextSource {
    pub mission: String,
    pub site_id: String,
    pub space_id: String,
    pub home_path: Option<String>,
    pub statements: Vec<StatementRegistryItem>,
}
impl PlanetContextSource {
    pub fn import_context() -> Result<PlanetContextSource, io::Error> {
        let app_path = env::current_dir();
        if app_path.is_err() {
            return Err(app_path.unwrap_err())
        }
        let app_path_str = app_path.unwrap();
        let app_path_str = app_path_str.to_str().unwrap();
        let path_planet_context: &str = &*format!("{}/planet_context.yaml", app_path_str);
        let planet_context_str = fs::read_to_string(&path_planet_context)
            .expect("Something went wrong reading the YAML file");
        let mut planet_context_source: PlanetContextSource = serde_yaml::from_str(&planet_context_str).unwrap();
        let sys_home_dir = dirs::home_dir().unwrap();
        let sys_home_dir_str = sys_home_dir.as_os_str().to_str().unwrap();
        let home_path = format!("{home_dir}/.safefolder", home_dir=sys_home_dir_str).clone();
        planet_context_source.home_path = Some(home_path);
        // eprintln!("PlanetContextSource.import_context :: planet_context_source: {:#?}", &planet_context_source);
        return Ok(planet_context_source)
    }
}

#[derive(Debug, Clone)]
pub struct PlanetContext<'gb> {
    pub mission: &'gb str,
    pub site_id: &'gb str,
    pub space_id: &'gb str,
    pub home_path: Option<String>,
    pub statements: Vec<StatementRegistryItem>,
}
impl<'gb> PlanetContext<'gb> {
    pub fn import(planet_context_source: &'gb PlanetContextSource) -> PlanetContext<'gb> {
        let planet_context: PlanetContext<'gb> = PlanetContext{
            mission: &planet_context_source.mission,
            site_id: &planet_context_source.site_id,
            space_id: &planet_context_source.space_id,
            home_path: planet_context_source.home_path.clone(),
            statements: planet_context_source.statements.clone()
        };
        return planet_context
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContextSource {
    pub id: Option<String>,
    pub data: Option<HashMap<String, String>>,
    pub account_id: Option<String>,
    pub space_id: String,
    pub site_id: Option<String>,
}
impl ContextSource {
    pub fn defaults(space_id: String, site_id: String) -> Self {
        let mut site_id_wrap: Option<String> = None;
        let mut space_id = space_id.clone();
        if site_id != String::from("") {
            site_id_wrap = Some(site_id);
            if space_id == String::from(PRIVATE) {
                space_id = String::from("");
            }
        }
        let context_source: ContextSource = ContextSource{
            id: None,
            data: Some(HashMap::new()),
            space_id: space_id,
            account_id: None,
            site_id: site_id_wrap,
        };
        return context_source
    }
}

#[derive(Debug, Clone)]
pub struct Context<'gb> {
    pub id: Option<&'gb str>,
    pub data: Option<&'gb HashMap<String, String>>,
    pub account_id: Option<String>,
    pub space_id: &'gb str,
    pub site_id: Option<String>,
}
impl<'gb> Context<'gb> {
    pub fn defaults(context_source: &'gb ContextSource) -> Self {
        let context = Self{
            id: None,
            data: None,
            account_id: context_source.account_id.clone(),
            space_id: &context_source.space_id,
            site_id: context_source.site_id.clone(),
        };
        return context
    }
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
