extern crate xid;

use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError, ValidationErrors};

use crate::storage::constants::{FIELD_VERSION, FIELD_API_VERSION};
use crate::storage::*;

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct LanguageConfig {
    #[validate(required, custom="validate_language_codes")]
    pub codes: Option<Vec<String>>,
    #[validate(custom="validate_default_language")]
    pub default: String,
}

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct FieldConfig {
    #[validate(length(equal=20, code="length_equal"))]
    #[serde(default="generate_id")]
    pub id: Option<String>,
    #[validate(required)]
    pub name: Option<String>,
    #[validate(required)]
    pub field_type: Option<String>,
    pub default: Option<String>,
    #[validate(required)]
    #[serde(default="FieldConfig::version")]
    pub version: Option<String>,
    pub required: Option<bool>,
    #[validate(required)]
    #[serde(default="FieldConfig::api_version")]
    pub api_version: Option<String>,
    pub indexed: Option<bool>,
}

impl StorageField for FieldConfig {
    fn defaults() -> FieldConfig {
        return FieldConfig {
            id: None,
            name: None,
            field_type: None,
            default: Some(String::from("")),
            version: Some(String::from(FIELD_VERSION)),
            required: Some(false),
            api_version: Some(String::from(FIELD_API_VERSION)),
            indexed: Some(true),
        };
    }
    fn version() -> Option<String> {
        return Some(String::from(FIELD_VERSION));
    }
    fn api_version() -> Option<String> {
        return Some(String::from(FIELD_API_VERSION));
    }
    /// Checks that FieldConfig passes validations
    fn is_valid(&self) -> Result<(), ValidationErrors> {
        match self.validate() {
            Ok(_) => return Ok(()),
            Err(errors) => {
                return Err(errors);
            },
          };
    }
}

fn validate_default_language(language: &String) -> Result<(), ValidationError> {
    let language = &**language;
    let db_languages = get_db_languages();
    if db_languages.contains(&language) {
        return Ok(())
    } else {
        return Err(ValidationError::new("Invalid Default Language"));
    }
    
}

fn validate_language_codes(languages: &Vec<String>) -> Result<(), ValidationError> {
    let db_languages = get_db_languages();
    let mut check: bool = true;
    for language in languages.into_iter() {
        let language = &**language;
        if !db_languages.contains(&language) {
            check = false;
        }
    }
    if check {
        return Ok(())
    } else {
        return Err(ValidationError::new("Invalid Language"));
    }

}

// fn validate_str(_: &str) -> Result<(), ValidationError> {
//     if std::any::type_name::<&str>() == "&str" {
//         return Ok(())
//     } else {
//         return Err(ValidationError::new("Invalid String"));
//     }
// }

// fn validate_string(_: String) -> Result<(), ValidationError> {
//     if std::any::type_name::<&str>() == "String" {
//         return Ok(())
//     } else {
//         return Err(ValidationError::new("Invalid String"));
//     }
// }
