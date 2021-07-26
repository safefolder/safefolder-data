extern crate xid;

use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationErrors, ValidationError};

use crate::planet::validation::{CommandImportConfig, PlanetValidationError};
use crate::planet::PlanetContext;

use crate::storage::constants::{FIELD_VERSION, FIELD_API_VERSION};
use crate::storage::*;

use super::fetch_yaml_config;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DbTableConfig {
    pub language: Option<LanguageConfig>,
    pub fields: Option<Vec<FieldConfig>>,
}


#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct CreateTableConfig {
    #[validate(required)]
    pub command: Option<String>,
    #[validate]
    pub language: Option<LanguageConfig>,
    #[validate]
    pub fields: Option<Vec<FieldConfig>>,
}

impl CreateTableConfig {

    pub fn defaults() -> CreateTableConfig {
        let config: CreateTableConfig = CreateTableConfig{
            command: None,
            language: None,
            fields: None,
        };
        return config
    }

    pub fn import(&self, planet_context: &PlanetContext, yaml_path: &String) -> Result<CreateTableConfig, Vec<PlanetValidationError>> {
        let yaml_str: String = fetch_yaml_config(&yaml_path);
        // Deseralize the config entity
        let response: Result<CreateTableConfig, serde_yaml::Error> = serde_yaml::from_str(&yaml_str);
        let import_config: CommandImportConfig = CommandImportConfig{
            command: String::from(""),
            planet_context: planet_context,
        };
        match response {
            Ok(_) => {
                let config_model: CreateTableConfig = response.unwrap();
                let validate: Result<(), ValidationErrors> = config_model.validate();
                match validate {
                    Ok(_) => {
                        return Ok(config_model)
                    },
                    Err(errors) => {
                        let command = &config_model.command.unwrap();
                        let planet_errors: Vec<PlanetValidationError> = import_config.parse_validator(
                            command, errors);
                        return Err(planet_errors);
                    }
                }
            },
            Err(error) => {
                let mut planet_errors: Vec<PlanetValidationError> = Vec::new();
                planet_errors.push(import_config.parse_serde(&error));
                return Err(planet_errors);
            }
        }
    }

}

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct LanguageConfig {
    #[validate(required, custom="validate_language_codes")]
    pub codes: Option<Vec<String>>,
    #[validate(custom="validate_default_language")]
    pub default: String,
}

#[derive(Debug, Deserialize, Serialize, Validate, Clone)]
pub struct FieldConfig {
    #[validate(length(equal=20))]
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
        let object: FieldConfig = FieldConfig{
            id: None,
            name: None,
            field_type: None,
            default: Some(String::from("")),
            version: Some(String::from(FIELD_VERSION)),
            required: Some(false),
            api_version: Some(String::from(FIELD_API_VERSION)),
            indexed: Some(true),
        };
        return object;
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
