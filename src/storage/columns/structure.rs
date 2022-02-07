use std::collections::BTreeMap;
use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::planet::{PlanetError};
use crate::commands::folder::config::ColumnConfig;
use crate::storage::constants::*;
use crate::storage::columns::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SetColumn {
    pub config: ColumnConfig
}
impl SetColumn {
    pub fn defaults(field_config: &ColumnConfig) -> Self {
        let field_config = field_config.clone();
        let field_obj = Self{
            config: field_config
        };
        return field_obj
    }
}
impl StorageColumn for SetColumn {
    fn update_config_map(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let mut field_config_map = field_config_map.clone();
        let config = self.config.clone();
        let minimum = config.set_minimum;
        let maximum = config.set_maximum;
        if minimum.is_some() {
            let minimum = minimum.unwrap();
            field_config_map.insert(SET_MINIMUM.to_string(), minimum);
        }
        if maximum.is_some() {
            let maximum = maximum.unwrap();
            field_config_map.insert(SET_MAXIMUM.to_string(), maximum);
        }
        field_config_map.insert(IS_SET.to_string(), String::from("true"));
        return Ok(field_config_map)
    }
    fn build_config(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError> {
        let mut config = self.config.clone();
        let minimum = field_config_map.get(SET_MINIMUM);
        let maximum = field_config_map.get(SET_MAXIMUM);
        if minimum.is_some() {
            let minimum = minimum.unwrap();
            config.set_minimum = Some(minimum.clone());
        }
        if maximum.is_some() {
            let maximum = maximum.unwrap();
            config.set_maximum = Some(maximum.clone());
        }
        config.is_set = Some(String::from("true"));
        return Ok(config)
    }
    fn validate(&self, data: &Vec<String>) -> Result<Vec<String>, PlanetError> {
        let data = data.clone();
        // How to validate max and min???
        // Do later
        return Ok(data)
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