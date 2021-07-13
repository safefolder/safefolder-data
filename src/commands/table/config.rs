
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationErrors};

use crate::storage::config::{LanguageConfig, FieldConfig};
use crate::planet::validation::{CommandImportConfig,PlanetValidationError};

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

    pub fn is_valid(&self) -> Result<(), ValidationErrors> {
        // This is deprecated, since we do validation logic into the import operation
        match self.validate() {
            Ok(_) => return Ok(()),
            Err(errors) => {
                return Err(errors);
            },
          };
    }

    pub fn import(yaml_path: &String) -> Result<CreateTableConfig, Vec<PlanetValidationError>> {
        // Fetch yaml config
        let yaml_str: String = fetch_yaml_config(&yaml_path);
        // Deseralize the config entity
        let response: Result<CreateTableConfig, serde_yaml::Error> = serde_yaml::from_str(&yaml_str);
        // I execute the import logic once I have the types
        let config: CommandImportConfig = CommandImportConfig{
            command: String::from("CREATE TABLE"),
        };
        return config.import(response);
    }

}