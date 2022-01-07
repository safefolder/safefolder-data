pub mod text;
pub mod number;
pub mod formula;
pub mod date;
pub mod reference;

use std::collections::{BTreeMap, HashMap};
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

* SmallTextProperty                 [done] - text
* LongTextProperty                  [partly done] - text : This is the text field, needs to be updated based on full text search.
* CheckBoxProperty                  [done] - number
* SelectProperty                    [done] - text
* DateProperty                      [done] - date
* DurationProperty                  [done] - date
* NumberProperty                    [done] - number
* AuditTimeProperty                 [done] - date
* AuditByProperty                   [done] - text
* CurrencyProperty                  [done] - number
* PercentProperty                   [done] - number
* FormulaProperty                   [done] - formula

* LinkProperty                      [doing] - reference
* ReferenceProperty                 [doing] - A reference from a linked folder. Config with linked field. These
    are subqueries. We do subquery when get by id. Also when doing select and search operations.
-
These are not complex:
* GenerateIdProperty                [todo] - text : Random ids
* GeneratedNumberProperty           [todo] - number: Sequential number - number : Sequence number.
* LanguageProperty                  [todo] - text
* PhoneProperty                     [todo]
* EmailProperty                     [todo]
* UrlProperty                       [todo]
* RatingProperty                    [todo]

* SetProperty                       [todo] - For example, tags
* ObjectProperty                    [todo]

* SubFolderProperty                 [todo]: This links to another db file with some media data: photo, etc...
    In this case we also map into table config, so I can easily have list of folders for this table. I only do
    one level. Here I define background image for the folder.
* StatsProperty                     [todo]: Statistics on linked fields with formula support: AVERAGE, 
    COUNT, COUNTA, COUNTALL, SUM, MAX, AND, OR, XOR, CONCATENATE. I execute these formulas once I post 
    processed the links and references. I would need to parse in a way to use those number functions.
* FileProperty                      [todo] - Custom file and image management with IPFS. I add many.
* CommandProperty                   [todo]: This does processing for complex cases, like image manipulation

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

pub trait ObjectStorageProperty<'gb> {
    fn update_config_map(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
        properties_map: &HashMap<String, PropertyConfig>,
        table_name: &String,
    ) -> Result<BTreeMap<String, String>, PlanetError>;
    fn build_config(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<PropertyConfig, PlanetError>;
    fn validate(
        &self, 
        data: &Vec<String>, 
    ) -> Result<Vec<String>, PlanetError>;
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
