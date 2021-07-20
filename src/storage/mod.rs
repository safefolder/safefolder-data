extern crate xid;

pub mod fields;
pub mod table;
pub mod config;
pub mod constants;

use validator::{ValidationErrors};

use config::FieldConfig;

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
    let field_id = xid::new().to_string();
    if Some(&field_id).is_some() {
        return Some(field_id)
    } else {
        return None
    }
}

pub fn generate_id_bytes() -> [u8; 12] {
    let field_id = xid::new();
    let field_id_bytes = field_id.as_bytes();
    // let field_id_bytes = field_id_bytes.to_vec();
    return *field_id_bytes
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