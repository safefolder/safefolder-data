use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct LanguageConfig<'a> {
    #[validate(required, custom="validate_language_codes")]
    pub codes: Option<Vec<&'a str>>,
    #[validate(custom="validate_default_language")]
    pub default: &'a str,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct FieldConfig<'a> {
    #[validate(length(min=20, max=20))]
    id: &'a str,
    #[validate(required)]
    name: Option<&'a str>,
    #[validate(required)]
    field_type: Option<&'a str>,
    number_decimals: u8,
    default: &'a str,
    #[validate(required)]
    version: Option<&'a str>,
    required: bool,
    #[validate(required)]
    api_version: Option<&'a str>,
    indexed: bool,
}

fn validate_default_language(language: &str) -> Result<(), ValidationError> {
    println!("language: {}", language);
    Ok(())
}

fn validate_language_codes(languages: &Vec<&str>) -> Result<(), ValidationError> {
    println!("languages: {:?}", languages);
    Ok(())
}
