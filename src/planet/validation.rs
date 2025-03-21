extern crate colored;

use colored::*;
use validator::{ValidationErrors, ValidationErrorsKind, ValidationError};
use tr::tr;
use std::collections::{BTreeMap};

use crate::planet::constants;
use crate::planet::PlanetContext;

#[derive(Debug, Clone)]
pub struct PlanetValidationError {
    pub command: String,
    pub column: String,
    pub error_code: String,
    pub message: String,
}

struct ValidationMessageFields {
    column: String,
    value: ColoredString,  
}

pub struct CommandImportConfig<'gb> {
    pub command: String,
    pub planet_context: &'gb PlanetContext<'gb>,
}

impl<'gb> CommandImportConfig<'gb> {

    pub fn parse_serde(&self, error: &serde_yaml::Error) -> PlanetValidationError {
        println!("parse_serde :: error: {:#?}", error);
        let error_str: &str = &*error.to_string();
        println!("parse_serde :: error str: {}", error_str);
        let mut column: &str = "";
        let mut error_type: &str = "";
        let mut field_value: &str = "";
        if error_str.find(":").is_some() {
            let error_fields: Vec<&str> = error_str.split(":").collect();
            println!("parse_serde :: error_fields: {:?}", &error_fields);
            if error_fields.len() == 3 {
                column = error_fields[0];
                error_type = error_fields[1].trim();
                let error_fields_next: Vec<&str> = error_fields[2].split(",").collect();
                field_value = error_fields_next[0];        
            } else {
                // error_fields: ["text_search", " missing field `column_relevance` at line 6 column 12"]
                column = error_fields[0];
                error_type = "";
                field_value = error_fields[1];
            }
        } else {
            // duplicate column `command` at line 2 column 8
            if Some(error_str.find(constants::SERDE_ERROR_TYPE_DUPLICATE_FIELD)).is_some() {
                error_type = constants::SERDE_ERROR_TYPE_DUPLICATE_FIELD;
                let items_error_str: Vec<&str> = error_str.split("`").collect();
                column = items_error_str[1];
            }
        }
        let mut error: PlanetValidationError = PlanetValidationError{
            command: self.command.to_string(),
            column: String::from(column),
            error_code: String::from(""),
            message: String::from("")
        };
        match error_type {
            constants::SERDE_ERROR_TYPE_INVALID_TYPE => {
                // number: invalid type: string "pepito34", expected u64 at line 3 column 9
                error.error_code = String::from(constants::SERDE_ERROR_TYPE_INVALID_TYPE);
                error.message = tr!(
                    "{command}{sep} Invalid type for column {column} with value: {value}", 
                    column=format!("{}{}{}", String::from("").magenta(), column.magenta(), String::from("").magenta()),
                    value=field_value.green(),
                    command=self.command.blue(),
                    sep=String::from(":").blue(),
                );
                return error;
            },
            constants::SERDE_ERROR_TYPE_INVALID_VALUE => {
                error.error_code = String::from(constants::SERDE_ERROR_TYPE_INVALID_VALUE);
                error.message = tr!(
                    "{command}{sep} Invalid value for column {column}", 
                    column=format!("{}{}{}", String::from("\"").magenta(), column.magenta(), String::from("\"").magenta()),
                    command=self.command.blue(),
                    sep=String::from(":").blue(),
                );
                return error;
            },
            constants::SERDE_ERROR_TYPE_INVALID_LENGTH => {
                error.error_code = String::from(constants::SERDE_ERROR_TYPE_INVALID_LENGTH);
                error.message = tr!(
                    "{command}{sep} Invalid length for column {column}", 
                    column=format!("{}{}{}", String::from("\"").magenta(), column.magenta(), String::from("\"").magenta()),
                    command=self.command.blue(),
                    sep=String::from(":").blue(),
                );
                return error;
            },
            constants::SERDE_ERROR_TYPE_UNKOWN_VARIANT => {
                error.error_code = String::from(constants::SERDE_ERROR_TYPE_UNKOWN_VARIANT);
                error.message = tr!(
                    "{command}{sep} Unknown variant for column {column}", 
                    column=format!("{}{}{}", String::from("\"").magenta(), column.magenta(), String::from("\"").magenta()),
                    value=field_value,
                    command=self.command.blue(),
                    sep=String::from(":").blue(),
                );
                return error;
            },
            constants::SERDE_ERROR_TYPE_UNKNOWN_FIELD => {
                error.error_code = String::from(constants::SERDE_ERROR_TYPE_UNKNOWN_FIELD);
                error.message = tr!(
                    "{command}{sep} Unknown column for {column}", 
                    column=format!("{}{}{}", String::from("\"").magenta(), column.magenta(), String::from("\"").magenta()),
                    command=self.command.blue(),
                    sep=String::from(":").blue(),
                );
                return error;
            },
            constants::SERDE_ERROR_TYPE_MISSING_FIELD => {
                error.error_code = String::from(constants::SERDE_ERROR_TYPE_MISSING_FIELD);
                error.message = tr!(
                    "{command}{sep} Missing column", 
                    command=self.command,
                    sep=String::from(":").blue()
                );
                return error;
            },
            constants::SERDE_ERROR_TYPE_DUPLICATE_FIELD => {
                // duplicate column `command` at line 2 column 8
                error.error_code = String::from(constants::SERDE_ERROR_TYPE_DUPLICATE_FIELD);
                error.message = tr!(
                    "{command}{sep}: Duplicate column {column}", 
                    command=self.command.blue(),
                    column=format!("{}{}{}", String::from("\"").magenta(), column.magenta(), String::from("\"").magenta()),
                    sep=String::from(":").blue()
                );
                return error;
            },
            _ => {
                error.error_code = String::from("Error");
                error.message = tr!(
                    "Error on configuration field {column}, {field_value}.", 
                    command=self.command.blue(),
                    column=format!("{}{}{}", String::from("\"").magenta(), column.magenta(), String::from("\"").magenta()),
                    sep=String::from(":").blue(),
                    field_value=field_value
                );

                return error
            },
        }
    }

