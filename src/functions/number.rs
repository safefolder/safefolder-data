use std::str::FromStr;
use regex::{Regex, Captures};
use std::{collections::HashMap};
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;
use math::round;

use crate::functions::*;
// use crate::functions::constants::*;
// use crate::functions::Function;


lazy_static! {
    static ref RE_CEILING: Regex = Regex::new(r#"CEILING\([\s\n\t]{0,}((?P<number>[+-]?[0-9]+\.?[0-9]*|\.[0-9]+)|(?P<number_ref>\{[\w\s]+\}))[\s\n\t]{0,},[\s\n\t]{0,}(?P<significance>[+-]?[0-9]+\.?[0-9]*|\.[0-9]+)[\s\n\t]{0,}\)"#).unwrap();
    static ref RE_FLOOR: Regex = Regex::new(r#"FLOOR\([\s\n\t]{0,}((?P<number>[+-]?[0-9]+\.?[0-9]*|\.[0-9]+)|(?P<number_ref>\{[\w\s]+\}))[\s\n\t]{0,},[\s\n\t]{0,}(?P<significance>[+-]?[0-9]+\.?[0-9]*|\.[0-9]+)[\s\n\t]{0,}\)"#).unwrap();
    static ref RE_COUNT: Regex = Regex::new(r#"COUNT\([\s\n\t]{0,}(?P<attrs>[\w\W\s\n\t]{0,})[\s\n\t]{0,}\)"#).unwrap();
    static ref RE_COUNTA: Regex = Regex::new(r#"COUNTA\([\s\n\t]{0,}(?P<attrs>[\w\W\s\n\t]{0,})[\s\n\t]{0,}\)"#).unwrap();
    static ref RE_COUNTALL: Regex = Regex::new(r#"COUNTALL\([\s\n\t]{0,}(?P<attrs>[\w\W\s\n\t]{0,})[\s\n\t]{0,}\)"#).unwrap();
    static ref RE_EVEN: Regex = Regex::new(r#"EVEN\([\s\n\t]{0,}(?P<number>[+-]?[0-9]+\.?[0-9]*|\.[0-9]+)[\s\n\t]{0,}\)|EVEN\([\s\n\t]{0,}(?P<number_ref>\{[\w\s]+\})[\s\n\t]{0,}\)"#).unwrap();
    static ref RE_EXP: Regex = Regex::new(r#"EXP\([\s\n\t]{0,}(?P<number>[+-]?[0-9]+\.?[0-9]*|\.[0-9]+)[\s\n\t]{0,}\)|EXP\([\s\n\t]{0,}(?P<number_ref>\{[\w\s]+\})[\s\n\t]{0,}\)"#).unwrap();
    static ref RE_INT: Regex = Regex::new(r#"INT\([\s\n\t]{0,}(?P<number>[+-]?[0-9]+\.?[0-9]*|\.[0-9]+)[\s\n\t]{0,}\)|INT\([\s\n\t]{0,}(?P<number_ref>\{[\w\s]+\})[\s\n\t]{0,}\)"#).unwrap();
    static ref RE_LOG: Regex = Regex::new(r#"LOG\([\s\n\t]{0,}(?P<number>[+-]?[0-9]+\.?[0-9]*|\.[0-9]+)[\n\s\t]{0,},{0,}[\n\s\t]{0,}(?P<base>\d+){0,}[\s\n\t]{0,}\)|LOG\([\s\n\t]{0,}(?P<number_ref>\{[\w\s]+\})[\n\s\t]{0,},{0,}[\n\s\t]{0,}(?P<base_ref>\d+){0,}[\s\n\t]{0,}\)"#).unwrap();
    static ref RE_MOD: Regex = Regex::new(r#"MOD\([\s\n\t]{0,}(?P<number>[+-]?[0-9]+\.?[0-9]*|\.[0-9]+)[\n\s\t]{0,},[\n\s\t]{0,}(?P<divisor>\d+){0,}[\s\n\t]{0,}\)|MOD\([\s\n\t]{0,}(?P<number_ref>\{[\w\s]+\})[\n\s\t]{0,},[\n\s\t]{0,}(?P<divisor_ref>\d+){0,}[\s\n\t]{0,}\)"#).unwrap();
    static ref RE_POWER: Regex = Regex::new(r#"POWER\([\s\n\t]{0,}(?P<number>[+-]?[0-9]+\.?[0-9]*|\.[0-9]+)[\n\s\t]{0,},[\n\s\t]{0,}(?P<power>[+-]?[0-9]+\.?[0-9]*|\.[0-9]+){0,}[\s\n\t]{0,}\)|POWER\([\s\n\t]{0,}(?P<number_ref>\{[\w\s]+\})[\n\s\t]{0,},[\n\s\t]{0,}(?P<power_ref>[+-]?[0-9]+\.?[0-9]*|\.[0-9]+){0,}[\s\n\t]{0,}\)"#).unwrap();
    static ref RE_ROUND: Regex = Regex::new(r#"ROUND\([\s\n\t]{0,}(?P<number>[+-]?[0-9]+\.?[0-9]*|\.[0-9]+)[\n\s\t]{0,},[\n\s\t]{0,}(?P<digits>\d+){0,}[\s\n\t]{0,}\)|ROUND\([\s\n\t]{0,}(?P<number_ref>\{[\w\s]+\})[\n\s\t]{0,},[\n\s\t]{0,}(?P<digits_ref>\d+){0,}[\s\n\t]{0,}\)"#).unwrap();
    static ref RE_ROUND_UP: Regex = Regex::new(r#"ROUNDUP\([\s\n\t]{0,}(?P<number>[+-]?[0-9]+\.?[0-9]*|\.[0-9]+)[\n\s\t]{0,},[\n\s\t]{0,}(?P<digits>\d+){0,}[\s\n\t]{0,}\)|ROUNDUP\([\s\n\t]{0,}(?P<number_ref>\{[\w\s]+\})[\n\s\t]{0,},[\n\s\t]{0,}(?P<digits_ref>\d+){0,}[\s\n\t]{0,}\)"#).unwrap();
    static ref RE_ROUND_DOWN: Regex = Regex::new(r#"ROUNDDOWN\([\s\n\t]{0,}(?P<number>[+-]?[0-9]+\.?[0-9]*|\.[0-9]+)[\n\s\t]{0,},[\n\s\t]{0,}(?P<digits>\d+){0,}[\s\n\t]{0,}\)|ROUNDDOWN\([\s\n\t]{0,}(?P<number_ref>\{[\w\s]+\})[\n\s\t]{0,},[\n\s\t]{0,}(?P<digits_ref>\d+){0,}[\s\n\t]{0,}\)"#).unwrap();
    static ref RE_SQRT: Regex = Regex::new(r#"SQRT\([\s\n\t]{0,}(?P<number>[+-]?[0-9]+\.?[0-9]*|\.[0-9]+)[\s\n\t]{0,}\)|SQRT\([\s\n\t]{0,}(?P<number_ref>\{[\w\s]+\})[\s\n\t]{0,}\)"#).unwrap();
    static ref RE_VALUE: Regex = Regex::new(r#"VALUE\([\s\n\t]{0,}(?P<text>"[\w\d,.{0,}\$€{0,}]+")[\s\n\t]{0,}\)|VALUE\([\s\n\t]{0,}(?P<text_ref>\{[\w\s]+\})[\s\n\t]{0,}\)"#).unwrap();
    static ref RE_BOOLEAN: Regex = Regex::new(r#"TRUE\(\)|FALSE\(\)|TRUE|FALSE"#).unwrap();
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RoundOption {
    Basic,
    Up,
    Down,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum BooleanOption {
    True,
    False,
}

pub trait NumberFunction {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError>;
    fn execute(&self) -> Result<String, PlanetError>;
}
pub trait RoundNumberFunction {
    fn handle(&mut self, option: RoundOption) -> Result<FunctionParse, PlanetError>;
    fn execute(&self, option: RoundOption) -> Result<String, PlanetError>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ceiling {
    function: Option<FunctionParse>,
    data_map: Option<HashMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>
}
impl Ceiling {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<HashMap<String, String>>
    ) -> Self {
        return Self{function: function, data_map: data_map, attributes: None};
    }
}
impl NumberFunction for Ceiling {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_CEILING;
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
            function.validate = Some(expr.is_match(function_text.as_str()));
            if function.validate.unwrap() {
                let matches = &expr.captures(function_text.as_str()).unwrap();
                let attr_number = matches.name("number");
                let attr_number_ref = matches.name("number_ref");
                let attr_significance = matches.name("significance");
                let mut attributes_: Vec<String> = Vec::new();
                if attr_number.is_some() {
                    let number = attr_number.unwrap();
                    let number_string = number.as_str().to_string();
                    attributes_.push(number_string);
                } else if attr_number_ref.is_some() {
                    let number_ref_string = attr_number_ref.unwrap().as_str().to_string();
                    attributes_.push(number_ref_string);
                }
                if attr_significance.is_some() {
                    let significance = attr_significance.unwrap();
                    let significance_string = significance.as_str().to_string();
                    attributes_.push(significance_string);
                }
                function.attributes = Some(attributes_);
            }
        }
        if data_map_wrap.is_some() {
            self.attributes = Some(compiled_attributes);
            self.data_map = Some(data_map);
            function_result.text = Some(self.execute()?);
            function.result = Some(function_result.clone());
        }
        return Ok(function)
    }
    fn execute(&self) -> Result<String, PlanetError> {
        let attributes = self.attributes.clone().unwrap();
        let data_map = &self.data_map.clone().unwrap();
        let attribute_item = attributes[0].clone();
        let reference_value_wrap = attribute_item.reference_value;
        let significance_string = attributes[1].clone().value.unwrap_or_default();
        let mut significance: i8 = FromStr::from_str(&significance_string.as_str()).unwrap();
        significance = significance - 1;
        let is_reference = attribute_item.is_reference;
        let number: f64;
        if is_reference {
            let reference_value = reference_value_wrap.unwrap();
            let function_attr = FunctionAttributeNew::defaults(
                &reference_value, Some(true), Some(true)
            );
            let replaced_string = function_attr.replace(data_map).item_processed.unwrap();
            number = FromStr::from_str(replaced_string.as_str()).unwrap();
        } else {
            let number_str = attribute_item.value.unwrap_or_default().clone();
            let number_str = number_str.as_str();
            number = FromStr::from_str(number_str).unwrap();
        }
        let number = round::ceil(number, significance);
        let result = number.to_string();
        return Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Floor {
    function: Option<FunctionParse>,
    data_map: Option<HashMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>
}
impl Floor {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<HashMap<String, String>>
    ) -> Self {
        return Self{function: function, data_map: data_map, attributes: None};
    }
}
impl NumberFunction for Floor {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // FLOOR(number, significance)
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_FLOOR;
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
            function.validate = Some(expr.is_match(function_text.as_str()));
            if function.validate.unwrap() {
                let matches = &expr.captures(function_text.as_str()).unwrap();
                let attr_number = matches.name("number");
                let attr_number_ref = matches.name("number_ref");
                let attr_significance = matches.name("significance");
                let mut attributes_: Vec<String> = Vec::new();
                if attr_number.is_some() {
                    let number = attr_number.unwrap();
                    let number_string = number.as_str().to_string();
                    attributes_.push(number_string);
                } else if attr_number_ref.is_some() {
                    let number_ref_string = attr_number_ref.unwrap().as_str().to_string();
                    attributes_.push(number_ref_string);
                }
                if attr_significance.is_some() {
                    let significance = attr_significance.unwrap();
                    let significance_string = significance.as_str().to_string();
                    attributes_.push(significance_string);
                }
                function.attributes = Some(attributes_);
            }
        }
        if data_map_wrap.is_some() {
            self.attributes = Some(compiled_attributes);
            self.data_map = Some(data_map);
            function_result.text = Some(self.execute()?);
            function.result = Some(function_result.clone());
        }
        return Ok(function)
    }
    fn execute(&self) -> Result<String, PlanetError> {
        let attributes = self.attributes.clone().unwrap();
        let data_map = &self.data_map.clone().unwrap();
        let attribute_item = attributes[0].clone();
        let reference_value_wrap = attribute_item.reference_value;
        let significance_string = attributes[1].clone().value.unwrap_or_default();
        let mut significance: i8 = FromStr::from_str(&significance_string.as_str()).unwrap();
        significance = significance - 1;
        let is_reference = attribute_item.is_reference;
        let number: f64;
        if is_reference {
            let reference_value = reference_value_wrap.unwrap();
            let function_attr = FunctionAttributeNew::defaults(
                &reference_value, Some(true), Some(true)
            );
            let replaced_string = function_attr.replace(data_map).item_processed.unwrap();
            number = FromStr::from_str(replaced_string.as_str()).unwrap();
        } else {
            let number_str = attribute_item.value.unwrap_or_default().clone();
            let number_str = number_str.as_str();
            number = FromStr::from_str(number_str).unwrap();
        }
        let number = round::floor(number, significance);
        return Ok(number.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Count {
    function: Option<FunctionParse>,
    data_map: Option<HashMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>
}
impl Count {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<HashMap<String, String>>
    ) -> Self {
        return Self{function: function, data_map: data_map, attributes: None};
    }
}
impl NumberFunction for Count {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // COUNT(value1, value2, ...))
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_COUNT;
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
            function.validate = Some(expr.is_match(function_text.as_str()));
            if function.validate.unwrap() {
                let matches = &expr.captures(function_text.as_str()).unwrap();
                let attrs = matches.name("attrs");
                if attrs.is_some() {
                    let attrs = attrs.unwrap().as_str().to_string();
                    let mut attributes_: Vec<String> = Vec::new();
                    attributes_.push(attrs);
                    function.attributes = Some(attributes_);
                }
            }
        }
        if data_map_wrap.is_some() {
            self.attributes = Some(compiled_attributes);
            self.data_map = Some(data_map);
            function_result.text = Some(self.execute()?);
            function.result = Some(function_result.clone());
        }
        return Ok(function)
    }
    fn execute(&self) -> Result<String, PlanetError> {
        let attributes = self.attributes.clone().unwrap();
        let data_map = &self.data_map.clone().unwrap();
        let attribute_item = attributes[0].clone();
        let attrs = attribute_item.value.unwrap_or_default();
        let items: Vec<&str> = attrs.split(",").collect();
        let mut number_items: Vec<String> = Vec::new();
        for mut item in items {
            item = item.trim();
            let is_string = item.to_string().find("\"");
            let is_ref = item.to_string().find("{");
            if is_ref.is_some() {
                let item_ = &item.trim().to_string();
                let function_attr = FunctionAttribute::defaults(
                    item_, 
                    Some(true)
                );
                let result = function_attr.replace(data_map.clone());
                let result = result.item_processed.clone();
                let result = result.unwrap();
                let result = result.as_str();
                let is_string = result.to_string().find("\"");
                if is_string.is_none() {
                    number_items.push(result.to_string());
                }
            } else {
                if is_string.is_none() {
                    number_items.push(item.to_string());
                }
            }
        }
        let count = number_items.len();
        return Ok(count.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CountA {
    function: Option<FunctionParse>,
    data_map: Option<HashMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>
}
impl CountA {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<HashMap<String, String>>
    ) -> Self {
        return Self{function: function, data_map: data_map, attributes: None};
    }
}
impl NumberFunction for CountA {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // COUNT(value1, value2, ...))
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_COUNTA;
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
            function.validate = Some(expr.is_match(function_text.as_str()));
            if function.validate.unwrap() {
                let matches = &expr.captures(function_text.as_str()).unwrap();
                let attrs = matches.name("attrs");
                if attrs.is_some() {
                    let attrs = attrs.unwrap().as_str().to_string();
                    let mut attributes_: Vec<String> = Vec::new();
                    attributes_.push(attrs);
                    function.attributes = Some(attributes_);
                }
            }
        }
        if data_map_wrap.is_some() {
            self.attributes = Some(compiled_attributes);
            self.data_map = Some(data_map);
            function_result.text = Some(self.execute()?);
            function.result = Some(function_result.clone());
        }
        return Ok(function)
    }
    fn execute(&self) -> Result<String, PlanetError> {
        let attributes = self.attributes.clone().unwrap();
        let data_map = &self.data_map.clone().unwrap();
        let attribute_item = attributes[0].clone();
        let attrs = attribute_item.value.unwrap_or_default();
        let items: Vec<&str> = attrs.split(",").collect();
        let mut number_items: Vec<String> = Vec::new();
        for mut item in items {
            item = item.trim();
            let is_string = item.to_string().find("\"");
            let is_null = item.to_lowercase() == "null";
            let is_ref = item.to_string().find("{");
            if is_ref.is_some() {
                let item_ = &item.trim().to_string();
                let function_attr = FunctionAttribute::defaults(
                    item_, 
                    Some(true)
                );
                let result = function_attr.replace(data_map.clone());
                let result = result.item_processed.clone();
                let result = result.unwrap();
                let result = result.as_str();
                let is_string = result.to_string().find("\"");
                if is_string.is_some() {
                    if item != "\"\"" {
                        number_items.push(result.to_string());
                    }
                } else {
                    if is_null == false {
                        number_items.push(item.to_string());
                    }
                }
            } else {
                if is_string.is_some() {
                    if item != "\"\"" {
                        number_items.push(item.to_string());
                    }
                } else {
                    if is_null == false {
                        number_items.push(item.to_string());
                    }
                }
            }
        }
        let count = number_items.len();
        return Ok(count.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CountAll {
    function: Option<FunctionParse>,
    data_map: Option<HashMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>
}
impl CountAll {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<HashMap<String, String>>
    ) -> Self {
        return Self{function: function, data_map: data_map, attributes: None};
    }
}
impl NumberFunction for CountAll {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // COUNT(value1, value2, ...))
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_COUNTALL;
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
            function.validate = Some(expr.is_match(function_text.as_str()));
            if function.validate.unwrap() {
                let matches = &expr.captures(function_text.as_str()).unwrap();
                let attrs = matches.name("attrs");
                if attrs.is_some() {
                    let attrs = attrs.unwrap().as_str().to_string();
                    let mut attributes_: Vec<String> = Vec::new();
                    attributes_.push(attrs);
                    function.attributes = Some(attributes_);
                }
            }
        }
        if data_map_wrap.is_some() {
            self.attributes = Some(compiled_attributes);
            self.data_map = Some(data_map);
            function_result.text = Some(self.execute()?);
            function.result = Some(function_result.clone());
        }
        return Ok(function)
    }
    fn execute(&self) -> Result<String, PlanetError> {
        let attributes = self.attributes.clone().unwrap();
        // let data_map = &self.data_map.clone().unwrap();
        let attribute_item = attributes[0].clone();
        let attrs = attribute_item.value.unwrap_or_default();
        let items: Vec<&str> = attrs.split(",").collect();
        let mut number_items: Vec<String> = Vec::new();
        for mut item in items {
            item = item.trim();
            number_items.push(item.to_string());
        }
        let count = number_items.len();
        return Ok(count.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Even {
    function: Option<FunctionParse>,
    data_map: Option<HashMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>
}
impl Even {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<HashMap<String, String>>
    ) -> Self {
        return Self{function: function, data_map: data_map, attributes: None};
    }
}
impl NumberFunction for Even {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // EVEN(number)
        // EVEN(1.5) => 2
        // EVEN(3) => 4
        // EVEN(2) => 2
        // EVEN(-1) => -2
        // EVEN({Column}) => 2
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_EVEN;
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
            function.validate = Some(expr.is_match(function_text.as_str()));
            if function.validate.unwrap() {
                let matches = &expr.captures(function_text.as_str()).unwrap();
                let attr_number = matches.name("number");
                let attr_number_ref = matches.name("number_ref");
                let mut attributes_: Vec<String> = Vec::new();
                if attr_number.is_some() {
                    let number_string = attr_number.unwrap().as_str().to_string();
                    attributes_.push(number_string);
                } else if attr_number_ref.is_some() {
                    let number_ref = attr_number_ref.unwrap().as_str().to_string();
                    attributes_.push(number_ref);
                }
                function.attributes = Some(attributes_);
            }
        }
        if data_map_wrap.is_some() {
            self.attributes = Some(compiled_attributes);
            self.data_map = Some(data_map);
            function_result.text = Some(self.execute()?);
            function.result = Some(function_result.clone());
        }
        return Ok(function)
    }
    fn execute(&self) -> Result<String, PlanetError> {
        let attributes = self.attributes.clone().unwrap();
        let data_map = &self.data_map.clone().unwrap();
        let attribute_item = attributes[0].clone();
        let is_reference = attribute_item.is_reference;
        let number: f64;
        let mut rounded_int: i32;
        if is_reference {
            let number_ref = attribute_item.reference_value.unwrap_or_default();
            let function_attr = FunctionAttribute::defaults(
                &number_ref, 
                Some(true)
            );
            let result = function_attr.replace(data_map.clone());
            let result = result.item_processed.clone();
            number = FromStr::from_str(result.unwrap().as_str()).unwrap();
        } else {
            let number_string = attribute_item.value.unwrap_or_default();
            number = FromStr::from_str(number_string.as_str()).unwrap();
        }
        let rounded = number.round();
        rounded_int = FromStr::from_str(rounded.to_string().as_str()).unwrap();
        let is_even = rounded_int%2 == 0;
        if is_even == false && rounded_int > 0 {
            rounded_int += 1;
        } else if is_even == true && rounded_int < 0 {
            rounded_int -= 1;
        }
        return Ok(rounded_int.to_string())
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Exp {
    function: Option<FunctionParse>,
    data_map: Option<HashMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>
}
impl Exp {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<HashMap<String, String>>
    ) -> Self {
        return Self{function: function, data_map: data_map, attributes: None};
    }
}
impl NumberFunction for Exp {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // EXP(number)
        // EXP(1) => 2.71828183
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_EXP;
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
            function.validate = Some(expr.is_match(function_text.as_str()));
            if function.validate.unwrap() {
                let matches = &expr.captures(function_text.as_str()).unwrap();
                let attr_number = matches.name("number");
                let attr_number_ref = matches.name("number_ref");
                let mut attributes_: Vec<String> = Vec::new();
                if attr_number.is_some() {
                    let number_string = attr_number.unwrap().as_str().to_string();
                    attributes_.push(number_string);
                } else if attr_number_ref.is_some() {
                    let number_ref = attr_number_ref.unwrap().as_str().to_string();
                    attributes_.push(number_ref);
                }
                function.attributes = Some(attributes_);
            }
        }
        if data_map_wrap.is_some() {
            self.attributes = Some(compiled_attributes);
            self.data_map = Some(data_map);
            function_result.text = Some(self.execute()?);
            function.result = Some(function_result.clone());
        }
        return Ok(function)
    }
    fn execute(&self) -> Result<String, PlanetError> {
        let attributes = self.attributes.clone().unwrap();
        let data_map = &self.data_map.clone().unwrap();
        let attribute_item = attributes[0].clone();
        let is_reference = attribute_item.is_reference;
        let number: f64;
        let number_result: f64;
        if is_reference {
            let number_ref = attribute_item.reference_value.unwrap_or_default();
            let function_attr = FunctionAttribute::defaults(
                &number_ref, 
                Some(true)
            );
            let result = function_attr.replace(data_map.clone());
            let result_string = result.item_processed.clone().unwrap_or_default();
            number = FromStr::from_str(result_string.as_str()).unwrap();
        } else {
            let number_string = attribute_item.value.unwrap_or_default();
            number = FromStr::from_str(number_string.as_str()).unwrap();
        }
        number_result = number.exp();
        return Ok(number_result.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Int {
    function: Option<FunctionParse>,
    data_map: Option<HashMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>
}
impl Int {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<HashMap<String, String>>
    ) -> Self {
        return Self{function: function, data_map: data_map, attributes: None};
    }
}
impl NumberFunction for Int {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // INT(number)
        // INT(8.9) => 8
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_INT;
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
            function.validate = Some(expr.is_match(function_text.as_str()));
            if function.validate.unwrap() {
                let matches = &expr.captures(function_text.as_str()).unwrap();
                let attr_number = matches.name("number");
                let attr_number_ref = matches.name("number_ref");
                let mut attributes_: Vec<String> = Vec::new();
                if attr_number.is_some() {
                    let number_string = attr_number.unwrap().as_str().to_string();
                    attributes_.push(number_string);
                } else if attr_number_ref.is_some() {
                    let number_ref = attr_number_ref.unwrap().as_str().to_string();
                    attributes_.push(number_ref);
                }
                function.attributes = Some(attributes_);
            }
        }
        if data_map_wrap.is_some() {
            self.attributes = Some(compiled_attributes);
            self.data_map = Some(data_map);
            function_result.text = Some(self.execute()?);
            function.result = Some(function_result.clone());
        }
        return Ok(function)
    }
    fn execute(&self) -> Result<String, PlanetError> {
        let attributes = self.attributes.clone().unwrap();
        let data_map = &self.data_map.clone().unwrap();
        let attribute_item = attributes[0].clone();
        let is_reference = attribute_item.is_reference;
        let mut number: f64;
        if is_reference {
            let number_ref = attribute_item.reference_value.unwrap_or_default();
            let function_attr = FunctionAttribute::defaults(
                &number_ref, 
                Some(true)
            );
            let result = function_attr.replace(data_map.clone());
            let result_string = result.item_processed.clone().unwrap_or_default();
            number = FromStr::from_str(result_string.as_str()).unwrap();
        } else {
            let number_string = attribute_item.value.unwrap_or_default();
            number = FromStr::from_str(number_string.as_str()).unwrap();
        }
        number = number.trunc();
        let number_str = number.to_string();
        let number_str = number_str.as_str();
        let number_result: isize = FromStr::from_str(number_str).unwrap();
        return Ok(number_result.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Log {
    function: Option<FunctionParse>,
    data_map: Option<HashMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>
}
impl Log {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<HashMap<String, String>>
    ) -> Self {
        return Self{function: function, data_map: data_map, attributes: None};
    }
}
impl NumberFunction for Log {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // LOG(number)
        // LOG(number, [base])
        // LOG(10) = 1
        // LOG(8, 2) = 3
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_LOG;
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
            function.validate = Some(expr.is_match(function_text.as_str()));
            if function.validate.unwrap() {
                let matches = &expr.captures(function_text.as_str()).unwrap();
                let attr_number = matches.name("number");
                let attr_number_ref = matches.name("number_ref");
                let attr_base = matches.name("base");
                let attr_base_ref = matches.name("base_ref");
                let mut attributes_: Vec<String> = Vec::new();
                if attr_number.is_some() {
                    let number_string = attr_number.unwrap();
                    let number_string = number_string.as_str().to_string();
                    attributes_.push(number_string);
                } else if attr_number_ref.is_some() {
                    let number_ref = attr_number_ref.unwrap().as_str().to_string();
                    attributes_.push(number_ref);
                }
                if attr_base.is_some() {
                    let base = attr_base.unwrap().as_str();
                    let base: usize = FromStr::from_str(base).unwrap();
                    attributes_.push(base.to_string());
                }
                if attr_base_ref.is_some() {
                    let base = attr_base_ref.unwrap().as_str();
                    let base_ref: usize = FromStr::from_str(base).unwrap();
                    attributes_.push(base_ref.to_string());
                }
                function.attributes = Some(attributes_);
            }
        }
        if data_map_wrap.is_some() {
            self.attributes = Some(compiled_attributes);
            self.data_map = Some(data_map);
            function_result.text = Some(self.execute()?);
            function.result = Some(function_result.clone());
        }
        return Ok(function)
    }
    fn execute(&self) -> Result<String, PlanetError> {
        let attributes = self.attributes.clone().unwrap();
        let data_map = &self.data_map.clone().unwrap();
        let attribute_item = attributes[0].clone();
        let base_item = attributes[1].clone();
        let is_reference = attribute_item.is_reference;
        let is_base_reference = base_item.is_reference;
        let mut number: f64;
        let mut base: f64 = 10.0;
        if is_reference {
            let number_ref = attribute_item.reference_value.unwrap();
            let function_attr = FunctionAttribute::defaults(
                &number_ref, 
                Some(true)
            );
            let result = function_attr.replace(data_map.clone());
            let result = result.item_processed.clone();
            number = FromStr::from_str(result.unwrap().as_str()).unwrap();
        } else {
            let number_string = attribute_item.value.unwrap_or_default();
            number = FromStr::from_str(number_string.as_str()).unwrap();
        }
        if is_base_reference {
            let base_ref = base_item.reference_value.unwrap();
            let function_attr = FunctionAttribute::defaults(
                &base_ref, 
                Some(true)
            );
            let result = function_attr.replace(data_map.clone());
            let result = result.item_processed.clone();
            base = FromStr::from_str(result.unwrap().as_str()).unwrap();
        } else {
            if base_item.value.is_some() {
                let base_string = base_item.value.unwrap();
                base = FromStr::from_str(base_string.as_str()).unwrap();
            }
        }
        number = number.log(base);
        let number_str = number.to_string();
        let number_str = number_str.as_str();
        let number_result: isize = FromStr::from_str(number_str).unwrap();
        return Ok(number_result.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mod {
    function: Option<FunctionParse>,
    data_map: Option<HashMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>
}
impl Mod {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<HashMap<String, String>>
    ) -> Self {
        return Self{function: function, data_map: data_map, attributes: None};
    }
}
impl NumberFunction for Mod {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // MOD(number, divisor)
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_MOD;
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
            function.validate = Some(expr.is_match(function_text.as_str()));
            if function.validate.unwrap() {
                let matches = &expr.captures(function_text.as_str()).unwrap();
                let attr_number = matches.name("number");
                let attr_number_ref = matches.name("number_ref");
                let attr_divisor = matches.name("divisor");
                let attr_divisor_ref = matches.name("divisor_ref");
                let mut attributes_: Vec<String> = Vec::new();
                if attr_number.is_some() {
                    let number_string = attr_number.unwrap().as_str().to_string();
                    attributes_.push(number_string);
                } else if attr_number_ref.is_some() {
                    let number_ref = attr_number_ref.unwrap().as_str().to_string();
                    attributes_.push(number_ref);
                }
                if attr_divisor.is_some() {
                    let divisor = attr_divisor.unwrap().as_str();
                    attributes_.push(divisor.to_string());
                }
                if attr_divisor_ref.is_some() {
                    let divisor = attr_divisor_ref.unwrap().as_str();
                    attributes_.push(divisor.to_string());
                }
                function.attributes = Some(attributes_);
            }
        }
        if data_map_wrap.is_some() {
            self.attributes = Some(compiled_attributes);
            self.data_map = Some(data_map);
            function_result.text = Some(self.execute()?);
            function.result = Some(function_result.clone());
        }
        return Ok(function)
    }
    fn execute(&self) -> Result<String, PlanetError> {
        let attributes = self.attributes.clone().unwrap();
        let data_map = &self.data_map.clone().unwrap();
        let attribute_item = attributes[0].clone();
        let divisor_item = attributes[1].clone();
        let is_reference = attribute_item.is_reference;
        let is_divisor_reference = divisor_item.is_reference;
        let mut number: f64;
        let mut divisor: f64 = 10.0;
        if is_reference {
            let number_ref = attribute_item.reference_value.unwrap_or_default();
            let function_attr = FunctionAttribute::defaults(
                &number_ref, 
                Some(true)
            );
            let result = function_attr.replace(data_map.clone());
            let result = result.item_processed.clone();
            number = FromStr::from_str(result.unwrap().as_str()).unwrap();
        } else {
            let number_string = attribute_item.value.unwrap_or_default();
            number = FromStr::from_str(number_string.as_str()).unwrap();
        }
        if is_divisor_reference {
            let divisor_ref = divisor_item.reference_value.unwrap_or_default();
            let function_attr = FunctionAttribute::defaults(
                &divisor_ref, 
                Some(true)
            );
            let result = function_attr.replace(data_map.clone());
            let result = result.item_processed.clone();
            divisor = FromStr::from_str(result.unwrap().as_str()).unwrap();
        } else {
            if divisor_item.reference_value.is_some() {
                let divisor_string = divisor_item.reference_value.unwrap();
                divisor = FromStr::from_str(divisor_string.as_str()).unwrap();
            }
        }
        number = number%divisor;
        let number_str = number.to_string();
        let number_str = number_str.as_str();
        let number_result: isize = FromStr::from_str(number_str).unwrap();
        return Ok(number_result.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Power {
    function: Option<FunctionParse>,
    data_map: Option<HashMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>
}
impl Power {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<HashMap<String, String>>
    ) -> Self {
        return Self{function: function, data_map: data_map, attributes: None};
    }
}
impl NumberFunction for Power {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // POWER(number, power)
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_POWER;
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
            function.validate = Some(expr.is_match(function_text.as_str()));
            if function.validate.unwrap() {
                let matches = &expr.captures(function_text.as_str()).unwrap();
                let attr_number = matches.name("number");
                let attr_number_ref = matches.name("number_ref");
                let attr_power = matches.name("power");
                let attr_power_ref = matches.name("power_ref");
                let mut attributes_: Vec<String> = Vec::new();
                if attr_number.is_some() {
                    let number_string = attr_number.unwrap().as_str().to_string();
                    attributes_.push(number_string);
                } else if attr_number_ref.is_some() {
                    let number_ref = attr_number_ref.unwrap().as_str().to_string();
                    attributes_.push(number_ref);
                }
                if attr_power.is_some() {
                    let power_string = attr_power.unwrap().as_str().to_string();
                    attributes_.push(power_string);
                }
                if attr_power_ref.is_some() {
                    let power_ref_string = attr_power_ref.unwrap().as_str().to_string();
                    attributes_.push(power_ref_string);
                }
                function.attributes = Some(attributes_);
            }
        }
        if data_map_wrap.is_some() {
            self.attributes = Some(compiled_attributes);
            self.data_map = Some(data_map);
            function_result.text = Some(self.execute()?);
            function.result = Some(function_result.clone());
        }
        return Ok(function)
    }
    fn execute(&self) -> Result<String, PlanetError> {
        let attributes = self.attributes.clone().unwrap();
        let data_map = &self.data_map.clone().unwrap();
        let attribute_item = attributes[0].clone();
        let is_reference = attribute_item.is_reference;
        let power_item = attributes[1].clone();
        let is_power_reference = power_item.is_reference;
        let mut number: f64;
        let mut power: f64 = 10.0;
        if is_reference {
            let number_ref = attribute_item.reference_value.unwrap();
            let function_attr = FunctionAttribute::defaults(
                &number_ref, 
                Some(true)
            );
            let result = function_attr.replace(data_map.clone());
            let result = result.item_processed.clone();
            number = FromStr::from_str(result.unwrap().as_str()).unwrap();
        } else {
            let number_string = attribute_item.value.unwrap_or_default();
            number = FromStr::from_str(number_string.as_str()).unwrap();
        }
        if is_power_reference {
            let power_ref = power_item.reference_value.unwrap();
            let function_attr = FunctionAttribute::defaults(
                &power_ref, 
                Some(true)
            );
            let result = function_attr.replace(data_map.clone());
            let result = result.item_processed.clone();
            power = FromStr::from_str(result.unwrap().as_str()).unwrap();
        } else {
            if power_item.value.is_some() {
                let power_string = power_item.value.unwrap();
                power = FromStr::from_str(power_string.as_str()).unwrap();
            }
        }
        number = number.powf(power);
        let number_str = number.to_string();
        let number_str = number_str.as_str();
        let number_result: f64 = FromStr::from_str(number_str).unwrap();
        return Ok(number_result.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Round {
    function: Option<FunctionParse>,
    data_map: Option<HashMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>
}
impl Round {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<HashMap<String, String>>
    ) -> Self {
        return Self{function: function, data_map: data_map, attributes: None};
    }
}
impl RoundNumberFunction for Round {
    fn handle(&mut self, option: RoundOption) -> Result<FunctionParse, PlanetError> {
        // ROUND(number, digits)
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let mut function = function_parse.clone();
        let data_map_wrap = data_map.clone();
        let (
            function_text_wrap, 
            function_text, 
            compiled_attributes,
            mut function_result,
            data_map,
        ) = prepare_function_parse(function_parse, data_map.clone());
        let expr = &RE_ROUND;
        let expr_up = &RE_ROUND_UP;
        let expr_down = &RE_ROUND_DOWN;
        if function_text_wrap.is_some() {
            match option {
                RoundOption::Basic => {
                    function.validate = Some(expr.is_match(function_text.as_str()));
                }
                RoundOption::Up => {
                    function.validate = Some(expr_up.is_match(function_text.as_str()));
                }
                RoundOption::Down => {
                    function.validate = Some(expr_down.is_match(function_text.as_str()));
                }
            }
            if function.validate.unwrap() {
                let matches: Captures;
                match option {
                    RoundOption::Basic => {
                        matches = expr.captures(function_text.as_str()).unwrap();
                    }
                    RoundOption::Up => {
                        matches = expr_up.captures(function_text.as_str()).unwrap();
                    }
                    RoundOption::Down => {
                        matches = expr_down.captures(function_text.as_str()).unwrap();
                    }
                }
                let attr_number = matches.name("number");
                let attr_number_ref = matches.name("number_ref");
                let attr_digits = matches.name("digits");
                let attr_digits_ref = matches.name("digits_ref");
                let mut attributes_: Vec<String> = Vec::new();
                if attr_number.is_some() {
                    let number_string = attr_number.unwrap().as_str().to_string();
                    attributes_.push(number_string);
                } else if attr_number_ref.is_some() {
                    let number_ref = attr_number_ref.unwrap().as_str().to_string();
                    attributes_.push(number_ref);
                }
                if attr_digits.is_some() {
                    let digits_string = attr_digits.unwrap().as_str().to_string();
                    attributes_.push(digits_string);
                }
                if attr_digits_ref.is_some() {
                    let digits_ref_string = attr_digits_ref.unwrap().as_str().to_string();
                    attributes_.push(digits_ref_string);
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
    fn execute(&self, option: RoundOption) -> Result<String, PlanetError> {
        let attributes = self.attributes.clone().unwrap();
        let data_map = &self.data_map.clone().unwrap();
        let attribute_item = attributes[0].clone();
        let is_reference = attribute_item.is_reference;
        let digits_item = attributes[1].clone();
        let is_digits_reference = digits_item.is_reference;
        let mut number: f64;
        let mut digits: i8 = 2;
        if is_reference {
            let number_ref = attribute_item.reference_value.unwrap();
            let function_attr = FunctionAttribute::defaults(
                &number_ref, 
                Some(true)
            );
            let result = function_attr.replace(data_map.clone());
            let result = result.item_processed.clone();
            number = FromStr::from_str(result.unwrap().as_str()).unwrap();
        } else {
            let number_string = attribute_item.value.unwrap();
            number = FromStr::from_str(number_string.as_str()).unwrap();
        }
        if is_digits_reference {
            let number_ref = digits_item.reference_value.unwrap();
            let function_attr = FunctionAttribute::defaults(
                &number_ref, 
                Some(true)
            );
            let result = function_attr.replace(data_map.clone());
            let result = result.item_processed.clone();
            digits = FromStr::from_str(result.unwrap().as_str()).unwrap();
        } else {
            if digits_item.value.is_some() {
                let digits_string = digits_item.value.unwrap();
                digits = FromStr::from_str(digits_string.as_str()).unwrap();
            }
        }
        match option {
            RoundOption::Basic => {
                number = round::half_away_from_zero(number, digits);
            },
            RoundOption::Up => {
                number = round::ceil(number, digits);
            },
            RoundOption::Down => {
                number = round::floor(number, digits);
            },
        }
        let number_str = number.to_string();
        let number_str = number_str.as_str();
        let number_result: f64 = FromStr::from_str(number_str).unwrap();
        return Ok(number_result.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Sqrt {
    function: Option<FunctionParse>,
    data_map: Option<HashMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>
}
impl Sqrt {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<HashMap<String, String>>
    ) -> Self {
        return Self{function: function, data_map: data_map, attributes: None};
    }
}
impl NumberFunction for Sqrt {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // SQRT(number)
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_SQRT;
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
            function.validate = Some(expr.is_match(function_text.as_str()));
            if function.validate.unwrap() {
                let matches = RE_SQRT.captures(function_text.as_str()).unwrap();
                let attr_number = matches.name("number");
                let attr_number_ref = matches.name("number_ref");
                let mut attributes_: Vec<String> = Vec::new();
                if attr_number.is_some() {
                    let number_string = attr_number.unwrap().as_str().to_string();
                    attributes_.push(number_string);
                } else if attr_number_ref.is_some() {
                    let number_string = attr_number_ref.unwrap().as_str().to_string();
                    attributes_.push(number_string);
                }
                function.attributes = Some(attributes_);
            }
        }
        if data_map_wrap.is_some() {
            self.attributes = Some(compiled_attributes);
            self.data_map = Some(data_map);
            function_result.text = Some(self.execute()?);
            function.result = Some(function_result.clone());
        }
        return Ok(function)
    }
    fn execute(&self) -> Result<String, PlanetError> {
        let attributes = self.attributes.clone().unwrap();
        let data_map = &self.data_map.clone().unwrap();
        let attribute_item = attributes[0].clone();
        let is_reference = attribute_item.is_reference;
        let mut number: f64;
        if is_reference {
            let number_ref = attribute_item.reference_value.unwrap();
            let function_attr = FunctionAttribute::defaults(
                &number_ref, 
                Some(true)
            );
            let result = function_attr.replace(data_map.clone());
            let result = result.item_processed.clone();
            number = FromStr::from_str(result.unwrap().as_str()).unwrap();
        } else {
            let number_string = attribute_item.value.unwrap();
            number = FromStr::from_str(number_string.as_str()).unwrap();
        }
        number = number.sqrt();
        let number_str = number.to_string();
        let number_str = number_str.as_str();
        let number_result: f64 = FromStr::from_str(number_str).unwrap();
        return Ok(number_result.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Value {
    function: Option<FunctionParse>,
    data_map: Option<HashMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>
}
impl Value {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<HashMap<String, String>>
    ) -> Self {
        return Self{function: function, data_map: data_map, attributes: None};
    }
}
impl NumberFunction for Value {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // VALUE(text)
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_VALUE;
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
            function.validate = Some(expr.is_match(function_text.as_str()));
            if function.validate.unwrap() {
                let matches = &expr.captures(function_text.as_str()).unwrap();
                let attr_text = matches.name("text");
                let attr_text_ref = matches.name("text_ref");
                let mut attributes_: Vec<String> = Vec::new();
                if attr_text.is_some() {
                    let text = attr_text.unwrap().as_str().to_string();
                    attributes_.push(text);
                } else if attr_text_ref.is_some() {
                    let text_ref = attr_text_ref.unwrap().as_str().to_string();
                    attributes_.push(text_ref);
                }
                function.attributes = Some(attributes_);
            }
        }
        if data_map_wrap.is_some() {
            self.attributes = Some(compiled_attributes);
            self.data_map = Some(data_map);
            function_result.text = Some(self.execute()?);
            function.result = Some(function_result.clone());
        }
        return Ok(function)
    }
    fn execute(&self) -> Result<String, PlanetError> {
        let attributes = self.attributes.clone().unwrap();
        let data_map = &self.data_map.clone().unwrap();
        let attribute_item = attributes[0].clone();
        let is_reference = attribute_item.is_reference;
        let mut text: String;
        if is_reference {
            let text_ref = attribute_item.reference_value.unwrap();
            let function_attr = FunctionAttribute::defaults(
                &text_ref, 
                Some(true)
            );
            let result = function_attr.replace(data_map.clone());
            let result = result.item_processed.clone();
            text = result.unwrap().as_str().to_string();
        } else {
            text = attribute_item.value.unwrap();
        }
        let number: f64;
        text = text.replace("$", "").replace("€", "");
        text = text.replace("\"", "");
        let has_multiple_punc = text.find(",").is_some() && text.find(".").is_some();
        if has_multiple_punc {
            // , could be thousand sep or decimals
            // . could be thousand or decimals
            let index_dot = text.find(".").unwrap();
            let index_comma = text.find(",").unwrap();
            if index_comma > index_dot {
                // 1.200,98
                text = text.replace(".", "");
                text = text.replace(",", ".");
                // 1200.98
            } else {
                // 1,200.98
                text = text.replace(",", "");
                // 1200.98
            }
        } else {
            let index_comma = text.find(",");
            if index_comma.is_some() {
                // 920,98
                text = text.replace(",", ".");
                // 920.98
            }
        }
        number = FromStr::from_str(text.as_str()).unwrap();
        return Ok(number.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Boolean {
    function: Option<FunctionParse>,
    data_map: Option<HashMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>
}
impl Boolean {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<HashMap<String, String>>
    ) -> Self {
        return Self{function: function, data_map: data_map, attributes: None};
    }
}
impl NumberFunction for Boolean {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // TRUE() or FALSE()
        // TRUE or FALSE
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_BOOLEAN;
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
            function.validate = Some(expr.is_match(function_text.as_str()));
            if function.validate.unwrap() {
                let attributes_: Vec<String> = Vec::new();
                function.attributes = Some(attributes_);
            }
        }
        if data_map_wrap.is_some() {
            self.attributes = Some(compiled_attributes);
            self.data_map = Some(data_map);
            function_result.text = Some(self.execute()?);
            function.result = Some(function_result.clone());
        }
        return Ok(function)
    }
    fn execute(&self) -> Result<String, PlanetError> {
        let function_parse = &self.function.clone().unwrap().clone();
        let function_text = function_parse.clone().text.unwrap_or_default();
        let mut number: u8 = 0;
        if function_text.find("TRUE").is_some() {
            number = 1;
        }
        return Ok(number.to_string())
    }
}

pub fn check_float_compare(value: &f64, compare_to: &f64, op: FormulaOperator) -> Result<bool, PlanetError> {
    let mut check: bool = false;
    match op {
        FormulaOperator::Greater => {
            if value > compare_to {
                check = true;
            }
        },
        FormulaOperator::Smaller => {
            if value < compare_to {
                check = true;
            }
        },
        FormulaOperator::GreaterOrEqual => {
            if value >= compare_to {
                check = true;
            }
        },
        FormulaOperator::SmallerOrEqual => {
            if value <= compare_to {
                check = true;
            }
        },
        _ => {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("Operator not supported")),
                )
            );

        }
    }
    return Ok(check)
}

// CEILING(number, significance)
/* #[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CeilingFunction {
    pub function_text: String,
    pub number: Option<f64>,
    pub number_ref: Option<String>,
    pub significance: Option<i8>,
    pub data_map: Option<HashMap<String, String>>,
}
impl CeilingFunction {
    pub fn defaults(function_text: &String, data_map: Option<HashMap<String, String>>) -> CeilingFunction {
        // CEILING(number, significance)

        let matches = RE_CEILING.captures(function_text).unwrap();
        let attr_number = matches.name("number");
        let attr_number_ref = matches.name("number_ref");
        let attr_significance = matches.name("significance");

        let mut number_wrap: Option<f64> = None;
        let mut number_ref_wrap: Option<String> = None;
        let mut significance_wrap: Option<i8> = None;

        if attr_number.is_some() {
            let number = attr_number.unwrap();
            let number: f64 = FromStr::from_str(number.as_str()).unwrap();
            number_wrap = Some(number);
        } else if attr_number_ref.is_some() {
            number_ref_wrap = Some(attr_number_ref.unwrap().as_str().to_string());
        }
        if attr_significance.is_some() {
            let significance = attr_significance.unwrap();
            let significance: i8 = FromStr::from_str(significance.as_str()).unwrap();
            significance_wrap = Some(significance);
        }

        let obj = Self{
            function_text: function_text.clone(),
            number: number_wrap,
            number_ref: number_ref_wrap,
            significance: significance_wrap,
            data_map: data_map,
        };

        return obj
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = CeilingFunction::defaults(
            &function_text, None
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_CEILING));
        }
        return (number_fails, failed_functions);
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = CeilingFunction::defaults(
            &function_text,Some(data_map)
        );
        formula = concat_obj.replace(formula);
        return formula
    }
}
impl Function for CeilingFunction {
    fn validate(&self) -> bool {
        let expr = RE_CEILING.clone();
        let function_text = self.function_text.clone();
        let mut check = expr.is_match(&function_text);
        let number = self.number.clone();
        let number_ref = self.number_ref.clone();
        let significance = self.significance.clone();
        if check == false {
            return check
        }
        if number.is_none() && number_ref.is_none() {
            check = false;
        }
        if significance.is_none() {
            check = false
        }
        return check
    }
    fn replace(&mut self, formula: String) -> String {
        let data_map = self.data_map.clone().unwrap();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();
        let replacement_string: String;

        let number_wrap = self.number.clone();
        let number_ref_wrap = self.number_ref.clone();
        let significance = self.significance.clone().unwrap() - 1;
        let number: f64;

        if number_wrap.is_some() {
            number = number_wrap.unwrap();
        } else {
            let number_ref = number_ref_wrap.unwrap();
            let function_attr = FunctionAttribute::defaults(
                &number_ref, 
                Some(true)
            );
            let number_str = function_attr.replace(data_map.clone()).item_processed.unwrap();
            number = FromStr::from_str(number_str.as_str()).unwrap();
        }
        let number = round::ceil(number, significance);

        replacement_string = number.to_string();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        formula = format!("\"{}\"", formula);
        return formula;
    }
}
 */
// FLOOR(number, significance)
/* #[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FloorFunction {
    pub function_text: String,
    pub number: Option<f64>,
    pub number_ref: Option<String>,
    pub significance: Option<i8>,
    pub data_map: Option<HashMap<String, String>>,
}
impl FloorFunction {
    pub fn defaults(function_text: &String, data_map: Option<HashMap<String, String>>) -> FloorFunction {
        // FLOOR(number, significance)

        let matches = RE_FLOOR.captures(function_text).unwrap();
        let attr_number = matches.name("number");
        let attr_number_ref = matches.name("number_ref");
        let attr_significance = matches.name("significance");

        let mut number_wrap: Option<f64> = None;
        let mut number_ref_wrap: Option<String> = None;
        let mut significance_wrap: Option<i8> = None;

        if attr_number.is_some() {
            let number = attr_number.unwrap();
            let number: f64 = FromStr::from_str(number.as_str()).unwrap();
            number_wrap = Some(number);
        } else if attr_number_ref.is_some() {
            number_ref_wrap = Some(attr_number_ref.unwrap().as_str().to_string());
        }
        if attr_significance.is_some() {
            let significance = attr_significance.unwrap();
            let significance: i8 = FromStr::from_str(significance.as_str()).unwrap();
            significance_wrap = Some(significance);
        }

        let obj = Self{
            function_text: function_text.clone(),
            number: number_wrap,
            number_ref: number_ref_wrap,
            significance: significance_wrap,
            data_map: data_map,
        };

        return obj
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = FloorFunction::defaults(
            &function_text, None
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_FLOOR));
        }
        return (number_fails, failed_functions);
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = FloorFunction::defaults(
            &function_text,Some(data_map)
        );
        formula = concat_obj.replace(formula);
        return formula
    }
}
impl Function for FloorFunction {
    fn validate(&self) -> bool {
        let expr = RE_FLOOR.clone();
        let function_text = self.function_text.clone();
        let mut check = expr.is_match(&function_text);
        let number = self.number.clone();
        let number_ref = self.number_ref.clone();
        let significance = self.significance.clone();
        if check == false {
            return check
        }
        if number.is_none() && number_ref.is_none() {
            check = false;
        }
        if significance.is_none() {
            check = false
        }
        return check
    }
    fn replace(&mut self, formula: String) -> String {
        let data_map = self.data_map.clone().unwrap();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();
        let replacement_string: String;

        let number_wrap = self.number.clone();
        let number_ref_wrap = self.number_ref.clone();
        let significance = self.significance.clone().unwrap() - 1;
        let number: f64;

        if number_wrap.is_some() {
            number = number_wrap.unwrap();
        } else {
            let number_ref = number_ref_wrap.unwrap();
            let function_attr = FunctionAttribute::defaults(
                &number_ref, 
                Some(true)
            );
            let number_str = function_attr.replace(data_map.clone()).item_processed.unwrap();
            number = FromStr::from_str(number_str.as_str()).unwrap();
        }
        let number = round::floor(number, significance);

        replacement_string = number.to_string();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        formula = format!("\"{}\"", formula);
        return formula;
    }
}
 */
// COUNT(value1, value2, ...))
/* #[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CountFunction {
    pub function_text: String,
    pub attrs: Option<String>,
    pub data_map: Option<HashMap<String, String>>,
}
impl CountFunction {
    pub fn defaults(function_text: &String, data_map: Option<HashMap<String, String>>) -> CountFunction {
        // COUNT(value1, value2, ...))

        let matches = RE_COUNT.captures(function_text).unwrap();
        let attrs = matches.name("attrs");

        let mut attrs_wrap: Option<String> = None;
        if attrs.is_some() {
            let attrs = attrs.unwrap().as_str().to_string();
            attrs_wrap = Some(attrs);
        }

        let obj = Self{
            function_text: function_text.clone(),
            attrs: attrs_wrap,
            data_map: data_map,
        };

        return obj
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = CountFunction::defaults(
            &function_text, None
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_COUNT));
        }
        return (number_fails, failed_functions);
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = CountFunction::defaults(
            &function_text,Some(data_map)
        );
        formula = concat_obj.replace(formula);
        return formula
    }
}
impl Function for CountFunction {
    fn validate(&self) -> bool {
        let expr = RE_COUNT.clone();
        let function_text = self.function_text.clone();
        let mut check = expr.is_match(&function_text);
        if check == false {
            return check
        }
        let attrs_wrap = self.attrs.clone();
        if attrs_wrap.is_none() {
            check = false
        }
        return check
    }
    fn replace(&mut self, formula: String) -> String {
        let data_map = self.data_map.clone().unwrap();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();
        let replacement_string: String;

        let attrs = self.attrs.clone().unwrap();
        let items: Vec<&str> = attrs.split(",").collect();
        let mut number_items: Vec<String> = Vec::new();
        for mut item in items {
            item = item.trim();
            let is_string = item.to_string().find("\"");
            let is_ref = item.to_string().find("{");
            if is_ref.is_some() {
                let item_ = &item.trim().to_string();
                let function_attr = FunctionAttribute::defaults(
                    item_, 
                    Some(true)
                );
                let result = function_attr.replace(data_map.clone());
                let result = result.item_processed.clone();
                let result = result.unwrap();
                let result = result.as_str();
                let is_string = result.to_string().find("\"");
                if is_string.is_none() {
                    number_items.push(result.to_string());
                }
            } else {
                if is_string.is_none() {
                    number_items.push(item.to_string());
                }
            }
        }
        let count = number_items.len();

        replacement_string = count.to_string();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        return formula;
    }
}
 */

// COUNTA(value1, value2, ...))
/* #[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CountAFunction {
    pub function_text: String,
    pub attrs: Option<String>,
    pub data_map: Option<HashMap<String, String>>,
}
impl CountAFunction {
    pub fn defaults(function_text: &String, data_map: Option<HashMap<String, String>>) -> CountAFunction {
        // COUNTA(value1, value2, ...))
        // We take as empty null and ""

        let matches = RE_COUNTA.captures(function_text).unwrap();
        let attrs = matches.name("attrs");

        let mut attrs_wrap: Option<String> = None;
        if attrs.is_some() {
            let attrs = attrs.unwrap().as_str().to_string();
            attrs_wrap = Some(attrs);
        }

        let obj = Self{
            function_text: function_text.clone(),
            attrs: attrs_wrap,
            data_map: data_map,
        };

        return obj
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = CountAFunction::defaults(
            &function_text, None
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_COUNTA));
        }
        return (number_fails, failed_functions);
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = CountAFunction::defaults(
            &function_text,Some(data_map)
        );
        formula = concat_obj.replace(formula);
        return formula
    }
}
impl Function for CountAFunction {
    fn validate(&self) -> bool {
        let expr = RE_COUNTA.clone();
        let function_text = self.function_text.clone();
        let mut check = expr.is_match(&function_text);
        if check == false {
            return check
        }
        let attrs_wrap = self.attrs.clone();
        if attrs_wrap.is_none() {
            check = false
        }
        return check
    }
    fn replace(&mut self, formula: String) -> String {
        let data_map = self.data_map.clone().unwrap();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();
        let replacement_string: String;

        let attrs = self.attrs.clone().unwrap();
        let items: Vec<&str> = attrs.split(",").collect();
        let mut number_items: Vec<String> = Vec::new();
        for mut item in items {
            item = item.trim();
            let is_string = item.to_string().find("\"");
            let is_null = item.to_lowercase() == "null";
            let is_ref = item.to_string().find("{");
            if is_ref.is_some() {
                let item_ = &item.trim().to_string();
                let function_attr = FunctionAttribute::defaults(
                    item_, 
                    Some(true)
                );
                let result = function_attr.replace(data_map.clone());
                let result = result.item_processed.clone();
                let result = result.unwrap();
                let result = result.as_str();
                let is_string = result.to_string().find("\"");
                if is_string.is_some() {
                    if item != "\"\"" {
                        number_items.push(result.to_string());
                    }
                } else {
                    if is_null == false {
                        number_items.push(item.to_string());
                    }
                }
            } else {
                if is_string.is_some() {
                    if item != "\"\"" {
                        number_items.push(item.to_string());
                    }
                } else {
                    if is_null == false {
                        number_items.push(item.to_string());
                    }
                }
            }
        }
        let count = number_items.len();

        replacement_string = count.to_string();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        return formula;
    }
} */

// COUNTALL(value1, value2, ...))
/* #[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CountAllFunction {
    pub function_text: String,
    pub attrs: Option<String>,
    pub data_map: Option<HashMap<String, String>>,
}
impl CountAllFunction {
    pub fn defaults(function_text: &String, data_map: Option<HashMap<String, String>>) -> CountAllFunction {
        // COUNTALL(value1, value2, ...))
        // We count all, including nulls and empty values

        let matches = RE_COUNTALL.captures(function_text).unwrap();
        let attrs = matches.name("attrs");

        let mut attrs_wrap: Option<String> = None;
        if attrs.is_some() {
            let attrs = attrs.unwrap().as_str().to_string();
            attrs_wrap = Some(attrs);
        }

        let obj = Self{
            function_text: function_text.clone(),
            attrs: attrs_wrap,
            data_map: data_map,
        };

        return obj
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = CountAllFunction::defaults(
            &function_text, None
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_COUNTALL));
        }
        return (number_fails, failed_functions);
    }
    pub fn do_replace(function_text: &String, mut formula: String) -> String {
        let mut concat_obj = CountAllFunction::defaults(
            &function_text, None
        );
        formula = concat_obj.replace(formula);
        return formula
    }
}
impl Function for CountAllFunction {
    fn validate(&self) -> bool {
        let expr = RE_COUNTALL.clone();
        let function_text = self.function_text.clone();
        let mut check = expr.is_match(&function_text);
        if check == false {
            return check
        }
        let attrs_wrap = self.attrs.clone();
        if attrs_wrap.is_none() {
            check = false
        }
        return check
    }
    fn replace(&mut self, formula: String) -> String {
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();
        let replacement_string: String;

        let attrs = self.attrs.clone().unwrap();
        let items: Vec<&str> = attrs.split(",").collect();
        let mut number_items: Vec<String> = Vec::new();
        for mut item in items {
            item = item.trim();
            number_items.push(item.to_string());
        }
        let count = number_items.len();

        replacement_string = count.to_string();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        return formula;
    }
}
 */

// EVEN(number)
/* #[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EvenFunction {
    pub function_text: String,
    pub number: Option<f64>,
    pub number_ref: Option<String>,
    pub data_map: Option<HashMap<String, String>>,
}
impl EvenFunction {
    pub fn defaults(function_text: &String, data_map: Option<HashMap<String, String>>) -> EvenFunction {
        // EVEN(number)
        // EVEN(1.5) => 2
        // EVEN(3) => 4
        // EVEN(2) => 2
        // EVEN(-1) => -2
        // EVEN({Column}) => 2

        let matches = RE_EVEN.captures(function_text).unwrap();
        let attr_number = matches.name("number");
        let attr_number_ref = matches.name("number_ref");

        let mut number_wrap: Option<f64> = None;
        let mut number_ref_wrap: Option<String> = None;

        if attr_number.is_some() {
            let number: f64 = FromStr::from_str(attr_number.unwrap().as_str()).unwrap();
            number_wrap = Some(number);
        } else if attr_number_ref.is_some() {
            let number_ref = attr_number_ref.unwrap().as_str().to_string();
            number_ref_wrap = Some(number_ref);
        }

        let obj = Self{
            function_text: function_text.clone(),
            number: number_wrap,
            number_ref: number_ref_wrap,
            data_map: data_map,
        };

        return obj
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = EvenFunction::defaults(
            &function_text, None
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_EVEN));
        }
        return (number_fails, failed_functions);
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = EvenFunction::defaults(
            &function_text,Some(data_map)
        );
        formula = concat_obj.replace(formula);
        return formula
    }
}
impl Function for EvenFunction {
    fn validate(&self) -> bool {
        let expr = RE_EVEN.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    fn replace(&mut self, formula: String) -> String {
        let data_map = self.data_map.clone().unwrap();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();

        let number_wrap = self.number.clone();
        let number_ref_wrap = self.number_ref.clone();
        let mut number: f64 = 0.0;
        let mut rounded_int: i32;
        if number_wrap.is_some() {
            number = number_wrap.unwrap();
        } else if number_ref_wrap.is_some() {
            let number_ref = number_ref_wrap.unwrap();
            let function_attr = FunctionAttribute::defaults(
                &number_ref, 
                Some(true)
            );
            let result = function_attr.replace(data_map.clone());
            let result = result.item_processed.clone();
            number = FromStr::from_str(result.unwrap().as_str()).unwrap();
        }

        let rounded = number.round();
        rounded_int = FromStr::from_str(rounded.to_string().as_str()).unwrap();
        let is_even = rounded_int%2 == 0;
        if is_even == false && rounded_int > 0 {
            rounded_int += 1;
        } else if is_even == true && rounded_int < 0 {
            rounded_int -= 1;
        }

        let replacement_string = rounded_int.to_string();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        return formula;
    }
} */

// EXP(number)
/* #[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExpFunction {
    pub function_text: String,
    pub number: Option<f64>,
    pub number_ref: Option<String>,
    pub data_map: Option<HashMap<String, String>>,
}
impl ExpFunction {
    pub fn defaults(function_text: &String, data_map: Option<HashMap<String, String>>) -> ExpFunction {
        // EXP(number)
        // EXP(1) => 2.71828183

        let matches = RE_EXP.captures(function_text).unwrap();
        let attr_number = matches.name("number");
        let attr_number_ref = matches.name("number_ref");

        let mut number_wrap: Option<f64> = None;
        let mut number_ref_wrap: Option<String> = None;

        if attr_number.is_some() {
            let number: f64 = FromStr::from_str(attr_number.unwrap().as_str()).unwrap();
            number_wrap = Some(number);
        } else if attr_number_ref.is_some() {
            let number_ref = attr_number_ref.unwrap().as_str().to_string();
            number_ref_wrap = Some(number_ref);
        }

        let obj = Self{
            function_text: function_text.clone(),
            number: number_wrap,
            number_ref: number_ref_wrap,
            data_map: data_map,
        };

        return obj
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = ExpFunction::defaults(
            &function_text, None
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_EXP));
        }
        return (number_fails, failed_functions);
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = ExpFunction::defaults(
            &function_text,Some(data_map)
        );
        formula = concat_obj.replace(formula);
        return formula
    }
}
impl Function for ExpFunction {
    fn validate(&self) -> bool {
        let expr = RE_EXP.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    fn replace(&mut self, formula: String) -> String {
        let data_map = self.data_map.clone().unwrap();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();

        let number_wrap = self.number.clone();
        let number_ref_wrap = self.number_ref.clone();
        let mut number: f64 = 0.0;
        let number_result: f64;
        if number_wrap.is_some() {
            number = number_wrap.unwrap();
        } else if number_ref_wrap.is_some() {
            let number_ref = number_ref_wrap.unwrap();
            let function_attr = FunctionAttribute::defaults(
                &number_ref, 
                Some(true)
            );
            let result = function_attr.replace(data_map.clone());
            let result = result.item_processed.clone();
            number = FromStr::from_str(result.unwrap().as_str()).unwrap();
        }

        number_result = number.exp();

        let replacement_string = number_result.to_string();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        return formula;
    }
} */

// INT(number)
/* #[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IntFunction {
    pub function_text: String,
    pub number: Option<f64>,
    pub number_ref: Option<String>,
    pub data_map: Option<HashMap<String, String>>,
}
impl IntFunction {
    pub fn defaults(function_text: &String, data_map: Option<HashMap<String, String>>) -> IntFunction {
        // INT(number)
        // INT(8.9) => 8

        let matches = RE_INT.captures(function_text).unwrap();
        let attr_number = matches.name("number");
        let attr_number_ref = matches.name("number_ref");

        let mut number_wrap: Option<f64> = None;
        let mut number_ref_wrap: Option<String> = None;

        if attr_number.is_some() {
            let number: f64 = FromStr::from_str(attr_number.unwrap().as_str()).unwrap();
            number_wrap = Some(number);
        } else if attr_number_ref.is_some() {
            let number_ref = attr_number_ref.unwrap().as_str().to_string();
            number_ref_wrap = Some(number_ref);
        }

        let obj = Self{
            function_text: function_text.clone(),
            number: number_wrap,
            number_ref: number_ref_wrap,
            data_map: data_map,
        };

        return obj
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = IntFunction::defaults(
            &function_text, None
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_INT));
        }
        return (number_fails, failed_functions);
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = IntFunction::defaults(
            &function_text,Some(data_map)
        );
        formula = concat_obj.replace(formula);
        return formula
    }
}
impl Function for IntFunction {
    fn validate(&self) -> bool {
        let expr = RE_INT.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    fn replace(&mut self, formula: String) -> String {
        let data_map = self.data_map.clone().unwrap();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();

        let number_wrap = self.number.clone();
        let number_ref_wrap = self.number_ref.clone();
        let mut number: f64 = 0.0;
        if number_wrap.is_some() {
            number = number_wrap.unwrap();
        } else if number_ref_wrap.is_some() {
            let number_ref = number_ref_wrap.unwrap();
            let function_attr = FunctionAttribute::defaults(
                &number_ref, 
                Some(true)
            );
            let result = function_attr.replace(data_map.clone());
            let result = result.item_processed.clone();
            number = FromStr::from_str(result.unwrap().as_str()).unwrap();
        }
        number = number.trunc();
        let number_str = number.to_string();
        let number_str = number_str.as_str();
        let number_result: isize = FromStr::from_str(number_str).unwrap();

        let replacement_string = number_result.to_string();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        return formula;
    }
} */

// LOG(number)
// LOG(number, [base])
/* #[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogFunction {
    pub function_text: String,
    pub number: Option<f64>,
    pub number_ref: Option<String>,
    pub base: Option<usize>,
    pub base_ref: Option<usize>,
    pub data_map: Option<HashMap<String, String>>,
}
impl LogFunction {
    pub fn defaults(function_text: &String, data_map: Option<HashMap<String, String>>) -> LogFunction {
        // LOG(number)
        // LOG(number, [base])
        // LOG(10) = 1
        // LOG(8, 2) = 3

        let matches = RE_LOG.captures(function_text).unwrap();
        let attr_number = matches.name("number");
        let attr_number_ref = matches.name("number_ref");
        let attr_base = matches.name("base");
        let attr_base_ref = matches.name("base_ref");

        let mut number_wrap: Option<f64> = None;
        let mut number_ref_wrap: Option<String> = None;
        let mut base_wrap: Option<usize> = None;
        let mut base_ref_wrap: Option<usize> = None;

        if attr_number.is_some() {
            let number: f64 = FromStr::from_str(attr_number.unwrap().as_str()).unwrap();
            number_wrap = Some(number);
        } else if attr_number_ref.is_some() {
            let number_ref = attr_number_ref.unwrap().as_str().to_string();
            number_ref_wrap = Some(number_ref);
        }
        if attr_base.is_some() {
            let base = attr_base.unwrap().as_str();
            let base: usize = FromStr::from_str(base).unwrap();
            base_wrap = Some(base);
        }
        if attr_base_ref.is_some() {
            let base = attr_base_ref.unwrap().as_str();
            let base_ref: usize = FromStr::from_str(base).unwrap();
            base_ref_wrap = Some(base_ref);
        }

        let obj = Self{
            function_text: function_text.clone(),
            number: number_wrap,
            number_ref: number_ref_wrap,
            base: base_wrap,
            base_ref: base_ref_wrap,
            data_map: data_map,
        };

        return obj
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = LogFunction::defaults(
            &function_text, None
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_LOG));
        }
        return (number_fails, failed_functions);
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = LogFunction::defaults(
            &function_text,Some(data_map)
        );
        formula = concat_obj.replace(formula);
        return formula
    }
}
impl Function for LogFunction {
    fn validate(&self) -> bool {
        let expr = RE_LOG.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    fn replace(&mut self, formula: String) -> String {
        let data_map = self.data_map.clone().unwrap();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();

        let number_wrap = self.number.clone();
        let number_ref_wrap = self.number_ref.clone();
        let base_wrap = self.base.clone();
        let base_ref_wrap = self.base_ref.clone();
        let mut number: f64 = 0.0;
        let mut base: f64 = 10.0;
        if number_wrap.is_some() {
            number = number_wrap.unwrap();
        } else if number_ref_wrap.is_some() {
            let number_ref = number_ref_wrap.unwrap();
            let function_attr = FunctionAttribute::defaults(
                &number_ref, 
                Some(true)
            );
            let result = function_attr.replace(data_map.clone());
            let result = result.item_processed.clone();
            number = FromStr::from_str(result.unwrap().as_str()).unwrap();
        }
        if base_wrap.is_some() {
            let base_: usize = base_wrap.unwrap();
            base = FromStr::from_str(base_.to_string().as_str()).unwrap();
        }
        if base_ref_wrap.is_some() {
            let base_: usize = base_ref_wrap.unwrap();
            base = FromStr::from_str(base_.to_string().as_str()).unwrap();
        }
        number = number.log(base);
        let number_str = number.to_string();
        let number_str = number_str.as_str();
        let number_result: isize = FromStr::from_str(number_str).unwrap();

        let replacement_string = number_result.to_string();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        return formula;
    }
} */

// MOD(number, divisor)
/* #[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModFunction {
    pub function_text: String,
    pub number: Option<f64>,
    pub number_ref: Option<String>,
    pub divisor: Option<usize>,
    pub divisor_ref: Option<usize>,
    pub data_map: Option<HashMap<String, String>>,
}
impl ModFunction {
    pub fn defaults(function_text: &String, data_map: Option<HashMap<String, String>>) -> ModFunction {
        // MOD(number, divisor)

        let matches = RE_MOD.captures(function_text).unwrap();
        let attr_number = matches.name("number");
        let attr_number_ref = matches.name("number_ref");
        let attr_divisor = matches.name("divisor");
        let attr_divisor_ref = matches.name("divisor_ref");

        let mut number_wrap: Option<f64> = None;
        let mut number_ref_wrap: Option<String> = None;
        let mut divisor_wrap: Option<usize> = None;
        let mut divisor_ref_wrap: Option<usize> = None;

        if attr_number.is_some() {
            let number: f64 = FromStr::from_str(attr_number.unwrap().as_str()).unwrap();
            number_wrap = Some(number);
        } else if attr_number_ref.is_some() {
            let number_ref = attr_number_ref.unwrap().as_str().to_string();
            number_ref_wrap = Some(number_ref);
        }
        if attr_divisor.is_some() {
            let divisor = attr_divisor.unwrap().as_str();
            let divisor: usize = FromStr::from_str(divisor).unwrap();
            divisor_wrap = Some(divisor);
        }
        if attr_divisor_ref.is_some() {
            let divisor = attr_divisor_ref.unwrap().as_str();
            let divisor_ref: usize = FromStr::from_str(divisor).unwrap();
            divisor_ref_wrap = Some(divisor_ref);
        }

        let obj = Self{
            function_text: function_text.clone(),
            number: number_wrap,
            number_ref: number_ref_wrap,
            divisor: divisor_wrap,
            divisor_ref: divisor_ref_wrap,
            data_map: data_map,
        };

        return obj
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = ModFunction::defaults(
            &function_text, None
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_MOD));
        }
        return (number_fails, failed_functions);
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = ModFunction::defaults(
            &function_text,Some(data_map)
        );
        formula = concat_obj.replace(formula);
        return formula
    }
}
impl Function for ModFunction {
    fn validate(&self) -> bool {
        let expr = RE_MOD.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    fn replace(&mut self, formula: String) -> String {
        let data_map = self.data_map.clone().unwrap();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();

        let number_wrap = self.number.clone();
        let number_ref_wrap = self.number_ref.clone();
        let divisor_wrap = self.divisor.clone();
        let divisor_ref_wrap = self.divisor_ref.clone();
        let mut number: f64 = 0.0;
        let mut divisor: f64 = 10.0;
        if number_wrap.is_some() {
            number = number_wrap.unwrap();
        } else if number_ref_wrap.is_some() {
            let number_ref = number_ref_wrap.unwrap();
            let function_attr = FunctionAttribute::defaults(
                &number_ref, 
                Some(true)
            );
            let result = function_attr.replace(data_map.clone());
            let result = result.item_processed.clone();
            number = FromStr::from_str(result.unwrap().as_str()).unwrap();
        }
        if divisor_wrap.is_some() {
            let divisor_: usize = divisor_wrap.unwrap();
            divisor = FromStr::from_str(divisor_.to_string().as_str()).unwrap();
        }
        if divisor_ref_wrap.is_some() {
            let divisor_: usize = divisor_ref_wrap.unwrap();
            divisor = FromStr::from_str(divisor_.to_string().as_str()).unwrap();
        }
        number = number%divisor;
        let number_str = number.to_string();
        let number_str = number_str.as_str();
        let number_result: isize = FromStr::from_str(number_str).unwrap();

        let replacement_string = number_result.to_string();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        return formula;
    }
} */

// POWER(number, power)
/* #[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PowerFunction {
    pub function_text: String,
    pub number: Option<f64>,
    pub number_ref: Option<String>,
    pub power: Option<f64>,
    pub power_ref: Option<f64>,
    pub data_map: Option<HashMap<String, String>>,
}
impl PowerFunction {
    pub fn defaults(function_text: &String, data_map: Option<HashMap<String, String>>) -> PowerFunction {
        // POWER(number, power)

        let matches = RE_POWER.captures(function_text).unwrap();
        let attr_number = matches.name("number");
        let attr_number_ref = matches.name("number_ref");
        let attr_power = matches.name("power");
        let attr_power_ref = matches.name("power_ref");

        let mut number_wrap: Option<f64> = None;
        let mut number_ref_wrap: Option<String> = None;
        let mut power_wrap: Option<f64> = None;
        let mut power_ref_wrap: Option<f64> = None;

        if attr_number.is_some() {
            let number: f64 = FromStr::from_str(attr_number.unwrap().as_str()).unwrap();
            number_wrap = Some(number);
        } else if attr_number_ref.is_some() {
            let number_ref = attr_number_ref.unwrap().as_str().to_string();
            number_ref_wrap = Some(number_ref);
        }
        if attr_power.is_some() {
            let power = attr_power.unwrap().as_str();
            let power: f64 = FromStr::from_str(power).unwrap();
            power_wrap = Some(power);
        }
        if attr_power_ref.is_some() {
            let power = attr_power_ref.unwrap().as_str();
            let power_ref: f64 = FromStr::from_str(power).unwrap();
            power_ref_wrap = Some(power_ref);
        }

        let obj = Self{
            function_text: function_text.clone(),
            number: number_wrap,
            number_ref: number_ref_wrap,
            power: power_wrap,
            power_ref: power_ref_wrap,
            data_map: data_map,
        };

        return obj
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = PowerFunction::defaults(
            &function_text, None
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_POWER));
        }
        return (number_fails, failed_functions);
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = PowerFunction::defaults(
            &function_text,Some(data_map)
        );
        formula = concat_obj.replace(formula);
        return formula
    }
}
impl Function for PowerFunction {
    fn validate(&self) -> bool {
        let expr = RE_POWER.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    fn replace(&mut self, formula: String) -> String {
        let data_map = self.data_map.clone().unwrap();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();

        let number_wrap = self.number.clone();
        let number_ref_wrap = self.number_ref.clone();
        let power_wrap = self.power.clone();
        let power_ref_wrap = self.power_ref.clone();
        let mut number: f64 = 0.0;
        let mut power: f64 = 10.0;
        if number_wrap.is_some() {
            number = number_wrap.unwrap();
        } else if number_ref_wrap.is_some() {
            let number_ref = number_ref_wrap.unwrap();
            let function_attr = FunctionAttribute::defaults(
                &number_ref, 
                Some(true)
            );
            let result = function_attr.replace(data_map.clone());
            let result = result.item_processed.clone();
            number = FromStr::from_str(result.unwrap().as_str()).unwrap();
        }
        if power_wrap.is_some() {
            let power_: f64 = power_wrap.unwrap();
            power = FromStr::from_str(power_.to_string().as_str()).unwrap();
        }
        if power_ref_wrap.is_some() {
            let power_: f64 = power_ref_wrap.unwrap();
            power = FromStr::from_str(power_.to_string().as_str()).unwrap();
        }
        number = number.powf(power);
        let number_str = number.to_string();
        let number_str = number_str.as_str();
        let number_result: f64 = FromStr::from_str(number_str).unwrap();

        let replacement_string = number_result.to_string();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        return formula;
    }
} */

// ROUND(number, digits)
// ROUNDUP(number, digits)
// ROUNDDOWN(number, digits)
/* #[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoundFunction {
    pub function_text: String,
    pub number: Option<f64>,
    pub number_ref: Option<String>,
    pub digits: Option<i8>,
    pub digits_ref: Option<i8>,
    pub option: RoundOption,
    pub data_map: Option<HashMap<String, String>>,
}
impl RoundFunction {
    pub fn defaults(function_text: &String, option: RoundOption, data_map: Option<HashMap<String, String>>) -> RoundFunction {
        // ROUND(number, digits)
        // ROUNDUP(number, digits)
        // ROUNDDOWN(number, digits)

        let matches: Captures;
        match option {
            RoundOption::Basic => {
                matches = RE_ROUND.captures(function_text).unwrap();
            },
            RoundOption::Up => {
                matches = RE_ROUND_UP.captures(function_text).unwrap();
            },
            RoundOption::Down => {
                matches = RE_ROUND_DOWN.captures(function_text).unwrap();
            },
        }
        let attr_number = matches.name("number");
        let attr_number_ref = matches.name("number_ref");
        let attr_digits = matches.name("digits");
        let attr_digits_ref = matches.name("digits_ref");

        let mut number_wrap: Option<f64> = None;
        let mut number_ref_wrap: Option<String> = None;
        let mut digits_wrap: Option<i8> = None;
        let mut digits_ref_wrap: Option<i8> = None;

        if attr_number.is_some() {
            let number: f64 = FromStr::from_str(attr_number.unwrap().as_str()).unwrap();
            number_wrap = Some(number);
        } else if attr_number_ref.is_some() {
            let number_ref = attr_number_ref.unwrap().as_str().to_string();
            number_ref_wrap = Some(number_ref);
        }
        if attr_digits.is_some() {
            let digits = attr_digits.unwrap().as_str();
            let digits: i8 = FromStr::from_str(digits).unwrap();
            digits_wrap = Some(digits);
        }
        if attr_digits_ref.is_some() {
            let digits = attr_digits_ref.unwrap().as_str();
            let digits_ref: i8 = FromStr::from_str(digits).unwrap();
            digits_ref_wrap = Some(digits_ref);
        }

        let obj = Self{
            function_text: function_text.clone(),
            number: number_wrap,
            number_ref: number_ref_wrap,
            digits: digits_wrap,
            digits_ref: digits_ref_wrap,
            option: option,
            data_map: data_map,
        };

        return obj
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>),
        option: RoundOption
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = RoundFunction::defaults(
            &function_text, option.clone(), None
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            match option {
                RoundOption::Basic => {
                    failed_functions.push(String::from(FUNCTION_ROUND));
                },
                RoundOption::Up => {
                    failed_functions.push(String::from(FUNCTION_ROUNDUP));
                },
                RoundOption::Down => {
                    failed_functions.push(String::from(FUNCTION_ROUNDDOWN));
                },
            }
            
        }
        return (number_fails, failed_functions);
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String, option: RoundOption) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = RoundFunction::defaults(
            &function_text, option,Some(data_map)
        );
        formula = concat_obj.replace(formula);
        return formula
    }
}
impl Function for RoundFunction {
    fn validate(&self) -> bool {
        let expr: Regex;
        match self.option {
            RoundOption::Basic => {
                expr = RE_ROUND.clone();
            },
            RoundOption::Up => {
                expr = RE_ROUND_UP.clone();
            },
            RoundOption::Down => {
                expr = RE_ROUND_DOWN.clone();
            },
        }
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    fn replace(&mut self, formula: String) -> String {
        let data_map = self.data_map.clone().unwrap();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();
        let option = self.option.clone();

        let number_wrap = self.number.clone();
        let number_ref_wrap = self.number_ref.clone();
        let digits_wrap = self.digits.clone();
        let digits_ref_wrap = self.digits_ref.clone();
        let mut number: f64 = 0.0;
        let mut digits: i8 = 2;
        if number_wrap.is_some() {
            number = number_wrap.unwrap();
        } else if number_ref_wrap.is_some() {
            let number_ref = number_ref_wrap.unwrap();
            let function_attr = FunctionAttribute::defaults(
                &number_ref, 
                Some(true)
            );
            let result = function_attr.replace(data_map.clone());
            let result = result.item_processed.clone();
            number = FromStr::from_str(result.unwrap().as_str()).unwrap();
        }
        if digits_wrap.is_some() {
            let digits_: i8 = digits_wrap.unwrap();
            digits = FromStr::from_str(digits_.to_string().as_str()).unwrap();
        }
        if digits_ref_wrap.is_some() {
            let digits_: i8 = digits_ref_wrap.unwrap();
            digits = FromStr::from_str(digits_.to_string().as_str()).unwrap();
        }
        match option {
            RoundOption::Basic => {
                number = round::half_away_from_zero(number, digits);
            },
            RoundOption::Up => {
                number = round::ceil(number, digits);
            },
            RoundOption::Down => {
                number = round::floor(number, digits);
            },
        }
        let number_str = number.to_string();
        let number_str = number_str.as_str();
        let number_result: f64 = FromStr::from_str(number_str).unwrap();

        let replacement_string = number_result.to_string();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        return formula;
    }
} */

// SQRT(number)
/* #[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SqrtFunction {
    pub function_text: String,
    pub number: Option<f64>,
    pub number_ref: Option<String>,
    pub data_map: Option<HashMap<String, String>>,
}
impl SqrtFunction {
    pub fn defaults(function_text: &String, data_map: Option<HashMap<String, String>>) -> SqrtFunction {
        // SQRT(number)

        let matches = RE_SQRT.captures(function_text).unwrap();
        let attr_number = matches.name("number");
        let attr_number_ref = matches.name("number_ref");

        let mut number_wrap: Option<f64> = None;
        let mut number_ref_wrap: Option<String> = None;

        if attr_number.is_some() {
            let number: f64 = FromStr::from_str(attr_number.unwrap().as_str()).unwrap();
            number_wrap = Some(number);
        } else if attr_number_ref.is_some() {
            let number_ref = attr_number_ref.unwrap().as_str().to_string();
            number_ref_wrap = Some(number_ref);
        }

        let obj = Self{
            function_text: function_text.clone(),
            number: number_wrap,
            number_ref: number_ref_wrap,
            data_map: data_map,
        };

        return obj
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = SqrtFunction::defaults(
            &function_text, None
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_SQRT));            
        }
        return (number_fails, failed_functions);
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = SqrtFunction::defaults(
            &function_text,Some(data_map)
        );
        formula = concat_obj.replace(formula);
        return formula
    }
}
impl Function for SqrtFunction {
    fn validate(&self) -> bool {
        let expr = RE_SQRT.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    fn replace(&mut self, formula: String) -> String {
        let data_map = self.data_map.clone().unwrap();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();

        let number_wrap = self.number.clone();
        let number_ref_wrap = self.number_ref.clone();
        let mut number: f64 = 0.0;
        if number_wrap.is_some() {
            number = number_wrap.unwrap();
        } else if number_ref_wrap.is_some() {
            let number_ref = number_ref_wrap.unwrap();
            let function_attr = FunctionAttribute::defaults(
                &number_ref, 
                Some(true)
            );
            let result = function_attr.replace(data_map.clone());
            let result = result.item_processed.clone();
            number = FromStr::from_str(result.unwrap().as_str()).unwrap();
        }
        number = number.sqrt();
        let number_str = number.to_string();
        let number_str = number_str.as_str();
        let number_result: f64 = FromStr::from_str(number_str).unwrap();

        let replacement_string = number_result.to_string();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        return formula;
    }
} */

// VALUE(text)
/* #[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ValueFunction {
    pub function_text: String,
    pub text: Option<String>,
    pub text_ref: Option<String>,
    pub data_map: Option<HashMap<String, String>>,
}
impl ValueFunction {
    pub fn defaults(function_text: &String, data_map: Option<HashMap<String, String>>) -> ValueFunction {
        // VALUE(text)

        let matches = RE_VALUE.captures(function_text).unwrap();
        let attr_text = matches.name("text");
        let attr_text_ref = matches.name("text_ref");

        let mut text_wrap: Option<String> = None;
        let mut text_ref_wrap: Option<String> = None;

        if attr_text.is_some() {
            let text = attr_text.unwrap().as_str().to_string();
            text_wrap = Some(text);
        } else if attr_text_ref.is_some() {
            let text_ref = attr_text_ref.unwrap().as_str().to_string();
            text_ref_wrap = Some(text_ref);
        }

        let obj = Self{
            function_text: function_text.clone(),
            text: text_wrap,
            text_ref: text_ref_wrap,
            data_map: data_map,
        };

        return obj
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>)
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = ValueFunction::defaults(
            &function_text, None
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_VALUE));            
        }
        return (number_fails, failed_functions);
    }
    pub fn do_replace(function_text: &String, data_map: HashMap<String, String>, mut formula: String) -> String {
        let data_map = data_map.clone();
        let mut concat_obj = ValueFunction::defaults(
            &function_text,Some(data_map)
        );
        formula = concat_obj.replace(formula);
        return formula
    }
}
impl Function for ValueFunction {
    fn validate(&self) -> bool {
        let expr = RE_VALUE.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    fn replace(&mut self, formula: String) -> String {
        let data_map = self.data_map.clone().unwrap();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();

        let text_wrap = self.text.clone();
        let text_ref_wrap = self.text_ref.clone();
        let mut text: String = String::from("");
        if text_wrap.is_some() {
            text = text_wrap.unwrap();
        } else if text_ref_wrap.is_some() {
            let text_ref = text_ref_wrap.unwrap();
            let function_attr = FunctionAttribute::defaults(
                &text_ref, 
                Some(true)
            );
            let result = function_attr.replace(data_map.clone());
            let result = result.item_processed.clone();
            text = result.unwrap().as_str().to_string();
        }
        let number: f64;
        text = text.replace("$", "").replace("€", "");
        text = text.replace("\"", "");
        let has_multiple_punc = text.find(",").is_some() && text.find(".").is_some();
        if has_multiple_punc {
            // , could be thousand sep or decimals
            // . could be thousand or decimals
            let index_dot = text.find(".").unwrap();
            let index_comma = text.find(",").unwrap();
            if index_comma > index_dot {
                // 1.200,98
                text = text.replace(".", "");
                text = text.replace(",", ".");
                // 1200.98
            } else {
                // 1,200.98
                text = text.replace(",", "");
                // 1200.98
            }
        } else {
            let index_comma = text.find(",");
            if index_comma.is_some() {
                // 920,98
                text = text.replace(",", ".");
                // 920.98
            }
        }
        number = FromStr::from_str(text.as_str()).unwrap();

        let replacement_string = number.to_string();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        return formula;
    }
} */

// TRUE() or FALSE()
/* #[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BooleanFunction {
    pub function_text: String,
    pub option: BooleanOption,
    pub data_map: Option<HashMap<String, String>>,
}
impl BooleanFunction {
    pub fn defaults(function_text: &String, option: BooleanOption, data_map: Option<HashMap<String, String>>) -> BooleanFunction {
        // TRUE() or FALSE()

        let obj = Self{
            function_text: function_text.clone(),
            option: option,
            data_map: data_map,
        };

        return obj
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>),
        option: BooleanOption
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = BooleanFunction::defaults(
            &function_text, option.clone(), None
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            match option {
                BooleanOption::False => {
                    failed_functions.push(String::from(FUNCTION_FALSE));
                },
                BooleanOption::True => {
                    failed_functions.push(String::from(FUNCTION_TRUE));
                },
            }
        }
        return (number_fails, failed_functions);
    }
    pub fn do_replace(
        function_text: &String, 
        mut formula: String,
        option: BooleanOption
    ) -> String {
        let mut concat_obj = BooleanFunction::defaults(
            &function_text, option, None
        );
        formula = concat_obj.replace(formula);
        return formula
    }
}
impl Function for BooleanFunction {
    fn validate(&self) -> bool {
        let expr = RE_BOOLEAN.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    fn replace(&mut self, formula: String) -> String {
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();
        let option = self.option.clone();
        let number: u8;
        match option {
            BooleanOption::False => {
                number = 0;
            },
            BooleanOption::True => {
                number = 1;
            },
        }
        let replacement_string = number.to_string();

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        return formula;
    }
}
 */