extern crate sled;
extern crate slug;

use std::str::FromStr;
use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::collections::BTreeMap;
use colored::Colorize;
use rust_decimal::prelude::ToPrimitive;
use validator::{Validate, ValidationErrors};
use serde::{Deserialize, Serialize};
use tr::tr;
use serde_encrypt::{
    serialize::impls::BincodeSerializer, shared_key::SharedKey, traits::SerdeEncryptSharedKey,
    AsSharedKey, EncryptedMessage,
};
use slug::slugify;

use crate::planet::constants::*;
use crate::storage::{generate_id};
use crate::planet::{PlanetError};
use crate::commands::folder::config::{DbFolderConfig};
use crate::storage::constants::*;
use crate::storage::properties::*;
// use crate::functions::*;

pub trait FolderSchema {
    fn defaults(
        home_dir: Option<&str>,
        account_id: Option<&str>,
        space_id: Option<&str>,
        site_id: Option<&str>,
    ) -> Result<DbFolder, PlanetError>;
    fn create(&self, db_data: &DbData) -> Result<DbData, PlanetError>;
    fn update(&self, db_data: &DbData) -> Result<DbData, PlanetError>;
    fn get(&self, id: &String) -> Result<DbData, PlanetError>;
    fn get_by_name(&self, folder_name: &str) -> Result<Option<DbData>, PlanetError>;
}

pub trait FolderItem {
    fn defaults(
        home_dir: &str,
        account_id: &str,
        space_id: &str,
        site_id: &str,
        box_id: &str,
        folder_id: &str,
        db_folder: &DbFolder,
    ) -> Result<DbFolderItem, PlanetError>;
    fn insert(&self, folder_name: &String, db_data: &DbData) -> Result<DbData, PlanetError>;
    fn update(&self, db_data: &DbData) -> Result<DbData, PlanetError>;
    fn get(
        &self, 
        folder_name: &String, 
        by: GetItemOption, 
        properties: Option<Vec<String>>
    ) -> Result<DbData, PlanetError>;
    fn select(&self, 
        folder_name: &String, 
        r#where: Option<String>, 
        page: Option<usize>,
        number_items: Option<usize>,
        properties: Option<Vec<String>>,
    ) -> Result<SelectResult, PlanetError>;
    fn count(&self, 
        folder_name: &String, 
        r#where: Option<String>, 
    ) -> Result<SelectCountResult, PlanetError>;
    fn total_count(&self) -> Result<SelectCountResult, PlanetError>;
}

// lifetimes: gb (global, for contexts), db, bs

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoutingData {
    pub account_id: Option<String>,
    pub space_id: Option<String>,
    pub box_id: Option<String>,
    pub site_id: Option<String>,
    pub ipfs_cid: Option<String>,
}

impl RoutingData {
    pub fn defaults(
        account_id: Option<String>, 
        site_id: Option<&str>,
        space_id: Option<&str>, 
        box_id: Option<&str>, 
        ipfs_cid: Option<String>,
    ) -> Option<RoutingData> {
        let mut routing = RoutingData{
            account_id: account_id,
            site_id: None,
            space_id: None,
            box_id: None,
            ipfs_cid: ipfs_cid,
        };
        if site_id.is_some() {
            let site_id = site_id.unwrap().to_string();
            routing.site_id = Some(site_id);
        }
        if space_id.is_some() {
            let space_id = space_id.unwrap().to_string();
            routing.space_id = Some(space_id);
        }
        if box_id.is_some() {
            let box_id = box_id.unwrap().to_string();
            routing.box_id = Some(box_id);
        }
        let routing_wrap = Some(routing);
        return routing_wrap
    }
}

