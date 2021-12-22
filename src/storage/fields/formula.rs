use std::collections::BTreeMap;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use tr::tr;
use serde_yaml;

use crate::planet::{PlanetError};
use crate::commands::table::config::FieldConfig;
use crate::storage::constants::*;
use crate::functions::{execute_formula, Formula};
use crate::storage::fields::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FormulaField {
    pub config: FieldConfig
}
impl FormulaField {
    pub fn defaults(field_config: &FieldConfig) -> Self {
        let field_config = field_config.clone();
        let field_obj = Self{
            config: field_config
        };
        return field_obj
    }
}
impl FormulaField {
    pub fn update_config_map(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
        field_name_map: &BTreeMap<String, String>,
        field_type_map: &BTreeMap<String, String>,
        db_table: &DbTable,
        table_name: &String,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let mut field_config_map = field_config_map.clone();
        let config = self.config.clone();
        let formula = config.formula;
        if formula.is_some() {
            let formula = formula.unwrap();
            let formula_format = config.formula_format.unwrap();
            let field_type_map = field_type_map.clone();
            let field_name_map = field_name_map.clone();
            let db_table = db_table.clone();
            let table_name = table_name.clone();
            let formula_compiled = Formula::defaults(
                &formula,
                &formula_format,
                None,
                Some(field_type_map),
                Some(field_name_map),
                Some(db_table),
                Some(table_name),
                false,
                None
            )?;
            field_config_map.insert(String::from(FORMULA), formula);
            field_config_map.insert(String::from(FORMULA_FORMAT), formula_format);
            let formula_serialized = serde_yaml::to_string(&formula_compiled).unwrap();
            field_config_map.insert(String::from(FORMULA_COMPILED), formula_serialized);
        }
        return Ok(field_config_map)
    }
    pub fn build_config(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<FieldConfig, PlanetError> {
        let mut config = self.config.clone();
        let formula = field_config_map.get(FORMULA);
        let formula_compiled = field_config_map.get(FORMULA_COMPILED);
        let formula_format = field_config_map.get(FORMULA_FORMAT);
        let mut formula_wrap: Option<String> = None;
        let mut formula_compiled_wrap: Option<String> = None;
        let mut formula_format_wrap: Option<String> = None;
        if formula_compiled.is_some() {
            let formula_compiled = formula_compiled.unwrap().clone();
            let formula = formula.unwrap().clone();
            formula_compiled_wrap = Some(formula_compiled);
            formula_wrap = Some(formula);
            let formula_format = formula_format.unwrap().clone();
            formula_format_wrap = Some(formula_format);
        }
        config.formula = formula_wrap;
        config.formula_format = formula_format_wrap;
        config.formula_compiled = formula_compiled_wrap;
        return Ok(config)
    }
    pub fn validate(&self, 
        data_map: &BTreeMap<String, String>,
        field_config_map: &BTreeMap<String, FieldConfig>,
    ) -> Result<String, PlanetError> {
        let config = self.config.clone();
        let field_config_map = field_config_map.clone();
        let data_map = data_map.clone();
        let formula_compiled_str = config.formula_compiled.clone();
        // execute formula and return result string
        if formula_compiled_str.is_some() {
            let formula_compiled_str = formula_compiled_str.unwrap();
            let formula_compiled: Formula = serde_yaml::from_str(
                formula_compiled_str.as_str()
            ).unwrap();
            let formula_result = execute_formula(&formula_compiled, &data_map, &field_config_map)?;
            return Ok(formula_result);
        } else {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("Formula not found on formula field")),
                )
            );
        }
    }
    pub fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String {
        let field_config = self.config.clone();
        let mut value = value.clone();
        // eprintln!("FormulaField.get_yaml_out :: field_config: {:#?}", field_config.clone());
        // eprintln!("FormulaField.get_yaml_out :: value: {:?}", &value);
        let field_name = field_config.name.unwrap();
        let mut yaml_string = yaml_string.clone();
        let field = &field_name.truecolor(
            YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
        );
        let formula_format = field_config.formula_format.unwrap();
        // eprintln!("FormulaField.get_yaml_out :: formula_format: {:?}", &formula_format);
        if &formula_format == FORMULA_FORMAT_TEXT {
            value = format!("{}", 
                value.truecolor(YAML_COLOR_ORANGE[0], YAML_COLOR_ORANGE[1], YAML_COLOR_ORANGE[2]), 
            );
            // eprintln!("FormulaField.get_yaml_out :: Text : value: {}", &value);
        } else if &formula_format == FORMULA_FORMAT_NUMBER {
            // eprintln!("FormulaField.get_yaml_out :: Number : value: {}", &value);
            value = value.replace("\"", "");
            value = format!("{}", value.truecolor(
                YAML_COLOR_YELLOW[0], YAML_COLOR_YELLOW[1], YAML_COLOR_YELLOW[2]
            ));
            
        } else if &formula_format == FORMULA_FORMAT_CHECK {
            value = value.replace("\"", "");
            if value == String::from("1") {
                value = String::from("true");
            } else if value == String::from("0") {
                value = String::from("false");
            }
            value = format!("{}", 
                value.truecolor(YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]), 
            );
        } else if &formula_format == FORMULA_FORMAT_DATE {
            value = format!("{}", 
                value.truecolor(YAML_COLOR_ORANGE[0], YAML_COLOR_ORANGE[1], YAML_COLOR_ORANGE[2]), 
            );
        }
        // eprintln!("FormulaField.get_yaml_out :: value: {}", &value);
        yaml_string.push_str(format!("  {field}: {value}\n", field=field, value=value).as_str());
        return yaml_string;
    }
}
