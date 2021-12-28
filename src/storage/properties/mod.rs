pub mod text;
pub mod number;
pub mod formula;
pub mod date;

use std::collections::BTreeMap;
use tr::tr;
use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::planet::{PlanetError};
use crate::storage::folder::{DbData, DbFolder};
use crate::commands::folder::config::*;

/*
These are the core fields implemented so we can tackle the security and permissions system

Table fields
============

* SmallTextField                [done] - text
* LongTextField                 [done] - text : This is the text field, needs to be updated based on full text search.
* CheckBoxField                 [done] - number
* SelectField                   [done] - text
* DateField                     [done] - date
* Duration                      [done] - date
* NumberField                   [done] - number
* AuditTimeField                [done] - date
* AuditByField                  [done] - text
* CurrencyField                 [done] - number
* PercentField                  [done] - number
* FormulaField                  [done] - formula

* LinkField                     [doing] - reference
* ReferenceField                [todo]: A reference from a linked folder. Config with linked field.

These are not complex:
* GenerateIdField               [todo] - text : Random ids
* GeneratedNumberField          [todo] - number: Sequential number - number : Sequence number.
* LanguageField                 [todo] - text
* PhoneField                    [todo]
* EmailField                    [todo]
* UrlField                      [todo]
* RatingField                   [todo]

* SetProfperty                  [todo]
* ObjectProperty                [todo]

* FolderField                   [todo]: This links to another db file with some media data: photo, etc...
    In this case we also map into table config, so I can easily have list of folders for this table. I only do
    one level. Here I define background image for the folder.
* StatsField                    [todo]: Statistics on linked fields with formula support: AVERAGE, 
    COUNT, COUNTA, COUNTALL, SUM, MAX, AND, OR, XOR, CONCATENATE. I execute these formulas once I post 
    processed the links and references. I would need to parse in a way to use those number functions.
* FileField                     [todo] - Custom file and image management with IPFS. I add many.

I might add for images these functions:
1. resize
2. thumb
3. blur and other simple operations

Above fields gives us what we need as EXCEL functions into the formula field. Formula can provide a combination 
of these function fields, which are not needed.

**xlformula_engine**
let formula = parse_formula::parse_string_to_formula(&"=1+2", None::<NoCustomFunction>);
let result = calculate::calculate_formula(formula, None::<NoReference>);
println!("Result is {}", calculate::result_to_string(result));

I can have some excel functions plus my custom functions.

For seq data queries, we use a formula AND, XOR, OR, etc... in a yaml we can do multi line and looks fine with
indents.

Then on the app, we have a visual way to add functions, helper content, etc...

*/

pub trait StorageProperty {
    fn update_config_map(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, PlanetError>;
    fn build_config(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<PropertyConfig, PlanetError>;
    fn validate(&self, data: &String) -> Result<String, PlanetError>;
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String;
}

pub trait FormulaStorageProperty {
    fn update_config_map(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
        field_name_map: &BTreeMap<String, String>,
        field_type_map: &BTreeMap<String, String>,
        db_table: &DbFolder,
        table_name: &String,
    ) -> Result<BTreeMap<String, String>, PlanetError>;
    fn build_config(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<PropertyConfig, PlanetError>;
    fn validate(&self, data: &String) -> Result<String, PlanetError>;
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String;
}


pub trait ValidateProperty {
    fn is_valid(&self, value: Option<&String>) -> Result<bool, PlanetError>;
}
pub trait ValidateManyProperty {
    fn is_valid(&self, value: Option<Vec<String>>) -> Result<bool, PlanetError>;
}
pub trait ValidateFormulaProperty {
    fn is_valid(&self, value: Option<&String>) -> Result<bool, PlanetError>;
}

pub trait ProcessProperty {
    fn process(
        &self,
        insert_data_map: BTreeMap<String, String>,
        db_data: DbData
    ) -> Result<DbData, PlanetError>;
}
pub trait ProcessManyProperty {
    fn process(
        &self,
        insert_data_collections_map: BTreeMap<String, Vec<String>>,
        db_data: DbData
    ) -> Result<DbData, PlanetError>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PropertyType {
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
