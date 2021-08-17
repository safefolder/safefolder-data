extern crate xid;

pub mod fields;
pub mod table;
pub mod constants;

use std::collections::HashMap;

use validator::{ValidationErrors};
use crate::commands::table::config::FieldConfig;
use crate::storage::table::DbData;

pub trait ConfigStorageField {
    fn defaults(
        options: Option<Vec<String>>
    ) -> FieldConfig;
    fn version() -> Option<String>;
    fn api_version() -> Option<String>;
    fn is_valid(&self) -> Result<(), ValidationErrors>;
    fn generate_id() -> Option<String> {
        return generate_id();
    }
    fn map_object_db(&self) -> HashMap<String, String>;
    fn map_collections_db(&self) -> HashMap<String, Vec<HashMap<String, String>>>;
    fn parse_from_db(db_data: &DbData) -> Vec<FieldConfig>;
    fn map_objects_db(&self) -> HashMap<String, Vec<HashMap<String, String>>>;
    fn get_field_id_map(fields: &Vec<FieldConfig>) -> HashMap<String, FieldConfig>;
    fn get_name_field(db_data: &DbData) -> Option<FieldConfig>;
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