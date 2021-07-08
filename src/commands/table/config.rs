
use serde::{Deserialize, Serialize};
use validator::{Validate};

use crate::storage::config::{LanguageConfig, FieldConfig};

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateTableConfig<'a> {
    #[validate(required)]
    command: Option<&'a str>,
    #[validate(required)]
    pub language: Option<LanguageConfig<'a>>,
    #[validate(required)]
    pub fields: Option<Vec<FieldConfig<'a>>>,
}
