extern crate tr;
extern crate colored;

use std::collections::{BTreeMap, HashMap};
use std::time::Instant;

use tr::tr;
use colored::*;
use regex::Regex;

use crate::commands::folder::config::{CreateFolderConfig, PropertyConfig};
use crate::commands::folder::{Command};
use crate::commands::{CommandRunner};
use crate::storage::{ConfigStorageProperty};
use crate::storage::folder::{DbFolder, FolderSchema, DbData, RoutingData};
use crate::planet::{
    PlanetContext, 
    PlanetError,
    Context, 
    validation::PlanetValidationError,
};
use crate::storage::constants::*;
use crate::planet::constants::*;

pub struct CreateFolder<'gb> {
    pub planet_context: &'gb PlanetContext<'gb>,
    pub context: &'gb Context<'gb>,
    pub config: CreateFolderConfig,
}

impl<'gb> Command<DbData> for CreateFolder<'gb> {

    fn run(&self) -> Result<DbData, PlanetError> {
        let t_1 = Instant::now();
        let home_dir = self.planet_context.home_path.unwrap_or_default();
        let account_id = self.context.account_id.unwrap_or_default();
        let space_id = self.context.space_id.unwrap_or_default();
        let site_id = self.context.site_id.unwrap_or_default();
        let result: Result<DbFolder, PlanetError> = DbFolder::defaults(
            Some(home_dir),
            Some(account_id),
            Some(space_id),
            Some(site_id),
        );
        match result {
            Ok(_) => {
                let command = self.config.command.clone().unwrap_or_default();
                let expr = Regex::new(r#"(CREATE FOLDER) "(?P<folder_name>[a-zA-Z0-9_ ]+)"#).unwrap();
                let table_name_match = expr.captures(&command).unwrap();
                let folder_name = &table_name_match["folder_name"].to_string();
                let config = self.config.clone();

                // routing parameters
                let account_id = Some(self.context.account_id.unwrap_or_default().to_string());
                let site_id = self.context.site_id;
                let space_id = self.context.space_id;
                let box_id = self.context.box_id;

                // db folder options with language data
                let db_folder: DbFolder = result.unwrap();
                let mut data: BTreeMap<String, String> = BTreeMap::new();
                let language = config.language.unwrap();
                let language_codes_list = language.codes.unwrap();
                let language_codes_str = language_codes_list.join(",");
                let language_default = language.default;
                data.insert(String::from(LANGUAGE_CODES), language_codes_str);
                data.insert(String::from(LANGUAGE_DEFAULT), language_default);

                // config data
                let mut data_objects: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
                let mut data_collections: BTreeMap<String, Vec<BTreeMap<String, String>>> = BTreeMap::new();
                let mut properties = config.properties.unwrap().clone();
                let mut field_ids: Vec<BTreeMap<String, String>> = Vec::new();

                // name field
                let name_field_config = config.name.unwrap();
                properties.insert(0, name_field_config);
                let mut field_name_map: BTreeMap<String, String> = BTreeMap::new();
                // populate field_type_map and field_name_map
                let mut field_type_map: BTreeMap<String, String> = BTreeMap::new();
                let mut properties_map: HashMap<String, PropertyConfig> = HashMap::new();
                for field in properties.iter() {
                    let field_attrs = field.clone();
                    let field_name = field.name.clone().unwrap();
                    let property_type = field.property_type.clone();
                    let mut field_id_map: BTreeMap<String, String> = BTreeMap::new();
                    let field_id = field_attrs.id.unwrap_or_default();
                    field_id_map.insert(String::from(ID), field_id.clone());
                    properties_map.insert(field_name.clone(), field.clone());
                    let _ = &field_ids.push(field_id_map);
                    if property_type.is_some() {
                        let property_type = property_type.unwrap();
                        field_type_map.insert(field_name.clone(), property_type);
                    }
                    let field_name_str = field_name.as_str();
                    if field_name_map.get(field_name_str).is_some() == false {
                        // id => name
                        field_name_map.insert(field_name.clone(), field_id.clone());
                    } else {
                        return Err(
                            PlanetError::new(
                                500, 
                                Some(tr!("There is already a field with name \"{}\"", &field_name)),
                            )
                        );
                    }
                }
                for field in properties.iter() {
                    // field simple attributes
                    let field_attrs = field.clone();
                    let field_name = field_attrs.name.unwrap_or_default().clone();
                    let map = &field.map_object_db(
                        self.planet_context,
                        self.context,
                        &properties_map,
                        &db_folder,
                        folder_name,
                    )?;
                    data_objects.insert(String::from(field_name.clone()), map.clone());
                    // field complex attributes like select_data
                    let map_list = &field.map_collections_db()?;
                    let map_list = map_list.clone();
                    // data_collections = map_list.clone();
                    data_collections.extend(map_list);
                }
                // eprintln!("CreateFolder.run :: data_objects_new: {:#?}", &data_objects_new);
                data_collections.insert(String::from(PROPERTY_IDS), field_ids);
                // routing
                let routing_wrap = RoutingData::defaults(
                    account_id, 
                    site_id, 
                    space_id,
                    box_id,
                    None,
                );
                let mut data_wrap: Option<BTreeMap<String, String>> = None;
                let mut data_collections_wrap: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>> = None;
                let mut data_objects_wrap: Option<BTreeMap<String, BTreeMap<String, String>>> = None;
                if data.len() > 0 {
                    data_wrap = Some(data);
                }
                if data_collections.len() > 0 {
                    data_collections_wrap = Some(data_collections);
                }
                if data_objects.len() > 0 {
                    data_objects_wrap = Some(data_objects);
                }
                let db_data: DbData = DbData::defaults(
                    &folder_name, 
                    data_wrap,
                    data_collections_wrap,
                    data_objects_wrap,
                    None,
                    routing_wrap,
                    None,
                )?;
                // Onl output TEMP the choices data to include in insert
                let mut mine = db_data.clone().data_collections.unwrap();
                mine.remove("field_ids");
                eprintln!("CreateFolder.run :: db_data: {:#?}", mine);
                eprintln!("CreateFolder.run :: db_data all: {:#?}", db_data.clone());

                let response: DbData = db_folder.create(&db_data)?;
                let response_src = response.clone();
                // response.id
                let folder_name = &response.name.unwrap_or_default();
                let folder_id = &response.id.unwrap();

                //
                // Related folders, I need to update their config, like Links
                //
                let properties = response.data_objects.unwrap();
                let mut linked_folder_ids: Vec<String> = Vec::new();
                let mut map_property_names: BTreeMap<String, String> = BTreeMap::new();
                let mut map_property_ids: BTreeMap<String, String> = BTreeMap::new();
                for (_, v) in properties {
                    let property_type = v.get(PROPERTY_TYPE);
                    let property_name = v.get(NAME);
                    let property_id = v.get(ID);
                    if property_type.is_some() {
                        let property_type = property_type.unwrap();
                        let property_name = property_name.unwrap();
                        let property_id = property_id.unwrap();
                        if property_type == PROPERTY_TYPE_LINK {
                            let linked_folder_id = v.get(LINKED_FOLDER_ID);
                            if linked_folder_id.is_some() {
                                let linked_folder_id = linked_folder_id.unwrap();
                                let has_id = linked_folder_ids.contains(linked_folder_id);
                                if !has_id {
                                    linked_folder_ids.push(linked_folder_id.clone());
                                    map_property_names.insert(
                                        linked_folder_id.clone(), property_name.clone()
                                    );
                                    map_property_ids.insert(
                                        linked_folder_id.clone(), property_id.clone()
                                    );
                                }
                            }
                        }
                    }
                }
                // Get each folder from db_folder instance and update with link to this created table
                let local_property_map = db_data.data_objects.unwrap();
                for link_folder_id in linked_folder_ids {
                    let linked_folder = db_folder.get(&link_folder_id);
                    if linked_folder.is_ok() {
                        let mut linked_folder = linked_folder.unwrap();
                        let mut map = linked_folder.data_objects.unwrap();
                        let property_name = map_property_names.get(&link_folder_id).unwrap();
                        let mut remote_property_map: BTreeMap<String, String> = local_property_map.get(
                            property_name
                        ).unwrap().clone();
                        // Update property map with properties for local field, Link with link_folder_id being 
                        // this local property
                        remote_property_map.insert(String::from(LINKED_FOLDER_ID), folder_id.clone());
                        remote_property_map.insert(String::from(NAME), folder_name.clone());
                        remote_property_map.insert(String::from(MANY), String::from(TRUE));
                        map.insert(folder_name.clone(), remote_property_map);
                        linked_folder.data_objects = Some(map);
                        let mut property_ids_map = linked_folder.data_collections.unwrap();
                        let mut property_ids = property_ids_map.get(
                            PROPERTY_IDS
                        ).unwrap().clone();
                        let property_id = map_property_ids.get(&link_folder_id.clone()).unwrap();
                        let mut element: BTreeMap<String, String> = BTreeMap::new();
                        element.insert(String::from(ID),property_id.clone());
                        property_ids.push(element);
                        property_ids_map.insert(String::from(PROPERTY_IDS), property_ids);
                        linked_folder.data_collections = Some(property_ids_map);
                        eprintln!("CreateFolder.run :: linked_folder: {:#?}", &linked_folder);
                        db_folder.update(&linked_folder)?;
                    }
                }

                println!();
                let quote_color = format!("{}", String::from("\""));
                println!("Created folder {} :: {} => {}",
                    format!("{}{}{}", &quote_color.blue(), &folder_name.blue(), &quote_color.blue()),
                    &folder_id.magenta(),
                    format!("{}{}{}", &quote_color.green(), &folder_name.green(), &quote_color.green()),
                );
                eprintln!("CreateFolder.run :: time: {} µs", &t_1.elapsed().as_micros());

                let _mine = db_folder.get_by_name(folder_name);

                Ok(response_src)
                },
            Err(error) => {
                Err(error)
            }
        }
    }

    fn runner(runner: &CommandRunner, path_yaml: &String) -> () {
        let config_ = CreateFolderConfig::defaults(None);
        let config: Result<CreateFolderConfig, Vec<PlanetValidationError>> = config_.import(
            runner.planet_context,
            &path_yaml
        );
        match config {
            Ok(_) => {
                let create_folder: CreateFolder = CreateFolder{
                    planet_context: runner.planet_context,
                    context: runner.context,
                    config: config.unwrap(),
                };
                let result = create_folder.run();
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
