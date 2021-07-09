extern crate tr;

use validator::{ValidationErrors};

use crate::commands::table::config::CreateTableConfig;
use crate::commands::table::Command;

pub struct CreateTable<'a> {
    pub config: CreateTableConfig,
    pub account_id: &'a str,
    pub space_id: &'a str,
}

impl<'a> Command for CreateTable<'a> {

    fn validate(&self) -> Result<(), ValidationErrors> {
        match self.config.is_valid() {
            Ok(_) => Ok(()),
            Err(e) => {
                return Err(e);
            }
        }
    }

    fn run(&self) -> () {
        // Insert into account "tables" the config
        println!("I run create table....");
    }

}

