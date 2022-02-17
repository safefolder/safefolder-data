extern crate rust_stemmers;

use std::collections::{BTreeMap, HashMap};
use colored::Colorize;
use std::fs::File;
use mime_guess;
// use json;
use serde_yaml;

use crate::planet::{PlanetError};
use crate::commands::folder::config::ColumnConfig;
use crate::storage::constants::*;
use crate::storage::columns::*;

// #[derive(Debug, Clone)]
// pub struct FileVersionData {
//     pub id: String,
//     pub name: Option<String>,
//     pub file_type: String,
//     pub content_type: String,
//     pub size: Option<usize>,
//     pub width: Option<usize>,
//     pub height: Option<usize>,
//     pub properties: Option<BTreeMap<String, String>>,
// }

// I would need to adapt to achiever data structure. I can work with data_objects and store an object
// versions can be serialized in YAML format, and make it work with write and later read doing the serial work
// meta can be serialized json object, maybe yaml???

// file type is the human version of mime content types, making a map

// #[derive(Debug, Clone)]
// pub struct FileData {
//     pub id: String,
//     pub file_name: String,
//     pub versions: Option<Vec<FileVersionData>>,
//     pub title: Option<String>,
//     pub content_type: String,
//     pub file_type: String,
//     pub tags: Option<Vec<String>>,
//     pub description: Option<String>,
//     pub created_time: Option<String>,
//     pub last_modified_time: Option<String>,
//     pub size: Option<usize>,
//     pub width: Option<usize>,
//     pub height: Option<usize>,
//     pub meta: Option<json::JsonValue>,
//     pub properties: Option<BTreeMap<String, String>>,
// }

#[derive(Debug, Clone)]
pub struct FileColumn {
    pub config: ColumnConfig,
}
impl FileColumn {
    pub fn defaults(
        column_config: &ColumnConfig,
    ) -> Self {
        let column_config = column_config.clone();
        let field_obj = Self{
            config: column_config,
        };
        return field_obj
    }
    pub fn validate(
        &self, 
        paths: &Vec<String>
    ) -> Result<Vec<String>, PlanetError> {
        // Column Type Yaml???? It would store and serialize automatically, YAMLColumn, JSONColumn.
        // I return data_objects and data_collections
        // data_objects...
        // - {Column}: All data, basic properties, meta.
        // data_collections
        // - {Column}__versions: All versions for the case of images, with image manipulation, or versions of 
        //      docs modified. I place here the thumb version 300x??? landscape or ???x300 portrait for views.
        // - {Column}__tags : All tags coming from the file, with object and "value" key, being value the tag.
        // Files are stored by id into the "{space}/files" directory. originals and versions.
        //          {file_id}.achieverenc
        // Transaction support, removing the file from OS in case we can't write into database.
        // I need a way to roll back the column file write
        let paths = paths.clone();
        let config = self.config.clone();
        let content_types_wrap = config.content_types;
        let mut content_types: Vec<String> = Vec::new();
        if content_types_wrap.is_some() {
            content_types = content_types_wrap.unwrap();
        }
        for path in paths.clone() {
            let path_fields: Vec<&str> = path.split("/").collect();
            let file_name = path_fields.last().unwrap().clone();
            eprint!("FileColumn.validate :: file_name: {}", file_name);
            // 1. Check path exists, raise error if does not exist
            let file = File::open(path.clone());
            if file.is_err() {
                let error = file.unwrap_err();
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!(
                            "Could not open File at \"{}\". Error: {:?}.", &path, &error
                        )),
                    )
                );
            }
            let _file = file.unwrap();
            // 2. Validate file is allowed in config content types. Some crate??? Using file name????
            if content_types.len() > 0 {
                let mime_guess = mime_guess::from_path(file_name);
                let content_type = mime_guess.first();
                if content_type.is_some() {
                    let content_type = content_type.unwrap();
                    let content_type = content_type.to_string();
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
            }
            // 3. Proccess with Tika to get text and metadata
            // I return list of parsed data for all contents related to the file, like images in a word document
            // Title
            // File Name
            // Created Time
            // Last Modified Time
            // Path: Relative to achiever planet home, which differs in platforms.
            // File Type: Title for file type, like "Microsoft Word", etc...
            // Content Type: application/pdf, etc...
            // Creator
            // Tags : Into data_collections, {Column}__tags
            // Width
            // Height
            // Size
            // 4. Generate thumb image version in case image
            // 5. Populate data_objects and data_collections
        }
        return Ok(paths.clone())
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
