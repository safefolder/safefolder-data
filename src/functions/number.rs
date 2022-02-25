use std::str::FromStr;
use regex::{Regex, Captures};
use std::{collections::BTreeMap};
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;
use math::round;
use crate::functions::*;

lazy_static! {
    pub static ref RE_NUMBER_ATTRS: Regex = Regex::new(r#"("[\w\s-]*")|([+-]?[0-9]+\.?[0-9]*|\.[0-9]+)|(\{[\w\s]+\})|([A-Z]+\(["\w\s-]+\))|(null)"#).unwrap();
    pub static ref RE_CEILING: Regex = Regex::new(r#"^CEILING\([\s\n\t]{0,}((?P<number>(([+-]?[0-9]+\.?[0-9]*|\.[0-9]+)|([A-Z]+\(.[^(]+\))))|(?P<number_ref>\{[\w\s]+\}))[\s\n\t]{0,},[\s\n\t]{0,}(?P<significance>(([+-]?[0-9]+\.?[0-9]*|\.[0-9]+)|(\{[\w\s]+\})|([A-Z]+\(.+\))))[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_FLOOR: Regex = Regex::new(r#"^FLOOR\([\s\n\t]{0,}((?P<number>(([+-]?[0-9]+\.?[0-9]*|\.[0-9]+)|([A-Z]+\(.[^)]*\))))|(?P<number_ref>\{[\w\s]+\}))[\s\n\t]{0,},[\s\n\t]{0,}(?P<significance>(([+-]?[0-9]+\.?[0-9]*|\.[0-9]+)|(\{[\w\s]+\})|([A-Z]+\(.*\))))[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_COUNT: Regex = Regex::new(r#"^COUNT\([\s\n\t]{0,}(?P<attrs>[\w\W\s\n\t]{0,})[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_COUNTA: Regex = Regex::new(r#"^COUNTA\([\s\n\t]{0,}(?P<attrs>[\w\W\s\n\t]{0,})[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_COUNTALL: Regex = Regex::new(r#"^COUNTALL\([\s\n\t]{0,}(?P<attrs>[\w\W\s\n\t]{0,})[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_EVEN: Regex = Regex::new(r#"^EVEN\([\s\n\t]{0,}(?P<number>(([+-]?[0-9]+\.?[0-9]*|\.[0-9]+)|([A-Z]+\(.[^)]*\))))[\s\n\t]{0,}\)|EVEN\([\s\n\t]{0,}(?P<number_ref>\{[\w\s]+\})[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_EXP: Regex = Regex::new(r#"^EXP\([\s\n\t]{0,}(?P<number>(([+-]?[0-9]+\.?[0-9]*|\.[0-9]+)|([A-Z]+\(.[^)]*\))))[\s\n\t]{0,}\)|^EXP\([\s\n\t]{0,}(?P<number_ref>\{[\w\s]+\})[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_INT: Regex = Regex::new(r#"^INT\([\s\n\t]{0,}(?P<number>(([+-]?[0-9]+\.?[0-9]*|\.[0-9]+)|([A-Z]+\(.[^)]*\))))[\s\n\t]{0,}\)|^INT\([\s\n\t]{0,}(?P<number_ref>\{[\w\s]+\})[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_LOG: Regex = Regex::new(r#"^LOG\([\s\n\t]{0,}(?P<number>(([+-]?[0-9]+\.?[0-9]*|\.[0-9]+)|(\{[\w\s]*\})|([A-Z]+\(.[^)]*\))))[\n\s\t]{0,},{0,}[\n\s\t]{0,}(?P<base>((\d+)|(\{[\w\s]+\})|([A-Z]+\(.[^)]*\)))){0,}[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_MOD: Regex = Regex::new(r#"^MOD\([\s\n\t]{0,}(?P<number>(([+-]?[0-9]+\.?[0-9]*|\.[0-9]+)|(\{[\w\s]+\})|([A-Z]+\(.[^(]*\))))[\n\s\t]{0,},[\n\s\t]{0,}(?P<divisor>((\d+)|(\{[\w\s]+\})|([A-Z]+\(.[^(]*\)))){0,}[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_POWER: Regex = Regex::new(r#"^POWER\([\s\n\t]{0,}(?P<number>(([+-]?[0-9]+\.?[0-9]*|\.[0-9]+)|(\{[\w\s]+\})|([A-Z]+\(.[^)]*\))))[\n\s\t]{0,},[\n\s\t]{0,}(?P<power>(([+-]?[0-9]+\.?[0-9]*|\.[0-9]+)|(\{[\w\s]+\})|([A-Z]+\(.[^)]*\)))){0,}[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_ROUND: Regex = Regex::new(r#"^ROUND\([\s\n\t]{0,}(?P<number>(([+-]?[0-9]+\.?[0-9]*|\.[0-9]+)|(\{[\w\s]+\})|([A-Z]+\(.[^(]*\))))[\n\s\t]{0,},[\n\s\t]{0,}(?P<digits>((\d+)|(\{[\w\s]+\})|([A-Z]+\(.[^(]*\)))){0,}[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_ROUND_UP: Regex = Regex::new(r#"^ROUNDUP\([\s\n\t]{0,}(?P<number>(([+-]?[0-9]+\.?[0-9]*|\.[0-9]+)|(\{[\w\s]+\})|([A-Z]+\(.[^(]*\))))[\n\s\t]{0,},[\n\s\t]{0,}(?P<digits>((\d+)|(\{[\w\s]+\})|([A-Z]+\(.[^(]*\)))){0,}[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_ROUND_DOWN: Regex = Regex::new(r#"^ROUNDDOWN\([\s\n\t]{0,}(?P<number>(([+-]?[0-9]+\.?[0-9]*|\.[0-9]+)|(\{[\w\s]+\})|([A-Z]+\(.[^(]*\))))[\n\s\t]{0,},[\n\s\t]{0,}(?P<digits>((\d+)|(\{[\w\s]+\})|([A-Z]+\(.[^(]*\)))){0,}[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_SQRT: Regex = Regex::new(r#"^SQRT\([\s\n\t]{0,}(?P<number>(([+-]?[0-9]+\.?[0-9]*|\.[0-9]+)|(\{[\w\s]+\})|([A-Z]+\(.[^)]*\))))[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_VALUE: Regex = Regex::new(r#"^VALUE\([\s\n\t]{0,}(?P<text>(("[\w\d,.{0,}\$€{0,}]+")|(\{[\w\s]+\})|([A-Z]+\(.[^)]*\))))[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_BOOLEAN: Regex = Regex::new(r#"^TRUE\(\)|^FALSE\(\)|TRUE|FALSE"#).unwrap();
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
    data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
    attributes: Option<Vec<FunctionAttributeItem>>,
    field_config_map: BTreeMap<String, ColumnConfig>,
}
impl Ceiling {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
        field_config_map: &BTreeMap<String, ColumnConfig>
    ) -> Self {
        let field_config_map = field_config_map.clone();
        return Self{
            function: function, 
            data_map: data_map, 
            attributes: None,
            field_config_map: field_config_map
        };
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
        let field_config_map = self.field_config_map.clone();
        let attribute_item = attributes[0].clone();
        let number_string = attribute_item.get_value(data_map, &field_config_map)?;
        let number_str = number_string.as_str();
        let mut number: f64 = FromStr::from_str(number_str).unwrap();
        let significance_item = attributes[1].clone();
        let significance_string = significance_item.get_value(data_map, &field_config_map)?;
        let mut significance: i8 = FromStr::from_str(&significance_string.as_str()).unwrap();
        significance = significance - 1;
        number = round::ceil(number, significance);
        let result = number.to_string();
        return Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Floor {
    function: Option<FunctionParse>,
    data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
    attributes: Option<Vec<FunctionAttributeItem>>,
    field_config_map: BTreeMap<String, ColumnConfig>
}
impl Floor {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
        field_config_map: &BTreeMap<String, ColumnConfig>
    ) -> Self {
        let field_config_map = field_config_map.clone();
        return Self{
            function: function, 
            data_map: data_map, 
            attributes: None,
            field_config_map: field_config_map
        };
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
        let field_config_map = self.field_config_map.clone();
        let attribute_item = attributes[0].clone();
        let number_string = attribute_item.get_value(data_map, &field_config_map)?;
        let number_str = number_string.as_str();
        let mut number: f64 = FromStr::from_str(number_str).unwrap();
        let significance_item = attributes[1].clone();
        let significance_string = significance_item.get_value(data_map, &field_config_map)?;
        let mut significance: i8 = FromStr::from_str(&significance_string.as_str()).unwrap();
        significance = significance - 1;
        number = round::floor(number, significance);
        return Ok(number.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Count {
    function: Option<FunctionParse>,
    data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
    attributes: Option<Vec<FunctionAttributeItem>>,
    field_config_map: BTreeMap<String, ColumnConfig>
}
impl Count {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
        field_config_map: &BTreeMap<String, ColumnConfig>
    ) -> Self {
        let field_config_map = field_config_map.clone();
        return Self{
            function: function, 
            data_map: data_map, 
            attributes: None,
            field_config_map: field_config_map
        };
    }
}
impl NumberFunction for Count {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // COUNT(value1, value2, ...))
        // I need to pass to execute a list of attributes!!! not a single attribute
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_COUNT;
        let expr_attrs = &RE_NUMBER_ATTRS;
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
                let mut attributes_: Vec<String> = Vec::new();
                let matches_iter = expr_attrs.captures_iter(function_text.as_str());
                for match_ in matches_iter {
                    let match_str = match_.get(0).unwrap().as_str();
                    let match_string = match_str.to_string();
                    attributes_.push(match_string);
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
        let field_config_map = self.field_config_map.clone();
        let mut items: Vec<String> = Vec::new();
        for attribute in attributes {
            let attribute_value = attribute.get_value(data_map, &field_config_map)?;
            items.push(attribute_value);
        }
        let count = items.len();
        return Ok(count.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CountA {
    function: Option<FunctionParse>,
    data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
    attributes: Option<Vec<FunctionAttributeItem>>,
    field_config_map: BTreeMap<String, ColumnConfig>
}
impl CountA {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
        field_config_map: &BTreeMap<String, ColumnConfig>
    ) -> Self {
        let field_config_map = field_config_map.clone();
        return Self{
            function: function, 
            data_map: data_map, 
            attributes: None,
            field_config_map: field_config_map
        };
    }
}
impl NumberFunction for CountA {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // COUNT(value1, value2, ...))
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_COUNTA;
        let expr_attrs = &RE_NUMBER_ATTRS;
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
                let mut attributes_: Vec<String> = Vec::new();
                let matches_iter = expr_attrs.captures_iter(function_text.as_str());
                for match_ in matches_iter {
                    let match_str = match_.get(0).unwrap().as_str();
                    let match_string = match_str.to_string();
                    attributes_.push(match_string);
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
        let field_config_map = self.field_config_map.clone();
        let mut items: Vec<String> = Vec::new();
        for attribute in attributes {
            let attribute_value = attribute.get_value(data_map, &field_config_map)?;
            let attr_type = attribute.attr_type;
            match attr_type {
                AttributeType::Text => {
                    items.push(attribute_value)
                },
                _ => {
                }
            }
        }
        let count = items.len();
        return Ok(count.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CountAll {
    function: Option<FunctionParse>,
    data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
    attributes: Option<Vec<FunctionAttributeItem>>,
    field_config_map: BTreeMap<String, ColumnConfig>,
}
impl CountAll {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
        field_config_map: &BTreeMap<String, ColumnConfig>,
    ) -> Self {
        let field_config_map = field_config_map.clone();
        return Self{
            function: function, 
            data_map: data_map, 
            attributes: None,
            field_config_map: field_config_map
        };
    }
}
impl NumberFunction for CountAll {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // COUNTALL(value1, value2, ...))
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_COUNTALL;
        let expr_attrs = &RE_NUMBER_ATTRS;
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
                let mut attributes_: Vec<String> = Vec::new();
                let matches_iter = expr_attrs.captures_iter(function_text.as_str());
                for match_ in matches_iter {
                    let match_str = match_.get(0).unwrap().as_str();
                    let match_string = match_str.to_string();
                    attributes_.push(match_string);
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
        let count = attributes.len();
        return Ok(count.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Even {
    function: Option<FunctionParse>,
    data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
    attributes: Option<Vec<FunctionAttributeItem>>,
    field_config_map: BTreeMap<String, ColumnConfig>
}
impl Even {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
        field_config_map: &BTreeMap<String, ColumnConfig>
    ) -> Self {
        let field_config_map = field_config_map.clone();
        return Self{
            function: function, 
            data_map: data_map, 
            attributes: None,
            field_config_map: field_config_map
        };
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
        let field_config_map = self.field_config_map.clone();
        let attribute_item = attributes[0].clone();
        let attribute_value = attribute_item.get_value(data_map, &field_config_map)?;
        let number: f64 = FromStr::from_str(attribute_value.as_str()).unwrap();
        let mut rounded_int: i32;
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
    data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
    attributes: Option<Vec<FunctionAttributeItem>>,
    field_config_map: BTreeMap<String, ColumnConfig>,
}
impl Exp {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
        field_config_map: &BTreeMap<String, ColumnConfig>
    ) -> Self {
        let field_config_map = field_config_map.clone();
        return Self{
            function: function, 
            data_map: data_map, 
            attributes: None,
            field_config_map: field_config_map
        };
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
        let field_config_map = self.field_config_map.clone();
        let attribute_item = attributes[0].clone();
        let attribute_value = attribute_item.get_value(data_map, &field_config_map)?;
        let number: f64 = FromStr::from_str(attribute_value.as_str()).unwrap();
        let number_result: f64;
        number_result = number.exp();
        return Ok(number_result.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Int {
    function: Option<FunctionParse>,
    data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
    attributes: Option<Vec<FunctionAttributeItem>>,
    field_config_map: BTreeMap<String, ColumnConfig>
}
impl Int {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
        field_config_map: &BTreeMap<String, ColumnConfig>,
    ) -> Self {
        let field_config_map = field_config_map.clone();
        return Self{
            function: function, 
            data_map: data_map, 
            attributes: None,
            field_config_map: field_config_map
        };
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
        let field_config_map = self.field_config_map.clone();
        let attribute_item = attributes[0].clone();
        let attribute_value = attribute_item.get_value(data_map, &field_config_map)?;
        let mut number: f64 = FromStr::from_str(attribute_value.as_str()).unwrap();
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
    data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
    attributes: Option<Vec<FunctionAttributeItem>>,
    field_config_map: BTreeMap<String, ColumnConfig>
}
impl Log {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
        field_config_map: &BTreeMap<String, ColumnConfig>
    ) -> Self {
        let field_config_map = field_config_map.clone();
        return Self{
            function: function, 
            data_map: data_map, 
            attributes: None,
            field_config_map: field_config_map
        };
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
        let field_config_map = self.field_config_map.clone();
        let attribute_item = attributes[0].clone();
        let attribute_value = attribute_item.get_value(data_map, &field_config_map)?;
        let mut number: f64 = FromStr::from_str(attribute_value.as_str()).unwrap();
        let mut base: f64 = 10.0;
        if attributes.len() == 2 {
            let base_item = attributes[1].clone();
            let base_value = base_item.get_value(data_map, &field_config_map)?;
            base = FromStr::from_str(base_value.as_str()).unwrap();
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
    data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
    attributes: Option<Vec<FunctionAttributeItem>>,
    field_config_map: BTreeMap<String, ColumnConfig>,
}
impl Mod {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
        field_config_map: &BTreeMap<String, ColumnConfig>
    ) -> Self {
        let field_config_map = field_config_map.clone();
        return Self{
            function: function, 
            data_map: data_map, 
            attributes: None,
            field_config_map: field_config_map
        };
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
        let field_config_map = self.field_config_map.clone();
        let attribute_item = attributes[0].clone();
        let attribute_value = attribute_item.get_value(data_map, &field_config_map)?;
        let mut number: f64 = FromStr::from_str(attribute_value.as_str()).unwrap();
        let mut divisor: f64 = 10.0;
        if attributes.len() == 2 {
            let divisor_item = attributes[1].clone();
            let divisor_value = divisor_item.get_value(data_map, &field_config_map)?;
            divisor = FromStr::from_str(divisor_value.as_str()).unwrap();
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
    data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
    attributes: Option<Vec<FunctionAttributeItem>>,
    field_config_map: BTreeMap<String, ColumnConfig>
}
impl Power {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
        field_config_map: &BTreeMap<String, ColumnConfig>
    ) -> Self {
        let field_config_map = field_config_map.clone();
        return Self{
            function: function, 
            data_map: data_map, 
            attributes: None,
            field_config_map: field_config_map
        };
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
        let field_config_map = self.field_config_map.clone();
        let attribute_item = attributes[0].clone();
        let attribute_value = attribute_item.get_value(data_map, &field_config_map)?;
        // let is_reference = attribute_item.is_reference;
        // let is_power_reference = power_item.is_reference;
        let mut number: f64 = FromStr::from_str(attribute_value.as_str()).unwrap();
        let mut power: f64 = 10.0;
        if attributes.len() == 2 {
            let power_item = attributes[1].clone();
            let power_value = power_item.get_value(data_map, &field_config_map)?;
            power = FromStr::from_str(power_value.as_str()).unwrap();
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
    data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
    attributes: Option<Vec<FunctionAttributeItem>>,
    field_config_map: BTreeMap<String, ColumnConfig>
}
impl Round {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
        field_config_map: &BTreeMap<String, ColumnConfig>
    ) -> Self {
        let field_config_map = field_config_map.clone();
        return Self{
            function: function, 
            data_map: data_map, 
            attributes: None,
            field_config_map: field_config_map
        };
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
        let field_config_map = self.field_config_map.clone();
        let attribute_item = attributes[0].clone();
        let attribute_value = attribute_item.get_value(data_map, &field_config_map)?;
        // let is_reference = attribute_item.is_reference;
        // let is_digits_reference = digits_item.is_reference;
        let mut number: f64 = FromStr::from_str(attribute_value.as_str()).unwrap();
        let mut digits: i8 = 2;
        if attributes.len() == 2 {
            let digits_item = attributes[1].clone();
            let digits_value = digits_item.get_value(data_map, &field_config_map)?;
            digits = FromStr::from_str(digits_value.as_str()).unwrap();
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
    data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
    attributes: Option<Vec<FunctionAttributeItem>>,
    field_config_map: BTreeMap<String, ColumnConfig>,
}
impl Sqrt {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
        field_config_map: &BTreeMap<String, ColumnConfig>
    ) -> Self {
        let field_config_map = field_config_map.clone();
        return Self{
            function: function, 
            data_map: data_map, 
            attributes: None,
            field_config_map: field_config_map
        };
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
        let field_config_map = self.field_config_map.clone();
        let attribute_item = attributes[0].clone();
        let attribute_value = attribute_item.get_value(data_map, &field_config_map)?;
        let mut number: f64 = FromStr::from_str(attribute_value.as_str()).unwrap();
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
    data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
    attributes: Option<Vec<FunctionAttributeItem>>,
    field_config_map: BTreeMap<String, ColumnConfig>,
}
impl Value {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
        field_config_map: &BTreeMap<String, ColumnConfig>,
    ) -> Self {
        let field_config_map = field_config_map.clone();
        return Self{
            function: function, 
            data_map: data_map, 
            attributes: None,
            field_config_map: field_config_map,
        };
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
        let field_config_map = self.field_config_map.clone();
        let attribute_item = attributes[0].clone();
        let mut text = attribute_item.get_value(data_map, &field_config_map)?;
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
    data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
    attributes: Option<Vec<FunctionAttributeItem>>,
    field_config_map: BTreeMap<String, ColumnConfig>,
}
impl Boolean {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
        field_config_map: &BTreeMap<String, ColumnConfig>,
    ) -> Self {
        let field_config_map = field_config_map.clone();
        return Self{
            function: function, 
            data_map: data_map, 
            attributes: None,
            field_config_map: field_config_map
        };
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
        FormulaOperator::Eq => {
            if value == compare_to {
                check = true;
            }
        },
        _ => {}
    }
    return Ok(check)
}
