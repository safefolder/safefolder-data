extern crate sled;

use std::str::FromStr;
use std::collections::HashMap;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use tr::tr;

use crate::planet::{PlanetError};
use crate::storage::table::{RowItem};

/*
These are the core fields implemented so we can tackle the security and permissions system

* 01. SmallTextField                [impl]
* 02. LongTextField                 [impl]
* 03. CheckBoxField                 [impl]
* 04. MultipleSelectField
* 05. SingleSelectField
* 06. DateField
* 07. NumberField                   [impl]
* 08. AuditTimeField
* 09. AuditByField
* 10. LinkField (This probably later once I have more ops from DbRow to get items, etc...)
* 11. NameField
* 12. CurrencyField
* 13. PercentField
* 14. EmailField
* 15. UrlField
* 16. CountField (This is parameters of COUNT() query when we go seq in table, defines query)
* 17. GenerateIdField
* 18. GenerateNumberField
* 19. LanguageField
* 20. NumberCollectionField
* 21. SmallTextCollectionField
* 22. FormulaField

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


pub trait VerifyProcessField {
    fn is_valid(&self, value: Option<&String>) -> Result<bool, PlanetError>;
    fn process(
        data_map: &HashMap<String, String>, 
        required: bool, 
        version: String, 
        field_name: String,
        insert_data: Vec<RowItem>
    ) -> Result<Vec<RowItem>, PlanetError>;
}

pub trait StringValueField {
    fn get_value(&self, value: Option<&String>) -> Option<String>;
}

pub trait NumberValueField {
    fn get_value(&self, value: Option<&String>) -> Option<i32>;
}
pub trait BoolValueField {
    fn get_value(&self, value: Option<&String>) -> Option<bool>;
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SmallTextField {
    pub name: String,
    pub version: String,
    pub required: bool,
}
impl VerifyProcessField for SmallTextField {
    fn is_valid(&self, value: Option<&String>) -> Result<bool, PlanetError> {
        if value.is_none() && self.required == true {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!(
                        "Field {}{}{} is required", 
                        String::from("\"").blue(), self.name.blue(), String::from("\"").blue()
                    )),
                )
            );
        } else {
            return Ok(true)
        }
    }
    fn process(
        data_map: &HashMap<String, String>, 
        required: bool, 
        version: String, 
        field_name: String,
        mut insert_data: Vec<RowItem>
    ) -> Result<Vec<RowItem>, PlanetError> {
        let value_string_ = data_map.get(&field_name).unwrap().clone();
        let field = Self{
            name: field_name.clone(),
            required: required,
            version: version,
        };
        let is_valid = field.is_valid(Some(&value_string_))?;
        if is_valid == true {
            let value = field.get_value(Some(&value_string_)).unwrap_or_default();
            let row_item: RowItem = RowItem(FieldType::SmallText(value.clone()));
            insert_data.push(row_item);
            return Ok(insert_data);
        } else {
            return Err(error_validate_process("Small Text", &field_name))
        }
    }
}
impl StringValueField for SmallTextField {
    fn get_value(&self, value: Option<&String>) -> Option<String> {
        if value.is_none() {
            return None
        } else {
            let value_final = value.unwrap().clone();
            return Some(value_final);
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LongTextField {
    pub name: String,
    pub version: String,
    pub required: bool,
}
impl VerifyProcessField for LongTextField {
    fn is_valid(&self, value: Option<&String>) -> Result<bool, PlanetError> {
        if value.is_none() && self.required == true {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!(
                        "Field {}{}{} is required", 
                        String::from("\"").blue(), self.name.blue(), String::from("\"").blue()
                    )),
                )
            );
        } else {
            return Ok(true)
        }
    }
    fn process(
        data_map: &HashMap<String, String>, 
        required: bool, 
        version: String, 
        field_name: String,
        mut insert_data: Vec<RowItem>
    ) -> Result<Vec<RowItem>, PlanetError> {
        let value_string_ = data_map.get(&field_name).unwrap().clone();
        let field = Self{
            name: field_name.clone(),
            required: required,
            version: version,
        };
        let is_valid = field.is_valid(Some(&value_string_))?;
        if is_valid == true {
            let value = field.get_value(Some(&value_string_)).unwrap_or_default();
            let row_item: RowItem = RowItem(FieldType::LongText(value.clone()));
            insert_data.push(row_item);
            return Ok(insert_data);
        } else {
            return Err(error_validate_process("Long Text", &field_name))
        }
    }
}
impl StringValueField for LongTextField {
    fn get_value(&self, value: Option<&String>) -> Option<String> {
        if value.is_none() {
            return None
        } else {
            let value_final = value.unwrap().clone();
            return Some(value_final);
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CheckBoxField {
    pub name: String,
    pub version: String,
    pub required: bool,
}
impl VerifyProcessField for CheckBoxField {
    fn is_valid(&self, value: Option<&String>) -> Result<bool, PlanetError> {
        if value.is_none() && self.required == true {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!(
                        "Field {}{}{} is required", 
                        String::from("\"").blue(), self.name.blue(), String::from("\"").blue()
                    )),
                )
            );
        } else {
            let value_str = value.unwrap().as_str();
            if value_str == "true" || value_str == "false" {
                return Ok(true);
            } else {
                return Ok(false)
            }
        }
    }
    fn process(
        data_map: &HashMap<String, String>, 
        required: bool, 
        version: String, 
        field_name: String,
        mut insert_data: Vec<RowItem>
    ) -> Result<Vec<RowItem>, PlanetError> {
        let value_string_ = data_map.get(&field_name).unwrap().clone();
        let field = Self{
            name: field_name.clone(),
            required: required,
            version: version,
        };
        let is_valid = field.is_valid(Some(&value_string_))?;
        if is_valid == true {
            let value = field.get_value(Some(&value_string_)).unwrap_or_default();
            let row_item: RowItem = RowItem(FieldType::CheckBox(value.clone()));
            insert_data.push(row_item);
            return Ok(insert_data);
        } else {
            return Err(error_validate_process("CheckBox", &field_name))
        }
    }
}
impl BoolValueField for CheckBoxField {
    fn get_value(&self, value: Option<&String>) -> Option<bool> {
        if value.is_none() {
            return None
        } else {
            let value_str = value.unwrap().as_str();
            if value_str == "true" {
                return Some(true);
            } else {
                return Some(false);
            }            
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NumberField {
    pub name: String,
    pub version: String,
    pub required: bool,
}
impl VerifyProcessField for NumberField {
    fn is_valid(&self, value: Option<&String>) -> Result<bool, PlanetError> {
        if value.is_none() && self.required == true {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!(
                        "Field {}{}{} is required", 
                        String::from("\"").blue(), self.name.blue(), String::from("\"").blue()
                    )),
                )
            );
        } else {
            let value_str = value.unwrap().as_str();
            let result = i32::from_str(value_str);
            match result {
                Ok(_) => {
                    let value: i32 = result.unwrap();
                    return Ok(true);
                },
                Err(_) => {
                    return Ok(false)
                }
            }
        }
    }
    fn process(
        data_map: &HashMap<String, String>, 
        required: bool, 
        version: String, 
        field_name: String,
        mut insert_data: Vec<RowItem>
    ) -> Result<Vec<RowItem>, PlanetError> {
        let value_string_ = data_map.get(&field_name).unwrap().clone();
        let field = Self{
            name: field_name.clone(),
            required: required,
            version: version,
        };
        let is_valid = field.is_valid(Some(&value_string_))?;
        if is_valid == true {
            let value = field.get_value(Some(&value_string_)).unwrap_or_default();
            let row_item: RowItem = RowItem(FieldType::NumberField(value.clone()));
            insert_data.push(row_item);
            return Ok(insert_data);
        } else {
            return Err(error_validate_process("Number", &field_name))
        }
    }
}

impl NumberValueField for NumberField {
    fn get_value(&self, value: Option<&String>) -> Option<i32> {
        if value.is_none() {
            return None
        } else {
            let value_str = value.unwrap().as_str();
            let value: i32 = i32::from_str(value_str).unwrap();
            return Some(value)
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FieldType {
    SmallText(String),
    LongText(String),
    CheckBox(bool),
    NumberField(i32),
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
