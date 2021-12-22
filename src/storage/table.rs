extern crate sled;
extern crate slug;

use std::str::FromStr;
use std::time::Instant;
use std::collections::BTreeMap;
use colored::Colorize;
use validator::{Validate, ValidationErrors};
use serde::{Deserialize, Serialize};
use tr::tr;
use serde_encrypt::{
    serialize::impls::BincodeSerializer, shared_key::SharedKey, traits::SerdeEncryptSharedKey,
    AsSharedKey, EncryptedMessage,
};
use slug::slugify;

use crate::planet::constants::*;
use crate::storage::{generate_id, ConfigStorageField};
use crate::planet::{PlanetError, PlanetContext, Context};
use crate::commands::table::config::{DbTableConfig, FieldConfig};
use crate::storage::constants::*;
use crate::storage::fields::*;
use crate::functions::*;

pub trait Schema<'gb> {
    fn defaults(planet_context: &'gb PlanetContext<'gb>, context: &'gb Context<'gb>) -> Result<DbTable<'gb>, PlanetError>;
    fn create(&self, db_data: &DbData) -> Result<DbData, PlanetError>;
    fn get(&self, id: &String) -> Result<DbData, PlanetError>;
    fn get_by_name(&self, table_name: &str) -> Result<Option<DbData>, PlanetError>;
}

pub trait Row<'gb> {
    fn defaults(
        table_file: &str,
        db_table: &'gb DbTable<'gb>,
        planet_context: &'gb PlanetContext<'gb>, 
        context: &'gb Context<'gb>
    ) -> Result<DbRow<'gb>, PlanetError>;
    fn insert(&self, table_name: &String, db_data: &DbData) -> Result<DbData, PlanetError>;
    fn get(&self, table_name: &String, by: GetItemOption, fields: Option<Vec<String>>) -> Result<DbData, PlanetError>;
    fn select(&self, 
        table_name: &String, 
        r#where: Option<String>, 
        page: Option<usize>,
        number_items: Option<usize>,
        fields: Option<Vec<String>>,
    ) -> Result<SelectResult, PlanetError>;
    fn count(&self, 
        table_name: &String, 
        r#where: Option<String>, 
    ) -> Result<SelectCountResult, PlanetError>;
    fn total_count(&self) -> Result<SelectCountResult, PlanetError>;
}

// lifetimes: gb (global, for contexts), db, bs

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoutingData {
    pub account_id: Option<String>,
    pub space_id: Option<String>,
    pub ipfs_cid: Option<String>,
}

