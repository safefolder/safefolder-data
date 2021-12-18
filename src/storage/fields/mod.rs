pub mod text;
pub mod number;
pub mod formula;
pub mod date;

use std::collections::HashMap;
use tr::tr;
use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::planet::{PlanetError};
use crate::storage::table::{DbData, DbTable};
use crate::commands::table::config::*;

/*
These are the core fields implemented so we can tackle the security and permissions system

* 01. SmallTextField                [impl] - text
* 02. LongTextField                 [impl] - text : This is the text field.
* 03. CheckBoxField                 [impl] - number
* 05. SelectField                   [impl] - text
* 06. DateField                     [impl] - date
* 06A Duration                      [impl] - date
* 07. NumberField                   [impl] - number
* 08. AuditTimeField                [impl] - date
* 09. AuditByField                  - text
* 10. LinkField (This probably later once I have more ops from DbRow to get items, etc...)
* 11. CurrencyField                 - number
* 12. PercentField                  - number
* 13. CountField (This is parameters of COUNT() query when we go seq in table, defines query) - agg
* 14. GenerateIdField               - text : Random ids
* 15. GeneratedNumberField : Sequential number - number : Sequence number.
* 16. LanguageField                 - text
* 17. NumberCollectionField : Is this like SetField???
* 18. SmallTextCollectionField : ????
* 19. FormulaField                  [impl] - formula
* 20. SetField: List of items in a field, strings, numbers, etc... All same type, which goes into the definition on the schema table
* 21. ObjectField: Object embedded with additional information, to group data into objects.

Above fields gives us what we need as EXCEL functions into the formula field. Formula can provide a combination of
these function fields, which are not needed.

**xlformula_engine**
let formula = parse_formula::parse_string_to_formula(&"=1+2", None::<NoCustomFunction>);
let result = calculate::calculate_formula(formula, None::<NoReference>);
println!("Result is {}", calculate::result_to_string(result));

I can have some excel functions plus my custom functions.

For seq data queries, we use a formula AND, XOR, OR, etc... in a yaml we can do multi line and looks fine with
indents.

Then on the app, we have a visual way to add functions, helper content, etc...

*/

pub trait StorageField {
    fn update_config_map(
        &mut self, 
        field_config_map: &HashMap<String, String>,
    ) -> Result<HashMap<String, String>, PlanetError>;
    fn build_config(
        &mut self, 
        field_config_map: &HashMap<String, String>,
    ) -> Result<FieldConfig, PlanetError>;
    fn validate(&self, data: &String) -> Result<String, PlanetError>;
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String;
}

pub trait FormulaStorageField {
    fn update_config_map(
        &mut self, 
        field_config_map: &HashMap<String, String>,
        field_name_map: &HashMap<String, String>,
        field_type_map: &HashMap<String, String>,
        db_table: &DbTable,
        table_name: &String,
    ) -> Result<HashMap<String, String>, PlanetError>;
    fn build_config(
        &mut self, 
        field_config_map: &HashMap<String, String>,
    ) -> Result<FieldConfig, PlanetError>;
    fn validate(&self, data: &String) -> Result<String, PlanetError>;
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String;
}


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
