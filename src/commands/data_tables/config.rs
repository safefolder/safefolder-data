use serde::{Serialize, Deserialize};
use validator::{Validate, ValidationError};

#[derive(Debug, PartialEq, Serialize, Deserialize, Validate)]
pub struct CreateTableConfig {
    #[validate(required)]
    pub Command: &'a str,
    #[validate(required)]
    pub Language: Language,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Validate)]
pub struct Language {
    #[validate(required, custom="validate_language_codes")]
    pub Codes: &'a Vec,
    #[validate(custom="validate_default_language")]
    pub Default: &'a str,
}


fn validate_default_language(language: &str) -> Result<(), ValidationError> {
    Ok(())
}

fn validate_language_codes(languages: &str) -> Result<(), ValidationError> {
    Ok(())
}