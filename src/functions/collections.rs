use std::str::FromStr;
use regex::{Regex, Captures};
use rust_decimal::prelude::ToPrimitive;
use std::{collections::BTreeMap};
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;

use crate::functions::*;


lazy_static! {
    static ref RE_MIN: Regex = Regex::new(r#"^MIN\((?P<sequence>[\d\s,.-]+)\)|MIN\((?P<sequence_ref>\{[\w\s]+\})\)"#).unwrap();
    static ref RE_MAX: Regex = Regex::new(r#"^MAX\((?P<sequence>[\d\s,.-]+)\)|MAX\((?P<sequence_ref>\{[\w\s]+\})\)"#).unwrap();
    static ref RE_AVG: Regex = Regex::new(r#"^AVG\((?P<sequence>[\d\s,.-]+)\)|MAX\((?P<sequence_ref>\{[\w\s]+\})\)"#).unwrap();
    static ref RE_SUM: Regex = Regex::new(r#"^SUM\((?P<sequence>[\d\s,.-]+)\)|MAX\((?P<sequence_ref>\{[\w\s]+\})\)"#).unwrap();
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StatOption {
    Min,
    Max,
    Avg,
    Sum
}

pub trait CollectionStatsFunction {
    fn handle(&mut self, option: StatOption) -> Result<FunctionParse, PlanetError>;
    fn execute(&self, option: StatOption) -> Result<String, PlanetError>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Stats {
    function: Option<FunctionParse>,
    data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
    attributes: Option<Vec<FunctionAttributeItem>>,
    column_config_map: BTreeMap<String, ColumnConfig>
}
impl Stats {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
        column_config_map: &BTreeMap<String, ColumnConfig>
    ) -> Self {
        let column_config_map = column_config_map.clone();
        return Self{
            function: function, 
            data_map: data_map, 
            attributes: None,
            column_config_map: column_config_map
        };
    }
}
impl CollectionStatsFunction for Stats {
    fn handle(&mut self, option: StatOption) -> Result<FunctionParse, PlanetError> {
        // MIN(1,2,3,4)
        // MIN({Column})
        // MAX(1,2,3,4)
        // MAX({Column})
        // AVG({Column})
        // SUM({Column})
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr_min = &RE_MIN;
        let expr_max = &RE_MAX;
        let expr_avg = &RE_AVG;
        let expr_sum = &RE_SUM;
        let mut function = function_parse.clone();
        let data_map_wrap = data_map.clone();
        let (
            function_text_wrap, 
            function_text, 
            compiled_attributes,
            mut function_result,
            data_map,
        ) = prepare_function_parse(function_parse, data_map.clone());
        if function_text_wrap.is_some() {
            match option {
                StatOption::Min => {
                    function.validate = Some(expr_min.is_match(function_text.as_str()));
                },
                StatOption::Max => {
                    function.validate = Some(expr_max.is_match(function_text.as_str()));
                },
                StatOption::Avg => {
                    function.validate = Some(expr_avg.is_match(function_text.as_str()));
                },
                StatOption::Sum => {
                    function.validate = Some(expr_sum.is_match(function_text.as_str()));
                },
            }
            if function.validate.unwrap() {
                let matches: Captures;
                match option {
                    StatOption::Min => {
                        matches = expr_min.captures(function_text.as_str()).unwrap();
                    },
                    StatOption::Max => {
                        matches = expr_max.captures(function_text.as_str()).unwrap();
                    },
                    StatOption::Avg => {
                        matches = expr_max.captures(function_text.as_str()).unwrap();
                    },
                    StatOption::Sum => {
                        matches = expr_max.captures(function_text.as_str()).unwrap();
                    },
                }        
                let attr_sequence = matches.name("sequence");
                let attr_sequence_ref = matches.name("sequence_ref");
                let mut attributes_: Vec<String> = Vec::new();
        
                if attr_sequence.is_some() {
                    let sequence: String = attr_sequence.unwrap().as_str().to_string();
                    attributes_.push(sequence);
                }
                if attr_sequence_ref.is_some() {
                    let sequence_ref: String = attr_sequence_ref.unwrap().as_str().to_string();
                    attributes_.push(sequence_ref);
                }
                function.attributes = Some(attributes_);
            }
        }
        if data_map_wrap.is_some() {
            self.attributes = Some(compiled_attributes);
            self.data_map = Some(data_map);
            function_result.text = Some(self.execute(option)?);
            function.result = Some(function_result.clone());
        }
        return Ok(function)
    }
    fn execute(&self, option: StatOption) -> Result<String, PlanetError> {
        let attributes = self.attributes.clone().unwrap();
        let data_map = &self.data_map.clone().unwrap();
        let column_config_map = self.column_config_map.clone();
        let result = validate_collection_stats(
            &attributes,
            &column_config_map
        );
        if result.is_err() {
            let error = result.unwrap_err();
            return Err(error)
        }
        let tuple = result.unwrap();
        let column_type = tuple.0;
        let is_collection_attribute = tuple.2;
        let mut sequence_list: Vec<f64> = Vec::new();
        let replacement_string: String;
        if is_collection_attribute {
            let column_type = column_type.as_str();
            if column_type == COLUMN_TYPE_NUMBER || column_type == COLUMN_TYPE_GENERATE_NUMBER ||
                column_type == COLUMN_TYPE_RATING {
                let attribute = &attributes[0];
                let result = attribute.get_values(data_map, &column_config_map);
                if result.is_err() {
                    let error = result.unwrap_err();
                    return Err(error)
                }
                let value_list = result.unwrap();
                for item in value_list {
                    let has_dot = item.clone().find(".");
                    let mut item_string = item.trim().to_string();
                    if has_dot.is_none() {
                        item_string.push_str(".0");
                    }
                    let item_number: f64 = FromStr::from_str(item_string.as_str()).unwrap();
                    sequence_list.push(item_number);
                }
            }
        } else {
            let sequence_item = attributes[0].clone();
            let mut sequence = sequence_item.get_value(data_map, None, &column_config_map)?;
            sequence = sequence.replace("\"", "");
            let sequence_str_list: Vec<&str> = sequence.split(",").collect();
            for mut item in sequence_str_list {
                let has_dot = item.clone().find(".");
                item = item.trim();
                let mut item_string: String = item.to_string();
                if has_dot.is_none() {
                    item_string.push_str(".0");
                }
                let item_number: f64 = FromStr::from_str(item_string.as_str()).unwrap();
                sequence_list.push(item_number);
            }
        }
        let stat_result: f64;
        match option {
            StatOption::Min => {
                let mut min: f64 = sequence_list[0];
                for item in sequence_list {
                    if item < min {
                        min = item
                    }
                }
                stat_result = min;
            },
            StatOption::Max => {
                let mut max: f64 = sequence_list[0];
                for item in sequence_list {
                    if item > max {
                        max = item
                    }
                }                
                stat_result = max;
            },
            StatOption::Avg => {
                let mut sum: f64 = 0.0;
                for item in sequence_list.clone() {
                    sum += item;
                }
                let number_items = sequence_list.len();
                let number_items = number_items.to_f64().unwrap();
                let avg = sum/number_items;
                stat_result = avg;
            },
            StatOption::Sum => {
                let mut sum: f64 = 0.0;
                for item in sequence_list {
                    sum += item;
                }
                stat_result = sum;
            }
        }
        replacement_string = stat_result.to_string();
        return Ok(replacement_string)
    }
}
