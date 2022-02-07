pub mod text;
pub mod number;
pub mod formula;
pub mod date;
pub mod reference;
pub mod structure;

use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use tr::tr;
use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::planet::{PlanetError};
use crate::storage::folder::{DbData, TreeFolder};
use crate::planet::constants::*;
use crate::commands::folder::config::*;

/*
These are the core fields implemented so we can tackle the security and permissions system

Table fields
============

* SmallTextColumn                 [done] - text
* LongTextColumn                  [partly done] - text : This is the text field, needs to be updated based on full text search.
* CheckBoxColumn                  [done] - number
* SelectColumn                    [done] - text
* DateColumn                      [done] - date
* DurationColumn                  [done] - date
* NumberColumn                    [done] - number
* AuditTimeColumn                 [done] - date
* AuditByColumn                   [done] - text
* CurrencyColumn                  [done] - number
* PercentColumn                   [done] - number
* FormulaColumn                   [done] - formula
* LinkColumn                      [done] - reference
* ReferenceColumn                 [done] - A reference from a linked folder. Config with linked field. These
    are subqueries. We do subquery when get by id. Also when doing select and search operations.
* LanguageColumn                  [done] - text
* TextColumn                      [done] - text
* GenerateIdColumn                [done] - text : Random ids
* GenerateNumberColumn            [done] - number: Sequential number - number : Sequence number.
* PhoneColumn                     [done]
* EmailColumn                     [done]
* UrlColumn                       [done]
* RatingColumn                    [done]

-
These are not complex:

* SetColumn                       [todo] - For example, tags
* ObjectColumn                    [todo]

* SubFolderColumn                 [todo]: This links to another db file with some media data: photo, etc...
    In this case we also map into table config, so I can easily have list of folders for this table. I only 
    do one level. Here I define background image for the folder.
* StatsColumn                     [todo]: Statistics on linked fields with formula support: AVERAGE, 
    COUNT, COUNTA, COUNTALL, SUM, MAX, AND, OR, XOR, CONCATENATE. I execute these formulas once I post 
    processed the links and references. I would need to parse in a way to use those number functions.
* FileColumn                      [todo] - Custom file and image management with IPFS. I add many.
* CommandColumn                   [todo]: This does processing for complex cases, like image manipulation

Above fields gives us what we need as EXCEL functions into the formula field. Formula can provide a 
combination of these function fields, which are not needed.

**xlformula_engine**
let formula = parse_formula::parse_string_to_formula(&"=1+2", None::<NoCustomFunction>);
let result = calculate::calculate_formula(formula, None::<NoReference>);
println!("Result is {}", calculate::result_to_string(result));

I can have some excel functions plus my custom functions.

For seq data queries, we use a formula AND, XOR, OR, etc... in a yaml we can do multi line and looks fine with
indents.

Then on the app, we have a visual way to add functions, helper content, etc...

*/

pub trait StorageColumn {
    fn update_config_map(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, PlanetError>;
    fn build_config(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError>;
    fn validate(&self, data: &Vec<String>) -> Result<Vec<String>, PlanetError>;
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String;
}

pub fn validate_set(config: &ColumnConfig, data: &Vec<String>) -> Result<(), PlanetError> {
    let config = config.clone();
    let column_name = config.name.unwrap_or_default();
    let is_set = config.is_set;
    let set_maximum = config.set_maximum;
    let set_minimum = config.set_minimum;
    let number_items = data.len();
    if is_set.is_some() {
        let is_set = is_set.unwrap().to_lowercase();
        // validate when is not a set and I have many items
        if is_set != String::from(TRUE) && number_items > 1 {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!(
                        "Column data for \"{}\" is not a set and number items is higher than 1.", 
                        &column_name
                    ))
                )
            );
        }
        // validate maximum and minimum for set
        if set_maximum.is_some() {
            let set_maximum = set_maximum.unwrap();
            let set_maximum: usize = FromStr::from_str(&set_maximum).unwrap();
            if number_items > set_maximum {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!(
                            "Number items in \"{}\" is higher than maximum, \"{}\".",
                            &column_name, &set_maximum
                        ))
                    )
                );
            }
        }
        if set_minimum.is_some() {
            let set_minimum = set_minimum.unwrap();
            let set_minimum: usize = FromStr::from_str(&set_minimum).unwrap();
            if number_items < set_minimum {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!(
                            "Number items in \"{}\" is lower than minimum, \"{}\".",
                            &column_name, &set_minimum
                        ))
                    )
                );
            }
        }
    }
    return Ok(())
}

pub trait ObjectStorageColumn<'gb> {
    fn update_config_map(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
        properties_map: &HashMap<String, ColumnConfig>,
        table_name: &String,
    ) -> Result<BTreeMap<String, String>, PlanetError>;
    fn build_config(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError>;
    fn validate(
        &self, 
        data: &Vec<String>, 
    ) -> Result<Vec<String>, PlanetError>;
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String;
}

pub trait FormulaStorageColumn {
    fn update_config_map(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
        field_name_map: &BTreeMap<String, String>,
        field_type_map: &BTreeMap<String, String>,
        db_table: &TreeFolder,
        table_name: &String,
    ) -> Result<BTreeMap<String, String>, PlanetError>;
    fn build_config(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError>;
    fn validate(&self, data: &String) -> Result<String, PlanetError>;
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String;
}


pub trait ValidateColumn {
    fn is_valid(&self, value: Option<&String>) -> Result<bool, PlanetError>;
}
pub trait ValidateManyColumn {
    fn is_valid(&self, value: Option<Vec<String>>) -> Result<bool, PlanetError>;
}
pub trait ValidateFormulaColumn {
    fn is_valid(&self, value: Option<&String>) -> Result<bool, PlanetError>;
}

pub trait ProcessColumn {
    fn process(
        &self,
        insert_data_map: BTreeMap<String, String>,
        db_data: DbData
    ) -> Result<DbData, PlanetError>;
}
pub trait ProcessManyColumn {
    fn process(
        &self,
        insert_data_collections_map: BTreeMap<String, Vec<String>>,
        db_data: DbData
    ) -> Result<DbData, PlanetError>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ColumnType {
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
