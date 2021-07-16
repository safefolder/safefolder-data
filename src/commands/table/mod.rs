pub mod schema;
pub mod config;

use validator::{ValidationErrors};
use tr::tr;
use std::fs;

use crate::planet::PlanetContext;
use crate::commands::CommandRunner;


pub trait ImportConfig {
    fn import_new(planet_context: &PlanetContext, yaml_path: &String);
}

// pub trait CommandConfig {

//     fn defaults() {
//         println!("Hello");
//     };

// }

pub trait Command {
    // fn validate(&self) -> Result<(), ValidationErrors>;
    fn run(&self);
    fn runner(runner: &CommandRunner, path_yaml: &String) -> ();
}


pub trait CommandConfig {
    fn is_valid(&self) -> Result<(), ValidationErrors>;
    fn import(&self, yaml_config: String);
}

pub fn fetch_yaml_config(yaml_path: &String) -> String {
    let yaml_config = fs::read_to_string(&yaml_path)
    .expect(&*tr!("Something went wrong reading the YAML file"));
    return yaml_config
}
