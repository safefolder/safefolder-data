extern crate xid;

pub mod fields;
pub mod table;
pub mod constants;

use validator::{ValidationErrors};
use crate::commands::table::config::FieldConfig;

pub trait StorageField {
    fn defaults() -> FieldConfig;
    fn version() -> Option<String>;
    fn api_version() -> Option<String>;
    fn is_valid(&self) -> Result<(), ValidationErrors>;
    fn generate_id() -> Option<String> {
        return generate_id();
    }
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