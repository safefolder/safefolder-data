extern crate colored;

use colored::*;
use serde::{Deserialize, Serialize};
use validator::{Validate};
use tr::tr;

use crate::planet::constants;

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct PlanetValidationError {
    pub command: String,
    pub field: String,
    pub error_code: String,
    pub message: String,
}

pub struct CommandImportConfig {
    pub command: String,
}

impl CommandImportConfig {

    pub fn parse_serde(&self, error: &serde_yaml::Error) -> PlanetValidationError {
        let error_str: &str = &*error.to_string();
        let error_fields: Vec<&str> = error_str.split(":").collect();
        let field = error_fields[0];
        let error_type = error_fields[1].trim();
        let error_fields_next: Vec<&str> = error_fields[2].split(",").collect();
        let field_value: &str = error_fields_next[0];
        let mut error: PlanetValidationError = PlanetValidationError{
            command: self.command.to_string(),
            field: String::from(field),
            error_code: String::from(""),
            message: String::from("")
        };
        match error_type {
            constants::SERDE_ERROR_TYPE_INVALID_TYPE => {
                // number: invalid type: string "pepito34", expected u64 at line 3 column 9
                error.error_code = String::from(constants::SERDE_ERROR_TYPE_INVALID_TYPE);
                error.message = tr!(
                    "{command}{sep} Invalid type for field {field} with value: {value}", 
                    field=format!("{}{}{}", String::from("").green(), field.green(), String::from("").green()),
                    value=field_value.green(),
                    command=self.command.blue(),
                    sep=String::from(":").blue()
                );
                return error;
            },
            constants::SERDE_ERROR_TYPE_INVALID_VALUE => {
                error.error_code = String::from(constants::SERDE_ERROR_TYPE_INVALID_VALUE);
                error.message = tr!(
                    "{command}: Invalid value for field \"{field}\"", 
                    field=field,
                    command=self.command.blue()
                );
                return error;
            },
            constants::SERDE_ERROR_TYPE_INVALID_LENGTH => {
                error.error_code = String::from(constants::SERDE_ERROR_TYPE_INVALID_LENGTH);
                error.message = tr!(
                    "{command}: Invalid length for field \"{field}\"", 
                    field=field,
                    command=self.command
                );
                return error;
            },
            constants::SERDE_ERROR_TYPE_UNKOWN_VARIANT => {
                error.error_code = String::from(constants::SERDE_ERROR_TYPE_UNKOWN_VARIANT);
                error.message = tr!(
                    "{command}: Unknown variant for field \"{field}\"", 
                    field=field,
                    value=field_value,
                    command=self.command
                );
                return error;
            },
            constants::SERDE_ERROR_TYPE_UNKNOWN_FIELD => {
                error.error_code = String::from(constants::SERDE_ERROR_TYPE_UNKNOWN_FIELD);
                error.message = tr!(
                    "{command}: Unknown field for \"{field}\"", 
                    field=field,
                    command=self.command
                );
                return error;
            },
            constants::SERDE_ERROR_TYPE_MISSING_FIELD => {
                error.error_code = String::from(constants::SERDE_ERROR_TYPE_MISSING_FIELD);
                error.message = tr!(
                    "{command}: Missing field", 
                    command=self.command
                );
                return error;
            },
            constants::SERDE_ERROR_TYPE_DUPLICATE_FIELD => {
                error.error_code = String::from(constants::SERDE_ERROR_TYPE_DUPLICATE_FIELD);
                error.message = tr!(
                    "{command}: Duplicate field", 
                    command=self.command
                );
                return error;
            },
            _ => {return error},
        }
    }

    pub fn parse_validator(&self) -> Vec<PlanetValidationError> {
        let errors: Vec<PlanetValidationError> = Vec::new();
        // Here we parse the validator errors and inject into PlanetValidationError
        return errors
    }

    pub fn import<T>(&self, response: Result<T, serde_yaml::Error>) -> Result<T, Vec<PlanetValidationError>> {

        match response {
            Ok(_) => {
                let errors: Vec<PlanetValidationError> = self.parse_validator();
                let number_errors = errors.len();
                match number_errors {
                    0 => {
                        println!("no validator errors, return");
                        return Ok(response.unwrap());
                    },
                    _ => {
                        println!("I got validator errors");
                        return Err(errors)
                    }
                }                
            },
            Err(error) => {
                let mut planet_errors: Vec<PlanetValidationError> = Vec::new();
                planet_errors.push(self.parse_serde(&error));
                return Err(planet_errors);
            }
        }

    }

}