impl SerdeEncryptSharedKey for NameTree {
    type S = BincodeSerializer<Self>;  // you can specify serializer implementation (or implement it by yourself).
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct NameTree {
    name: String
}

// This structure would apply for SchemaData and RowData, we would need to convert from one to the other
// data has field_id -> value, so if we change property name would not be affected

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct DbData {
    #[validate(required)]
    pub id: Option<String>,
    #[validate(required)]
    pub slug: Option<String>,
    #[validate(required)]
    pub name: Option<String>,
    pub routing: Option<BTreeMap<String, String>>,
    pub options: Option<BTreeMap<String, String>>,
    pub context: Option<BTreeMap<String, String>>,
    pub data: Option<BTreeMap<String, String>>,
    pub data_collections: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
    pub data_objects: Option<BTreeMap<String, BTreeMap<String, String>>>,
}
impl DbData {
    pub fn defaults(
        name: &String, 
        data: Option<BTreeMap<String, String>>, 
        data_collections: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>, 
        data_objects: Option<BTreeMap<String, BTreeMap<String, String>>>, 
        options: Option<BTreeMap<String, String>>, 
        routing: Option<RoutingData>, 
        context: Option<BTreeMap<String, String>>
    ) -> Result<DbData, PlanetError> {
        let slug = slugify(name);
        let mut routing_map_: Option<BTreeMap<String, String>> = None;
        if routing.is_some() {
            let mut routing_map: BTreeMap<String, String> = BTreeMap::new();
            let routing = routing.unwrap();
            let account_id = routing.account_id.clone().clone();
            let site_id = routing.site_id.clone();
            let space_id = routing.space_id.clone();
            let box_id = routing.box_id.clone();
            let ipfs_cid = routing.ipfs_cid.clone().unwrap_or_default();
            if account_id.is_some() {
                routing_map.insert(String::from(ACCOUNT_ID), account_id.unwrap().to_string());
            }
            if site_id.is_some() {
                routing_map.insert(String::from(SITE_ID), site_id.unwrap());
            }

            if space_id.is_some() {
                routing_map.insert(String::from(SPACE_ID), space_id.unwrap());
            }
            if box_id.is_some() {
                routing_map.insert(String::from(BOX_ID), box_id.unwrap());
            }            
            routing_map.insert(String::from(IPFS_CID), ipfs_cid);
            routing_map_ = Some(routing_map);
        }
        let db_data = Self{
            id: generate_id(),
            slug: Some(slug),
            name: Some(format!("{}", name)),
            routing: routing_map_,
            options: options,
            data: data,
            data_collections: data_collections,
            data_objects: data_objects,
            context: context,
        };
        let validate: Result<(), ValidationErrors> = db_data.validate();
        match validate {
            Ok(_) => {
                return Ok(db_data);
            },
            Err(_) => {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!("Could not validate folder data")),
                    )
                );
            }
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchemaData {
    pub id: Option<String>,
    pub routing: RoutingData,
    pub name: String,
    pub config: DbFolderConfig,
}

impl SchemaData {
    pub fn defaults(
        name: &String, 
        config: &DbFolderConfig, 
        account_id: Option<&str>, 
        site_id: Option<&str>, 
        space_id: Option<&str>, 
        box_id: Option<&str>, 
    ) -> SchemaData {
        let mut routing = RoutingData{
            account_id: None,
            site_id: None,
            space_id: None,
            box_id: None,
            ipfs_cid: None,
        };
        if account_id.is_some() {
            let account_id = account_id.unwrap().to_string();
            routing.account_id = Some(account_id);
        }
        if site_id.is_some() {
            let site_id = site_id.unwrap().to_string();
            routing.site_id = Some(site_id);
        }
        if space_id.is_some() {
            let space_id = space_id.unwrap().to_string();
            routing.space_id = Some(space_id);
        }
        if box_id.is_some() {
            let box_id = box_id.unwrap().to_string();
            routing.box_id = Some(box_id);
        }
        let schema_data = SchemaData{
            id: generate_id(),
            routing: routing,
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
pub struct DbFolder {
    pub home_dir: Option<String>,
    pub account_id: Option<String>,
    pub space_id: Option<String>,
    pub site_id: Option<String>,
    pub box_id: Option<String>,
    db: sled::Db,
}

impl FolderSchema for DbFolder {

    fn defaults(
        home_dir: Option<&str>,
        account_id: Option<&str>,
        space_id: Option<&str>,
        site_id: Option<&str>,
    ) -> Result<DbFolder, PlanetError> {
        let mut path: String = String::from("");
        let home_dir = home_dir.unwrap_or_default();
        let account_id = account_id.unwrap_or_default();
        let space_id = space_id.unwrap_or_default();
        if account_id != "" && space_id != "" {
            println!("DbFolder.open :: account_id and space_id have been informed");
        } else if site_id.is_none() && space_id == "private" {
            path = format!("{home}/private/folders.db", home=&home_dir);
        } else {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("Sites not yet supported, only private spaces")),
                )
            )
        }
        let site_id = site_id.unwrap_or_default();
        eprintln!("DbFolder.defaults :: path: {}", &path);
        let config: sled::Config = sled::Config::default()
            .use_compression(true)
            .path(path);
        let result: Result<sled::Db, sled::Error> = config.open();
        match result {
            Ok(_) => {
                let db = result.unwrap();
                let db_folder: DbFolder= DbFolder{
                    home_dir: Some(home_dir.to_string()),
                    account_id: Some(account_id.to_string()),
                    space_id: Some(space_id.to_string()),
                    site_id: Some(site_id.to_string()),
                    box_id: None,
                    db: db,
                };
                Ok(db_folder)
            },
            Err(err) => {
                eprintln!("{:?}", &err);
                let planet_error = PlanetError::new(
                    500, 
                    Some(tr!("Could not open folder database")),
                );
                Err(planet_error)
            }
        }
    }