    fn get_validation_message_items(&self, main_error_field: &str, error_field: &str, error: &ValidationError) -> ValidationMessageFields {
        let message_field: ValidationMessageFields;
        if main_error_field.len() == 0 {
            message_field = ValidationMessageFields{
                column: format!(
                    "{}{}{}", 
                    String::from("\"").magenta(), 
                    error_field.magenta(),
                    String::from("\"").magenta(), 
                ),
                value: error.params.get("equal").unwrap().to_string().green(),
            };
        } else {
            // main_error_field: command
            let error_field_equal = error.params.get("equal");
            if error_field_equal.is_some() {
                message_field = ValidationMessageFields{
                    column: format!(
                        "{}{}{}.{}{}{}", 
                        String::from("\"").magenta(), 
                        main_error_field.magenta(),
                        String::from("\"").magenta(),
                        String::from("\"").magenta(), 
                        error_field.magenta(),
                        String::from("\"").magenta(), 
                    ),
                    value: error.params.get("equal").unwrap().to_string().green(),
                };
            } else {
                // main_error_field: command
                message_field = ValidationMessageFields{
                    column: format!(
                        "{}{}{}", 
                        String::from("\"").magenta(), 
                        main_error_field.magenta(),
                        String::from("\"").magenta(),
                    ),
                    value: String::from("Hola").green(),
                };
            }
        }
        return message_field;
    }

