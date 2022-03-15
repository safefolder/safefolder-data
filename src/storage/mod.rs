extern crate xid;

pub mod folder;
pub mod constants;
pub mod columns;
pub mod space;

use std::collections::{BTreeMap,HashMap};

use validator::{ValidationErrors};
use crate::statements::folder::schema::*;
use crate::storage::folder::{DbData, TreeFolder};
use crate::planet::{PlanetError, Context, PlanetContext};
use crate::planet::constants::*;

pub trait ConfigStorageColumn {
    fn defaults(
        options: Option<Vec<String>>
    ) -> ColumnConfig;
    fn version() -> Option<String>;
    fn is_valid(&self) -> Result<(), ValidationErrors>;
    fn generate_id() -> Option<String> {
        return generate_id();
    }
    fn create_config(
        &self, 
        planet_context: &PlanetContext,
        context: &Context,
        properties_map: &HashMap<String, ColumnConfig>,
        db_folder: &TreeFolder,
        folder_name: &String
    ) -> Result<BTreeMap<String, String>, PlanetError>;
    fn get_column_config_map(
        planet_context: &PlanetContext,
        context: &Context,
        folder: &DbData
    ) -> Result<BTreeMap<String, ColumnConfig>, PlanetError>;
    fn map_collections_db(&self) -> Result<BTreeMap<String, Vec<BTreeMap<String, String>>>, PlanetError>;
    fn get_config(
        planet_context: &PlanetContext,
        context: &Context,
        db_data: &DbData
    ) -> Result<Vec<ColumnConfig>, PlanetError>;
    fn map_objects_db(&self) -> Result<BTreeMap<String, Vec<BTreeMap<String, String>>>, PlanetError>;
    fn get_column_id_map(properties: &Vec<ColumnConfig>) -> Result<BTreeMap<String, ColumnConfig>, PlanetError>;
    fn get_name_column(db_data: &DbData) -> Option<ColumnConfig>;
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
        LANGUAGE_SPANISH, 
        LANGUAGE_ENGLISH,
        LANGUAGE_FRENCH,
        LANGUAGE_GERMAN,
        LANGUAGE_ITALIAN,
        LANGUAGE_PORTUGUESE,
        LANGUAGE_NORWEGIAN,
        LANGUAGE_SWEDISH,
        LANGUAGE_DANISH,
    ];
    return languages
}