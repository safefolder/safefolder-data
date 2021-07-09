
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationErrors};

use crate::storage::config::{LanguageConfig, FieldConfig};

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
        match self.validate() {
            Ok(_) => return Ok(()),
            Err(errors) => {
                return Err(errors);
            },
          };
    }
}