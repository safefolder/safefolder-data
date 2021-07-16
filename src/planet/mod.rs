pub mod validation;
pub mod constants;

extern crate serde_yaml;
extern crate colored;

use serde::{Deserialize, Serialize};
use validator::{Validate};

use tr::tr;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
// use colored::*;

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct PlanetContext {
    pub mission: String,
    pub validation_errors: Option<HashMap<String, String>>,
}

impl PlanetContext {
    pub fn import_context() -> Result<PlanetContext, io::Error> {
        let home_path = env::current_dir();
        if home_path.is_err() {
            return Err(home_path.unwrap_err())
        }
        let home_path_str = home_path.unwrap();
        let home_path_str = home_path_str.to_str().unwrap();
        let path_planet_context: &str = &*format!("{}/planet_context.yaml", home_path_str);
        if Some(path_planet_context).is_some() {
            let planet_context_str = fs::read_to_string(&path_planet_context)
            .expect("Something went wrong reading the YAML file");
            let mut planet_context: PlanetContext = serde_yaml::from_str(&planet_context_str).unwrap();
            
            // Planet validation errors with i18m support
            let mut validation_errors: HashMap<String, String> = HashMap::new();
            validation_errors.insert(String::from("length_min"), 
                tr!("{command}: Length for field \"{field}\" is higher than {value}."));
            validation_errors.insert(String::from("length_max"), 
                tr!("{command}: Length for field \"{field}\" is lower than {value}."));
            validation_errors.insert(String::from("length_equal"), 
                tr!("{command}: Length for field \"{field}\" is not equal to {value}."));
            validation_errors.insert(String::from("range_min"), 
                tr!("{command}: Range for field \"{field}\" does not meet. {} is lower than allowed range."));
            validation_errors.insert(String::from("range_max"), 
                tr!("{command}: Range for field \"{field}\" does not meet. {} is higher than allowed range"));
            validation_errors.insert(String::from("must_match"), 
                tr!("{command}: Value from field \"{field_origin}\" does not match with field \"{field_matched}\""));
            validation_errors.insert(String::from("contains"), 
                tr!("{command}: Field \"{field}\" does not contain \"{value}\""));
            validation_errors.insert(String::from("regex"), 
                tr!("{command}: String expression formula does not match (Regex) for field \"{field}\""));
                validation_errors.insert(String::from("required"), 
                tr!("{command}: Field \"{field}\" is required"));
            
            planet_context.validation_errors = Some(validation_errors);
            return Ok(planet_context);
        } else {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, 
                &*tr!("Could not import planet context")))
        }
    }

}

pub struct Context {
    pub id: Option<String>,
    pub data: HashMap<String, String>,
}