    fn get_by_name(&self, folder_name: &str) -> Result<Option<DbData>, PlanetError> {
        // I travel folder for account_id if any, space id if any and folder name
        let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        let iter = self.db.iter();
        let mut number_items = 0;
        let mut matched_item: Option<DbData> = None;
        let ctx_account_id = self.account_id.clone().unwrap_or_default();
        let ctx_account_id = ctx_account_id.as_str();
        let ctx_space_id = self.space_id.clone().unwrap_or_default();
        let ctx_space_id = ctx_space_id.as_str();
        for result in iter {
            let tuple = result.unwrap();
            let item_db = tuple.1.to_vec();
            let item_ = EncryptedMessage::deserialize(item_db).unwrap();
            let item_ = DbData::decrypt_owned(
                &item_, 
                &shared_key);
            let item = item_.unwrap();
            let item_source = item.clone();
            let item_name = &item.name.unwrap();
            eprintln!("DbFolder.get_by_name :: name: {}", item_name);
            let matches_name_none = &item_name.to_lowercase() == &folder_name.to_lowercase();
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
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!(
                            "Could not fetch item from database, more than 1 item matches folder name."
                        ))
                    )
                )
            },
            _ => {
                return Err(PlanetError::new(500, Some(tr!("Item not found in folders."))))
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
        let folder_name = db_data.name.clone().unwrap();
        let result_table_exists: Result<Option<DbData>, PlanetError> = self.get_by_name(
            &folder_name.as_str()
        );
        let table_exists_error = *&result_table_exists.clone().is_err();
        let table_exists = *&table_exists_error == false && *&result_table_exists.clone().unwrap().is_some();
        if table_exists == true {
            let table_name_str = format!("\"{}\"", &folder_name).magenta();
            return Err(PlanetError::new(
                500, 
                Some(tr!("Folder {} already exists", &table_name_str))));
        } else if *&table_exists_error == true {
            let error = result_table_exists.unwrap_err();
            return Err(PlanetError::new(
                500, 
                Some(tr!("Error checking folder \"{}\": \"{}\"", &folder_name, &error.message))));
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
                println!("DbFolder.create :: id: {:?} name: {}", &id, &folder_name);
                let _db_response = response.clone().unwrap();
                // eprintln!("DbFolder.create :: db_response : {:?}", &db_response);
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
                Err(PlanetError::new(500, Some(tr!(
                    "Could not write folder config into database, generic data error."
                ))))
            }
        }
    }
    fn update(&self, folder: &DbData) -> Result<DbData, PlanetError> {
        let mut folder = folder.clone();
        let folder_name = folder.name.clone().unwrap();
        let result_table_exists: Result<Option<DbData>, PlanetError> = self.get_by_name(
            &folder_name.as_str()
        );
        let table_exists_error = *&result_table_exists.clone().is_err();
        let table_exists = *&table_exists_error == false && *&result_table_exists.clone().unwrap().is_some();
        if table_exists == false {
            let table_name_str = format!("\"{}\"", &folder_name).magenta();
            return Err(PlanetError::new(
                500, 
                Some(tr!("Folder {} does not exists", &table_name_str))));
        } else if *&table_exists_error == true {
            let error = result_table_exists.unwrap_err();
            return Err(PlanetError::new(
                500, 
                Some(tr!("Error checking folder \"{}\": \"{}\"", &folder_name, &error.message))));
        }
        let folder_db = result_table_exists.unwrap().unwrap();
        folder.id = folder_db.id;
        let id_db = folder.clone().id.unwrap();
        let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        let encrypted_schema = folder.clone().encrypt(&shared_key).unwrap();
        let encoded: Vec<u8> = encrypted_schema.serialize();
        let id_db = xid::Id::from_str(id_db.as_str()).unwrap();
        let id_db = id_db.as_bytes();
        let response = &self.db.insert(id_db, encoded);
        match response {
            Ok(_) => {
                let _db_response = response.clone().unwrap();
                // eprintln!("DbFolder.update :: folder_name: {} db_response : {:?}", 
                //     &folder_name, &db_response
                // );
                Ok(folder.clone())
            }
            Err(_) => {
                Err(PlanetError::new(500, Some(tr!("Could not write folder schema"))))
            }
        }
    }
}

