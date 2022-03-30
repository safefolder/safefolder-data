use std::collections::BTreeMap;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;
use regex::{Regex};

use crate::planet::{PlanetError};
use crate::statements::folder::schema::*;
use crate::statements::*;
use crate::storage::constants::*;
use crate::storage::columns::*;

lazy_static! {
    pub static ref RE_STATEMENTS: Regex = Regex::new(r#"(?P<Statement>(?P<StatementHeader>[\w\s]*)\([\s\S][^)]+\);)"#).unwrap();
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatementColumn {
    pub config: ColumnConfig
}
impl StatementColumn {
    pub fn defaults(field_config: &ColumnConfig) -> Self {
        let field_config = field_config.clone();
        let field_obj = Self{
            config: field_config
        };
        return field_obj
    }
}
impl EnvDbStorageColumn for StatementColumn {
    fn create_config(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
        env: &Environment,
        space_database: &SpaceDatabase
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let mut field_config_map = field_config_map.clone();
        let env = env.clone();
        let space_database = space_database.clone();
        let column_config = self.config.clone();
        let statements = column_config.statements;
        if statements.is_some() {
            let statements = statements.unwrap();
            let statement_runner = StatementRunner{
                response_format: StatementResponseFormat::YAML
            };
            let expr = &RE_STATEMENTS;
            let statement_list = expr.captures_iter(&statements);
            for statement_match in statement_list {
                let statement_text = statement_match.name("Statement");
                if statement_text.is_some() {
                    let statement_text = statement_text.unwrap().as_str();
                    let result = statement_runner.call(
                        &env, 
                        Some(space_database.clone()), 
                        &statement_text.to_string(),
                        &StatementCallMode::Compile
                    );
                    if result.is_err() {
                        return Err(PlanetError::new(
                            500, 
                            Some(
                                tr!("Statements syntax error. Check your input and try again.")
                            ),
                        ));
                    }
                }
            }
            field_config_map.insert(STATEMENTS.to_string(), statements);
        }
        return Ok(field_config_map)
    }
    fn get_config(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError> {
        let mut config = self.config.clone();
        let field_config_map = field_config_map.clone();
        let statements = field_config_map.get(STATEMENTS);
        if statements.is_some() {
            let statements = statements.unwrap().clone();
            config.statements = Some(statements);
        }
        return Ok(config)
    }
    fn validate(
        &self, 
        _data: &Vec<String>,
        env: &Environment,
        space_database: &SpaceDatabase
    ) -> Result<Vec<String>, Vec<PlanetError>> {
        // let data = data.clone();
        let env = env.clone();
        let space_database = space_database.clone();
        let config = self.config.clone();
        let statements = config.statements;
        let mut all_errors: Vec<PlanetError> = Vec::new();
        let mut responses: Vec<String> = Vec::new();
        if statements.is_some() {
            let statements = statements.unwrap();
            let statement_runner = StatementRunner{
                response_format: StatementResponseFormat::YAML
            };
            let expr = &RE_STATEMENTS;
            let statement_list = expr.captures_iter(&statements);
            for statement_match in statement_list {
                let statement_text = statement_match.name("Statement");
                if statement_text.is_some() {
                    let statement_text = statement_text.unwrap().as_str();
                    // I need to replace {...} with data_map data
                    let result = statement_runner.call(
                        &env, 
                        Some(space_database.clone()), 
                        &statement_text.to_string(),
                        &StatementCallMode::Run
                    );
                    if result.is_err() {
                        let errors = result.unwrap_err();
                        for error in errors {
                            all_errors.push(error);
                        }
                    } else {
                        let response = result.unwrap();
                        responses.push(response);    
                    }
                }
            }
        }
        if all_errors.len() > 0 {
            return Err(all_errors)
        }
        return Ok(responses)
    }
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String {
        let field_config = self.config.clone();
        let field_name = field_config.name.unwrap();
        let mut yaml_string = yaml_string.clone();
        let field = &field_name.truecolor(
            YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
        );
        let value = format!("{}", value.to_string().truecolor(
            YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
        ));
        let value = format!("{}", value.to_string().truecolor(
            YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
        ));
        yaml_string.push_str(format!("  {field}: {value}\n", field=field, value=value).as_str());
        return yaml_string;
    }
}
