pub mod text;
pub mod number;
pub mod formula;

use std::collections::HashMap;
use tr::tr;
use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::planet::{PlanetError};
use crate::storage::table::{DbData};

pub trait ValidateField {
    fn is_valid(&self, value: Option<&String>) -> Result<bool, PlanetError>;
}
pub trait ValidateManyField {
    fn is_valid(&self, value: Option<Vec<String>>) -> Result<bool, PlanetError>;
}
pub trait ValidateFormulaField {
    fn is_valid(&self, value: Option<&String>) -> Result<bool, PlanetError>;
}

pub trait ProcessField {
    fn process(
        &self,
        insert_data_map: HashMap<String, String>,
        db_data: DbData
    ) -> Result<DbData, PlanetError>;
}
pub trait ProcessManyField {
    fn process(
        &self,
        insert_data_collections_map: HashMap<String, Vec<String>>,
        db_data: DbData
    ) -> Result<DbData, PlanetError>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FieldType {
    SmallText(String),
    LongText(String),
    CheckBox(bool),
    NumberField(i32),
    SelectField(String),
}

pub fn error_validate_process(field_type: &str, field_name: &str) -> PlanetError {
    let error = PlanetError::new(
        500, 
        Some(tr!(
            "Could not validate \"{field_type}\" field {}{}{}", 
            String::from("\"").blue(), &field_name.blue(), String::from("\"").blue(),
            field_type=field_type
        )),
    );
    return error;
}