impl DbFolder {
    pub fn get_field_id_map(
        db_folder: &DbFolder,
        folder_name: &String
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let folder = db_folder.get_by_name(folder_name).unwrap().unwrap();
        let db_fields = folder.data_objects.unwrap();
        let mut field_id_map: BTreeMap<String, String> = BTreeMap::new();
        for db_field in db_fields.keys() {
            let field_config = db_fields.get(db_field).unwrap();
            let field_id = field_config.get(ID).unwrap().clone();
            let field_name = db_field.clone();
            field_id_map.insert(field_id, field_name);
        }
        Ok(field_id_map)
    }
    pub fn get_field_name_map(
        db_folder: &DbFolder,
        folder_name: &String
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let folder = db_folder.get_by_name(folder_name)?;
        if folder.is_some() {
            let folder = folder.unwrap();
            let db_fields = folder.data_objects.unwrap();
            let mut field_id_map: BTreeMap<String, String> = BTreeMap::new();
            for db_field in db_fields.keys() {
                let field_config = db_fields.get(db_field).unwrap();
                let field_id = field_config.get(ID).unwrap().clone();
                let field_name = db_field.clone();
                field_id_map.insert(field_name, field_id);
            }
            Ok(field_id_map)
        } else {
            return Err(PlanetError::new(
                500, 
                Some(tr!("Folder does not exist")),
            ));
        }
    }
    // Get field_name -> field_type map
    pub fn get_field_type_map(
        folder: &DbData,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let db_fields = folder.data_objects.clone().unwrap();
        let mut field_type_map: BTreeMap<String, String> = BTreeMap::new();
        for db_field in db_fields.keys() {
            let field_name = db_field.clone();
            let field_map = db_fields.get(&field_name);
            if field_map.is_some() {
                let field_type = field_map.unwrap().get("field_type");
                if field_type.is_some() {
                    let field_type = field_type.unwrap().clone();
                    field_type_map.insert(field_name, field_type);
                }
            }
        }
        Ok(field_type_map)
    }
    pub fn get_item_data_by_field_names(
        item_data_id: Option<BTreeMap<String, String>>,
        field_id_map: BTreeMap<String, String>
    ) -> BTreeMap<String, String> {
        let mut item_data: BTreeMap<String, String> = BTreeMap::new();
        if item_data_id.is_some() {
            let item_data_id = item_data_id.unwrap();
            for field_id in item_data_id.keys() {
                let has_name = field_id_map.get(field_id);                    
                if has_name.is_some() {
                    let field_name = has_name.unwrap().clone();
                    let field_value = item_data_id.get(field_id).unwrap().clone();
                    item_data.insert(field_name, field_value);
                }
            }    
        }
        return item_data
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FolderItemElement(pub PropertyType);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FolderItemData {
    pub id: Option<String>,
    pub routing: RoutingData,
    pub data: Option<BTreeMap<String, FolderItemElement>>,
}
impl FolderItemData {
    pub fn defaults(
        account_id: Option<&String>, 
        site_id: Option<&String>,
        space_id: Option<&String>,
        box_id: Option<&String>,
    ) -> Self {
        let mut routing = RoutingData{
            account_id: None,
            site_id: None,
            space_id: None,
            box_id: None,
            ipfs_cid: None,
        };
        if account_id.is_some() {
            let account_id = account_id.unwrap().clone();
            routing.account_id = Some(account_id);
        }
        if site_id.is_some() {
            let site_id = site_id.unwrap().clone();
            routing.site_id = Some(site_id);
        }
        if space_id.is_some() {
            let space_id = space_id.unwrap().clone();
            routing.space_id = Some(space_id);
        }
        if box_id.is_some() {
            let box_id = box_id.unwrap().clone();
            routing.box_id = Some(box_id);
        }
        return Self{
            id: generate_id(),
            routing: routing,
            data: None,
        };
    }
}

pub enum GetItemOption {
    ById(String),
    ByName(String),
}

// let home_dir = planet_context.home_path.unwrap_or_default();
// let account_id = context.account_id.unwrap_or_default();
// let space_id = context.space_id.unwrap_or_default();
// let box_id = context.box_id.unwrap_or_default();


#[derive(Debug, Clone)]
pub struct DbFolderItem {
    pub db_folder: DbFolder,
    pub folder_id: Option<String>,
    pub home_dir: Option<String>,
    pub account_id: Option<String>,
    pub space_id: Option<String>,
    pub site_id: Option<String>,
    pub box_id: Option<String>,
}

impl DbFolderItem {

    fn get_partition(
        &self,
        item_id: &str,
    ) -> Result<u16, PlanetError> {
        let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        let account_id = self.account_id.clone().unwrap_or_default();
        let account_id = account_id.as_str();
        let space_id = self.space_id.clone().unwrap_or_default();
        let space_id = space_id.as_str();
        let box_id = self.box_id.clone().unwrap_or_default();
        let box_id = box_id.as_str();
        let site_id = self.site_id.clone().unwrap_or_default();
        let site_id = site_id.as_str();
        let partition:u16;
        let db = self.open_partitions()?;
        let id_db = xid::Id::from_str(item_id).unwrap();
        let id_db = id_db.as_bytes();
        let db_result = db.get(&id_db);
        if db_result.is_err() {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("Could not open partition database")),
                )
            )
        }
        let db_result = db_result.unwrap();
        if db_result.is_none() {
            // I need to assign a parition for this item_id, since none has been assigned, and write to db. Used for insert calls.
            let number_parition_items = db.len();
            let number_parition_items = number_parition_items.to_u16().unwrap();
            partition = number_parition_items/ITEMS_PER_PARTITION + 1;
            // write to db partition
            let mut data: BTreeMap<String, String> = BTreeMap::new();
            let item_id_string = item_id.to_string();
            data.insert(PARTITION.to_string(), partition.to_string());
            let routing_wrap = RoutingData::defaults(
                Some(account_id.to_string()),
                Some(site_id), 
                Some(space_id), 
                Some(box_id),
                None
            );    
            let db_data = DbData::defaults(
                &item_id_string, 
                Some(data), 
                None, 
                None, 
                None, 
                routing_wrap, 
                None
            )?;
            let encrypted_data = db_data.encrypt(&shared_key).unwrap();
            let encoded: Vec<u8> = encrypted_data.serialize();
            let response = db.insert(id_db, encoded);
            match response {
                Ok(_) => {
                    return Ok(partition)
                },
                Err(_) => {
                    return Err(
                        PlanetError::new(
                            500, 
                            Some(tr!("Could not write partition")),
                        )
                    )        
                }
            }
        }
        let db_result = db_result.unwrap();
        let item_db = db_result.to_vec();
        let item_ = EncryptedMessage::deserialize(item_db).unwrap();
        let item_ = DbData::decrypt_owned(
            &item_, 
            &shared_key);
        match item_ {
            Ok(_) => {
                let partition_db = item_.unwrap();
                let data = partition_db.data;
                if data.is_some() {
                    let data = data.unwrap();
                    let partition_wrap = data.get(PARTITION);
                    if partition_wrap.is_some() {
                        let partition_str = partition_wrap.unwrap().as_str();
                        partition = FromStr::from_str(partition_str).unwrap();
                        return Ok(partition)
                    }
                }
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!("Could not open partition database, no data")),
                    )
                )
            },
            Err(_) => {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!("Could not open partition database, encrypt and serialize error")),
                    )
                )
            }
        }
    }

    fn _get_current_partition(
        &self,
        folder_id: &str,
    ) -> Result<u16, PlanetError> {
        let mut path: String = String::from("");
        let home_dir = self.home_dir.clone().unwrap_or_default();
        let home_dir = home_dir.as_str();
        let account_id = self.account_id.clone().unwrap_or_default();
        let account_id = account_id.as_str();
        let space_id = self.space_id.clone().unwrap_or_default();
        let space_id = space_id.as_str();
        let box_id = self.box_id.clone().unwrap_or_default();
        let box_id = box_id.as_str();
        let partition:u16;
        if account_id != "" && space_id != "" {
            println!("DbFolderItem.get_current_partition :: account_id and space_id have been informed");
        } else if space_id == "private" {
            // .achiever-planet/{table_file} : platform wide folder (slug with underscore)
            path = format!(
                "{home}/private/boxes/{box_id}/folders/{folder_id}/partition.db", 
                home=&home_dir, 
                box_id=box_id,
                folder_id=folder_id
            );
        }
        eprintln!("DbFolderItem.get_current_partition :: path: {:?}", path);
        let config: sled::Config = sled::Config::default()
            .use_compression(true)
            .path(path);
        let result: Result<sled::Db, sled::Error> = config.open();
        match result {
            Ok(_) => {
                let db = result.unwrap();
                let number_parition_items = db.len();
                let number_parition_items = number_parition_items.to_u16().unwrap();
                partition = number_parition_items/ITEMS_PER_PARTITION + 1;
                return Ok(partition)
            },
            Err(_) => {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!("Could not open partition database")),
                    )
                )
            }
        }
    }

    fn open_partition_by_item(
        &self,
        item_id: &str,
    ) -> Result<sled::Db, PlanetError> {
        let partition = self.get_partition(item_id);
        let partition = partition.unwrap();
        if partition > MAX_PARTITIONS {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("Maximum number of partitions reached. Maximum number of items in a folder is 1,000,000")),
                )
            )
        }
        let db = self.open_partition(&partition)?;
        return Ok(db)
    }

    fn open_partition(
        &self,
        partition: &u16,
    ) -> Result<sled::Db, PlanetError> {
        let mut path: String = String::from("");
        let folder_id = self.folder_id.clone().unwrap_or_default();
        let folder_id = folder_id.as_str();
        let account_id = self.account_id.clone().unwrap_or_default();
        let account_id = account_id.as_str();
        let space_id = self.space_id.clone().unwrap_or_default();
        let space_id = space_id.as_str();
        let box_id = self.box_id.clone().unwrap_or_default();
        let box_id = box_id.as_str();
        let home_dir = self.home_dir.clone().unwrap_or_default();
        let home_dir = home_dir.as_str();
        let partition_str = partition.to_string();
        let partition_str = format!("{:0>4}", partition_str);
        if account_id != "" && space_id != "" {
            println!("DbFolderItem.open_partition :: account_id and space_id have been informed");
        } else if space_id == "private" {
            // .achiever-planet/{table_file} : platform wide folder (slug with underscore)
            path = format!(
                "{home}/private/boxes/{box_id}/folders/{folder_id}/{partition_str}.db", 
                home=&home_dir, 
                box_id=box_id,
                folder_id=folder_id,
                partition_str=partition_str,
            );
        }
        eprintln!("DbFolderItem.open_partition :: path: {:?}", path);
        let config: sled::Config = sled::Config::default()
            .use_compression(true)
            .path(path);
        let result: Result<sled::Db, sled::Error> = config.open();
        if result.is_err() {
            let error = result.unwrap_err();
            let message = error.to_string();
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("{}", message)),
                )
            )
        }
        let db = result.unwrap();
        return Ok(db)
    }

    fn close_partition(&self, db: &sled::Db) -> Result<usize, PlanetError> {
        let size = db.flush();
        if size.is_err() {
            let error = size.unwrap_err();
            let message = error.to_string();
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("{}", message)),
                )
            )
        }
        let size = size.unwrap();
        return Ok(size)
    }

    fn open_partitions(
        &self,
    ) -> Result<sled::Db, PlanetError> {
        let mut path: String = String::from("");
        let home_dir = self.home_dir.clone().unwrap_or_default();
        let home_dir = home_dir.as_str();
        let account_id = self.account_id.clone().unwrap_or_default();
        let account_id = account_id.as_str();
        let space_id = self.space_id.clone().unwrap_or_default();
        let space_id = space_id.as_str();
        let box_id = self.box_id.clone().unwrap_or_default();
        let box_id = box_id.as_str();
        let folder_id = self.folder_id.clone().unwrap_or_default();
        let folder_id = folder_id.as_str();
        if account_id != "" && space_id != "" {
            println!("DbFolderItem.open_partitions :: account_id and space_id have been informed");
        } else if space_id == "private" {
            // .achiever-planet/{table_file} : platform wide folder (slug with underscore)
            path = format!(
                "{home}/private/boxes/{box_id}/folders/{folder_id}/partition.db", 
                home=&home_dir, 
                box_id=box_id,
                folder_id=folder_id
            );
        }
        eprintln!("DbFolderItem.open_partitions :: path: {:?}", path);
        let config: sled::Config = sled::Config::default()
            .use_compression(true)
            .path(path);
        let result: Result<sled::Db, sled::Error> = config.open();
        if result.is_err() {
            let error = result.unwrap_err();
            let message = error.to_string();
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("{}", message)),
                )
            )
        }
        let db = result.unwrap();
        return Ok(db)
    }

    fn _close_partitions(&self, db: &sled::Db) -> Result<usize, PlanetError> {
        let size = db.flush();
        if size.is_err() {
            let error = size.unwrap_err();
            let message = error.to_string();
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("{}", message)),
                )
            )
        }
        let size = size.unwrap();
        return Ok(size)
    }

    fn get_partitions(&self) -> Result<Vec<u16>, PlanetError> {
        let mut list_partitions: Vec<u16> = Vec::new();
        let count = self.total_count()?;
        let number_items = count.total.to_u16().unwrap();
        // 1200 items => 2 partitions
        // 1000 items => 1 partition
        // 1950 items => 2 partitions
        let mut number_partitions = number_items/ITEMS_PER_PARTITION;
        let rest = number_items%ITEMS_PER_PARTITION;
        if rest != 0 {
            number_partitions += 1;
        }
        for item in 1..number_partitions {
            list_partitions.push(item);
        }
        return Ok(list_partitions)
    }

}