impl RoutingData {
    pub fn defaults(account_id: Option<String>, space_id: Option<String>, ipfs_cid: Option<String>) -> Option<RoutingData> {
        let mut routing_wrap: Option<RoutingData> = None;
        let account_id = account_id.unwrap_or_default();
        let space_id = space_id.unwrap_or_default();
        if account_id != "" && space_id != "" {
            let routing = RoutingData{
                account_id: Some(account_id.to_string()),
                space_id: Some(space_id.to_string()),
                ipfs_cid: ipfs_cid,
            };
            routing_wrap = Some(routing);
        }
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
// data has field_id -> value, so if we change field name would not be affected

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
            let account_id = routing.account_id.clone().unwrap_or_default();
            let space_id = routing.space_id.clone().unwrap_or_default();
            let ipfs_cid = routing.ipfs_cid.clone().unwrap_or_default();
            routing_map.insert(String::from(ACCOUNT_ID), account_id.to_string());
            routing_map.insert(String::from(SPACE_ID), space_id.to_string());
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
                        Some(tr!("Could not validate table data")),
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
                let db_table: DbTable= DbTable{
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

impl<'gb> DbTable<'gb> {
    pub fn get_field_id_map(
        db_table: &DbTable,
        table_name: &String
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let table = db_table.get_by_name(table_name).unwrap().unwrap();
        let db_fields = table.data_objects.unwrap();
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
        db_table: &DbTable,
        table_name: &String
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let table = db_table.get_by_name(table_name)?;
        if table.is_some() {
            let table = table.unwrap();
            let db_fields = table.data_objects.unwrap();
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
                Some(tr!("Table does not exist")),
            ));
        }
    }
    // Get field_name -> field_type map
    pub fn get_field_type_map(
        table: &DbData,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let db_fields = table.data_objects.clone().unwrap();
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
pub struct RowItem(pub FieldType);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RowData {
    pub id: Option<String>,
    pub routing: RoutingData,
    pub data: Option<BTreeMap<String, RowItem>>,
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

pub enum GetItemOption {
    ById(String),
    ByName(String),
}

#[derive(Debug, Clone)]
pub struct DbRow<'gb> {
    pub context: &'gb Context<'gb>,
    pub planet_context: &'gb PlanetContext<'gb>,
    pub db_table: &'gb DbTable<'gb>,
    db: sled::Db,
    raw_index: sled::Tree,
    idx_index: sled::Tree,
}

impl<'gb> Row<'gb> for DbRow<'gb> {

    fn defaults(
        table_file: &str,
        db_table: &'gb DbTable<'gb>,
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
                let raw_index = db.open_tree(INDEX_PROFILE_RAW);
                let idx_index = db.open_tree(INDEX_PROFILE_IDX);
                if raw_index.is_ok() && idx_index.is_ok() {
                    let raw_index = raw_index.unwrap();
                    let idx_index = idx_index.unwrap();
                    let db_row: DbRow = DbRow{
                        context: context,
                        planet_context: planet_context,
                        db: db,
                        db_table: db_table,
                        raw_index: raw_index,
                        idx_index: idx_index,
                    };
                    Ok(db_row)
                } else {
                    let planet_error = PlanetError::new(
                        500, 
                        Some(tr!("Could not open index trees")),
                    );
                    Err(planet_error)
                }
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

    fn insert(&self, table_name: &String, db_data: &DbData) -> Result<DbData, PlanetError> {
        let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        let encrypted_data = db_data.encrypt(&shared_key).unwrap();
        let encoded: Vec<u8> = encrypted_data.serialize();
        let id = db_data.id.clone().unwrap();
        let id_db = xid::Id::from_str(id.as_str()).unwrap();
        let id_db = id_db.as_bytes();
        // TODO: The call already provides the data inserted, check response so I don't need self.get since
        //       I already have the data.
        let response = &self.db.insert(id_db, encoded);
        match response {
            Ok(_) => {
                // Get item
                let item_ = self.get(&table_name, GetItemOption::ById(id), None);
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

    // We can get items by id (not changing string), and name (we search for slugified name)
    fn get(
        &self, 
        table_name: &String, 
        by: GetItemOption, 
        fields: Option<Vec<String>>
    ) -> Result<DbData, PlanetError> {
        let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        let item_db: Vec<u8>;
        match by {
            GetItemOption::ById(id) => {
                let id_db = xid::Id::from_str(&id).unwrap();
                let id_db = id_db.as_bytes();
                let db_result = self.db.get(&id_db);
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
                            item = self.filter_fields(&table_name, &fields, &item)?;
                            Ok(item)
                        }
                    },
                    Err(_) => {
                        Err(PlanetError::new(500, Some(tr!("Could not fetch item from database"))))
                    }
                }
            },
            GetItemOption::ByName(name) => {
                let mut found = false;
                let mut wrap_db_data: Option<DbData> = None;
                for db_result in self.db.iter() {
                    let (_, db_item) = db_result.unwrap();
                    let db_item = db_item.to_vec();
                    let item_ = EncryptedMessage::deserialize(db_item).unwrap();
                    let item_ = DbData::decrypt_owned(
                        &item_, 
                        &shared_key);
                    let item_db_data = item_.unwrap();
                    let item_db_data_ = item_db_data.clone();
                    let item_db_name = &item_db_data.name.unwrap();
                    if item_db_name == &name {
                        found = true;
                        wrap_db_data = Some(item_db_data_);
                        break
                    }
                }
                if found == false {
                    return Err(
                        PlanetError::new(
                            404, 
                            Some(tr!("Item does not exist"))
                        )
                    );
                }
                return Ok(wrap_db_data.unwrap())
            }
        }
    }
    fn count(&self, 
        table_name: &String, 
        r#where: Option<String>, 
    ) -> Result<SelectCountResult, PlanetError> {
        let select_result = self.select(
            table_name,
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
        let total = self.db.len();
        let result = SelectCountResult{
            time: t_1.elapsed().as_millis() as usize,
            total: total,
            data_count: total,
        };
        return Ok(result);
    }
    fn select(&self, 
        table_name: &String, 
        r#where: Option<String>, 
        page: Option<usize>,
        number_items_page: Option<usize>,
        fields: Option<Vec<String>>,
    ) -> Result<SelectResult, PlanetError> {
        let t_1 = Instant::now();
        let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        let iter = self.db.iter();
        let db_table = self.db_table.clone();
        let table = db_table.get_by_name(table_name)?.unwrap();
        let field_config_map = FieldConfig::get_field_config_map(&table).unwrap();
        let field_config_map_wrap = Some(field_config_map.clone());
        // eprintln!("DbRow.select :: table: {:#?}", &table);
        let fields_wrap = fields.clone();
        let has_fields = fields_wrap.is_some();
        let mut fields: Vec<String> = Vec::new();
        if fields_wrap.is_some() {
            fields = fields_wrap.unwrap();
        }
        
        let t_total_1 = Instant::now();
        let result_total_count = self.total_count()?;
        let total = result_total_count.total;
        eprintln!("DbRow.select :: get total: {} µs", &t_total_1.elapsed().as_micros());
        
        let mut items: Vec<DbData> = Vec::new();
        let page = page.unwrap();
        let number_items_page = number_items_page.unwrap();
        let where_formula = r#where.clone();
        let where_formula = where_formula.unwrap_or_default();
        let mut count = 1;
        // Think way to return total
        let mut select_result = SelectResult{
            total: total,
            time: 0,
            page: page,
            data: Vec::new(),
            data_count: 0,
        };
        let t_header = &t_1.elapsed().as_micros();
        eprintln!("DbRow.select :: t_header: {} µs", &t_header);

        let t_f_1 = Instant::now();
        // Check where_formula is assign or not
        let expr = &RE_FORMULA_ASSIGN;
        let is_assign_function = expr.is_match(&where_formula);
        eprintln!("DbRow.select :: is_assign_function: {}", &is_assign_function);
        let formula_query = Formula::defaults(
            &where_formula, 
            &String::from("bool"), 
            Some(table), 
            None, 
            None, 
            Some(db_table), 
            Some(table_name.clone()), 
            is_assign_function,
            field_config_map_wrap.clone()
        )?;
        // eprintln!("DbRow.select :: original formula_query: {:#?}", &formula_query);
        let t_f_2 = &t_f_1.elapsed().as_micros();
        eprintln!("select :: Time compile formula: {} µs", &t_f_2);

        for result in iter {
            let t_item_1 = Instant::now();
            let tuple = result.unwrap();
            let item_db = tuple.1.to_vec();
            let item_ = EncryptedMessage::deserialize(item_db).unwrap();
            let item_ = DbData::decrypt_owned(
                &item_, 
                &shared_key);
            eprintln!("DbRow.select :: [{}] encrypt & deser: {} µs", &count, &t_item_1.elapsed().as_micros());
            // let t_item_2 = Instant::now();
            let item = item_.unwrap().clone();
            let routing_response = item.clone().routing;
            if routing_response.is_some() {
                let routing = routing_response.unwrap();
                let account_id = routing.get(ACCOUNT_ID).unwrap().as_str();
                let space_id = routing.get(SPACE_ID).unwrap().as_str();
                if account_id == "" && space_id == "" {
                    continue
                }
            }

            let formula_matches: bool;
            let formula_result: String;
            let t_item_3 = Instant::now();
            let data_map = item.clone().data.unwrap();

            formula_result = execute_formula(&formula_query, &data_map, &field_config_map)?;
            if formula_result == String::from("1") {
                formula_matches = true;
            } else {
                formula_matches = false;
            }
            eprintln!("select :: formula_matches: {}", &formula_matches);
            eprintln!("DbRow.select :: [{}] formula exec: {} µs", &count, &t_item_3.elapsed().as_micros());

            let count_float: f64 = FromStr::from_str(count.to_string().as_str()).unwrap();
            let number_items_page_float: f64 = FromStr::from_str(
                number_items_page.to_string().as_str()).unwrap();
            let page_target = (count_float / number_items_page_float).round() + 1.0;
            let page_target = page_target as usize;
            if page_target == page && formula_matches {
                if &has_fields == &true {
                    let item_new = self.filter_fields(&table_name, &fields, &item)?;
                    items.push(item_new);
                } else {
                    items.push(item);
                }
            } else {
                continue
            }
            // let number_items_page_ = number_items_page as usize;
            count += 1;
            let t_item = t_item_1.elapsed().as_micros();
            eprintln!("DbRow.select :: item [{}] : {} µs", &count-1, &t_item);
        }
        select_result.data = items;
        select_result.data_count = select_result.data.len();
        select_result.time = t_1.elapsed().as_millis() as usize;
        eprintln!("DbRow.select :: total db time: {} µs", &t_1.elapsed().as_micros());
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

impl<'gb> DbRow<'gb> {

    pub fn filter_fields(&self, table_name: &String, fields: &Vec<String>, item: &DbData) -> Result<DbData, PlanetError> {
        let fields = fields.clone();
        let mut item = item.clone();
        // field_id => field_name
        let field_id_map= DbTable::get_field_id_map(
            &self.db_table,
            table_name
        )?;
        // data
        let mut data_new: BTreeMap<String, String> = BTreeMap::new();
        let db_data = item.data;
        if db_data.is_some() {
            for (field_db_id, field_value) in db_data.unwrap() {
                let field_db_name = &field_id_map.get(&field_db_id).unwrap().clone();
                for field in &fields {
                    if field.to_lowercase() == field_db_name.to_lowercase() {
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
                for field in &fields {
                    if field.to_lowercase() == field_db_name.to_lowercase() {
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
                for field in &fields {
                    if field.to_lowercase() == field_db_name.to_lowercase() {
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