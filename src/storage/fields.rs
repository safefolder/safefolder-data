extern crate sled;

use std::str::FromStr;
use std::collections::HashMap;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use tr::tr;

use crate::commands::table::constants::{FIELD_IDS, KEY, SELECT_OPTIONS, VALUE};
use crate::planet::{PlanetError};
use crate::storage::table::{DbData};
use crate::commands::table::config::FieldConfig;

/*
These are the core fields implemented so we can tackle the security and permissions system

* 01. SmallTextField                [impl]
* 02. LongTextField                 [impl]
* 03. CheckBoxField                 [impl]
* 04. MultipleSelectField           [doing]
* 05. SingleSelectField             [impl]
* 06. DateField
* 07. NumberField                   [impl]
* 08. AuditTimeField
* 09. AuditByField
* 10. LinkField (This probably later once I have more ops from DbRow to get items, etc...)
* 11. NameField (*)
* 12. CurrencyField
* 13. PercentField
* 14. EmailField ???? Maybe move to master fields when we do
* 15. UrlField ???? Maybe move to master fields when we do
* 16. CountField (This is parameters of COUNT() query when we go seq in table, defines query)
* 17. GenerateIdField
* 18. GenerateNumberField
* 19. LanguageField
* 20. NumberCollectionField
* 21. SmallTextCollectionField
* 22. FormulaField (*)
* 23. SetField: List of items in a field, strings, numbers, etc... All same type, which goes into the definition on the schema table
* 24. ObjectField: Object embedded with additional information, to group data into objects.

Functions / Formulas

* FormulaField: This would use excel functions, etc... to come up with value
* ConcatenateField
* DateFormatField
* DateModifyField
* DayField
* DivideField
* HourField
* JoinListField
* LengthField
* LowerField
* MonthField
* MultiplyField
* NowField
* ReplaceField
* SecondField
* SubtractField
* TodayField
* UpperField
* WeekField
* YearField

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

pub trait ValidateField {
    fn is_valid(&self, value: Option<&String>) -> Result<bool, PlanetError>;
}
pub trait ValidateManyField {
    fn is_valid(&self, value: Option<Vec<String>>) -> Result<bool, PlanetError>;
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
pub trait DbDumpString {
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String;
}
pub trait DbDumpBool {
    fn get_yaml_out(&self, yaml_string: &String, value: bool) -> String;
}
pub trait DbDumpNumber {
    fn get_yaml_out(&self, yaml_string: &String, value: &i32) -> String;
}
pub trait DbDumpSingleSelect {
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String;
}

pub trait StringValueField {
    fn get_value(&self, value_db: Option<&String>) -> Option<String>;
    fn get_value_db(&self, value: Option<&String>) -> Option<String>;
}
pub trait StringVectorValueField {
    fn get_value(&self, values_db: Option<Vec<HashMap<String, String>>>) -> Option<Vec<HashMap<String, String>>>;
    fn get_value_db(&self, value: Option<Vec<String>>) -> Option<Vec<HashMap<String, String>>>;
}
pub trait NumberValueField {
    fn get_value(&self, value_db: Option<&String>) -> Option<i32>;
    fn get_value_db(&self, value: Option<&i32>) -> Option<String>;
}
pub trait BoolValueField {
    fn get_value(&self, value_db: Option<&String>) -> Option<bool>;
    fn get_value_db(&self, value: Option<bool>) -> Option<String>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FieldType {
    SmallText(String),
    LongText(String),
    CheckBox(bool),
    NumberField(i32),
    SingleSelectField(String),
    MultipleSelectField(Vec<String>),
}

// SingleSelectField => 

// SingleSelectField which type on enum???
// All fields would go into String, Number and basic types?????

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

// Fieldds

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SmallTextField {
    pub field_config: FieldConfig
}

impl SmallTextField {
    pub fn defaults(field_config: &FieldConfig) -> Self {
        let field_config = field_config.clone();
        let field_obj = Self{
            field_config: field_config
        };
        return field_obj
    }
    pub fn init_do(field_config: &FieldConfig, data_map: HashMap<String, String>, mut db_data: DbData) -> Result<DbData, PlanetError> {
        let field_object = Self::defaults(field_config);
        db_data = field_object.process(data_map.clone(), db_data)?;
        return Ok(db_data)
    }
    pub fn init_get(
        field_config: &FieldConfig, 
        data: Option<&HashMap<String, String>>, 
        yaml_out_str: &String
    ) -> Result<String, PlanetError> {
        let field_config_ = field_config.clone();
        let field_id = field_config_.id.unwrap();
        let data = data.unwrap().clone();
        let field_obj = Self::defaults(&field_config);
        let value_db = data.get(&field_id).unwrap().clone();
        let value = field_obj.get_value(Some(&value_db)).unwrap();
        let yaml_out_str = field_obj.get_yaml_out(yaml_out_str, &value);
        return Ok(yaml_out_str)
    }
}

impl DbDumpString for SmallTextField {
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String {
        let field_config = self.field_config.clone();
        let field_name = field_config.name.unwrap();
        let mut yaml_string = yaml_string.clone();
        let field = &field_name.blue();
        let value = format!("{}{}{}", 
            String::from("\"").truecolor(255, 165, 0), 
            value.truecolor(255, 165, 0), 
            String::from("\"").truecolor(255, 165, 0)
        );
        yaml_string.push_str(format!("{field}: {value}\n", field=field, value=value).as_str());
        return yaml_string;
    }
}

impl ValidateField for SmallTextField {
    fn is_valid(&self, value: Option<&String>) -> Result<bool, PlanetError> {
        let field_config = self.field_config.clone();
        let required = field_config.required.unwrap();
        let name = field_config.name.unwrap();
        if value.is_none() && required == true {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!(
                        "Field {}{}{} is required", 
                        String::from("\"").blue(), name.blue(), String::from("\"").blue()
                    )),
                )
            );
        } else {
            return Ok(true)
        }
    }
}
impl ProcessField for SmallTextField {
    fn process(
        &self,
        insert_data_map: HashMap<String, String>,
        mut db_data: DbData
    ) -> Result<DbData, PlanetError> {
        let field_config = self.field_config.clone();
        let field_id = field_config.id.unwrap_or_default();
        let field_name = field_config.name.unwrap_or_default();
        let value_entry = insert_data_map.get(&field_name).unwrap().clone();
        let value_db = value_entry.clone();
        let field = Self{
            field_config: self.field_config.clone()
        };
        let mut data: HashMap<String, String> = HashMap::new();
        if db_data.data.is_some() {
            data = db_data.data.unwrap();
        }
        let is_valid = field.is_valid(Some(&value_db))?;
        if is_valid == true {
            &data.insert(field_id, value_db);
            db_data.data = Some(data);
            return Ok(db_data);
        } else {
            return Err(error_validate_process("Small Text", &field_name))
        }
    }
}
impl StringValueField for SmallTextField {
    fn get_value(&self, value_db: Option<&String>) -> Option<String> {
        if value_db.is_none() {
            return None
        } else {
            let value_final = value_db.unwrap().clone();
            return Some(value_final);
        }
    }
    fn get_value_db(&self, value: Option<&String>) -> Option<String> {
        if *&value.is_some() {
            let value = value.unwrap().clone();
            return Some(value);
        }
        return None
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LongTextField {
    pub field_config: FieldConfig
}

impl LongTextField {
    pub fn defaults(field_config: &FieldConfig) -> Self {
        let field_config = field_config.clone();
        let field_obj = Self{
            field_config: field_config
        };
        return field_obj
    }
    pub fn init_do(field_config: &FieldConfig, data_map: HashMap<String, String>, mut db_data: DbData) -> Result<DbData, PlanetError> {
        let field_object = Self::defaults(field_config);
        db_data = field_object.process(data_map.clone(), db_data)?;
        return Ok(db_data)
    }
    pub fn init_get(
        field_config: &FieldConfig, 
        data: Option<&HashMap<String, String>>, 
        yaml_out_str: &String
    ) -> Result<String, PlanetError> {
        let field_config_ = field_config.clone();
        let field_id = field_config_.id.unwrap();
        let data = data.unwrap().clone();
        let field_obj = Self::defaults(&field_config);
        let value_db = data.get(&field_id).unwrap().clone();
        let value = field_obj.get_value(Some(&value_db)).unwrap();
        let yaml_out_str = field_obj.get_yaml_out(yaml_out_str, &value);
        return Ok(yaml_out_str)
    }
}

impl DbDumpString for LongTextField {
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String {
        let field_config = self.field_config.clone();
        let field_name = field_config.name.unwrap();
        let mut yaml_string = yaml_string.clone();
        let field = &field_name.blue();
        let value = format!("{}{}{}", 
            String::from("\"").truecolor(255, 165, 0), 
            value.truecolor(255, 165, 0), 
            String::from("\"").truecolor(255, 165, 0)
        );
        yaml_string.push_str(format!("{field}: {value}\n", field=field, value=value).as_str());
        return yaml_string;
    }
}

impl ValidateField for LongTextField {
    fn is_valid(&self, value: Option<&String>) -> Result<bool, PlanetError> {
        let field_config = self.field_config.clone();
        let required = field_config.required.unwrap();
        let name = field_config.name.unwrap();
        if value.is_none() && required == true {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!(
                        "Field {}{}{} is required", 
                        String::from("\"").blue(), name.blue(), String::from("\"").blue()
                    )),
                )
            );
        } else {
            return Ok(true)
        }
    }
}
impl ProcessField for LongTextField {
    fn process(
        &self,
        insert_data_map: HashMap<String, String>,
        mut db_data: DbData
    ) -> Result<DbData, PlanetError> {
        let field_config = self.field_config.clone();
        let field_name = field_config.name.unwrap_or_default();
        let field_id = field_config.id.unwrap_or_default();
        let value_entry = insert_data_map.get(&field_name).unwrap().clone();
        let value_db = value_entry.clone();
        let field = Self{
            field_config: self.field_config.clone()
        };
        let mut data: HashMap<String, String> = HashMap::new();
        if db_data.data.is_some() {
            data = db_data.data.unwrap();
        }
        let is_valid = field.is_valid(Some(&value_db))?;
        if is_valid == true {
            &data.insert(field_id, value_db);
            db_data.data = Some(data);
            return Ok(db_data);
        } else {
            return Err(error_validate_process("Long Text", &field_name))
        }
    }
}
impl StringValueField for LongTextField {
    fn get_value(&self, value_db: Option<&String>) -> Option<String> {
        if value_db.is_none() {
            return None
        } else {
            let value_final = value_db.unwrap().clone();
            return Some(value_final);
        }
    }
    fn get_value_db(&self, value: Option<&String>) -> Option<String> {
        if *&value.is_some() {
            let value = value.unwrap().clone();
            return Some(value);
        }
        return None
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CheckBoxField {
    pub field_config: FieldConfig
}

impl CheckBoxField {
    pub fn defaults(field_config: &FieldConfig) -> Self {
        let field_config = field_config.clone();
        let field_obj = Self{
            field_config: field_config
        };
        return field_obj
    }
    pub fn init_do(field_config: &FieldConfig, data_map: HashMap<String, String>, mut db_data: DbData) -> Result<DbData, PlanetError> {
        let field_object = Self::defaults(field_config);
        db_data = field_object.process(data_map.clone(), db_data)?;
        return Ok(db_data)
    }
    pub fn init_get(
        field_config: &FieldConfig, 
        data: Option<&HashMap<String, String>>, 
        yaml_out_str: &String
    ) -> Result<String, PlanetError> {
        let field_config_ = field_config.clone();
        let field_id = field_config_.id.unwrap();
        let data = data.unwrap().clone();
        let field_obj = Self::defaults(&field_config);
        let value_db = data.get(&field_id).unwrap().clone();
        let value = field_obj.get_value(Some(&value_db)).unwrap();
        let yaml_out_str = field_obj.get_yaml_out(yaml_out_str, value);
        return Ok(yaml_out_str)
    }
}

impl DbDumpBool for CheckBoxField {
    fn get_yaml_out(&self, yaml_string: &String, value: bool) -> String {
        let field_config = self.field_config.clone();
        let field_name = field_config.name.unwrap();
        let mut yaml_string = yaml_string.clone();
        let field = &field_name.blue();
        let value = format!("{}", value.to_string().blue());
        yaml_string.push_str(format!("{field}: {value}\n", field=field, value=value).as_str());
        return yaml_string;
    }
}

impl ValidateField for CheckBoxField {
    fn is_valid(&self, value: Option<&String>) -> Result<bool, PlanetError> {
        let field_config = self.field_config.clone();
        let required = field_config.required.unwrap();
        let name = field_config.name.unwrap();
        eprintln!("CheckBoxField.is_valid :: value: {:?}", &value);
        if value.is_none() && required == true {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!(
                        "Field {}{}{} is required", 
                        String::from("\"").blue(), name.blue(), String::from("\"").blue()
                    )),
                )
            );
        } else {
            let value_str = value.unwrap().as_str();
            eprintln!("CheckBoxField.is_valid :: value_str: {:?}", &value_str);
            if value_str == "true" || value_str == "false" {
                return Ok(true);
            } else {
                return Ok(false)
            }
        }
    }
}
impl ProcessField for CheckBoxField {
    fn process(
        &self,
        insert_data_map: HashMap<String, String>,
        mut db_data: DbData
    ) -> Result<DbData, PlanetError> {
        let field_config = self.field_config.clone();
        let field_name = field_config.name.unwrap_or_default();
        let field_id = field_config.id.unwrap_or_default();
        let value_entry = insert_data_map.get(&field_name).unwrap().clone();
        let value_db = value_entry.clone();
        let field = Self{
            field_config: self.field_config.clone()
        };
        let mut data: HashMap<String, String> = HashMap::new();
        if db_data.data.is_some() {
            data = db_data.data.unwrap();
        }
        let is_valid = field.is_valid(Some(&value_db))?;
        if is_valid == true {
            &data.insert(field_id, value_db);
            db_data.data = Some(data);
            return Ok(db_data);
        } else {
            return Err(error_validate_process("CheckBox", &field_name))
        }
    }
}
impl BoolValueField for CheckBoxField {
    fn get_value(&self, value_db: Option<&String>) -> Option<bool> {
        if value_db.is_none() {
            return None
        } else {
            let value_str = value_db.unwrap().as_str();
            if value_str == "true" {
                return Some(true);
            } else {
                return Some(false);
            }
        }
    }
    fn get_value_db(&self, value: Option<bool>) -> Option<String> {
        if *&value.is_some() {
            let value = value.unwrap();
            if value == true {
                return Some(String::from("true"))
            } else {
                return Some(String::from("false"));
            }
        } else {
            return None
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NumberField {
    pub field_config: FieldConfig
}
impl NumberField {
    pub fn defaults(field_config: &FieldConfig) -> Self {
        let field_config = field_config.clone();
        let field_obj = Self{
            field_config: field_config
        };
        return field_obj
    }
    pub fn init_do(field_config: &FieldConfig, data_map: HashMap<String, String>, mut db_data: DbData) -> Result<DbData, PlanetError> {
        let field_object = Self::defaults(field_config);
        db_data = field_object.process(data_map.clone(), db_data)?;
        return Ok(db_data)
    }
    pub fn init_get(
        field_config: &FieldConfig, 
        data: Option<&HashMap<String, String>>, 
        yaml_out_str: &String
    ) -> Result<String, PlanetError> {
        let field_config_ = field_config.clone();
        let field_id = field_config_.id.unwrap();
        let data = data.unwrap().clone();
        let field_obj = Self::defaults(&field_config);
        let value_db = data.get(&field_id).unwrap().clone();
        let value = field_obj.get_value(Some(&value_db)).unwrap();
        let yaml_out_str = field_obj.get_yaml_out(yaml_out_str, &value);
        return Ok(yaml_out_str)
    }
}
impl DbDumpNumber for NumberField {
    fn get_yaml_out(&self, yaml_string: &String, value: &i32) -> String {
        let field_config = self.field_config.clone();
        let field_name = field_config.name.unwrap();
        let mut yaml_string = yaml_string.clone();
        let field = &field_name.blue();
        let value = format!("{}", value.to_string().truecolor(255, 255, 200));
        yaml_string.push_str(format!("{field}: {value}\n", field=field, value=value).as_str());
        return yaml_string;
    }
}
impl ValidateField for NumberField {
    fn is_valid(&self, value: Option<&String>) -> Result<bool, PlanetError> {
        let field_config = self.field_config.clone();
        let required = field_config.required.unwrap();
        let name = field_config.name.unwrap();
        if value.is_none() && required == true {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!(
                        "Field {}{}{} is required", 
                        String::from("\"").blue(), name.blue(), String::from("\"").blue()
                    )),
                )
            );
        } else {
            let value_str = value.unwrap().as_str();
            let result = i32::from_str(value_str);
            match result {
                Ok(_) => {
                    // let value: i32 = result.unwrap();
                    return Ok(true);
                },
                Err(_) => {
                    return Ok(false)
                }
            }
        }
    }
}
impl ProcessField for NumberField {
    fn process(
        &self,
        insert_data_map: HashMap<String, String>,
        mut db_data: DbData
    ) -> Result<DbData, PlanetError> {
        let field_config = self.field_config.clone();
        let field_name = field_config.name.unwrap_or_default();
        let field_id = field_config.id.unwrap_or_default();
        let value_entry = insert_data_map.get(&field_name).unwrap().clone();
        let value_db = value_entry.clone();
        let field = Self{
            field_config: self.field_config.clone()
        };
        let mut data: HashMap<String, String> = HashMap::new();
        if db_data.data.is_some() {
            data = db_data.data.unwrap();
        }
        let is_valid = field.is_valid(Some(&value_entry))?;
        if is_valid == true {
            &data.insert(field_id, value_db);
            db_data.data = Some(data);
            return Ok(db_data);
        } else {
            return Err(error_validate_process("Number", &field_name))
        }
    }
}
impl NumberValueField for NumberField {
    fn get_value(&self, value_db: Option<&String>) -> Option<i32> {
        if value_db.is_none() {
            return None
        } else {
            let value_str = value_db.unwrap().as_str();
            let value: i32 = i32::from_str(value_str).unwrap();
            return Some(value)
        }
    }
    fn get_value_db(&self, value: Option<&i32>) -> Option<String> {
        if *&value.is_some() {
            let value = value.unwrap();
            let value_str = value.to_string();
            return Some(value_str);
        } else {
            return None
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SelectField {
    pub field_config: FieldConfig,
    pub options_id_map: Option<HashMap<String, String>>,
    pub options_name_map: Option<HashMap<String, String>>,
}
impl SelectField {
    pub fn defaults(field_config: &FieldConfig, table: Option<&DbData>) -> Self {
        let field_config = field_config.clone();
        let mut field_obj = Self{
            field_config: field_config,
            options_id_map: None,
            options_name_map: None,
        };
        if table.is_some() {
            let table = table.unwrap();
            let mut options_id_map: HashMap<String, String> = HashMap::new();
            let mut options_name_map: HashMap<String, String> = HashMap::new();
            let field_config = &field_obj.field_config;
            let field_name = field_config.name.clone().unwrap();
            for data_collection in table.data_collections.clone() {
                // key for ordering: field_ids
                for key in data_collection.keys() {
                    if key.to_lowercase() != String::from(FIELD_IDS) {
                        // key: Status__select_options
                        let key_items: Vec<&str> = key.split("__").collect();
                        let key_field_name = key_items[0];
                        let key_field_type = key_items[1];
                        if key_field_type == SELECT_OPTIONS && key_field_name.to_lowercase() == field_name.to_lowercase() {
                            // Process, since we have a simple select field
                            // "Status__select_options": [
                            //     {
                            //         "key": "c48kg78smpv5gct3hfqg",
                            //         "value": "Draft",
                            //     },
                            //     ...
                            // ],
                            let options: &Vec<HashMap<String, String>> = data_collection.get(key).unwrap();
                            for option in options {
                                options_id_map.insert(
                                    option.get(KEY).unwrap().clone(), 
                                    option.get(VALUE).unwrap().clone()
                                );
                                options_name_map.insert(
                                    option.get(VALUE).unwrap().clone(), 
                                    option.get(KEY).unwrap().clone()
                                );
                            }
                        }
                    }
                }
            }
            field_obj.options_id_map = Some(options_id_map);
            field_obj.options_name_map = Some(options_name_map);
        }
        return field_obj;
    }
    pub fn init_do(
        field_config: &FieldConfig, 
        table: &DbData,
        data_map: HashMap<String, String>, 
        mut db_data: DbData
    ) -> Result<DbData, PlanetError> {
        let field_object = Self::defaults(field_config, Some(table));
        db_data = field_object.process(data_map.clone(), db_data)?;
        return Ok(db_data)
    }
    pub fn init_get(
        field_config: &FieldConfig, 
        table: &DbData,
        data: Option<&HashMap<String, String>>, 
        yaml_out_str: &String
    ) -> Result<String, PlanetError> {
        let field_config_ = field_config.clone();
        let field_id = field_config_.id.unwrap();
        let data = data.unwrap().clone();
        let field_obj = Self::defaults(&field_config, Some(table));
        let value_db = data.get(&field_id).unwrap().clone();
        let value = field_obj.get_value(Some(&value_db)).unwrap();
        let yaml_out_str = field_obj.get_yaml_out(yaml_out_str, &value);
        return Ok(yaml_out_str)
    }
}
impl DbDumpSingleSelect for SelectField {
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String {
        let field_config = self.field_config.clone();
        let field_name = field_config.name.unwrap();
        let mut yaml_string = yaml_string.clone();
        let field = &field_name.blue();
        let value = format!("{}{}{}", 
            String::from("\"").truecolor(255, 165, 0), 
            value.truecolor(255, 165, 0), 
            String::from("\"").truecolor(255, 165, 0)
        );
        yaml_string.push_str(format!("{field}: {value}\n", field=&field, value=value).as_str());
        return yaml_string;
    }
}
impl ValidateField for SelectField {
    fn is_valid(&self, value: Option<&String>) -> Result<bool, PlanetError> {
        // value represents the id for the option selected, like id->name
        let field_config = self.field_config.clone();
        let required = field_config.required.unwrap();
        let name = field_config.name.unwrap();
        // let field_name = self.field.name.clone().unwrap_or_default();
        if value.is_none() && required == true {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!(
                        "Field {}{}{} is required", 
                        String::from("\"").blue(), &name.blue(), String::from("\"").blue()
                    )),
                )
            );
        } else {
            // In case no many, just string with id. In case many, list of ids separated by commas
            let value_id = value.unwrap();
            let id_list: Vec<&str> = value_id.split(",").collect();
            let field_config = self.field_config.clone();
            // Check that value appears on the config for choices id -> value
            // The option id is obtained from the table config
            let options = field_config.options.unwrap();
            let options_name_map = &self.options_name_map.clone().unwrap();
            let options_id_map = &self.options_id_map.clone().unwrap();
            let mut verified = false;
            if *&id_list.len() == 1 {
                for select_option in options {
                    let select_id = options_name_map.get(&select_option);
                    if select_id.is_some() {
                        let select_id = options_name_map.get(&select_option).unwrap();
                        if select_id == value_id {
                            verified = true;
                            break;
                        }
                    }
                }
            } else {
                verified = true;
                for item_select_id in id_list {
                    let item_select_id = &item_select_id.to_string();
                    let select_option = options_id_map.get(item_select_id);
                    if select_option.is_none() {
                        verified = false;
                        break
                    }
                }
            }
            if verified == true {
                return Ok(true)
            } else {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!(
                            "Field {}{}{} is not configured with select id {}{}{}", 
                            String::from("\"").blue(), &name.blue(), String::from("\"").blue(),
                            String::from("\"").blue(), value_id, String::from("\"").blue(),
                        )),
                    )
                );
            }            
        }
    }
}
impl ProcessField for SelectField {
    fn process(
        &self,
        insert_data_map: HashMap<String, String>,
        mut db_data: DbData
    ) -> Result<DbData, PlanetError> {
        let field_config = self.field_config.clone();
        let field_name = field_config.name.unwrap_or_default();
        let field_id = field_config.id.unwrap_or_default();
        let value_entry = insert_data_map.get(&field_name).unwrap().clone();
        let value_db = value_entry.clone();
        let mut data: HashMap<String, String> = HashMap::new();
        if db_data.data.is_some() {
            data = db_data.data.unwrap();
        }
        let value_string_ = insert_data_map.get(&field_name).unwrap().clone();
        let is_valid = self.is_valid(Some(&value_string_))?;
        if is_valid == true {
            &data.insert(field_id, value_db);
            db_data.data = Some(data);
            return Ok(db_data);
        } else {
            return Err(error_validate_process("Single Select", &field_name))
        }
    }
}
impl StringValueField for SelectField {
    fn get_value(&self, value_db: Option<&String>) -> Option<String> {
        // value_db can be either single or multiple separated by commas
        let mut resolved_id: Option<String> = None;
        let options = self.field_config.options.clone().unwrap();
        let options_map = self.options_name_map.clone().unwrap();
        let options_id_map = self.options_id_map.clone().unwrap();
        if value_db.is_none() {
            return None
        } else {
            let value = value_db.unwrap();
            let value_items: Vec<&str> = value.split(",").collect();
            if *&value_items.len() == 1 {                
                for option_value in options.iter() {
                    let select_id = options_map.get(option_value).unwrap().clone();
                    if select_id.to_lowercase() == value.to_lowercase() {
                        resolved_id = Some(select_id);
                    }
                }
            } else {
                let mut resolved_ids: Vec<String> = Vec::new();
                for item_select_id in value_items {
                    // let select_option = options_id_map.get(value_item_id).unwrap().clone();
                    let item_select_id = &item_select_id.to_string();
                    let select_option = options_id_map.get(item_select_id);
                    if select_option.is_some() {
                        let select_option = select_option.unwrap().clone();
                        resolved_ids.push(item_select_id.clone());
                    }
                    resolved_id = Some(resolved_ids.join(","));
                }
            }
            return resolved_id;
        }
    }
    fn get_value_db(&self, value: Option<&String>) -> Option<String> {
        // value is the literal, the option string literal
        // I return the option id
        if *&value.is_some() {
            let value = value.unwrap();
            let options = self.field_config.options.clone().unwrap();
            let options_map = self.options_name_map.clone().unwrap();
            let mut select_id: Option<String> = None;
            for option_value in options.iter() {
                if option_value.to_lowercase() == value.to_lowercase() {
                    select_id = Some(options_map.get(option_value).unwrap().clone());
                    break
                }
            }
            return select_id;
        } else {
            return None
        }
    }
}
