extern crate sled;

use std::collections::HashMap;
use std::{thread};
use std::sync::{Arc, Mutex};

use tr::tr;

use crate::planet::{PlanetError};
use crate::storage::constants::*;

#[derive(Debug, Clone)]
pub struct SpaceDatabase {
    pub connection_pool: HashMap<String, sled::Db>,
}

impl SpaceDatabase {
    pub fn defaults(
        site_id: Option<&str>, 
        space_id: &str, 
        home_dir: Option<&str>,
        skip_data: Option<bool>,
    ) -> Result<Self, PlanetError> {
        let skip_data_wrap = skip_data.clone();
        let mut skip_data = false;
        if skip_data_wrap.is_some() {
            skip_data = skip_data_wrap.unwrap();
        }
        let home_dir = home_dir.unwrap_or_default();
        let space_id = space_id;
        let mut connection_pool: HashMap<String, sled::Db> = HashMap::new();
        // private space: Open private space db and workspace
        // site: Open site space db and site db
        if site_id.is_none() {
            // I don't have site, private space
            // I try to open db connections in threads for performance
            // space database
            let db_names: Vec<&str> = "database.db,workspace.db".split(",").collect();
            let connection_pool_: Arc<Mutex<HashMap<String, sled::Db>>> = Arc::new(Mutex::new(connection_pool));
            let space_id: Arc<Mutex<String>> = Arc::new(Mutex::new(space_id.to_string()));
            let home_dir: Arc<Mutex<String>> = Arc::new(Mutex::new(home_dir.to_string()));
            let errors: Arc<Mutex<Vec<PlanetError>>> = Arc::new(Mutex::new(Vec::new()));
            let mut handles= vec![];
            for db_name in db_names {
                let connection_pool_ = Arc::clone(&connection_pool_);
                let space_id = Arc::clone(&space_id);
                let home_dir = Arc::clone(&home_dir);
                let errors = Arc::clone(&errors);
                let mut key: &str = PRIVATE;
                if db_name == "workspace.db" {
                    key = WORKSPACE
                }
                if skip_data == true && key == PRIVATE {
                    continue
                }
                let handle = thread::spawn(move || {
                    // Open db, then sync into connection_pool
                    let home_dir = home_dir.lock().unwrap();
                    let path = format!("{home}/private/{file}", file=db_name, home=&home_dir);
                    let config: sled::Config = sled::Config::default()
                    .use_compression(true)
                    .path(path);
                    let result= config.open();
                    if result.is_ok() {
                        let database = result.unwrap();
                        let mut connection_pool_ = connection_pool_.lock().unwrap();
                        connection_pool_.insert(key.to_string(), database);
                    } else {
                        let space_id = space_id.lock().unwrap();
                        let mut errors = errors.lock().unwrap();
                        errors.push(
                            PlanetError::new(
                                500, 
                                Some(tr!("Error opening database.db on space \"{}\".", space_id)),
                            )
                        )
                    }
                });
                handles.push(handle);
            }
            for handle in handles {
                handle.join().unwrap();
            }
            connection_pool = connection_pool_.lock().unwrap().clone();
        } else {
            // I have site
            let site_id = site_id.unwrap();
            let db_names: Vec<&str> = "database.db,site.db".split(",").collect();
            let connection_pool_: Arc<Mutex<HashMap<String, sled::Db>>> = Arc::new(Mutex::new(connection_pool));
            let space_id: Arc<Mutex<String>> = Arc::new(Mutex::new(space_id.to_string()));
            let home_dir: Arc<Mutex<String>> = Arc::new(Mutex::new(home_dir.to_string()));
            let site_id: Arc<Mutex<String>> = Arc::new(Mutex::new(site_id.to_string()));
            let errors: Arc<Mutex<Vec<PlanetError>>> = Arc::new(Mutex::new(Vec::new()));
            let mut handles= vec![];
            for db_name in db_names {
                let connection_pool_ = Arc::clone(&connection_pool_);
                let space_id = Arc::clone(&space_id);
                let home_dir = Arc::clone(&home_dir);
                let site_id = Arc::clone(&site_id);
                let errors = Arc::clone(&errors);
                let space_id_ = space_id.lock().unwrap().clone();
                let mut key: String = space_id_.clone();
                if db_name == String::from("site.db") {
                    key = site_id.lock().unwrap().clone();
                }
                if skip_data == true && key == space_id_ {
                    continue
                }
                let handle = thread::spawn(move || {
                    // Open db, then sync into connection_pool
                    let home_dir = home_dir.lock().unwrap();
                    let site_id = site_id.lock().unwrap();
                    let path : String;
                    if key == space_id_ {
                        path = format!(
                            "{home}/sites/{site_id}/spaces/{space_id}/database.db", 
                            site_id=site_id, 
                            space_id=&space_id_,
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
                        let mut connection_pool_ = connection_pool_.lock().unwrap();
                        connection_pool_.insert(key.to_string(), database);
                    } else {
                        let space_id = space_id.lock().unwrap();
                        let mut errors = errors.lock().unwrap();
                        errors.push(
                            PlanetError::new(
                                500, 
                                Some(tr!("Error opening database.db on space \"{}\".", space_id)),
                            )
                        )
                    }
                });
                handles.push(handle);
            }
            for handle in handles {
                handle.join().unwrap();
            }
            connection_pool = connection_pool_.lock().unwrap().clone();
        }
        let obj = Self{
            connection_pool: connection_pool.clone()
        };
        return Ok(obj)
    }
}