impl FolderItem for DbFolderItem {

    fn defaults(
        home_dir: &str,
        account_id: &str,
        space_id: &str,
        site_id: &str,
        box_id: &str,
        folder_id: &str,
        db_folder: &DbFolder,
    ) -> Result<DbFolderItem, PlanetError> {
        let db_row: DbFolderItem = DbFolderItem{
            home_dir: Some(home_dir.to_string()),
            account_id: Some(account_id.to_string()),
            space_id: Some(space_id.to_string()),
            box_id: Some(box_id.to_string()),
            site_id: Some(site_id.to_string()),
            db_folder: db_folder.clone(),
            folder_id: Some(folder_id.to_string()),
        };
        Ok(db_row)
    }

    fn insert(&self, folder_name: &String, db_data: &DbData) -> Result<DbData, PlanetError> {
        let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        let encrypted_data = db_data.encrypt(&shared_key).unwrap();
        let encoded: Vec<u8> = encrypted_data.serialize();
        let id = db_data.id.clone().unwrap();
        let id_db = xid::Id::from_str(id.as_str()).unwrap();
        let id_db = id_db.as_bytes();
        let db = self.open_partition_by_item(&id)?;
        let response = &db.insert(id_db, encoded);
        match response {
            Ok(_) => {
                // Get item
                let item_ = self.get(&folder_name, GetItemOption::ById(id), None);
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
                Err(PlanetError::new(500, Some(tr!("Could not insert data"))))
            }
        }
    }

