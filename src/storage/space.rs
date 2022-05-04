extern crate sled;

use std::collections::HashMap;
use tr::tr;

use crate::planet::{PlanetError};
use crate::storage::constants::*;

#[derive(Debug, Clone)]
pub struct SpaceDatabase {
    pub connection_pool: HashMap<String, sled::Db>,
}

impl SpaceDatabase {
    pub fn defaults(
        site_id: Option<String>, 
        space_id: &str, 
        box_id: &str,
        home_dir: Option<&str>,
    ) -> Result<Self, PlanetError> {
        let home_dir = home_dir.unwrap_or_default();
        let space_id = space_id;
        let mut connection_pool: HashMap<String, sled::Db> = HashMap::new();
        // private space: Open private space db and workspace
        // site: Open site space db and site db
        let mut errors: Vec<PlanetError> = Vec::new();
        if site_id.is_none() {
            // I don't have site, private space
            // I try to open db connections in threads for performance
            // space database
            let db_names: Vec<&str> = "database.db,workspace.db".split(",").collect();
            for db_name in db_names {
                let key: &str;
                if db_name == "workspace.db" {
                    key = WORKSPACE;
                } else {
                    key = PRIVATE;
                }
                // Open db, then sync into connection_pool
                let path : String;
                if key == PRIVATE {
                    path = format!("{home}/private/{file}", file=db_name, home=&home_dir);
                } else {
                    path = format!("{home}/{file}", file=db_name, home=&home_dir);
                }
                let config: sled::Config = sled::Config::default()
                .use_compression(true)
                .path(path);
                let result= config.open();
                if result.is_ok() {
                    let database = result.unwrap();
                    connection_pool.insert(key.to_string(), database);
                } else {
                    errors.push(
                        PlanetError::new(
                            500, 
                            Some(tr!("Error opening database.db on space \"{}\".", space_id)),
                        )
                    )
                }
            }
        } else {
            // I have site
            let site_id = site_id.unwrap();
            let db_names: Vec<&str> = "database.db,site.db".split(",").collect();
            for db_name in db_names {
                let mut key: String = space_id.to_string();
                if db_name == String::from("site.db") {
                    key = site_id.to_string();
                }
                // Open db, then sync into connection_pool
                let path : String;
                if key == space_id {
                    path = format!(
                        "{home}/sites/{site_id}/spaces/{space_id}/boxes/{box_id}/database.db", 
                        site_id=site_id, 
                        space_id=&space_id,
                        box_id=box_id,
                        home=&home_dir
                    );
                } else {
                    path = format!(
                        "{home}/sites/{site_id}/site.db", 
                        site_id=site_id, 
                        home=&home_dir
                    );
                }
                let config: sled::Config = sled::Config::default()
                .use_compression(true)
                .path(path);
                let result= config.open();
                if result.is_ok() {
                    let database = result.unwrap();
                    connection_pool.insert(key.to_string(), database);
                } else {
                    errors.push(
                        PlanetError::new(
                            500, 
                            Some(tr!("Error opening database.db on space \"{}\".", space_id)),
                        )
                    )
                }
            }
        }
        let obj = Self{
            connection_pool: connection_pool.clone()
        };
        return Ok(obj)
    }
}