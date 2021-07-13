pub mod validation;
pub mod constants;

extern crate serde_yaml;

use serde::{Deserialize, Serialize};
use validator::{Validate};

use tr::tr;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;

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
            validation_errors.insert(String::from("length_min"), tr!("This is a message"));
            
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