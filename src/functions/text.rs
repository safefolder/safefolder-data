use std::str::FromStr;
use regex::Regex;
use std::{collections::BTreeMap};
use lazy_static::lazy_static;

use crate::planet::PlanetError;

use crate::functions::*;

lazy_static! {
    pub static ref RE_CONCAT_ATTRS: Regex = Regex::new(r#"("[\w\s-]+")|(\d+)|(\{[\w\s]+\})|([A-Z]+\(["\w\s-]+\))"#).unwrap();
    pub static ref RE_FORMAT_ATTR: Regex = Regex::new(r#"FORMAT\([\s]{0,}((?P<attr>"[\{\}\w\s-]+")|(?P<func>[A-Z]+\(["{}\w\s-]+\)))[\s]{0,}\)"#).unwrap();
    pub static ref RE_FORMAT_COLUMNS: Regex = Regex::new(r"(\{[\w\s-]+\})").unwrap();
    pub static ref RE_JOINLIST_ATTRS: Regex = Regex::new(r#"(?P<array>\{[\w\s\d,"-]+\}),[\s]*(?P<sep>(".[^)]+)|([A-Z]+\(.[^()]+\)))"#).unwrap();
    pub static ref RE_LEN: Regex = Regex::new(r#"^LEN\([\s]*((?P<string>"[\w\s-]+")|(?P<ref>\{[\w\s]+\})|(?P<func>[A-Z]+\(.[^)]+\)))[\s]*\)"#).unwrap();
    pub static ref RE_LOWER: Regex = Regex::new(r#"^LOWER\([\s]*((?P<string>"[\w\s-]+")|(?P<ref>\{[\w\s]+\})|(?P<func>[A-Z]+\(.[^)]+\)))[\s]*\)"#).unwrap();
    pub static ref RE_UPPER: Regex = Regex::new(r#"^UPPER\([\s]*((?P<string>"[\w\s-]+")|(?P<ref>\{[\w\s]+\})|(?P<func>[A-Z]+\(.[^)]+\)))[\s]*\)"#).unwrap();
    pub static ref RE_SINGLE_ATTR: Regex = Regex::new(r#"("[\w\s-]+")|(\{[\w\s]+\})"#).unwrap();
    pub static ref RE_REPLACE: Regex = Regex::new(r#"^REPLACE\([\n\s\t]{0,}(?P<old_text>(("*[{}\w\s]+"*)|([A-Z]+\(.[^()]+\))))[\n\s\t]{0,},[\n\s\t]{0,}(?P<start_num>\d)[\n\s\t]{0,},[\n\s\t]{0,}(?P<num_chars>\d)[\n\s\t]{0,},[\n\s\t]{0,}(?P<new_text>(("*[{}\w\s]+"*)|([A-Z]+\(.[^()]+\))))[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_MID: Regex = Regex::new(r#"^MID\([\s\n\t]{0,}((?P<text>(("*[\w\s]+"*)|([A-Z]+\(.[^)]+\))))|(?P<text_ref>\{[\w\s]+\}))[\s\n\t]{0,},[\s\n\t]{0,}(?P<start_num>\d+)[\s\n\t]{0,},[\s\n\t]{0,}(?P<num_chars>\d+)[\s\n\t]{0,}\)"#).unwrap();
    pub static ref RE_REPT: Regex = Regex::new(r#"^REPT\([\s\n\t]{0,}((?P<text>(("[\w\s\W]+")|([A-Z]+\(.[^)]+\))))|(?P<text_ref>\{[\w\s]+\}))[\s\n\t]{0,},[\s\n\t]{0,}(?P<number_times>\d+)[\s\t\n]{0,}\)"#).unwrap();
    pub static ref RE_SUBSTITUTE: Regex = Regex::new(r#"^SUBSTITUTE\([\s\n\t]{0,}((?P<text>(("[\w\s]+")|([A-Z]+\(.[^)]+\))))|(?P<text_ref>(\{[\w\s]+\})))[\s\n\t]{0,},[\s\n\t]{0,}(?P<old_text>(("[\w\s]+")[\s\n\t]{0,})|([A-Z]+\(.[^)]+\))),[\s\n\t]{0,}(?P<new_text>(("[\w\s]+")|([A-Z]+\(.[^)]+\))))[\s\t\n]{0,}\)"#).unwrap();
    pub static ref RE_TRIM: Regex = Regex::new(r#"^TRIM\([\s\n\t]{0,}((?P<text>"[\w\s]+")|(?P<text_ref>\{[\w\s]+\})|(?P<func>[A-Z]+\(.[^()]+\)))[\s\n\t]{0,}\)"#).unwrap();
}

pub trait TextFunction {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError>;
    fn execute(&self) -> Result<String, PlanetError>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Concat {
    function: Option<FunctionParse>,
    data_map: Option<BTreeMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>,
    field_config_map: BTreeMap<String, FieldConfig>,
}
impl Concat {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<BTreeMap<String, String>>,
        field_config_map: &BTreeMap<String, FieldConfig>
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
impl TextFunction for Concat {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_CONCAT_ATTRS;
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
                function.attributes = Some(get_vector_regex_attributes(
                    expr.captures_iter(function_text.as_str()))
                );
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
        let mut attributes_processed: Vec<String> = Vec::new();
        // Case 1: In a formula field, no ref, CONCAT("My ", "places")
        // Case 2: In a formula field, ref, CONCAT("My "", {Column})
        // Case 3: I have function as attribute: CONCAT("My "", TRIM({Column}))
        for attribute_item in attributes {
            let attribute = attribute_item.get_value(data_map, &field_config_map)?;
            attributes_processed.push(attribute);
        }
        let result = attributes_processed.join("");
        return Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Trim {
    function: Option<FunctionParse>,
    data_map: Option<BTreeMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>,
    field_config_map: BTreeMap<String, FieldConfig>,
}
impl Trim {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<BTreeMap<String, String>>,
        field_config_map: &BTreeMap<String, FieldConfig>,
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
impl TextFunction for Trim {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_TRIM;
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
                let matches = expr.captures(function_text.as_str()).unwrap();
                let attr_text = matches.name("text");
                let attr_text_ref = matches.name("text_ref");
                let attr_text_func = matches.name("func");
                let mut attributes_: Vec<String> = Vec::new();
                if attr_text.is_some() {
                    let attr_text = attr_text.unwrap().as_str().to_string();
                    attributes_.push(prepare_string_attribute(attr_text));
                } else if attr_text_ref.is_some() {
                    // reference
                    let attr_text_ref = attr_text_ref.unwrap().as_str().to_string();
                    attributes_.push(attr_text_ref);
                } else {
                    // function
                    let attr_text_func = attr_text_func.unwrap().as_str().to_string();
                    attributes_.push(attr_text_func);
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
        let attributes = attributes.clone();
        let attribute_item = attributes[0].clone();
        let attribute = attribute_item.get_value(data_map, &field_config_map)?;
        let result = attribute.trim().to_string();
        return Ok(result);
    
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Format {
    function: Option<FunctionParse>,
    data_map: Option<BTreeMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>,
    field_config_map: BTreeMap<String, FieldConfig>,
}
impl Format {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<BTreeMap<String, String>>,
        field_config_map: &BTreeMap<String, FieldConfig>,
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
impl TextFunction for Format {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // FORMAT("Hello-{Column A}-45")
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_FORMAT_ATTR;
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
                let captures = expr.captures(function_text.as_str());
                let mut attributes_: Vec<String> = Vec::new();
                if captures.is_some() {
                    let captures = captures.unwrap();
                    let attr = captures.name("attr");
                    let func = captures.name("func");
                    if attr.is_some() {
                        let attr = attr.unwrap().as_str().to_string();
                        attributes_.push(attr);
                    }
                    if func.is_some() {
                        let func = func.unwrap().as_str().to_string();
                        attributes_.push(func);
                    }
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
        // FORMAT("Hello-{Column A}-45")
        let attributes = attributes.clone();
        let attribute_item = attributes[0].clone();
        let value = attribute_item.value.unwrap_or_default();
        // I don't have reference attribute, neither function attribute
        let attribute: String = value;
        let mut attribute_new = attribute.clone();
        let column_list = RE_FORMAT_COLUMNS.captures_iter(&attribute);
        for column in column_list {
            let mut column_attribute = column.get(0).unwrap().as_str().to_string();
            column_attribute = column_attribute.replace("{", "").replace("}", "");
            let item_value = data_map.get(&column_attribute);
            if item_value.is_some() {
                let item_value = item_value.unwrap();
                let replace_value = format!("{}{}{}", String::from("{"), &column_attribute, String::from("}"));
                attribute_new = attribute_new.replace(&replace_value, item_value);
            }
        }
        return Ok(attribute_new)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JoinList {
    function: Option<FunctionParse>,
    data_map: Option<BTreeMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>,
    field_config_map: BTreeMap<String, FieldConfig>,
}
impl JoinList {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<BTreeMap<String, String>>,
        field_config_map: &BTreeMap<String, FieldConfig>,
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
impl TextFunction for JoinList {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // JOINLIST({My Column}, ",")
        // JOINLIST({1, 2, 3}, ",") => "1,2,3"
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_JOINLIST_ATTRS;
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
                let attr_map = RE_JOINLIST_ATTRS.captures(function_text.as_str()).unwrap();
                // array is either a reference or a set/list of values
                let array = attr_map.name("array").unwrap().as_str().to_string();
                let sep = attr_map.name("sep").unwrap().as_str().replace("\"", "");
                let mut attributes_: Vec<String> = Vec::new();
                attributes_.push(prepare_string_attribute(array));
                attributes_.push(prepare_string_attribute(sep));
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
        // Excel array: {column_a, column_b; column_2_a, column_2_b} (like a table: columns and rows)
        // One dimension, simply separate by commas
        // A database column might have a list: Set, Links, etc...
        // JOINLIST({My Column}, ",")
        // JOINLIST({1, 2, 3}, ",") => "1,2,3"
        // {1, 2, 3} => array: "{1, 2, 3}" array_items: ["1", "2", "3"]
        // {"a", "b", "c"} => array: "{\"a\", \"b\", \"c\"}" array_items: ["a", "b", "c"]
        // {Column A} => array "{Column A}"" array_items: []
        let attributes = attributes.clone();
        let array = &attributes[0];
        let column_name = &array.name;
        let array_value = &array.value;
        let separator = &attributes[1];
        let separator_value = &separator.value;
        // {1,2,3} or {"a","b","c"} or {Column}
        let mut array_items_wrap: Option<Vec<String>> = None;
        if array_value.is_some() {
            let array_value = array_value.clone().unwrap();
            if array_value.find(",").is_some() {
                let array_value = array_value.replace("{", "").replace("}", "");
                let array_items = array_value.split(",");
                let array_items: Vec<&str> = array_items.collect();
                let array_items: Vec<String> = array_items.iter().map(|s| s.trim().to_string()).collect();
                array_items_wrap = Some(array_items);
            }
        }
        let mut replacement_string = String::from("");
        if array_items_wrap.is_some() {
            // {1,2,3} or {"a","b","c"}
            let array_items = array_items_wrap.unwrap();
            let sep = separator_value.clone().unwrap();
            replacement_string = array_items.join(&sep);
        } else {
            // Column A
            // I need to get list of items from data_map
            let column_name = column_name.clone().unwrap();
            let column = column_name.replace("{", "").replace("}", "");
            let array_items = data_map.get(&column);
            let sep = separator_value.clone().unwrap();
            if array_items.is_some() {
                let array_items = array_items.unwrap();
                let items = array_items.split(",");
                let items: Vec<&str> = items.collect();
                replacement_string = items.join(&sep);
            }
        }
        return Ok(replacement_string)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Length {
    function: Option<FunctionParse>,
    data_map: Option<BTreeMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>,
    field_config_map: BTreeMap<String, FieldConfig>,
}
impl Length {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<BTreeMap<String, String>>,
        field_config_map: &BTreeMap<String, FieldConfig>,
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
impl TextFunction for Length {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // LEN("my string") => 8
        // LEN({My Column}) => 23
        // LEN(TRIM(" hello world "))
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_LEN;
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
                let matches = expr.captures(function_text.as_str()).unwrap();
                let attr_string = matches.name("string");
                let attr_ref = matches.name("ref");
                let attr_func = matches.name("func");
                let mut attributes_: Vec<String> = Vec::new();
                if attr_string.is_some() {
                    let attr_string = attr_string.unwrap().as_str().to_string();
                    attributes_.push(attr_string);
                } else if attr_ref.is_some() {
                    let attr_ref = attr_ref.unwrap().as_str().to_string();
                    attributes_.push(attr_ref);
                } else if attr_func.is_some() {
                    let attr_func = attr_func.unwrap().as_str().to_string();
                    attributes_.push(attr_func);
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
        let attributes = attributes.clone();
        let attribute_item = attributes[0].clone();
        let attribute = attribute_item.get_value(data_map, &field_config_map)?;
        let length = attribute.len().to_string();
        return Ok(length)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Lower {
    function: Option<FunctionParse>,
    data_map: Option<BTreeMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>,
    field_config_map: BTreeMap<String, FieldConfig>,
}
impl Lower {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<BTreeMap<String, String>>,
        field_config_map: &BTreeMap<String, FieldConfig>,
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
impl TextFunction for Lower {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // LOWER("my string")
        // LOWER({My Column})
        // LOWER(TRIM(" HELLO WORLD "))
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_LOWER;
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
                let matches = expr.captures(function_text.as_str()).unwrap();
                let attr_string = matches.name("string");
                let attr_ref = matches.name("ref");
                let attr_func = matches.name("func");
                let mut attributes_: Vec<String> = Vec::new();
                if attr_string.is_some() {
                    let attr_string = attr_string.unwrap().as_str().to_string();
                    attributes_.push(attr_string);
                } else if attr_ref.is_some() {
                    let attr_ref = attr_ref.unwrap().as_str().to_string();
                    attributes_.push(attr_ref);
                } else if attr_func.is_some() {
                    let attr_func = attr_func.unwrap().as_str().to_string();
                    attributes_.push(attr_func);
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
        let attributes = attributes.clone();
        let attribute_item = attributes[0].clone();
        let attribute = attribute_item.get_value(data_map, &field_config_map)?;
        let result_lower = attribute.to_lowercase();
        return Ok(result_lower)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Upper {
    function: Option<FunctionParse>,
    data_map: Option<BTreeMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>,
    field_config_map: BTreeMap<String, FieldConfig>,
}
impl Upper {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<BTreeMap<String, String>>,
        field_config_map: &BTreeMap<String, FieldConfig>,
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
impl TextFunction for Upper {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // UPPER("my string")
        // UPPER({My Column})
        // UPPER(TRIM(" HELLO WORLD "))
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_UPPER;
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
                let matches = expr.captures(function_text.as_str()).unwrap();
                let attr_string = matches.name("string");
                let attr_ref = matches.name("ref");
                let attr_func = matches.name("func");
                let mut attributes_: Vec<String> = Vec::new();
                if attr_string.is_some() {
                    let attr_string = attr_string.unwrap().as_str().to_string();
                    attributes_.push(attr_string);
                } else if attr_ref.is_some() {
                    let attr_ref = attr_ref.unwrap().as_str().to_string();
                    attributes_.push(attr_ref);
                } else if attr_func.is_some() {
                    let attr_func = attr_func.unwrap().as_str().to_string();
                    attributes_.push(attr_func);
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
        let attributes = attributes.clone();
        let attribute_item = attributes[0].clone();
        let attribute = attribute_item.get_value(data_map, &field_config_map)?;
        let result_upper = attribute.to_uppercase();
        return Ok(result_upper)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Replace {
    function: Option<FunctionParse>,
    data_map: Option<BTreeMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>,
    field_config_map: BTreeMap<String, FieldConfig>,
}
impl Replace {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<BTreeMap<String, String>>,
        field_config_map: &BTreeMap<String, FieldConfig>,
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
impl TextFunction for Replace {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // REPLACE(old_text, start_num, num_chars, new_text)
        // REPLACE(old_text, start_num, num_chars, new_text)
        // REPLACE({My Column}, start_num, num_chars, new_text)
        // old_text     Required. Text in which you want to replace some characters.
        // start_num    Required. The position of the character in old_text that you want to replace with new_text.
        // num_chars    Required. The number of characters in old_text that you want REPLACE to replace with new_text.
        // new_text     Required. The text that will replace characters in old_text.
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_REPLACE;
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
                let matches = expr.captures(function_text.as_str()).unwrap();
                let old_text = matches.name("old_text").unwrap().as_str().to_string();
                let start_num= matches.name("start_num").unwrap().as_str().to_string();
                let num_chars = matches.name("num_chars").unwrap().as_str().to_string();
                let new_text = matches.name("new_text").unwrap().as_str().to_string();
                let mut attributes_: Vec<String> = Vec::new();
                attributes_.push(old_text);
                attributes_.push(start_num);
                attributes_.push(num_chars);
                attributes_.push(new_text);
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
        let attributes = attributes.clone();
        let old_text = attributes[0].clone();
        let old_text_value = old_text.get_value(data_map, &field_config_map)?;
        let start_num = attributes[1].clone();
        let start_num_value = start_num.get_value(data_map, &field_config_map)?;
        let start_num_value: u32 = FromStr::from_str(start_num_value.as_str()).unwrap();
        let num_chars = attributes[2].clone();
        let num_chars_value = num_chars.get_value(data_map, &field_config_map)?;
        let num_chars_value: u32 = FromStr::from_str(num_chars_value.as_str()).unwrap();
        let new_text = attributes[3].clone();
        let new_text_value = new_text.get_value(data_map, &field_config_map)?;
        let replacement_string: String;
        let mut piece: String = String::from("");
        for (i, item) in old_text_value.chars().enumerate() {
            let i = i as u32;
            let length = *&piece.len();
            let length = length as u32;
            if &i >= &start_num_value && length < num_chars_value {
                piece.push(item);
            }
        }
        replacement_string = old_text_value.replace(&piece, &new_text_value);
        return Ok(replacement_string)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mid {
    function: Option<FunctionParse>,
    data_map: Option<BTreeMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>,
    field_config_map: BTreeMap<String, FieldConfig>,
}
impl Mid {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<BTreeMap<String, String>>,
        field_config_map: &BTreeMap<String, FieldConfig>,
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
impl TextFunction for Mid {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // MID(text, start_num, num_chars)
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_MID;
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
                let matches = expr.captures(function_text.as_str()).unwrap();
                let attr_text = matches.name("text");
                let attr_text_ref = matches.name("text_ref");
                let attr_start_num = matches.name("start_num").unwrap().as_str().to_string();
                let attr_num_chars = matches.name("num_chars").unwrap().as_str().to_string();
                let mut attributes_: Vec<String> = Vec::new();
                if attr_text.is_some() {
                    let attr_text = attr_text.unwrap().as_str().to_string();
                    attributes_.push(prepare_string_attribute(attr_text));
                } else {
                    let attr_text_ref = attr_text_ref.unwrap().as_str().to_string();
                    attributes_.push(attr_text_ref);
                }
                attributes_.push(attr_start_num);
                attributes_.push(attr_num_chars);
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
        let attributes = attributes.clone();
        let text = attributes[0].clone();
        let text_value = text.get_value(data_map, &field_config_map)?;
        let start_num = attributes[1].clone();
        let start_num_value = start_num.get_value(data_map, &field_config_map)?;
        let start_num_value: usize = FromStr::from_str(start_num_value.as_str()).unwrap();
        let num_chars = attributes[2].clone();
        let num_chars_value = num_chars.get_value(data_map, &field_config_map)?;
        let num_chars_value: usize = FromStr::from_str(num_chars_value.as_str()).unwrap();
        
        let mut text_new = String::from("");
        for (i, char) in text_value.chars().enumerate() {
            let count = i+1;
            if count >= start_num_value && count <= num_chars_value {
                let char_ = char.to_string();
                text_new.push_str(char_.as_str());
            }
        }
        return Ok(text_new)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rept {
    function: Option<FunctionParse>,
    data_map: Option<BTreeMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>,
    field_config_map: BTreeMap<String, FieldConfig>,
}
impl Rept {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<BTreeMap<String, String>>,
        field_config_map: &BTreeMap<String, FieldConfig>,
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
impl TextFunction for Rept {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // REPT(text, number_times)
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_REPT;
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
                let matches = expr.captures(function_text.as_str()).unwrap();
                let attr_text = matches.name("text");
                let attr_text_ref = matches.name("text_ref");
                let attr_number_times = matches.name("number_times").unwrap().as_str().to_string();
                let mut attributes_: Vec<String> = Vec::new();
                if attr_text.is_some() {
                    let attr_text = attr_text.unwrap().as_str().to_string();
                    attributes_.push(prepare_string_attribute(attr_text));
                } else {
                    let attr_text_ref = attr_text_ref.unwrap().as_str().to_string();
                    attributes_.push(attr_text_ref);
                }
                attributes_.push(attr_number_times);
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
        let attributes = attributes.clone();
        let text = attributes[0].clone();
        let text_value = text.get_value(data_map, &field_config_map)?;
        let number_times = attributes[1].clone();
        let number_times_value = number_times.get_value(data_map, &field_config_map)?;
        let number_times_value: usize = FromStr::from_str(number_times_value.as_str()).unwrap();
        let text_value_str = text_value.as_str();
        let text_value_str = text_value_str.repeat(number_times_value);
        return Ok(text_value_str)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Substitute {
    function: Option<FunctionParse>,
    data_map: Option<BTreeMap<String, String>>,
    attributes: Option<Vec<FunctionAttributeItem>>,
    field_config_map: BTreeMap<String, FieldConfig>,
}
impl Substitute {
    pub fn defaults(
        function: Option<FunctionParse>, 
        data_map: Option<BTreeMap<String, String>>,
        field_config_map: &BTreeMap<String, FieldConfig>,
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
impl TextFunction for Substitute {
    fn handle(&mut self) -> Result<FunctionParse, PlanetError> {
        // SUBSTITUTE(text, old_text, new_text)
        let function_parse = &self.function.clone().unwrap();
        let data_map = self.data_map.clone();
        let expr = &RE_SUBSTITUTE;
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
                let matches = expr.captures(function_text.as_str()).unwrap();
                let attr_text = matches.name("text");
                let attr_text_ref = matches.name("text_ref");
                let attr_old_text = matches.name("old_text").unwrap().as_str().to_string();
                let attr_new_text = matches.name("new_text").unwrap().as_str().to_string();
                let mut attributes_: Vec<String> = Vec::new();
                if attr_text.is_some() {
                    let attr_text = attr_text.unwrap().as_str().to_string();
                    attributes_.push(prepare_string_attribute(attr_text));
                } else {
                    let attr_text_ref = attr_text_ref.unwrap().as_str().to_string();
                    attributes_.push(attr_text_ref);
                }
                attributes_.push(prepare_string_attribute(attr_old_text));
                attributes_.push(prepare_string_attribute(attr_new_text));
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
        let attributes = attributes.clone();
        let text = attributes[0].clone();
        let mut text_value = text.get_value(data_map, &field_config_map)?;
        let old_text = attributes[1].clone();
        let old_text_value = old_text.get_value(data_map, &field_config_map)?;
        let old_text_value = old_text_value.as_str();
        let new_text = attributes[2].clone();
        let new_text_value = new_text.get_value(data_map, &field_config_map)?;
        let new_text_value = new_text_value.as_str();

        text_value = text_value.replace(old_text_value, new_text_value);
        return Ok(text_value)
    }
}

pub fn check_string_equal(name: &String, value: &String) -> Result<bool, PlanetError> {
    let check: bool;
    if name.to_lowercase() == value.to_lowercase() {
        check = true
    } else {
        check = false
    }
    return Ok(check)
}

pub fn check_str_equal(name: &str, value: &str) -> Result<bool, PlanetError> {
    let check: bool;
    if name.to_lowercase() == value.to_lowercase() {
        check = true
    } else {
        check = false
    }
    return Ok(check)
}
