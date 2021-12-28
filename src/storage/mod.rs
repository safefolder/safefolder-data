extern crate xid;

// pub mod fields_old;
pub mod folder;
pub mod constants;
pub mod properties;

use std::collections::BTreeMap;

use validator::{ValidationErrors};
use crate::commands::folder::config::PropertyConfig;
use crate::storage::folder::{DbData, DbFolder};
use crate::planet::PlanetError;

pub trait ConfigStorageProperty {
    fn defaults(
        options: Option<Vec<String>>
    ) -> PropertyConfig;
    fn version() -> Option<String>;
    fn api_version() -> Option<String>;
    fn is_valid(&self) -> Result<(), ValidationErrors>;
    fn generate_id() -> Option<String> {
        return generate_id();
    }
    fn map_object_db(
        &self, 
        property_type_map: &BTreeMap<String, String>,
        property_name_map: &BTreeMap<String, String>,
        db_folder: &DbFolder,
        folder_name: &String
    ) -> Result<BTreeMap<String, String>, PlanetError>;
    fn get_property_config_map(folder: &DbData) -> Result<BTreeMap<String, PropertyConfig>, PlanetError>;
    fn map_collections_db(&self) -> Result<BTreeMap<String, Vec<BTreeMap<String, String>>>, PlanetError>;
    fn parse_from_db(db_data: &DbData) -> Result<Vec<PropertyConfig>, PlanetError>;
    fn map_objects_db(&self) -> Result<BTreeMap<String, Vec<BTreeMap<String, String>>>, PlanetError>;
    fn get_property_id_map(properties: &Vec<PropertyConfig>) -> Result<BTreeMap<String, PropertyConfig>, PlanetError>;
    fn get_name_property(db_data: &DbData) -> Option<PropertyConfig>;
}

pub fn generate_id() -> Option<String> {
    let field_id = xid::new();
    if Some(&field_id).is_some() {
        return Some(field_id.to_string())
    } else {
        return None
    }
}

pub fn get_db_languages() -> Vec<&'static str> {
    let languages = vec![
        "spanish", 
        "english",
        "french",
        "german",
        "italian",
        "portuguese",
        "norweian",
        "swedish",
        "danish",
    ];
    return languages
}