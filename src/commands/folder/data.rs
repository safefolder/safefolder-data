extern crate tr;
extern crate colored;
extern crate slug;

use std::collections::{BTreeMap, HashMap};
use std::time::Instant;

use tr::tr;
use colored::*;
use regex::Regex;
use slug::slugify;
use std::str::FromStr;


use crate::commands::folder::config::*;
use crate::storage::constants::*;
use crate::commands::folder::{Command};
use crate::commands::{CommandRunner};
use crate::planet::constants::{ID, NAME};
use crate::storage::folder::{DbFolder, DbFolderItem, FolderItem, FolderSchema, DbData, GetItemOption};
use crate::storage::folder::*;
use crate::storage::{ConfigStorageProperty, generate_id};
use crate::planet::{
    PlanetContext, 
    PlanetError,
    Context,
    validation::PlanetValidationError,
};
use crate::storage::properties::{text::*, StorageProperty, ObjectStorageProperty};
use crate::storage::properties::number::*;
use crate::storage::properties::date::*;
use crate::storage::properties::formula::*;
use crate::storage::properties::reference::*;

pub struct InsertIntoFolder<'gb> {
    pub planet_context: &'gb PlanetContext<'gb>,
    pub context: &'gb Context<'gb>,
    pub db_folder: &'gb DbFolder<'gb>,
    pub config: InsertIntoFolderConfig,
}

impl<'gb> InsertIntoFolder<'gb> {

    fn get_insert_id_data_map(
        &self,
        insert_data_map: &BTreeMap<String, String>,
        folder_data: &BTreeMap<String, BTreeMap<String, String>>,
    ) -> (BTreeMap<String, String>, BTreeMap<String, String>) {
        let mut insert_id_data_map: BTreeMap<String, String> = BTreeMap::new();
        let mut insert_id_data_objects_map: BTreeMap<String, String> = BTreeMap::new();
        for (name, value) in insert_data_map.clone() {
            let id_map = folder_data.get(&name);
            if id_map.is_some() {
                let id_map = id_map.unwrap();
                let id = id_map.get(ID);
                let property_type = id_map.get(PROPERTY_TYPE);
                if id.is_some() {
                    let id = id.unwrap().clone();
                    let property_type = property_type.unwrap().clone();
                    if property_type == PROPERTY_TYPE_LINK.to_string() {
                        // property_id => remote folder item id
                        insert_id_data_objects_map.insert(id, value);
                    } else {
                        insert_id_data_map.insert(id, value);
                    }
                }
            }
        }
        return (insert_id_data_map, insert_id_data_objects_map)
    }

    fn get_insert_id_data_collections_map(
        &self,
        insert_data_collections_map: Option<BTreeMap<String, Vec<String>>>,
        folder_data: &BTreeMap<String, BTreeMap<String, String>>,
    ) -> BTreeMap<String, Vec<String>> {
        let mut insert_id_data_collections_map: BTreeMap<String, Vec<String>> = BTreeMap::new();
        if insert_data_collections_map.is_some() {
            // I receive a map of list of ids
            let insert_data_collections_map = insert_data_collections_map.unwrap();
            for (name, id_list) in insert_data_collections_map {
                let id_map = folder_data.get(&name);
                if id_map.is_some() {
                    let id_map = id_map.unwrap();
                    let id = id_map.get(ID);
                    if id.is_some() {
                        let id = id.unwrap().clone();
                        insert_id_data_collections_map.insert(id, id_list);
                    }
                }
            }
        }
        return insert_id_data_collections_map
    }

