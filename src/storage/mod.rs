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

// pub fn generate_id_bytes() -> [u8; 12] {
//     let field_id = xid::new();
//     let field_id_bytes = field_id.as_bytes();
//     // let field_id_bytes = field_id_bytes.to_vec();
//     return *field_id_bytes
// }

// pub fn parse_id_string(id: &String) -> Result<[u8; 12], PlanetError> {
//     let id = hex::decode(id);
//     match id {
//         Ok(_) => {
//             let id_vector = id.unwrap();
//             let mut id_final: ArrayVec<u8, 12> = ArrayVec::<u8, 12>::new();
//             for chunk in id_vector {
//                 id_final.push(chunk);
//             }
//             let mine = id_final[..];
//             Ok(id_final[..])
//         },
//         Err(_) => {
//             Err(PlanetError::new(500, Some(tr!("Could not generate hex form of identifier"))))
//         }
//     }
// }

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