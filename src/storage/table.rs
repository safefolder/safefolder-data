extern crate sled;

use std::str::FromStr;
use serde_json;
use std::collections::HashMap;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use tr::tr;
use serde_encrypt::{
    serialize::impls::BincodeSerializer, shared_key::SharedKey, traits::SerdeEncryptSharedKey,
    AsSharedKey, EncryptedMessage,
};

use crate::planet::constants::*;
use crate::storage::{generate_id};
use crate::planet::{PlanetError, PlanetContext, Context};
use crate::commands::table::config::DbTableConfig;
use crate::storage::constants::CHILD_PRIVATE_KEY_ARRAY;
use crate::storage::fields::*;

pub trait Schema<'gb> {
    fn defaults(planet_context: &'gb PlanetContext<'gb>, context: &'gb Context<'gb>) -> Result<DbTable<'gb>, PlanetError>;
    fn create(&self, db_data: &DbData) -> Result<DbData, PlanetError>;
    fn get(&self, id: &String) -> Result<DbData, PlanetError>;
    fn get_by_name(&self, table_name: &str) -> Result<Option<DbData>, PlanetError>;
}

pub trait Row<'gb> {
    fn defaults(
        table_file: &str, 
        planet_context: &'gb PlanetContext<'gb>, 
        context: &'gb Context<'gb>
    ) -> Result<DbRow<'gb>, PlanetError>;
    fn insert(&self, db_data: &DbData) -> Result<DbData, PlanetError>;
    fn get(&self, id: &String) -> Result<DbData, PlanetError>;
}

// lifetimes: gb (global, for contexts), db, bs

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoutingData {
    pub account_id: Option<String>,
    pub space_id: Option<String>,
    pub ipfs_cid: Option<String>,
}

// pub struct RowData {
//     pub id: Option<String>,
//     pub routing: RoutingData,
//     pub data: Option<HashMap<String, RowItem>>,
// }

// This structure would apply for SchemaData and RowData, we would need to convert from one to the other
// data has field_id -> value, so if we change field name would not be affected

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbData {
    pub id: Option<String>,
    pub name: Option<String>,
    pub routing: Option<HashMap<String, String>>,
    pub options: Option<HashMap<String, String>>,
    pub data: Option<HashMap<String, String>>,
    pub data_collections: Option<HashMap<String, Vec<HashMap<String, String>>>>,
    pub data_objects: Option<HashMap<String, HashMap<String, String>>>,
}
impl DbData {
    pub fn defaults(
        name: &String, 
        data: Option<HashMap<String, String>>, 
        data_collections: Option<HashMap<String, Vec<HashMap<String, String>>>>, 
        data_objects: Option<HashMap<String, HashMap<String, String>>>, 
        options: Option<HashMap<String, String>>, 
        routing: Option<&RoutingData>, 
    ) -> DbData {
        let mut routing_map_: Option<HashMap<String, String>> = None;
        if routing.is_some() {
            let mut routing_map: HashMap<String, String> = HashMap::new();
            let routing = routing.unwrap();
            let account_id = routing.account_id.clone().unwrap_or_default();
            let space_id = routing.space_id.clone().unwrap_or_default();
            let ipfs_cid = routing.ipfs_cid.clone().unwrap_or_default();
            routing_map.insert(String::from(ACCOUNT_ID), account_id.to_string());
            routing_map.insert(String::from(SPACE_ID), space_id.to_string());
            routing_map.insert(String::from(IPFS_CID), ipfs_cid);
            let routing_map_ = Some(routing_map);
        }
        let db_data = Self{
            id: generate_id(),
            name: Some(format!("{}", name)),
            routing: routing_map_,
            options: options,
            data: data,
            data_collections: data_collections,
            data_objects: data_objects,
        };
        return db_data
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchemaData {
    pub id: Option<String>,
    pub routing: RoutingData,
    pub name: String,
    pub config: DbTableConfig,
}

impl SchemaData {
    pub fn defaults(name: &String, config: &DbTableConfig, account_id: &str, space_id: &str) -> SchemaData {
        let schema_data = SchemaData{
            id: generate_id(),
            routing: RoutingData{
                account_id: Some(account_id.to_string()),
                space_id: Some(space_id.to_string()),
                ipfs_cid: None,
            },
            name: format!("{}", name),
            config: config.clone(),
        };
        return schema_data
    }
}

impl SerdeEncryptSharedKey for DbData {
    type S = BincodeSerializer<Self>;  // you can specify serializer implementation (or implement it by yourself).
}

#[derive(Debug, Clone)]
pub struct DbTable<'gb> {
    pub context: &'gb Context<'gb>,
    pub planet_context: &'gb PlanetContext<'gb>,
    db: sled::Db,
}

impl<'gb> Schema<'gb> for DbTable<'gb> {

