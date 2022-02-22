extern crate rust_stemmers;
extern crate dirs;

use std::collections::{BTreeMap, HashMap};
use colored::Colorize;
use std::fs::File;
use mime_guess;
use json;
use reqwest::blocking::Client;
use serde_yaml;

use crate::planet::{PlanetError};
use crate::commands::folder::config::ColumnConfig;
use crate::storage::constants::*;
use crate::storage::columns::*;
use crate::storage::folder::{DbFile, RoutingData, TreeFolderItem};
use crate::storage::generate_id;
use crate::storage::space::SpaceDatabase;

#[derive(Debug, Clone)]
pub struct FileColumn {
    pub config: ColumnConfig,
    pub db_folder_item: Option<TreeFolderItem>,
    pub space_database: Option<SpaceDatabase>,
}
impl FileColumn {
    pub fn defaults(
        column_config: &ColumnConfig,
        db_folder_item: Option<TreeFolderItem>,
        space_database: Option<SpaceDatabase>
    ) -> Self {
        let column_config = column_config.clone();
        let field_obj = Self{
            config: column_config,
            db_folder_item: db_folder_item,
            space_database: space_database
        };
        return field_obj
    }
    pub fn validate(
        &self, 
        paths: &Vec<String>,
        data_objects: &BTreeMap<String, BTreeMap<String, String>>,
        data_collections: &BTreeMap<String, Vec<BTreeMap<String, String>>>,
        routing: Option<RoutingData>,
        home_dir: &String,
    ) -> Result<(
            Vec<String>,
            Vec<String>,
            BTreeMap<String, BTreeMap<String, String>>, 
            BTreeMap<String, Vec<BTreeMap<String, String>>>
        ), PlanetError> {
        let mut db_folder_item = self.db_folder_item.clone().unwrap();
        let mut data_collections = data_collections.clone();
        let mut data_objects = data_objects.clone();
        let paths = paths.clone();
        let config = self.config.clone();
        let column_id = &config.id.unwrap();
        let content_types_wrap = config.content_types;
        let mut content_types: Vec<String> = Vec::new();
        if content_types_wrap.is_some() {
            content_types = content_types_wrap.unwrap();
        }
        let mut document_texts: Vec<String> = Vec::new();
        let mut file_ids: Vec<String> = Vec::new();
        for path in paths.clone() {
            let path_fields: Vec<&str> = path.split("/").collect();
            let file_name = path_fields.last().unwrap().clone();
            // 1. Check path exists, raise error if does not exist
            let file = File::open(path.clone());
            if file.is_err() {
                let error = file.unwrap_err();
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!(
                            "Could not open File at \"{}\". Error: \"{}\".", &path, &error
                        )),
                    )
                );
            }
            let file = file.unwrap();
            // 2. Validate file is allowed in config content types. Some crate??? Using file name????
            let mime_guess = mime_guess::from_path(file_name);
            let content_type_wrap = mime_guess.first();
            let mut content_type: String = String::from("");
            if content_type_wrap.is_some() {
                let content_type_ = content_type_wrap.unwrap();
                content_type = content_type_.to_string();
            }
            if content_types.len() > 0 {
                let check = content_types.contains(&content_type);
                if !check {
                    return Err(
                        PlanetError::new(
                            500, 
                            Some(tr!(
                                "File with path \"{}\" has content type not supported: \"{}\"", 
                                &path, &content_type
                            )),
                        )
                    );
                }
            }
            // TODO: generate file type from custom map
            let file_type: String = String::from("");
            let metadata = &file.metadata().unwrap();
            let file_size = metadata.len();
            if metadata.is_dir() {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!(
                            "Path \"\" is location of a directory instead of a file.", &path
                        )),
                    )
                );
            }
            let client = Client::new();
            let url = format!("http://{host}:{port}/rmeta/text", host=TIKA_HOST, port=TIKA_PORT);
            let res = client.put(&url)
            .body(file)
            .send();
            let mut file_id: Option<String> = None;
            if res.is_ok() {
                let response = res.unwrap();
                let response = response.text().unwrap();
                let json_document = json::parse(&response);
                let mut my_map: BTreeMap<String, String> = BTreeMap::new();
                if json_document.is_ok() {
                    let json_document = json_document.unwrap();
                    let main_document = &json_document[0];
                    let title = &main_document["dc:title"];
                    let created_time = &main_document["dcterms:created"];
                    let last_modified_time = &main_document["dcterms:modified"];
                    let content_type = &main_document["Content-Type"];
                    let creator = &main_document["dc:creator"];
                    let subject = &main_document["dc:subject"];
                    let description = &main_document["dc:description"];
                    let category = &main_document["cp:category"];
                    let image_width = &main_document["Image Width"];
                    let image_height = &main_document["Image Height"];
                    let text = &main_document["X-TIKA:content"];
                    let id = generate_id();
                    // file type
                    if id.is_some() {
                        let id = id.unwrap();
                        file_id = Some(id.clone());
                        my_map.insert(
                            ID.to_string(), 
                            id
                        );    
                    }
                    if !title.is_null() {
                        my_map.insert(
                            FILE_PROP_TITLE.to_string(), 
                            title.to_string()
                        );
                    }
                    my_map.insert(
                        FILE_PROP_FILE_NAME.to_string(), 
                        file_name.to_string()
                    );
                    my_map.insert(
                        FILE_PROP_SIZE.to_string(), 
                        file_size.to_string()
                    );
                    if !created_time.is_null() {
                        my_map.insert(
                            FILE_PROP_CREATED_TIME.to_string(), 
                            created_time.to_string()
                        );
                    }
                    if !last_modified_time.is_null() {
                        my_map.insert(
                            FILE_PROP_LAST_MODIFIED_TIME.to_string(), 
                            last_modified_time.to_string()
                        );
                    }
                    if !content_type.is_null() {
                        my_map.insert(
                            FILE_PROP_CONTENT_TYPE.to_string(), 
                            content_type.to_string()
                        );
                    }
                    if !creator.is_null() {
                        my_map.insert(
                            FILE_PROP_CREATOR.to_string(), 
                            creator.to_string()
                        );
                    }
                    if !category.is_null() {
                        my_map.insert(
                            FILE_PROP_CATEGORY.to_string(), 
                            category.to_string()
                        );
                    }
                    if !subject.is_null() {
                        if subject.is_string() {
                            my_map.insert(
                                FILE_PROP_SUBJECT.to_string(), 
                                subject.to_string()
                            );
                        } else {
                            my_map.insert(
                                FILE_PROP_SUBJECT.to_string(), 
                                subject[0].to_string()
                            );
                            let keywords = subject[1].to_string();
                            let check = keywords.contains(",");
                            if check {
                                let keyword_list: Vec<&str> = keywords.split(",").collect();
                                let mut list: Vec<BTreeMap<String, String>> = Vec::new();
                                for keyword in keyword_list {
                                    let keyword = keyword.trim();
                                    let mut my_map: BTreeMap<String, String> = BTreeMap::new();
                                    my_map.insert(VALUE.to_string(), keyword.to_string());
                                    list.push(my_map);
                                }
                                let key = format!("{}__tags", column_id);
                                data_collections.insert(key, list);
                            } else {
                                let keyword = keywords;
                                let mut list: Vec<BTreeMap<String, String>> = Vec::new();
                                let mut my_map: BTreeMap<String, String> = BTreeMap::new();
                                my_map.insert(VALUE.to_string(), keyword.to_string());
                                list.push(my_map);
                                let key = format!("{}__tags", column_id);
                                data_collections.insert(key, list);
                            }
                            my_map.insert(
                                FILE_PROP_SUBJECT.to_string(), 
                                subject[0].to_string()
                            );
                        }
                    }
                    if !image_width.is_null() {
                        let width_str = image_width.to_string();
                        let fields: Vec<&str> = width_str.split(" pixels").collect();
                        my_map.insert(
                            FILE_PROP_IMAGE_WIDTH.to_string(), 
                            fields[0].to_string()
                        );
                    }
                    if !image_height.is_null() {
                        let height_str = image_height.to_string();
                        let fields: Vec<&str> = height_str.split(" pixels").collect();
                        my_map.insert(
                            FILE_PROP_IMAGE_HEIGHT.to_string(), 
                            fields[0].to_string()
                        );
                    }
                    if !description.is_null() {
                        my_map.insert(
                            FILE_PROP_DESCRIPTION.to_string(), 
                            description.to_string()
                        );
                    }
                    if !text.is_null() {
                        let mut text = text.to_string();
                        text = text.replace("\n", "");
                        text = text.replace("\t", "");
                        text = text.replace("\r", "");
                        document_texts.push(text);
                    }
                    // let text = &main_document["X-TIKA:content"];
                    // main_document.remove("X-TIKA:content");
                    my_map.insert(
                        FILE_PROP_METADATA.to_string(), 
                        response
                    );
                    data_objects.insert(column_id.clone(), my_map);
                } else {
                    return Err(
                        PlanetError::new(
                            500, 
                            Some(tr!(
                                "Error processing metadata response."
                            )),
                        )
                    );
                }
            } else {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!(
                            "Error processing file metadata."
                        )),
                    )
                );
            }
            // Write file into database or home OS dir
            if file_id.is_some() {
                let file_id = file_id.unwrap();
                let mut file = File::open(path.clone()).unwrap();
                let size = file.metadata().unwrap().len();
                let db_file = DbFile::defaults(
                    &file_id, 
                    &file_name.to_string(), 
                    &mut file,
                    &content_type,
                    &file_type,
                    routing.clone(),
                    home_dir
                );
                if db_file.is_err() {
                    return Err(
                        PlanetError::new(
                            500, 
                            Some(tr!(
                                "Error writing into file database."
                            )),
                        )
                    );                    
                }
                let mut db_file = db_file.unwrap();
                if size < MAX_FILE_DB {
                    let file_id = db_folder_item.write_file(&db_file);
                    if file_id.is_err() {
                        return Err(
                            PlanetError::new(
                                500, 
                                Some(tr!(
                                    "Error writing into file database."
                                )),
                            )
                        );
                    }
                    let file_id = file_id.unwrap();
                    file_ids.push(file_id.clone());
                } else {
                    // File into OS home directory for space
                    let file_id = db_file.write_file(&mut file, &db_folder_item);
                    if file_id.is_ok() {
                        let file_id = file_id.unwrap();
                        file_ids.push(file_id);
                    }
                }
            } else {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!(
                            "Error generating file id."
                        )),
                    )
                );
            }
        }
        return Ok(
            (
                file_ids.clone(),
                document_texts.clone(),
                data_objects.clone(),
                data_collections.clone()
            )
        )
    }
}
impl StorageColumnBasic for FileColumn {
    fn create_config(
        &mut self, 
        column_config_map: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let mut column_config_map = column_config_map.clone();
        let config = self.config.clone();
        let many = config.many;
        if many.is_some() {
            let many = many.unwrap();
            column_config_map.insert(String::from(MANY), many.to_string());
        }
        let content_types = config.content_types;
        if content_types.is_some() {
            let content_types = content_types.unwrap();
            let content_types_serialized = serde_yaml::to_string(&content_types).unwrap();
            column_config_map.insert(String::from(CONTENT_TYPES), content_types_serialized);
        }
        return Ok(column_config_map)
    }
    fn get_config(
        &mut self, 
        column_config_map: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError> {
        let mut config = self.config.clone();
        let column_config_map = column_config_map.clone();
        let many = column_config_map.get(MANY);
        let content_types = column_config_map.get(CONTENT_TYPES);
        if many.is_some() {
            let many = many.unwrap();
            let many = many.clone() == String::from(TRUE);
            config.many = Some(many);
        }
        if content_types.is_some() {
            let content_types = content_types.unwrap();
            let content_types: Vec<String> = serde_yaml::from_str(content_types).unwrap();
            config.content_types = Some(content_types);
        }
        return Ok(config)
    }
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String {
        let column_config = self.config.clone();
        let column_name = column_config.name.unwrap();
        let mut yaml_string = yaml_string.clone();
        let field = &column_name.truecolor(
            YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
        );
        let value = format!("{}", 
            value.truecolor(YAML_COLOR_ORANGE[0], YAML_COLOR_ORANGE[1], YAML_COLOR_ORANGE[2]), 
        );
        yaml_string.push_str(format!("  {field}: {value}\n", field=field, value=value).as_str());
        return yaml_string;
    }
}

pub fn get_file_types() -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    map.insert(String::from("application/vnd.hzn-3d-crossword"), String::from("3D Crossword Plugin"));
    // TODO: continue with all mime types
    // https://www.freeformatter.com/mime-types-list.html#mime-types-list
    return map
}

pub fn get_file_type(content_type: &str) -> Result<String, PlanetError> {
    let map = get_file_types();
    let result = map.get(content_type);
    if result.is_none() {
        return Err(
            PlanetError::new(
                500, 
                Some(tr!(
                    "Content type \"{}\" not supported.", content_type
                )),
            )
        );
    }
    let file_type = result.unwrap();
    return Ok(file_type.clone())
}