    pub fn run(&self) -> Result<DbData, Vec<PlanetError>> {
        let t_1 = Instant::now();
        let command = self.config.command.clone().unwrap_or_default();
        let expr = Regex::new(r#"(INSERT INTO FOLDER) "(?P<folder_name>[a-zA-Z0-9_ ]+)"#).unwrap();
        let folder_name_match = expr.captures(&command).unwrap();
        let folder_name = &folder_name_match["folder_name"].to_string();
        // eprintln!("InsertIntoFolder.run :: folder_name: {}", folder_name);

        // routing
        let account_id = Some(self.context.account_id.unwrap_or_default().to_string());
        let site_id = self.context.site_id;
        let space_id = self.context.space_id;
        let box_id = self.context.box_id;

        // folder
        let mut errors: Vec<PlanetError> = Vec::new();
        let folder = self.db_folder.get_by_name(folder_name);
        if folder.is_err() {
            let error = folder.unwrap_err();
            errors.push(error);
            return Err(errors);
        }
        let folder = folder.unwrap();
        if *&folder.is_none() {
            errors.push(
                PlanetError::new(
                    500, 
                    Some(tr!("Could not find folder {}", &folder_name)),
                )
            );
            return Err(errors);
        }    

        let folder = folder.unwrap();
        let folder_name = &folder.clone().name.unwrap();
        // eprintln!("InsertIntoFolder.run :: Got folder! folder_name: {}", folder_name);
        let folder_id = folder.clone().id.unwrap();

        let result: Result<DbFolderItem<'gb>, PlanetError> = DbFolderItem::defaults(
            folder_id.as_str(),
            self.db_folder,
            self.planet_context,
            self.context,
        );
        match result {
            Ok(_) => {
                let db_row: DbFolderItem<'gb> = result.unwrap();

                // routing
                let routing_wrap = RoutingData::defaults(
                    account_id,
                    site_id, 
                    space_id, 
                    box_id,
                    None
                );
                
                // eprintln!("InsertIntoFolder.run :: folder: {:#?}", &folder);

                // I need a way to get list of instance PropertyConfig (properties)
                let config_properties = PropertyConfig::parse_from_db(
                    self.planet_context,
                    self.context,
                    &folder
                );
                if config_properties.is_err() {
                    let error = config_properties.unwrap_err();
                    errors.push(error);
                    return Err(errors);
                }
                let config_properties = config_properties.unwrap();
                // eprintln!("InsertIntoFolder.run :: config_properties: {:#?}", &config_properties);

                let insert_data_map: BTreeMap<String, String> = self.config.data.clone().unwrap();
                let insert_data_collections_map = self.config.data_collections.clone();
                // I need to have {id} -> Value
                let folder_data = folder.clone().data_objects.unwrap();

                // get id => value for data, data_objects and data_collections
                let (
                    insert_id_data_map, 
                    insert_id_data_objects_map
                ) = self.get_insert_id_data_map(
                    &insert_data_map, &folder_data
                );
                let insert_id_data_collections_map = self.get_insert_id_data_collections_map(
                    insert_data_collections_map, &folder_data
                );
                
                // process insert config data_collections
                // User authentication
                // TODO: Complete when implement the permission system exchange token by user_id
                let user_id = generate_id().unwrap();
                
                // let insert_data_collections_map = self.config.data_collections.clone().unwrap();
                // eprintln!("InsertIntoFolder.run :: insert_data__collections_map: {:#?}", &insert_data_collections_map);
                // TODO: Change for the item name
                // We will use this when we have the Name property, which is required in all tables
                // eprintln!("InsertIntoFolder.run :: routing_wrap: {:#?}", &routing_wrap);

                // Keep in mind on name attribute for DbData
                // 1. Can be small text or any other property, so we need to do validation and generation of data...
                // 2. Becaouse if formula is generated from other properties, is generated number or id is also generated
                // I also need a set of properties allowed to be name (Small Text, Formula), but this in future....
                // name on YAML not required, since can be generated
                // Check property type and attribute to validate
                // So far only take Small Text
                let name_field: PropertyConfig = PropertyConfig::get_name_property(&folder).unwrap();
                let name_field_type = name_field.property_type.unwrap().clone();
                let insert_name = self.config.name.clone();
                // Only support so far Small Text and needs to be informed in YAML with name
                if name_field_type != PROPERTY_TYPE_SMALL_TEXT.to_string() || insert_name.is_none() {
                    errors.push(
                        PlanetError::new(
                            500, 
                            Some(tr!("You need to include name property when inserting data into database.
                             Only \"Small Text\" supported so far")),
                        )
                    );
                }
                let name = insert_name.unwrap();
                // Check name does not exist
                // eprintln!("InsertIntoFolder.run :: name: {}", &name);
                let name_exists = self.check_name_exists(&folder_name, &name, &db_row);
                eprintln!("InsertIntoFolder.run :: record name_exists: {}", &name_exists);
                if name_exists {
                    errors.push(
                        PlanetError::new(
                            500, 
                            Some(tr!("A record with name \"{}\" already exists in database", &name)),
                        )
                    );
                }

                // Instantiate DbData and validate
                let mut db_context: BTreeMap<String, String> = BTreeMap::new();
                db_context.insert(FOLDER_NAME.to_string(), folder_name.clone());
                let db_data = DbData::defaults(
                    &name,
                    None,
                    None,
                    None,
                    None,
                    routing_wrap,
                    None,
                );
                if db_data.is_err() {
                    let error = db_data.unwrap_err();
                    errors.push(error);
                    return Err(errors)
                }
                let mut db_data = db_data.unwrap();
                let mut data: BTreeMap<String, String> = BTreeMap::new();
                let mut data_objects: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
                let mut data_collections: BTreeMap<String, Vec<BTreeMap<String, String>>> = BTreeMap::new();
                let mut property_config_map: BTreeMap<String, PropertyConfig> = BTreeMap::new();
                for property in config_properties.clone() {
                    let field_name = property.name.clone().unwrap();
                    property_config_map.insert(field_name, property.clone());
                }
                let mut links_map: HashMap<String, Vec<PropertyConfig>> = HashMap::new();
                let mut links_data_map: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();
                for property in config_properties {
                    let mut property_data: Option<Vec<String>> = None;
                    let property_config = property.clone();
                    let property_type = property.property_type.unwrap_or_default();
                    let property_type = property_type.as_str();
                    let property_name = property.name.unwrap();
                    eprintln!("InsertIntoFolder.run :: \"{}\" property_type: {}", &property_name, &property_type);
                    // I always have a property id
                    let property_id = property.id.unwrap_or_default();
                    
                    let data_item = insert_id_data_map.get(&property_id);
                    if data_item.is_some() {
                        eprintln!("InsertIntoFolder.run :: have data_item...");
                        let data_item = data_item.unwrap().clone();
                        let mut items = Vec::new();
                        items.push(data_item);
                        property_data = Some(items);
                    }
                    let data_objects_item = insert_id_data_objects_map.get(
                        &property_id
                    );
                    if data_objects_item.is_some() && property_data.is_none() {
                        eprintln!("InsertIntoFolder.run :: have data_objects_item...");
                        let mut items = Vec::new();
                        items.push(data_objects_item.unwrap().clone());
                        property_data = Some(items);
                    }
                    let data_collections_item = insert_id_data_collections_map.get(
                        &property_id
                    );
                    if data_collections_item.is_some() && property_data.is_none() {
                        eprintln!("InsertIntoFolder.run :: have data_collections_item...");
                        let data_collections_item = data_collections_item.unwrap().clone();
                        property_data = Some(data_collections_item);
                    }
                    // In case we don't have any value and is system generated we skip
                    eprintln!("InsertIntoFolder.run :: before check .. property_data: {:?}", &property_data);
                    if property_data.is_none() &&
                        (
                            property_type != PROPERTY_TYPE_FORMULA && 
                            property_type != PROPERTY_TYPE_CREATED_TIME && 
                            property_type != PROPERTY_TYPE_LAST_MODIFIED_TIME
                        ) {
                            eprintln!("InsertIntoFolder.run :: \"{}\" I skip...", &property_name);
                        continue
                    }
                    let property_data_: Vec<String>;
                    if property_data.is_some() {
                        property_data_ = property_data.clone().unwrap().clone();
                    } else {
                        let mut items = Vec::new();
                        items.push(String::from(""));
                        property_data_ = items;
                    }
                    let property_data = property_data_;
                    eprintln!("InsertIntoFolder.run :: \"{}\" property_data: {:?}", &property_name, &property_data);
                    let mut property_data_wrap: Result<String, PlanetError> = Ok(String::from(""));
                    let mut skip_data_assign = false;
                    match property_type {
                        PROPERTY_TYPE_SMALL_TEXT => {
                            let obj = SmallTextProperty::defaults(&property_config);
                            property_data_wrap = obj.validate(&property_data[0]);
                        },
                        PROPERTY_TYPE_LONG_TEXT => {
                            let obj = LongTextProperty::defaults(&property_config);
                            property_data_wrap = obj.validate(&property_data[0]);
                        },
                        PROPERTY_TYPE_CHECKBOX => {
                            let obj = CheckBoxProperty::defaults(&property_config);
                            property_data_wrap = obj.validate(&property_data[0]);
                        },
                        PROPERTY_TYPE_NUMBER => {
                            let obj = NumberProperty::defaults(&property_config);
                            property_data_wrap = obj.validate(&property_data[0]);
                        },
                        PROPERTY_TYPE_SELECT => {
                            let obj = SelectProperty::defaults(
                                &property_config, 
                                Some(&folder)
                            );
                            property_data_wrap = obj.validate(&property_data[0]);
                        },
                        PROPERTY_TYPE_FORMULA => {
                            let obj = FormulaProperty::defaults(&property_config);
                            property_data_wrap = obj.validate(&data, &property_config_map);
                        },
                        PROPERTY_TYPE_DATE => {
                            let obj = DateProperty::defaults(&property_config);
                            property_data_wrap = obj.validate(&property_data[0]);
                        },
                        PROPERTY_TYPE_DURATION => {
                            let obj = DurationProperty::defaults(&property_config);
                            property_data_wrap = obj.validate(&property_data[0]);
                        },
                        PROPERTY_TYPE_CREATED_TIME => {
                            let obj = AuditDateProperty::defaults(&property_config);
                            property_data_wrap = obj.validate(&property_data[0]);
                        },
                        PROPERTY_TYPE_LAST_MODIFIED_TIME => {
                            let obj = AuditDateProperty::defaults(&property_config);
                            property_data_wrap = obj.validate(&property_data[0]);
                        },
                        PROPERTY_TYPE_CREATED_BY => {
                            let obj = AuditByProperty::defaults(&property_config);
                            property_data_wrap = obj.validate(&user_id);
                        },
                        PROPERTY_TYPE_LAST_MODIFIED_BY => {
                            let obj = AuditByProperty::defaults(&property_config);
                            property_data_wrap = obj.validate(&user_id);
                        },
                        PROPERTY_TYPE_CURRENCY => {
                            let obj = CurrencyProperty::defaults(&property_config);
                            property_data_wrap = obj.validate(&property_data[0]);
                        },
                        PROPERTY_TYPE_PERCENTAGE => {
                            let obj = PercentageProperty::defaults(&property_config);
                            property_data_wrap = obj.validate(&property_data[0]);
                        },
                        PROPERTY_TYPE_LINK => {
                            let obj = LinkProperty::defaults(
                                self.planet_context,
                                self.context,
                                &property_config,
                                Some(&self.db_folder),
                            );
                            let result = obj.validate(&property_data);
                            if result.is_err() {
                                let error = result.clone().err().unwrap();
                                errors.push(error);
                            }
                            let id_list = result.unwrap();
                            let many = property_config.many.unwrap();
                            if many {
                                let mut items: Vec<BTreeMap<String, String>> = Vec::new();
                                for item_id in id_list.clone() {
                                    let mut map: BTreeMap<String, String> = BTreeMap::new();
                                    map.insert(ID.to_string(), item_id);
                                    items.push(map);
                                }
                                data_collections.insert(property_id.clone(), items);
                            } else {
                                let mut map: BTreeMap<String, String> = BTreeMap::new();
                                let value = id_list[0].clone();
                                map.insert(ID.to_string(), value);
                                data_objects.insert(property_id.clone(), map);
                            }
                            skip_data_assign = true;
                            // links_map
                            let linked_folder_id = property_config.clone().linked_folder_id.unwrap();
                            let map_item = links_map.get(
                                &linked_folder_id
                            );
                            if map_item.is_some() {
                                let mut array = map_item.unwrap().clone();
                                array.push(property_config);
                                links_map.insert(property_id.clone(), array.clone());
                            } else {
                                let mut array: Vec<PropertyConfig> = Vec::new();
                                array.push(property_config);
                                links_map.insert(property_id.clone(), array);
                            }
                            // links_data_map
                            // address folder id => {"Home Addresses" => [jdskdsj], "Work Addresses": [djdks8dsjk]}
                            let map_item_data = links_data_map.get(&linked_folder_id);
                            if map_item_data.is_some() {
                                let mut my_map = map_item_data.unwrap().clone();
                                let my_list_wrap = my_map.get(&property_name.clone());
                                let mut my_list: Vec<String>;
                                if my_list_wrap.is_some() {
                                    my_list = my_list_wrap.unwrap().clone();
                                } else {
                                    my_list = Vec::new();
                                }
                                for item_id in id_list.clone() {
                                    my_list.push(item_id);
                                }
                                my_map.insert(property_name.clone(), my_list);
                                links_data_map.insert(property_id.clone(), my_map);
                            } else {
                                let mut my_map: HashMap<String, Vec<String>> = HashMap::new();
                                let mut my_list: Vec<String> = Vec::new();
                                for item_id in id_list.clone() {
                                    my_list.push(item_id);
                                }
                                my_map.insert(property_name.clone(), my_list);
                                links_data_map.insert(property_id.clone(), my_map);
                            }
                        },
                        _ => {
                            errors.push(
                                PlanetError::new(
                                    500, 
                                    Some(tr!("Field \"{}\" not supported.", &property_type)),
                                )
                            );
                        }
                    };
                    eprintln!("InsertIntoFolder.run :: \"{}\" skip_data_assign: {} data: {} objects: {} collections: {}", 
                        &property_name,
                        &skip_data_assign,
                        &property_data_wrap.is_ok(),
                        &data_objects.len(),
                        &data_collections.len(),
                    );
                    if skip_data_assign == false {
                        let tuple = handle_field_response(
                            &property_data_wrap, &errors, &property_id, &data
                        );
                        data = tuple.0;
                        errors = tuple.1;
                    }
                }
                if errors.len() > 0 {
                    return Err(errors)
                }
                db_data.data = Some(data);
                db_data.data_objects = None;
                db_data.data_collections = None;
                if data_objects.keys().len() != 0 {
                    db_data.data_objects = Some(data_objects);
                }
                if data_collections.keys().len() != 0 {
                    db_data.data_collections = Some(data_collections);
                }
                eprintln!("InsertIntoFolder.run :: I will write: {:#?}", &db_data);
                let response= db_row.insert(&folder_name, &db_data);
                if response.is_err() {
                    let error = response.unwrap_err();
                    errors.push(error);
                    return Err(errors)
                }
                let response = response.unwrap();
                let id_record = response.clone().id.unwrap();
                eprintln!("InsertIntoFolder.run :: time: {} µs", &t_1.elapsed().as_micros());
                // links
                for (property_id, config_property_list) in links_map {
                    // Get db item for this link
                    for config in config_property_list {
                        let many = config.many.unwrap();
                        let remote_folder_id = config.linked_folder_id;
                        if remote_folder_id.is_none() {
                            continue
                        }
                        let remote_folder_id = remote_folder_id.unwrap();
                        let folder = self.db_folder.get(&remote_folder_id).unwrap();
                        let folder_name = folder.name.unwrap();
                        let main_data_map = links_data_map.get(&property_id);
                        if main_data_map.is_some() {
                            let main_data_map = main_data_map.unwrap();
                            for (_property_name, id_list) in main_data_map {
                                for item_id in id_list {
                                    let result: Result<DbFolderItem<'gb>, PlanetError> = DbFolderItem::defaults(
                                        remote_folder_id.as_str(),
                                        self.db_folder,
                                        self.planet_context,
                                        self.context,
                                    );
                                    if result.is_err() {
                                        // Return error about database problem
                                    }
                                    let db_row = result.unwrap();
                                    let linked_item = db_row.get(
                                        &folder_name, 
                                        GetItemOption::ById(item_id.clone()), 
                                        None
                                    );
                                    if linked_item.is_ok() {
                                        let mut linked_item = linked_item.unwrap();
                                        // I may need to update to data_objects or data_collections
                                        if many {
                                            let data_collections_wrap = linked_item.data_collections;
                                            let mut data_collections: BTreeMap<String, Vec<BTreeMap<String, String>>>;
                                            if data_collections_wrap.is_some() {
                                                data_collections = data_collections_wrap.unwrap();
                                            } else {
                                                data_collections = BTreeMap::new();
                                            }
                                            let list_wrap = data_collections.get(&property_id);
                                            let mut list: Vec<BTreeMap<String, String>>;
                                            if list_wrap.is_some() {
                                                list = list_wrap.unwrap().clone();
                                                let mut item_object: BTreeMap<String, String> = BTreeMap::new();
                                                item_object.insert(ID.to_string(), id_record.clone());
                                                list.push(item_object);
                                            } else {
                                                list = Vec::new();
                                                let mut item_object: BTreeMap<String, String> = BTreeMap::new();
                                                item_object.insert(ID.to_string(), id_record.clone());
                                                list.push(item_object);
                                            }
                                            data_collections.insert(property_id.clone(), list);
                                            linked_item.data_collections = Some(data_collections);
                                            let _linked_item = db_row.update(&linked_item);
                                        } else {
                                            let data_objects_wrap = linked_item.data_objects;
                                            let mut data_objects: BTreeMap<String, BTreeMap<String, String>>;
                                            if data_objects_wrap.is_some() {
                                                data_objects = data_objects_wrap.unwrap();
                                            } else {
                                                data_objects = BTreeMap::new();
                                            }
                                            let mut item_object: BTreeMap<String, String> = BTreeMap::new();
                                            item_object.insert(ID.to_string(), id_record.clone());
                                            data_objects.insert(property_id.clone(), item_object);
                                            linked_item.data_objects = Some(data_objects);
                                            let _linked_item = db_row.update(&linked_item);
                                        }
                                    } else {
                                        let error = linked_item.unwrap_err();
                                        eprintln!("InsertIntoFolder.run :: I have error on get linked_item: {}", &error.message);
                                    }
                                }
                            }
                        }
                    }
                }
                return Ok(response);
            },
            Err(error) => {
                errors.push(error);
                return Err(errors);
            }
        }
    }

    pub fn runner(runner: &CommandRunner, path_yaml: &String) -> () {
        let t_1 = Instant::now();
        let config_ = InsertIntoFolderConfig::defaults(None);
        let config: Result<InsertIntoFolderConfig, Vec<PlanetValidationError>> = config_.import(
            runner.planet_context,
            &path_yaml
        );
        match config {
            Ok(_) => {
                let db_folder= DbFolder::defaults(
                    runner.planet_context,
                    runner.context,
                ).unwrap();
        
                let insert_into_table: InsertIntoFolder = InsertIntoFolder{
                    planet_context: runner.planet_context,
                    context: runner.context,
                    config: config.unwrap(),
                    db_folder: &db_folder,
                };
                let result: Result<_, Vec<PlanetError>> = insert_into_table.run();
                match result {
                    Ok(_) => {
                        println!();
                        println!("{}", String::from("[OK]").green());
                        eprint!("Time: {} ms", &t_1.elapsed().as_millis());
                    },
                    Err(errors) => {
                        let count = 1;
                        println!();
                        println!("{}", tr!("I found these errors").red().bold());
                        println!("{}", "--------------------".red());
                        println!();
                        for error in errors {
                            println!(
                                "{}{} {}", 
                                count.to_string().blue(),
                                String::from('.').blue(),
                                error.message
                            );
                        }
                    }
                }
            },
            Err(errors) => {
                println!();
                println!("{}", tr!("I found these errors").red().bold());
                println!("{}", "--------------------".red());
                println!();
                let mut count = 1;
                for error in errors {
                    println!(
                        "{}{} {}", 
                        count.to_string().blue(), 
                        String::from('.').blue(), 
                        error.message
                    );
                    count += 1;
                }
            }
        }
    }
}

