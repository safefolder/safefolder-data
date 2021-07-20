// use serde_json::Value;
use std::io;
use std::{collections::HashMap, time::{SystemTime, UNIX_EPOCH}};
use serde::{Deserialize, Serialize};

use crate::storage::{generate_id, generate_id_bytes};
use crate::storage::config::{FieldConfig, LanguageConfig};

pub trait Table {
    fn insert(&self, db_row: DbRow);
    fn update(&self, id: &str, db_row: DbRow);
    fn delete(&self, id: &str);
    fn read(&self, id: &str);
    fn select(&self, query: DbQuery);
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbTableConfig {
    pub language: Option<LanguageConfig>,
    pub fields: Option<Vec<FieldConfig>>,
}


#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DbTable {
    pub id: Option<[u8; 12]>,
    pub display_name: String,
    pub config: DbTableConfig,
}

impl DbTable {
    pub fn defaults(display_name: &String, config: DbTableConfig) -> DbTable {
        let db_table: DbTable = DbTable{
            id: Some(generate_id_bytes()),
            display_name: format!("{}", display_name),
            config: config,
        };
        return db_table;
    }
    pub fn create(&self) -> Result<DbTable, io::Error> {
        let encoded: Vec<u8> = bincode::serialize(&self).unwrap();
        let _config = sled::Config::default()
            .use_compression(true)
            .path("my_tables_db");
        let db: sled::Db = _config.open().unwrap();
        let response = db.insert(&self.id.unwrap(), encoded);
        match response {
            Ok(_) => {
                let item_db = db.get(&self.id.unwrap()).unwrap().unwrap();
                let item_: Result<DbTable, _> = bincode::deserialize(&item_db);
                match item_ {
                    Ok(_) => {
                        let item = item_.unwrap();
                        Ok(item)
                    },
                    Err(error) => {
                        let my_error = io::Error::new(io::ErrorKind::ConnectionRefused, error);
                        Err(my_error)
                    }
                }
            },
            Err(error) => {
                // I return io error, so is not linked to data provider, sled so far
                let my_error = io::Error::new(io::ErrorKind::ConnectionRefused, error);
                Err(my_error)
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DbFieldValue {
    pub id: Option<String>,
    pub config_id: Option<String>,
    pub name: Option<String>,
    pub value: Option<String>,
    pub options: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DbRow {
    pub id: Option<String>,
    pub table: Option<DbTable>,
    pub display_name: String,
    pub fields: Option<Vec<DbFieldValue>>,
    pub fields_stemming: Option<Vec<DbFieldValue>>,
    pub options: Option<HashMap<String, String>>,
}


//TODO: I will need the schema from create table config, so I can apply some helper methods of conversion

impl DbRow {
    pub fn defaults(display_name: &String, table: &DbTable) -> DbRow {
        let db_row: DbRow = DbRow{
            id: generate_id(),
            display_name: format!("{}", display_name),
            fields: Some(Vec::new()),
            fields_stemming: None,
            options: None,
            table: Some(table.clone()),
        };
        return db_row
    }
}

impl Table for DbTable{

    fn insert(&self, db_row: DbRow) {
        println!("DbDoc.insert...");
        // Bincode
        let t_1 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let encoded: Vec<u8> = bincode::serialize(&db_row).unwrap();
        let t_2 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        // let diff = t_2-t_1;
        println!("DbDoc.insert :: bincode perf encode: {:?}", &(t_2-t_1));
        println!("DbDoc.insert :: encoded: {:?} length: {}", encoded, encoded.len());
        let t_1 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let decoded: Result<DbTable, bincode::Error> = bincode::deserialize(&encoded[..]);
        // println!("DbDoc.insert :: error: {:?}", decoded.unwrap_err());
        let t_2 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        println!("DbDoc.insert :: bincode perf decode: {:?}", &(t_2-t_1));
        // JSON
        // let t_1 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        // let config_value = serde_json::to_value(&db_doc).unwrap();
        // let json_encoded = serde_json::to_string(&config_value).unwrap();
        // let t_2 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        // let diff = t_2-t_1;
        // println!("DbDoc.insert :: json perf encode: {:?}", &diff);
        // println!("DbDoc.insert :: encoded json: length: {}", json_encoded.len());
        // println!("DbDoc.insert :: {:?}", json_encoded);
        // let t_1 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        // let json_decoded: Result<Value, serde_json::Error> = serde_json::from_str(&json_encoded);
        // let db_doc: Result<DbDoc, serde_json::Error> = serde_json::from_value(json_decoded.unwrap());
        // let t_2 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        // println!("DbDoc.insert :: json perf decode: {:?}", &(t_2-t_1));
        // println!("DbDoc.insert :: json decoded: {:#?}", &db_doc.unwrap());
    }

    fn update(&self, id: &str, db_row: DbRow) {
        println!("KeyValueTable.update ...");
        println!("id: {}", id.to_string())
    }

    fn delete(&self, id: &str) {
        println!("KeyValueTable.delete ...");
        println!("id: {}", id)
    }

    fn read(&self, id: &str) {
        println!("KeyValueTable.read ...");
        println!("id: {id}", id=id)
    }

    fn select(&self, query: DbQuery) {
        println!("KeyValueTable.select ...");
        println!("select...");
    }

}

pub struct DbQuery {
    pub sql_where: String,
    pub page: u8,
    pub number_items: u8,
}