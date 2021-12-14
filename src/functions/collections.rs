use std::str::FromStr;
use regex::{Regex, Captures};
use std::{collections::HashMap};
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;

use crate::functions::*;


lazy_static! {
    static ref RE_MIN: Regex = Regex::new(r#"^MIN\((?P<sequence>[\d\s,.-]+)\)|MIN\((?P<sequence_ref>\{[\w\s]+\})\)"#).unwrap();
    static ref RE_MAX: Regex = Regex::new(r#"^MAX\((?P<sequence>[\d\s,.-]+)\)|MAX\((?P<sequence_ref>\{[\w\s]+\})\)"#).unwrap();
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StatOption {
    Min,
    Max,
}

pub trait CollectionStatsFunction {
    fn handle(&mut self, option: StatOption) -> Result<FunctionParse, PlanetError>;
    fn execute(&self, option: StatOption) -> Result<String, PlanetError>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Stats {
    function: Option<FunctionParse>,
    data_map: Option<HashMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>
}
impl Stats {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<HashMap<String, String>>
    ) -> Self {
        return Self{function: function, data_map: data_map, attributes: None};
    }
}
impl CollectionStatsFunction for Stats {
    fn handle(&mut self, option: StatOption) -> Result<FunctionParse, PlanetError> {
        // MIN(1,2,3,4)
        // MIN({Column})
        // MAX(1,2,3,4)
        // MAX({Column})
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr_min = &RE_MIN;
        let expr_max = &RE_MAX;
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
        let sequence_item = attributes[0].clone();
        let mut sequence = sequence_item.get_value(data_map)?;
        let replacement_string: String;
        sequence = sequence.replace("\"", "");
        let mut sequence_list: Vec<f64> = Vec::new();
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
        }
        replacement_string = stat_result.to_string();
        return Ok(replacement_string)
    }
}
