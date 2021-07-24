extern crate sled;

use std::str::FromStr;
// use std::{collections::HashMap, time::{SystemTime, UNIX_EPOCH}};
use serde::{Deserialize, Serialize};
use tr::tr;

use crate::storage::{generate_id};
use crate::planet::{PlanetError, PlanetContext, Context};
use crate::commands::table::config::DbTableConfig;

pub trait Schema<'gb> {
    fn defaults(planet_context: &'gb PlanetContext<'gb>, context: &'gb Context<'gb>) -> Result<DbTable<'gb>, PlanetError>;
    fn create(&self, schema_data: &SchemaData) -> Result<SchemaData, PlanetError>;
    fn get(&self, id: &String) -> Result<SchemaData, PlanetError>;
    fn get_by_name(&self, table_name: &str) -> Result<Option<SchemaData>, PlanetError>;
}

// pub trait Table {
//     fn insert(&self, db_row: DbRow);
//     fn update(&self, id: &str, db_row: DbRow);
//     fn delete(&self, id: &str);
//     fn read(&self, id: &str);
//     fn select(&self, query: DbQuery);
// }

// lifetimes: gb (global, for contexts), db, bs

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchemaData {
    pub id: Option<String>,
    pub account_id: Option<String>,
    pub space_id: Option<String>,
    pub name: String,
    pub config: DbTableConfig,
}