    fn update(&self, db_data: &DbData) -> Result<DbData, PlanetError> {
        let db_data = db_data.clone();
        let id = db_data.id.clone().unwrap();
        let id_db = xid::Id::from_str(id.as_str()).unwrap();
        let id_db = id_db.as_bytes();
        let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        let encrypted_data = db_data.encrypt(&shared_key).unwrap();
        let encoded: Vec<u8> = encrypted_data.serialize();
        let db = self.open_partition_by_item(&id)?;
        let response = &db.insert(id_db, encoded);
        match response {
            Ok(_) => {
                let response = response.clone().unwrap();
                let item_db = response.unwrap().to_vec();
                let item = EncryptedMessage::deserialize(
                    item_db
                ).unwrap();
                let item = DbData::decrypt_owned(
                    &item, 
                    &shared_key
                );
                match item {
                    Ok(_) => {
                        let item = item.unwrap();
                        Ok(item)
                    },
                    Err(_) => {
                        let error = item.unwrap_err();
                        Err(PlanetError::new(
                            500, 
                            Some(tr!(
                                "Could not update data, encryption error: {}", error.to_string()
                            )
                        )))
                    }
                }
            },
            Err(_) => {
                Err(PlanetError::new(500, Some(tr!("Could not update data"))))
            }
        }
    }

