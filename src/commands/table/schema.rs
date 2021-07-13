extern crate tr;

use validator::{ValidationErrors};

use crate::commands::table::config::CreateTableConfig;
use crate::commands::table::{Command, CommandConfig};
use crate::planet::{PlanetContext, Context};

pub struct CreateTable<'a> {
    pub planet_context: &'a PlanetContext,
    pub context: &'a Context,
    pub config: CreateTableConfig,
    pub account_id: Option<&'a str>,
    pub space_id: Option<&'a str>,
}

impl<'a> Command for CreateTable<'a> {

    fn validate(&self) -> Result<(), ValidationErrors> {
        match self.config.is_valid() {
            Ok(_) => Ok(()),
            Err(e) => {
                println!("Hola...");
                println!("{}", e);
                return Err(e);
            }
        }
    }

    fn run(&self) -> () {
        // Insert into account "tables" the config
        println!("I run create table....");
    }

}