impl SchemaData {
    pub fn defaults(name: &String, config: &DbTableConfig, account_id: &String, space_id: &String) -> SchemaData {
        let schema_data = SchemaData{
            id: generate_id(),
            name: format!("{}", name),
            config: config.clone(),
            account_id: Some(account_id.clone()),
            space_id: Some(space_id.clone()),
        };
        return schema_data
    }
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
        let home_dir = planet_context.home_path.clone().unwrap();
        if *&context.account_id.unwrap() != "" && *&context.space_id.unwrap() != "" {
            println!("DbTable.open :: account_id and space_id have been informed");
        } else {
            // .achiever-planet/tables/tables.db : platform wide table schemas
            path = format!("{home}/tables/tables.db", home=&home_dir);
            println!("DbTable.open :: path: {}", &path);
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

    fn get_by_name(&self, table_name: &str) -> Result<Option<SchemaData>, PlanetError> {
        // I travel table for account_id if any, space id if any and table name
        let iter = self.db.iter();
        let mut number_items = 0;
        let mut matched_item: Option<SchemaData> = None;
        for result in iter {
            let tuple = result.unwrap();
            let item_db = tuple.1;
            let item: SchemaData = bincode::deserialize(&item_db).unwrap();
            let item_source = item.clone();
            let matches_name_none = &item.name.to_lowercase() == &table_name.to_lowercase();
            let mut check_account: bool = true;
            let ctx_account_id = self.context.account_id.clone().unwrap();
            let ctx_space_id = self.context.space_id.clone().unwrap();
            if self.context.account_id.is_some() && self.context.space_id.is_some() {
                if (item.account_id.is_some() && &item.account_id.unwrap() != ctx_account_id) || 
                (item.space_id.is_some() && &item.space_id.unwrap() != ctx_space_id) {
                    check_account = false;
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

    fn get(&self, id: &String) -> Result<SchemaData, PlanetError> {
        let id_db = xid::Id::from_str(id).unwrap();
        let id_db = id_db.as_bytes();
        let item_db = &self.db.get(&id_db).unwrap().unwrap();
        let item_: Result<SchemaData, _> = bincode::deserialize(&item_db);
        match item_ {
            Ok(_) => {
                let item = item_.unwrap();
                Ok(item)
            },
            Err(_) => {
                Err(PlanetError::new(500, Some(tr!("Could not fetch item from database"))))
            }
        }
    }

    fn create(&self, schema_data: &SchemaData) -> Result<SchemaData, PlanetError> {
        let table_name = &schema_data.name;
        let result_table_exists: Result<Option<SchemaData>, PlanetError> = self.get_by_name(
            &table_name.as_str()
        );
        let table_exists_error = *&result_table_exists.is_err();
        let table_exists = *&table_exists_error == false && *&result_table_exists.unwrap().is_some();
        if table_exists == true {
            return Err(PlanetError::new(
                500, 
                Some(tr!("Table \"{}\" already exists", &table_name))));
        } else if *&table_exists_error == true {
            return Err(PlanetError::new(
                500, 
                Some(tr!("Error checking table \"{}\"", &table_name))));
        }
        let encoded: Vec<u8> = bincode::serialize(schema_data).unwrap();
        let id = &schema_data.id.clone().unwrap();
        let id_db = xid::Id::from_str(id).unwrap();
        let id_db = id_db.as_bytes();
        let response = &self.db.insert(&id_db, encoded);
        match response {
            Ok(_) => {
                println!("DbTable.create :: id: {:?}", &id);
                let item_ = self.get(id);
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


// #[derive(Debug, Deserialize, Serialize, Clone)]
// pub struct DbFieldValue {
//     pub id: Option<String>,
//     pub config_id: Option<String>,
//     pub name: Option<String>,
//     pub value: Option<String>,
//     pub options: Option<HashMap<String, String>>,
// }

// #[derive(Debug, Deserialize, Serialize, Clone)]
// pub struct DbRow<'db, 'gb> {
//     pub id: Option<String>,
//     pub table: Option<DbTable<'db, 'gb>>,
//     pub display_name: String,
//     pub fields: Option<Vec<DbFieldValue>>,
//     pub fields_stemming: Option<Vec<DbFieldValue>>,
//     pub options: Option<HashMap<String, String>>,
// }


// //TODO: I will need the schema from create table config, so I can apply some helper methods of conversion

// // impl DbRow {
// //     pub fn defaults(display_name: &String, table: &DbTable) -> DbRow {
// //         let db_row: DbRow = DbRow{
// //             id: generate_id(),
// //             display_name: format!("{}", display_name),
// //             fields: Some(Vec::new()),
// //             fields_stemming: None,
// //             options: None,
// //             table: Some(table.clone()),
// //         };
// //         return db_row
// //     }
// // }

// // impl Table for DbTable{

// //     fn insert(&self, db_row: DbRow) {
// //         println!("DbDoc.insert...");
// //         // Bincode
// //         let t_1 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
// //         let encoded: Vec<u8> = bincode::serialize(&db_row).unwrap();
// //         let t_2 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
// //         // let diff = t_2-t_1;
// //         println!("DbDoc.insert :: bincode perf encode: {:?}", &(t_2-t_1));
// //         println!("DbDoc.insert :: encoded: {:?} length: {}", encoded, encoded.len());
// //         let t_1 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
// //         let decoded: Result<DbTable, bincode::Error> = bincode::deserialize(&encoded[..]);
// //         // println!("DbDoc.insert :: error: {:?}", decoded.unwrap_err());
// //         let t_2 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
// //         println!("DbDoc.insert :: bincode perf decode: {:?}", &(t_2-t_1));
// //         // JSON
// //         // let t_1 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
// //         // let config_value = serde_json::to_value(&db_doc).unwrap();
// //         // let json_encoded = serde_json::to_string(&config_value).unwrap();
// //         // let t_2 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
// //         // let diff = t_2-t_1;
// //         // println!("DbDoc.insert :: json perf encode: {:?}", &diff);
// //         // println!("DbDoc.insert :: encoded json: length: {}", json_encoded.len());
// //         // println!("DbDoc.insert :: {:?}", json_encoded);
// //         // let t_1 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
// //         // let json_decoded: Result<Value, serde_json::Error> = serde_json::from_str(&json_encoded);
// //         // let db_doc: Result<DbDoc, serde_json::Error> = serde_json::from_value(json_decoded.unwrap());
// //         // let t_2 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
// //         // println!("DbDoc.insert :: json perf decode: {:?}", &(t_2-t_1));
// //         // println!("DbDoc.insert :: json decoded: {:#?}", &db_doc.unwrap());
// //     }

// //     fn update(&self, id: &str, db_row: DbRow) {
// //         println!("KeyValueTable.update ...");
// //         println!("id: {}", id.to_string())
// //     }

// //     fn delete(&self, id: &str) {
// //         println!("KeyValueTable.delete ...");
// //         println!("id: {}", id)
// //     }

// //     fn read(&self, id: &str) {
// //         println!("KeyValueTable.read ...");
// //         println!("id: {id}", id=id)
// //     }

// //     fn select(&self, query: DbQuery) {
// //         println!("KeyValueTable.select ...");
// //         println!("select...");
// //     }

// // }

// pub struct DbQuery {
//     pub sql_where: String,
//     pub page: u8,
//     pub number_items: u8,
// }