impl<'gb> InsertIntoFolder<'gb> {
    pub fn check_name_exists(&self, folder_name: &String, name: &String, db_row: &DbFolderItem) -> bool {
        let check: bool;
        let name = name.clone();
        let result = db_row.get(&folder_name, GetItemOption::ByName(name), None);
        eprintln!("InsertIntoFolder.check_name_exists :: get response: {:#?}", &result);
        match result {
            Ok(_) => {
                check = true
            },
            Err(_) => {
                check = false
            }
        }
        return check
    }
}

pub struct GetFromFolder<'gb> {
    pub planet_context: &'gb PlanetContext<'gb>,
    pub context: &'gb Context<'gb>,
    pub db_folder: &'gb DbFolder<'gb>,
    pub config: GetFromFolderConfig,
}

impl<'gb> Command<String> for GetFromFolder<'gb> {

    fn run(&self) -> Result<String, PlanetError> {
        let command = self.config.command.clone().unwrap_or_default();
        let expr = Regex::new(r#"(GET FROM TABLE) "(?P<folder_name>[a-zA-Z0-9_ ]+)""#).unwrap();
        let table_name_match = expr.captures(&command).unwrap();
        let folder_name = &table_name_match["folder_name"].to_string();
        let folder_file = slugify(&folder_name);
        let folder_file = folder_file.as_str().replace("-", "_");

        let result: Result<DbFolderItem<'gb>, PlanetError> = DbFolderItem::defaults(
            &folder_file,
            self.db_folder,
            self.planet_context,
            self.context,
        );
        match result {
            Ok(_) => {
                // let data_config = self.config.data.clone();
                let db_row: DbFolderItem<'gb> = result.unwrap();
                // I need to get SchemaData and schema for the folder
                // I go through properties in order to build RowData                
                let folder = self.db_folder.get_by_name(folder_name)?;
                if *&folder.is_none() {
                    return Err(
                        PlanetError::new(
                            500, 
                            Some(tr!("Could not find folder {}", &folder_name)),
                        )
                    );
                }
                let folder = folder.unwrap();
                let data_collections = folder.clone().data_collections;
                let field_ids = data_collections.unwrap().get(PROPERTY_IDS).unwrap().clone();
                let config_properties = PropertyConfig::parse_from_db(
                    self.planet_context,
                    self.context,
                    &folder
                )?;
                let field_id_map: BTreeMap<String, PropertyConfig> = PropertyConfig::get_property_id_map(&config_properties)?;
                let properties = self.config.data.clone().unwrap().properties;
                eprintln!("GetFromFolder.run :: properties: {:?}", &properties);
                let item_id = self.config.data.clone().unwrap().id.unwrap();
                // Get item from database
                let db_data = db_row.get(&folder_name, GetItemOption::ById(item_id), properties)?;
                // data and basic properties
                let data = db_data.data;
                let mut yaml_out_str = String::from("---\n");
                // id
                let id_yaml_value = self.config.data.clone().unwrap().id.unwrap().truecolor(
                    YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
                );
                let id_yaml = format!("{}", 
                    id_yaml_value.truecolor(YAML_COLOR_ORANGE[0], YAML_COLOR_ORANGE[1], YAML_COLOR_ORANGE[2]), 
                );
                yaml_out_str.push_str(format!("{property}: {value}\n", 
                    property=String::from(ID).truecolor(
                        YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
                    ), 
                    value=&id_yaml
                ).as_str());
                // name
                let name_yaml_value = &db_data.name.unwrap().clone();
                let name_yaml = format!("{}", 
                    name_yaml_value.truecolor(YAML_COLOR_ORANGE[0], YAML_COLOR_ORANGE[1], YAML_COLOR_ORANGE[2]), 
                );
                yaml_out_str.push_str(format!("{property}: {value}\n", 
                    property=String::from(NAME).truecolor(
                        YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
                    ), 
                    value=&name_yaml
                ).as_str());
                yaml_out_str.push_str(format!("{}\n", 
                    String::from("data:").truecolor(YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]),
                ).as_str());
                if data.is_some() {
                    // property_id -> string value
                    let data = data.unwrap();
                    // I need to go through in same order as properties were registered in PropertyConfig when creating schema
                    for field_id_data in field_ids {
                        let property_id = field_id_data.get(ID).unwrap();
                        let property_config = field_id_map.get(property_id).unwrap().clone();
                        let field_config_ = property_config.clone();
                        let property_type = property_config.property_type.unwrap();
                        let property_type =property_type.as_str();
                        let value = data.get(property_id);
                        if value.is_none() {
                            continue
                        }
                        let value = value.unwrap();
                        // Get will return YAML document for the data
                        match property_type {
                            PROPERTY_TYPE_SMALL_TEXT => {
                                let obj = SmallTextProperty::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            PROPERTY_TYPE_LONG_TEXT => {
                                let obj = LongTextProperty::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            PROPERTY_TYPE_CHECKBOX => {
                                let obj = CheckBoxProperty::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            PROPERTY_TYPE_NUMBER => {
                                let obj = NumberProperty::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            PROPERTY_TYPE_SELECT => {
                                let obj = SelectProperty::defaults(&field_config_, Some(&folder));
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            PROPERTY_TYPE_FORMULA => {
                                let obj = FormulaProperty::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            PROPERTY_TYPE_DATE => {
                                let obj = DateProperty::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            PROPERTY_TYPE_DURATION => {
                                let obj = DurationProperty::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },                            
                            PROPERTY_TYPE_CREATED_TIME => {
                                let obj = AuditDateProperty::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            PROPERTY_TYPE_LAST_MODIFIED_TIME => {
                                let obj = AuditDateProperty::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            PROPERTY_TYPE_CREATED_BY => {
                                let obj = AuditByProperty::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            PROPERTY_TYPE_LAST_MODIFIED_BY => {
                                let obj = AuditByProperty::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            PROPERTY_TYPE_CURRENCY => {
                                let obj = CurrencyProperty::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            PROPERTY_TYPE_PERCENTAGE => {
                                let obj = PercentageProperty::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            _ => {
                                yaml_out_str = yaml_out_str;
                            }
                        }
                    }
                }
                eprintln!("{}", yaml_out_str);
                return Ok(yaml_out_str);
            },
            Err(error) => {
                return Err(error);
            }
        }
    }

    fn runner(runner: &CommandRunner, path_yaml: &String) -> () {
        let config_ = GetFromFolderConfig::defaults(
            String::from("")
        );
        let config: Result<GetFromFolderConfig, Vec<PlanetValidationError>> = config_.import(
            runner.planet_context,
            &path_yaml
        );
        match config {
            Ok(_) => {
                let db_folder= DbFolder::defaults(
                    runner.planet_context,
                    runner.context,
                ).unwrap();

                let insert_into_table: GetFromFolder = GetFromFolder{
                    planet_context: runner.planet_context,
                    context: runner.context,
                    config: config.unwrap(),
                    db_folder: &db_folder,
                };
                let result: Result<_, PlanetError> = insert_into_table.run();
                match result {
                    Ok(_) => {
                        println!();
                        println!("{}", String::from("[OK]").green());
                    },
                    Err(error) => {
                        let count = 1;
                        println!();
                        println!("{}", tr!("I found these errors").red().bold());
                        println!("{}", "--------------------".red());
                        println!();
                        println!(
                            "{}{} {}", 
                            count.to_string().blue(),
                            String::from('.').blue(),
                            error.message
                        );
                    }
                }
            },
            Err(errors) => {
                println!();
                println!("{}", tr!("I found these errors").red().bold());
                println!("{}", "--------------------".red());
                println!();
                let mut count = 1;
                for error in errors {
                    println!(
                        "{}{} {}", 
                        count.to_string().blue(), 
                        String::from('.').blue(), 
                        error.message
                    );
                    count += 1;
                }
            }
        }
    }
}

pub struct SelectFromFolder<'gb> {
    pub planet_context: &'gb PlanetContext<'gb>,
    pub context: &'gb Context<'gb>,
    pub db_folder: &'gb DbFolder<'gb>,
    pub config: SelectFromFolderConfig,
}

impl<'gb> Command<String> for SelectFromFolder<'gb> {

    fn run(&self) -> Result<String, PlanetError> {
        let command = self.config.command.clone().unwrap_or_default();
        let expr = Regex::new(r#"(SELECT FROM TABLE) "(?P<folder_name>[a-zA-Z0-9_ ]+)""#).unwrap();
        let table_name_match = expr.captures(&command).unwrap();
        let folder_name = &table_name_match["folder_name"].to_string();
        let folder_file = slugify(&folder_name);
        let folder_file = folder_file.as_str().replace("-", "_");
        eprintln!("SelectFromFolder.run :: folder_file: {}", &folder_file);

        let result: Result<DbFolderItem<'gb>, PlanetError> = DbFolderItem::defaults(
            &folder_file,
            self.db_folder,
            self.planet_context,
            self.context,
        );
        match result {
            Ok(_) => {
                let db_row: DbFolderItem<'gb> = result.unwrap();
                let config = self.config.clone();
                let r#where = config.r#where;
                let page = config.page;
                let number_items = config.number_items;
                let properties = config.properties;
                let mut page_wrap: Option<usize> = None;
                let mut number_items_wrap: Option<usize> = None;
                if page.is_some() {
                    let page_string = page.unwrap();
                    let page_number: usize = FromStr::from_str(page_string.as_str()).unwrap();
                    page_wrap = Some(page_number)
                }
                if number_items.is_some() {
                    let number_items_string = number_items.unwrap();
                    let number_items: usize = FromStr::from_str(number_items_string.as_str()).unwrap();
                    number_items_wrap = Some(number_items)
                }
                let result = db_row.select(
                    folder_name, 
                    r#where, 
                    page_wrap, 
                    number_items_wrap, 
                    properties,
                )?;
                eprintln!("SelectFromFolder :: result: {:#?}", &result);
                // Later on, I do pretty print
            },
            Err(error) => {
                return Err(error);
            }
        }

        return Ok(String::from(""));
    }
    fn runner(runner: &CommandRunner, path_yaml: &String) -> () {
        let config_ = SelectFromFolderConfig::defaults(
            None,
            None,
            None
        );
        let config: Result<SelectFromFolderConfig, Vec<PlanetValidationError>> = config_.import(
            runner.planet_context,
            &path_yaml
        );
        match config {
            Ok(_) => {
                let db_folder= DbFolder::defaults(
                    runner.planet_context,
                    runner.context,
                ).unwrap();

                let select_from_table: SelectFromFolder = SelectFromFolder{
                    planet_context: runner.planet_context,
                    context: runner.context,
                    config: config.unwrap(),
                    db_folder: &db_folder,
                };
                let result: Result<_, PlanetError> = select_from_table.run();
                match result {
                    Ok(_) => {
                        println!();
                        println!("{}", String::from("[OK]").green());
                    },
                    Err(error) => {
                        let count = 1;
                        println!();
                        println!("{}", tr!("I found these errors").red().bold());
                        println!("{}", "--------------------".red());
                        println!();
                        println!(
                            "{}{} {}", 
                            count.to_string().blue(),
                            String::from('.').blue(),
                            error.message
                        );
                    }
                }
            },
            Err(errors) => {
                println!();
                println!("{}", tr!("I found these errors").red().bold());
                println!("{}", "--------------------".red());
                println!();
                let mut count = 1;
                for error in errors {
                    println!(
                        "{}{} {}", 
                        count.to_string().blue(), 
                        String::from('.').blue(), 
                        error.message
                    );
                    count += 1;
                }
            }
        }
    }
}

fn handle_field_response(
    property_data: &Result<String, PlanetError>, 
    errors: &Vec<PlanetError>, 
    property_id: &String,
    data: &BTreeMap<String, String>
) -> (BTreeMap<String, String>, Vec<PlanetError>) {
    let property_data = property_data.clone();
    let mut errors = errors.clone();
    let mut data = data.clone();
    let property_id = property_id.clone();
    if property_data.is_err() {
        let err = property_data.unwrap_err();
        errors.push(err);
    } else {
        let property_data = property_data.unwrap();
        data.insert(property_id, property_data);
    }
    return (data, errors)
}