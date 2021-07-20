
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationErrors};

use crate::storage::config::{LanguageConfig, FieldConfig};
use crate::commands::ConfigSettings;
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
    settings: Option<ConfigSettings>,
}

impl CreateTableConfig {

    pub fn defaults(yaml_path: String, planet_context: PlanetContext) -> CreateTableConfig {
        let config: CreateTableConfig = CreateTableConfig{
            command: None,
            language: None,
            fields: None,
            settings: Some(ConfigSettings::defaults(yaml_path, planet_context)),
        };
        return config
    }

    pub fn import(&self) -> Result<CreateTableConfig, Vec<PlanetValidationError>> {
        // Fetch yaml config
        let settings: &ConfigSettings = &self.settings.clone().unwrap();
        let yaml_str: String = fetch_yaml_config(&settings.yaml_path.clone().unwrap());
        // Deseralize the config entity
        let response: Result<CreateTableConfig, serde_yaml::Error> = serde_yaml::from_str(&yaml_str);
        let import_config: CommandImportConfig = CommandImportConfig{
            command: &String::from(""),
            planet_context: &settings.planet_context.clone().unwrap(),
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