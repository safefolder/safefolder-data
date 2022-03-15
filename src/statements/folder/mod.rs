pub mod schema;
pub mod config;
pub mod data;

// use validator::{ValidationErrors};
use tr::tr;
use std::{fs};
// use crate::planet::PlanetError;
// use crate::statements::StatementConfig;


pub trait ImportConfig {
    fn import_new(&self) -> ();
}

impl<T: ImportConfig> ImportConfig for &T {
    fn import_new(&self) -> () {
        T::import_new(*self)
    }
}

// pub trait StatementConfig {
//     fn is_valid(&self) -> Result<(), ValidationErrors>;
//     fn import(&self, yaml_config: String);
// }

pub fn fetch_yaml_config(yaml_path: &String) -> String {
    let yaml_config = fs::read_to_string(&yaml_path)
    .expect(&*tr!("Something went wrong reading the YAML file"));
    return yaml_config
}
