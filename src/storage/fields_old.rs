extern crate sled;
extern crate xlformula_engine;

use std::str::FromStr;
use std::collections::HashMap;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use tr::tr;
use serde_yaml;

use crate::commands::table::constants::{FIELD_IDS, KEY, SELECT_OPTIONS, VALUE};
use crate::planet::constants::ID;
use crate::planet::{PlanetError};
use crate::storage::table::{DbData};
use crate::commands::table::config::FieldConfig;
use crate::storage::constants::*;
use crate::functions::{execute_formula, Formula};

/*
These are the core fields implemented so we can tackle the security and permissions system

* 01. SmallTextField                [impl] - text
* 02. LongTextField                 [impl] - text : This is the text field.
* 03. CheckBoxField                 [impl] - number
* 05. SelectField                   [impl] - text
* 06. DateField                     - date
* 07. NumberField                   [impl] - number
* 08. AuditTimeField                - audit
* 09. AuditByField                  - audit
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

// pub trait ValidateField {
//     fn is_valid(&self, value: Option<&String>) -> Result<bool, PlanetError>;
// }
// pub trait ValidateManyField {
//     fn is_valid(&self, value: Option<Vec<String>>) -> Result<bool, PlanetError>;
// }
// pub trait ValidateFormulaField {
//     fn is_valid(&self, value: Option<&String>) -> Result<bool, PlanetError>;
// }

// pub trait ProcessField {
//     fn process(
//         &self,
//         insert_data_map: HashMap<String, String>,
//         db_data: DbData
//     ) -> Result<DbData, PlanetError>;
// }
// pub trait ProcessManyField {
//     fn process(
//         &self,
//         insert_data_collections_map: HashMap<String, Vec<String>>,
//         db_data: DbData
//     ) -> Result<DbData, PlanetError>;
// }
// pub trait DbDumpString {
//     fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String;
// }
// pub trait DbDumpBool {
//     fn get_yaml_out(&self, yaml_string: &String, value: bool) -> String;
// }
// pub trait DbDumpNumber {
//     fn get_yaml_out(&self, yaml_string: &String, value: &i32) -> String;
// }
// pub trait DbDumpSingleSelect {
//     fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String;
// }

// pub trait StringValueField {
//     fn get_value(&self, value_db: Option<&String>) -> Option<String>;
//     fn get_value_db(&self, value: Option<&String>) -> Option<String>;
// }
// pub trait StringVectorValueField {
//     fn get_value(&self, values_db: Option<Vec<HashMap<String, String>>>) -> Option<Vec<HashMap<String, String>>>;
//     fn get_value_db(&self, value: Option<Vec<String>>) -> Option<Vec<HashMap<String, String>>>;
// }
// pub trait NumberValueField {
//     fn get_value(&self, value_db: Option<&String>) -> Option<i32>;
//     fn get_value_db(&self, value: Option<&i32>) -> Option<String>;
// }
// pub trait BoolValueField {
//     fn get_value(&self, value_db: Option<&String>) -> Option<bool>;
//     fn get_value_db(&self, value: Option<bool>) -> Option<String>;
// }

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub enum FieldType {
//     SmallText(String),
//     LongText(String),
//     CheckBox(bool),
//     NumberField(i32),
//     SelectField(String),
// }

// // SingleSelectField => 

// // SingleSelectField which type on enum???
// // All fields would go into String, Number and basic types?????

// pub fn error_validate_process(field_type: &str, field_name: &str) -> PlanetError {
//     let error = PlanetError::new(
//         500, 
//         Some(tr!(
//             "Could not validate \"{field_type}\" field {}{}{}", 
//             String::from("\"").blue(), &field_name.blue(), String::from("\"").blue(),
//             field_type=field_type
//         )),
//     );
//     return error;
// }

// // Fieldds




