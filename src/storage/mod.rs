extern crate xid;

// pub mod fields_old;
pub mod table;
pub mod constants;
pub mod fields;

use std::collections::BTreeMap;

use validator::{ValidationErrors};
use crate::commands::table::config::FieldConfig;
use crate::storage::table::{DbData, DbTable};
use crate::planet::PlanetError;

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
    fn map_object_db(
        &self, 
        field_type_map: &BTreeMap<String, String>,
        field_name_map: &BTreeMap<String, String>,
        db_table: &DbTable,
        table_name: &String
    ) -> Result<BTreeMap<String, String>, PlanetError>;
    fn get_field_config_map(table: &DbData) -> Result<BTreeMap<String, FieldConfig>, PlanetError>;
    fn map_collections_db(&self) -> Result<BTreeMap<String, Vec<BTreeMap<String, String>>>, PlanetError>;
    fn parse_from_db(db_data: &DbData) -> Result<Vec<FieldConfig>, PlanetError>;
    fn map_objects_db(&self) -> Result<BTreeMap<String, Vec<BTreeMap<String, String>>>, PlanetError>;
    fn get_field_id_map(fields: &Vec<FieldConfig>) -> Result<BTreeMap<String, FieldConfig>, PlanetError>;
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