    fn parse_field_validations(&self, 
        command: &String,
        main_error_field: &str,
        error_field: &str,
        errors: Vec<ValidationError>
    ) -> Vec<PlanetValidationError> {
        let mut planet_errors: Vec<PlanetValidationError> = Vec::new();
        for error in errors {
            let mut planet_error: PlanetValidationError = PlanetValidationError {
                command: self.command.to_string(),
                column: String::from(error_field),
                error_code: format!("{}_equal", error.code),
                message: String::from(""),
                };
            let message_fields: ValidationMessageFields = self.get_validation_message_items(
                &main_error_field,
                &error_field,
                &error);
            if error.code == "length" && error.params.contains_key("equal") {
            planet_error.message = tr!(
                "{command}{sep}: {column} has length not equal to {value}",
                command=command.blue(),
                sep=String::from(":").blue(),
                column=message_fields.column,
                value=message_fields.value,
                );
                planet_errors.push(planet_error);
            } else if error.code == "length" && error.params.contains_key("min") {
                    planet_error.message = tr!(
                        "{command}{sep}: {column} has length lower than {value}",
                        command=command.blue(),
                        sep=String::from(":").blue(),
                        column=message_fields.column,
                        value=message_fields.value,
                    );
                    planet_errors.push(planet_error);
            } else if error.code == "length" && error.params.contains_key("max") {
                    planet_error.message = tr!(
                        "{command}{sep}: {column} has length higher than {value}",
                        command=command.blue(),
                        sep=String::from(":").blue(),
                        column=message_fields.column,
                        value=message_fields.value,
                    );
                    planet_errors.push(planet_error);
            } else if error.code == "required" {
                planet_error.message = tr!(
                    "{command}{sep}: {column} is required",
                    command=command.blue(),
                    sep=String::from(":").blue(),
                    column=message_fields.column,
                    );
                    planet_errors.push(planet_error);
            } else if error.code == "contains" {
                planet_error.message = tr!(
                    "{command}{sep}: {column} does not contain {value}.",
                    command=command.blue(),
                    sep=String::from(":").blue(),
                    column=message_fields.column,
                    value=message_fields.value,
                    );
                    planet_errors.push(planet_error);
            } else if error.code == "regex" {
                // [ValidationError { code: "regex", message: None, params: {"value": String("CREATE TABLE")} }]
                planet_error.message = tr!(
                    "{command}{sep}: {column} did not pass formatting validation. Check documentation.",
                    command=command.blue(),
                    sep=String::from(":").blue(),
                    column=message_fields.column,
                );
                planet_errors.push(planet_error);
            } else if error.code == "range" && error.params.contains_key("min") {
                planet_error.message = tr!(
                    "{command}{sep}: {column} value {value} is lower than defined range.",
                    command=command.blue(),
                    sep=String::from(":").blue(),
                    column=message_fields.column,
                );
                planet_errors.push(planet_error);
            } else if error.code == "range" && error.params.contains_key("max") {
                planet_error.message = tr!(
                    "{command}{sep}: {column} value {value} is higher than defined range.",
                    command=command.blue(),
                    sep=String::from(":").blue(),
                    column=message_fields.column,
                );
                planet_errors.push(planet_error);
            } else {
                let value_wrap = error.params.get("value");
                if value_wrap.is_some() {
                    let value = value_wrap.unwrap().to_string();
                    planet_error.message = tr!(
                        "{command}{sep}: {code} :: {value}.",
                        command=command.blue(),
                        sep=String::from(":").blue(),
                        code=error.code,
                        value=value
                    );
                    planet_errors.push(planet_error);
                } else {
                    planet_error.message = tr!(
                        "{command}{sep}: {code}.",
                        command=command.blue(),
                        sep=String::from(":").blue(),
                        code=error.code,
                    );
                    planet_errors.push(planet_error);
                }
            }
        }
        return planet_errors;
    }

    fn parse_struct_validations(&self, 
        command: &String,
        main_error_field: &str, 
        validation_errors: Box<ValidationErrors>
    ) -> Vec<PlanetValidationError> {
        let mut planet_errors: Vec<PlanetValidationError> = Vec::new();
        let list_errors = validation_errors.into_errors();
        for (error_field, error_kind) in list_errors {
            if let ValidationErrorsKind::Field(errors) = error_kind {
                planet_errors = self.parse_field_validations(
                    command, 
                    main_error_field, 
                    error_field, 
                    errors
                );
            }    
        }
        return planet_errors;
    }

    fn parse_list_validations(&self, 
        command: &String,
        main_error_field: &str, 
        errors: BTreeMap<usize, Box<ValidationErrors>>
    ) -> Vec<PlanetValidationError> {
        let mut planet_errors: Vec<PlanetValidationError> = Vec::new();
        for (_, validation_errors) in errors {
            let list_errors = validation_errors.into_errors();
            for (error_field, error_kind) in list_errors {
                if let ValidationErrorsKind::Field(errors) = error_kind {
                    for error in errors {
                        let mut errors_: Vec<ValidationError> = Vec::new();
                        errors_.push(error);
                        let planet_errors_ = self.parse_field_validations(
                            command, 
                            main_error_field, 
                            error_field, 
                            errors_
                        );
                        planet_errors.extend(planet_errors_);
                    }
                }
            }
        }
        return planet_errors;

    }

    pub fn parse_validator(&self, command: &String, errors: ValidationErrors) -> Vec<PlanetValidationError> {
        eprintln!("parse_validator :: command: {}", command);
        let all_errors = errors.into_errors();
        let mut planet_errors: Vec<PlanetValidationError> = Vec::new();
        for (error_field, error_kind) in all_errors {
            if let ValidationErrorsKind::List(errors) = error_kind {
                let planet_errors_: Vec<PlanetValidationError> = self.parse_list_validations(
                    command,
                    error_field, 
                    errors);
                planet_errors.extend(planet_errors_);
            } else if let ValidationErrorsKind::Field(errors) = error_kind {
                let planet_errors_: Vec<PlanetValidationError> = self.parse_field_validations(
                    command, 
                    error_field,
                    "",
                    errors
                );
                planet_errors.extend(planet_errors_);
            } else if let ValidationErrorsKind::Struct(errors) = error_kind {
                let planet_errors_: Vec<PlanetValidationError> = self.parse_struct_validations(
                    command, 
                    error_field, 
                    errors
                );
                planet_errors.extend(planet_errors_);
            }
        }
        return planet_errors;
    }

}