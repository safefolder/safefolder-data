
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationErrors};

use crate::storage::config::{LanguageConfig, FieldConfig};
use crate::planet::validation::{CommandImportConfig, PlanetValidationError};
use crate::planet::PlanetContext;

use super::fetch_yaml_config;

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

    pub fn import(planet_context: &PlanetContext, yaml_path: &String) -> Result<CreateTableConfig, Vec<PlanetValidationError>> {
        // Fetch yaml config
        let yaml_str: String = fetch_yaml_config(&yaml_path);
        // Deseralize the config entity
        let response: Result<CreateTableConfig, serde_yaml::Error> = serde_yaml::from_str(&yaml_str);
        let import_config: CommandImportConfig = CommandImportConfig{
            command: &String::from(""),
            planet_context: &planet_context,
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