    // We can get items by id (not changing string), and name (we search for slugified name)
    fn get(
        &self, 
        folder_name: &String, 
        by: GetItemOption, 
        fields: Option<Vec<String>>
    ) -> Result<DbData, PlanetError> {
        let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        let item_db: Vec<u8>;
        match by {
            GetItemOption::ById(id) => {
                eprintln!("DbFolderItem.get :: id: {}", &id);
                let id_db = xid::Id::from_str(&id).unwrap();
                let id_db = id_db.as_bytes();
                let db = self.open_partition_by_item(&id)?;
                let db_result = db.get(&id_db);
                if db_result.is_err() {
                    return Err(
                        PlanetError::new(
                            500, 
                            Some(tr!("Could not fetch item from database"))
                        )
                    );
                }
                let item_exsists = db_result.unwrap();
                if item_exsists.is_none() {
                    return Err(
                        PlanetError::new(
                            404, 
                            Some(tr!("Item does not exist"))
                        )
                    );
                }
                item_db = item_exsists.unwrap().to_vec();
                let item_ = EncryptedMessage::deserialize(item_db).unwrap();
                let item_ = DbData::decrypt_owned(
                    &item_, 
                    &shared_key);
                match item_ {
                    Ok(_) => {
                        let mut item = item_.unwrap();
                        if fields.is_none() {
                            Ok(item)    
                        } else {
                            // eprintln!("get :: item: {:#?}", &item);
                            // If fields is informed, then I need to remove from item.data fields not requested
                            // data: Some(
                            //     {
                            //         "c49qh6osmpv69nnrt33g": "pepito",
                            //         "c49qh6osmpv69nnrt35g": "c49qh6osmpv69nnrt370",
                            //         "c49qh6osmpv69nnrt34g": "true",
                            //         "c49qh6osmpv69nnrt350": "34",
                            //         "c49qh6osmpv69nnrt360": "c49qh6osmpv69nnrt380,c49qh6osmpv69nnrt390",
                            //         "c49qh6osmpv69nnrt340": "This is some description I want to include",
                            //     },
                            // ),
                            let fields = fields.unwrap();
                            item = self.filter_fields(&folder_name, &fields, &item)?;
                            Ok(item)
                        }
                    },
                    Err(_) => {
                        Err(PlanetError::new(500, Some(tr!("Could not fetch item from database"))))
                    }
                }
            },
            GetItemOption::ByName(name) => {
                let partitions = self.get_partitions()?;
                let this: Arc<Mutex<DbFolderItem>> = Arc::new(Mutex::new(self.clone()));
                eprintln!("DbFolderItem.get :: partitions: {:?}", &partitions);
                let mut handles= vec![];
                // Threads to fetch name from all partitions
                let wrap_db_data: Arc<Mutex<Option<DbData>>> = Arc::new(Mutex::new(None));
                let shared_key: Arc<Mutex<SharedKey>> = Arc::new(Mutex::new(shared_key));
                let name: Arc<Mutex<String>> = Arc::new(Mutex::new(name));
                for partition in partitions {
                    let this = Arc::clone(&this);
                    let wrap_db_data = Arc::clone(&wrap_db_data);
                    let shared_key = Arc::clone(&shared_key);
                    let name = Arc::clone(&name);
                    let handle = thread::spawn(move || {
                        // open partition and search for name
                        let this = this.lock().unwrap();
                        let shared_key = shared_key.lock().unwrap();
                        let name = name.lock().unwrap();
                        let db = this.open_partition(&partition).unwrap();
                        let mut my_wrap_db_data: Option<DbData> = None;
                        for db_result in db.iter() {
                            let (_, db_item) = db_result.unwrap();
                            let db_item = db_item.to_vec();
                            let item_ = EncryptedMessage::deserialize(
                                db_item
                            ).unwrap();
                            let item_ = DbData::decrypt_owned(
                                &item_, 
                                &shared_key
                            );
                            let item_db_data = item_.unwrap();
                            let item_db_data_ = item_db_data.clone();
                            let item_db_name = &item_db_data.name.unwrap();
                            if *item_db_name == *name {
                                my_wrap_db_data = Some(item_db_data_);
                                break;
                            }
                        }
                        let _ = this.close_partition(&db);
                        if my_wrap_db_data.is_some() {
                            let mut wrap_db_data = wrap_db_data.lock().unwrap();
                            *wrap_db_data = my_wrap_db_data;
                        }
                    });
                    handles.push(handle);
                }
                for handle in handles {
                    handle.join().unwrap();
                }
                let wrap_db_data_ = wrap_db_data.lock().unwrap().clone();
                let found = wrap_db_data_.is_some();
            
                if found == false {
                    return Err(
                        PlanetError::new(
                            404, 
                            Some(tr!("Item does not exist"))
                        )
                    );
                }
                return Ok(wrap_db_data_.unwrap())
            }
        }
    }
    fn count(&self, 
        folder_name: &String, 
        r#where: Option<String>, 
    ) -> Result<SelectCountResult, PlanetError> {
        let select_result = self.select(
            folder_name,
            r#where,
            Some(1),
            Some(1),
            None
        )?;
        let result = SelectCountResult{
            time: select_result.time,
            total: select_result.total,
            data_count: select_result.data_count,
        };
        return Ok(result)
    }
    fn total_count(&self) -> Result<SelectCountResult, PlanetError> {
        let t_1 = Instant::now();
        let db = self.open_partitions()?;
        let total = db.len();
        let result = SelectCountResult{
            time: t_1.elapsed().as_millis() as usize,
            total: total,
            data_count: total,
        };
        return Ok(result);
    }
    fn select(&self, 
        _folder_name: &String, 
        r#_where: Option<String>, 
        _page: Option<usize>,
        _number_items_page: Option<usize>,
        _fields: Option<Vec<String>>,
    ) -> Result<SelectResult, PlanetError> {
        // let t_1 = Instant::now();
        // let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        // let iter = self.db.iter();
        // let db_folder = self.db_folder.clone();
        // let folder = db_folder.get_by_name(folder_name)?.unwrap();
        // let field_config_map = PropertyConfig::get_property_config_map(
        //     self.planet_context,
        //     self.context,
        //     &folder
        // ).unwrap();
        // let field_config_map_wrap = Some(field_config_map.clone());
        // // eprintln!("DbFolderItem.select :: folder: {:#?}", &folder);
        // let fields_wrap = fields.clone();
        // let has_fields = fields_wrap.is_some();
        // let mut fields: Vec<String> = Vec::new();
        // if fields_wrap.is_some() {
        //     fields = fields_wrap.unwrap();
        // }
        
        // let t_total_1 = Instant::now();
        // let result_total_count = self.total_count()?;
        // let total = result_total_count.total;
        // eprintln!("DbFolderItem.select :: get total: {} µs", &t_total_1.elapsed().as_micros());
        
        // let mut items: Vec<DbData> = Vec::new();
        // let page = page.unwrap();
        // let number_items_page = number_items_page.unwrap();
        // let where_formula = r#where.clone();
        // let where_formula = where_formula.unwrap_or_default();
        // let mut count = 1;
        // Think way to return total
        let total = 0;
        let page = 0;
        let select_result = SelectResult{
            total: total,
            time: 0,
            page: page,
            data: Vec::new(),
            data_count: 0,
        };
        // let t_header = &t_1.elapsed().as_micros();
        // eprintln!("DbFolderItem.select :: t_header: {} µs", &t_header);

        // let t_f_1 = Instant::now();
        // // Check where_formula is assign or not
        // let expr = &RE_FORMULA_ASSIGN;
        // let is_assign_function = expr.is_match(&where_formula);
        // eprintln!("DbFolderItem.select :: is_assign_function: {}", &is_assign_function);
        // let formula_query = Formula::defaults(
        //     &where_formula, 
        //     &String::from("bool"), 
        //     Some(folder), 
        //     None, 
        //     Some(db_folder), 
        //     Some(folder_name.clone()), 
        //     is_assign_function,
        //     field_config_map_wrap.clone()
        // )?;
        // // eprintln!("DbFolderItem.select :: original formula_query: {:#?}", &formula_query);
        // let t_f_2 = &t_f_1.elapsed().as_micros();
        // eprintln!("select :: Time compile formula: {} µs", &t_f_2);

        // for result in iter {
        //     let t_item_1 = Instant::now();
        //     let tuple = result.unwrap();
        //     let item_db = tuple.1.to_vec();
        //     let item_ = EncryptedMessage::deserialize(item_db).unwrap();
        //     let item_ = DbData::decrypt_owned(
        //         &item_, 
        //         &shared_key);
        //     eprintln!("DbFolderItem.select :: [{}] encrypt & deser: {} µs", &count, &t_item_1.elapsed().as_micros());
        //     // let t_item_2 = Instant::now();
        //     let item = item_.unwrap().clone();
        //     let routing_response = item.clone().routing;
        //     if routing_response.is_some() {
        //         let routing = routing_response.unwrap();
        //         let account_id = routing.get(ACCOUNT_ID).unwrap().as_str();
        //         let space_id = routing.get(SPACE_ID).unwrap().as_str();
        //         if account_id == "" && space_id == "" {
        //             continue
        //         }
        //     }

        //     let formula_matches: bool;
        //     let formula_result: String;
        //     let t_item_3 = Instant::now();
        //     let data_map = item.clone().data.unwrap();

        //     formula_result = execute_formula(&formula_query, &data_map, &field_config_map)?;
        //     if formula_result == String::from("1") {
        //         formula_matches = true;
        //     } else {
        //         formula_matches = false;
        //     }
        //     eprintln!("select :: formula_matches: {}", &formula_matches);
        //     eprintln!("DbFolderItem.select :: [{}] formula exec: {} µs", &count, &t_item_3.elapsed().as_micros());

        //     let count_float: f64 = FromStr::from_str(count.to_string().as_str()).unwrap();
        //     let number_items_page_float: f64 = FromStr::from_str(
        //         number_items_page.to_string().as_str()).unwrap();
        //     let page_target = (count_float / number_items_page_float).round() + 1.0;
        //     let page_target = page_target as usize;
        //     if page_target == page && formula_matches {
        //         if &has_fields == &true {
        //             let item_new = self.filter_fields(&folder_name, &fields, &item)?;
        //             items.push(item_new);
        //         } else {
        //             items.push(item);
        //         }
        //     } else {
        //         continue
        //     }
        //     // let number_items_page_ = number_items_page as usize;
        //     count += 1;
        //     let t_item = t_item_1.elapsed().as_micros();
        //     eprintln!("DbFolderItem.select :: item [{}] : {} µs", &count-1, &t_item);
        // }
        // select_result.data = items;
        // select_result.data_count = select_result.data.len();
        // select_result.time = t_1.elapsed().as_millis() as usize;
        // eprintln!("DbFolderItem.select :: total db time: {} µs", &t_1.elapsed().as_micros());
        return Ok(select_result);
    }
}