    fn defaults(planet_context: &'gb PlanetContext<'gb>, context: &'gb Context<'gb>) -> 
        Result<DbTable<'gb>, PlanetError> {
        let mut path: String = String::from("");
        let home_dir = planet_context.home_path.unwrap_or_default();
        let account_id = context.account_id.unwrap_or_default();
        let space_id = context.space_id.unwrap_or_default();
        if account_id != "" && space_id != "" {
            println!("DbTable.open :: account_id and space_id have been informed");
        } else {
            // .achiever-planet/tables/tables.db : platform wide table schemas
            path = format!("{home}/tables/tables.db", home=&home_dir);
        }
        let config: sled::Config = sled::Config::default()
            .use_compression(true)
            .path(path);
        let result: Result<sled::Db, sled::Error> = config.open();
        match result {
            Ok(_) => {
                let db = result.unwrap();
                let db_table: DbTable = DbTable{
                    context: context,
                    planet_context: planet_context,
                    db: db
                };
                Ok(db_table)
            },
            Err(_) => {
                let planet_error = PlanetError::new(
                    500, 
                    Some(tr!("Could not open database")),
                );
                Err(planet_error)
            }
        }
    }

    fn get_by_name(&self, table_name: &str) -> Result<Option<DbData>, PlanetError> {
        // I travel table for account_id if any, space id if any and table name
        let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        let iter = self.db.iter();
        let mut number_items = 0;
        let mut matched_item: Option<DbData> = None;
        let ctx_account_id = self.context.account_id.unwrap_or_default();
        let ctx_space_id = self.context.space_id.unwrap_or_default();
        for result in iter {
            let tuple = result.unwrap();
            let item_db = tuple.1.to_vec();
            let item_ = EncryptedMessage::deserialize(item_db).unwrap();
            let item_ = DbData::decrypt_owned(
                &item_, 
                &shared_key);
            let item = item_.unwrap();
            let item_source = item.clone();
            let matches_name_none = &item.name.unwrap().to_lowercase() == &table_name.to_lowercase();
            let mut check_account: bool = true;
            let routing_response = item.routing;
            if routing_response.is_some() {
                let routing = routing_response.unwrap();
                let account_id = routing.get(ACCOUNT_ID).unwrap();
                let space_id = routing.get(SPACE_ID).unwrap();
                if ctx_account_id != "" && ctx_space_id != "" {
                    if (account_id != "" && account_id != ctx_account_id) || 
                    (space_id != "" && space_id != ctx_space_id) {
                        check_account = false;
                    }
                }
            }
            if matches_name_none && check_account {
                number_items += 1;
                matched_item = Some(item_source);
            }
        }
        match number_items {
            0 => {
                return Ok(None)
            },
            1 => {
                return Ok(matched_item)
            },
            2 => {
                return Err(PlanetError::new(500, Some(tr!("Could not fetch item from database"))))
            },
            _ => {
                return Err(PlanetError::new(500, Some(tr!("Could not fetch item from database"))))
            }
        }
    }

    fn get(&self, id: &String) -> Result<DbData, PlanetError> {
        let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        let id_db = xid::Id::from_str(id).unwrap();
        let id_db = id_db.as_bytes();
        let item_db = self.db.get(&id_db).unwrap().unwrap().to_vec();
        let item_ = EncryptedMessage::deserialize(item_db).unwrap();
        let item_ = DbData::decrypt_owned(
            &item_, 
            &shared_key);
        match item_ {
            Ok(_) => {
                let item = item_.unwrap();
                // println!("get :: item: {:?}", &item);
                Ok(item)
            },
            Err(_) => {
                Err(PlanetError::new(500, Some(tr!("Could not fetch item from database"))))
            }
        }
    }

    fn create(&self, db_data: &DbData) -> Result<DbData, PlanetError> {
        let table_name = db_data.name.clone().unwrap();
        let result_table_exists: Result<Option<DbData>, PlanetError> = self.get_by_name(
            &table_name.as_str()
        );
        let table_exists_error = *&result_table_exists.is_err();
        let table_exists = *&table_exists_error == false && *&result_table_exists.unwrap().is_some();
        if table_exists == true {
            let table_name_str = format!("\"{}\"", &table_name).magenta();
            return Err(PlanetError::new(
                500, 
                Some(tr!("Table {} already exists", &table_name_str))));
        } else if *&table_exists_error == true {
            return Err(PlanetError::new(
                500, 
                Some(tr!("Error checking table \"{}\"", &table_name))));
        }
        let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        let encrypted_schema = db_data.encrypt(&shared_key).unwrap();
        let encoded: Vec<u8> = encrypted_schema.serialize();
        let id = db_data.id.clone().unwrap();
        let id_db = xid::Id::from_str(id.as_str()).unwrap();
        let id_db = id_db.as_bytes();
        let response = &self.db.insert(id_db, encoded);
        match response {
            Ok(_) => {
                // println!("DbTable.create :: id: {:?}", &id);
                let item_ = self.get(&id);
                match item_ {
                    Ok(_) => {
                        let item = item_.unwrap();
                        Ok(item)
                    },
                    Err(error) => {
                        Err(error)
                    }
                }
            },
            Err(_) => {
                Err(PlanetError::new(500, Some(tr!("Could not write table schema"))))
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RowItem(pub FieldType);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RowData {
    pub id: Option<String>,
    pub routing: RoutingData,
    pub data: Option<HashMap<String, RowItem>>,
}
impl RowData {
    pub fn defaults(account_id: &String, space_id: &String) -> Self {
        return Self{
            id: generate_id(),
            routing: RoutingData{
                account_id: Some(account_id.clone()),
                space_id: Some(space_id.clone()),
                ipfs_cid: None
            },
            data: None,
        };
    }
}

#[derive(Debug, Clone)]
pub struct DbRow<'gb> {
    pub context: &'gb Context<'gb>,
    pub planet_context: &'gb PlanetContext<'gb>,
    db: sled::Db,
}

impl<'gb> Row<'gb> for DbRow<'gb> {

    fn defaults(
        table_file: &str,
        planet_context: &'gb PlanetContext<'gb>, 
        context: &'gb Context<'gb>
    ) -> Result<DbRow<'gb>, PlanetError> {
        let mut path: String = String::from("");
        let home_dir = planet_context.home_path.unwrap_or_default();
        let account_id = context.account_id.unwrap_or_default();
        let space_id = context.space_id.unwrap_or_default();
        if account_id != "" && space_id != "" {
            println!("DbRow.defaults :: account_id and space_id have been informed");
        } else {
            // .achiever-planet/{table_file} : platform wide table (slug with underscore)
            path = format!("{home}/tables/{table_file}.db", home=&home_dir, table_file=table_file);
        }
        eprintln!("DbRow.defaults :: path: {:?}", path);
        let config: sled::Config = sled::Config::default()
            .use_compression(true)
            .path(path);
        let result: Result<sled::Db, sled::Error> = config.open();
        match result {
            Ok(_) => {
                let db = result.unwrap();
                let db_row: DbRow = DbRow{
                    context: context,
                    planet_context: planet_context,
                    db: db
                };
                Ok(db_row)
            },
            Err(_) => {
                let planet_error = PlanetError::new(
                    500, 
                    Some(tr!("Could not open database")),
                );
                Err(planet_error)
            }
        }
    }

    fn insert(&self, db_data: &DbData) -> Result<DbData, PlanetError> {
        eprintln!("insert :: row_data: {:#?}", db_data);
        let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        let encrypted_data = db_data.encrypt(&shared_key).unwrap();
        let encoded: Vec<u8> = encrypted_data.serialize();
        eprintln!("insert :: data size: {}", encoded.len());
        let id = db_data.id.clone().unwrap();
        let id_db = xid::Id::from_str(id.as_str()).unwrap();
        let id_db = id_db.as_bytes();
        let response = &self.db.insert(id_db, encoded);
        match response {
            Ok(_) => {
                eprintln!("DbRow.insert :: id: {:?}", &id);
                eprintln!("DbRow.insert :: Will get item...");
                let item_ = self.get(&id);
                match item_ {
                    Ok(_) => {
                        let item = item_.unwrap();
                        Ok(item)
                    },
                    Err(error) => {
                        Err(error)
                    }
                }
            },
            Err(_) => {
                Err(PlanetError::new(500, Some(tr!("Could not write table schema"))))
            }
        }
    }

    fn get(&self, id: &String) -> Result<DbData, PlanetError> {
        let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        let id_db = xid::Id::from_str(id).unwrap();
        let id_db = id_db.as_bytes();
        let item_db = self.db.get(&id_db).unwrap().unwrap().to_vec();
        let item_ = EncryptedMessage::deserialize(item_db).unwrap();
        let item_ = DbData::decrypt_owned(
            &item_, 
            &shared_key);
        match item_ {
            Ok(_) => {
                let item = item_.unwrap();
                eprintln!("get :: item: {:?}", &item);
                Ok(item)
            },
            Err(_) => {
                Err(PlanetError::new(500, Some(tr!("Could not fetch item from database"))))
            }
        }
    }
}

pub struct DbQuery {
    pub sql_where: String,
    pub page: u8,
    pub number_items: u8,
}