#[derive(Debug, Clone)]
pub struct SelectResult {
    total: usize,
    time: usize,
    page: usize,
    data_count: usize,
    data: Vec<DbData>,
}

#[derive(Debug, Clone)]
pub struct SelectCountResult {
    total: usize,
    time: usize,
    data_count: usize,
}

impl DbFolderItem {

    pub fn filter_fields(&self, folder_name: &String, fields: &Vec<String>, item: &DbData) -> Result<DbData, PlanetError> {
        let fields = fields.clone();
        let mut item = item.clone();
        // field_id => field_name
        let field_id_map= DbFolder::get_field_id_map(
            &self.db_folder,
            folder_name
        )?;
        // data
        let mut data_new: BTreeMap<String, String> = BTreeMap::new();
        let db_data = item.data;
        if db_data.is_some() {
            for (field_db_id, field_value) in db_data.unwrap() {
                let field_db_name = &field_id_map.get(&field_db_id).unwrap().clone();
                for property in &fields {
                    if property.to_lowercase() == field_db_name.to_lowercase() {
                        data_new.insert(field_db_id.clone(), field_value.clone());
                    }
                }
            }
            item.data = Some(data_new);
        } else {
            item.data = None;
        }
        // data_collections
        let mut data_collections_new: BTreeMap<String, Vec<BTreeMap<String, String>>> = BTreeMap::new();
        let db_data_collections = item.data_collections;
        if db_data_collections.is_some() {
            for (field_db_id, items) in db_data_collections.unwrap() {
                let field_db_name = &field_id_map.get(&field_db_id).unwrap().clone();
                for property in &fields {
                    if property.to_lowercase() == field_db_name.to_lowercase() {
                        data_collections_new.insert(field_db_id.clone(), items.clone());
                    }
                }
            }
            item.data_collections = Some(data_collections_new);
        } else {
            item.data_collections = None;
        }
        // data_objects
        let mut data_objects_new: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
        let db_data_objects = item.data_objects;
        if db_data_objects.is_some() {
            for (field_db_id, map) in db_data_objects.unwrap() {
                let field_db_name = &field_id_map.get(&field_db_id).unwrap().clone();
                for property in &fields {
                    if property.to_lowercase() == field_db_name.to_lowercase() {
                        data_objects_new.insert(field_db_id.clone(), map.clone());
                    }
                }
            }
            item.data_objects = Some(data_objects_new);
        } else {
            item.data_objects = None;
        }
        return Ok(item);
    }
}

pub struct DbQuery {
    pub sql_where: String,
    pub page: u8,
    pub number_items: u8,
}