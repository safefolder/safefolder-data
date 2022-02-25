pub mod constants;
pub mod text;
pub mod date;
pub mod structure;
pub mod number;
pub mod collections;

use std::str::FromStr;
use std::collections::{BTreeMap,HashMap};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use regex::{Regex, CaptureMatches};
use tr::tr;
use xlformula_engine::{calculate, parse_formula, NoReference, NoCustomFunction};

use crate::storage::ConfigStorageColumn;
use crate::storage::folder::{DbData, TreeFolder, get_value_list};
use crate::commands::folder::config::ColumnConfig;
use crate::functions::constants::*;
use crate::functions::text::*;
use crate::functions::date::*;
use crate::functions::number::*;
use crate::functions::collections::*;
use crate::functions::structure::*;
use crate::planet::PlanetError;

lazy_static! {
    pub static ref RE_FORMULA_FUNCTIONS: Regex = Regex::new(r#"([a-zA-Z]+\(.+\))"#).unwrap();
    pub static ref RE_ACHIEVER_FUNCTIONS: Regex = Regex::new(r#"(?P<func>[A-Z]+\(.+\))|(?P<func_empty>[A-Z]+\(\))"#).unwrap();
    pub static ref RE_ACHIEVER_FUNCTIONS_PARTS : Regex = Regex::new(r#"(?P<func>[A-Z]+\([\w\s\d"\-\+:,\{\}.€\$=;]+\))|(?P<func_empty>[A-Z]+\(\))"#).unwrap();
    pub static ref RE_FORMULA_VALID: Regex = Regex::new(r#"(?im:\{[\w\s]+\})"#).unwrap();
    pub static ref RE_EMBED_FUNC: Regex = Regex::new(r#"\((?P<func_embed>[A-Z]+)"#).unwrap();
    pub static ref RE_STRING_MATCH: Regex = Regex::new(r#"(?P<string_match>"[\w\s]+"[\s\n\t]{0,}[=><][\s\n\t]{0,}"[\w\s]+")"#).unwrap();
    pub static ref RE_FORMULA_QUERY: Regex = Regex::new(r#"(?P<assign>\{[\s\w]+\}[\s\t]{0,}(?P<log_op>=|>|<|>=|<=)[\s\t]{0,}.+)|(?P<op>AND|OR|NOT|XOR)\((?P<attrs>.+)\)"#).unwrap();
    pub static ref RE_FORMULA_FIELD_FUNCTIONS: Regex = Regex::new(r#"(?P<func>[A-Z]+[("\d,-.;_:+$€\s\w{})]+)"#).unwrap();
    pub static ref RE_FUNCTION_ATTRS_OLD: Regex = Regex::new(r#"("[\w\s-]+")|(\{[\w\s]+\})|([A-Z]+\(["\w\s]+\))|([+-]?[0-9]+\.?[0-9]*|\.[0-9]+)"#).unwrap();
    pub static ref RE_FUNCTION_ATTRS: Regex = Regex::new(r#"[A-Z]+\((?P<attrs>.+)\)"#).unwrap();
    pub static ref RE_ATTR_TYPE_RESOLVE: Regex = Regex::new(r#"(?P<ref>\{[\w\s]+\}$)|(?P<formula>[A-Z]+\(.+\).*)|(?P<bool>TRUE|FALSE)|(?P<number>^[+-]?[0-9]+\.?[0-9]*|^\.[0-9]+)|(?P<null>null)|(?P<assign>\{[\w\s]+\}[\s]*[=<>]+[\s]*((\d+)|("*[\w\s]+"*)))|(?P<string>\\{0,}"*[,;_.\\$€:\-\+\{\}\w\s-]*\\{0,}"*)"#).unwrap();
    pub static ref RE_FORMULA_FUNCTION_PIECES: Regex = Regex::new(r#"[A-Z]+\(((.[^()]*)|())\)"#).unwrap();
    pub static ref RE_FORMULA_FUNCTION_VARIABLES: Regex = Regex::new(r#"(?P<func>\$func_\d)"#).unwrap();
    pub static ref RE_FORMULA_VARIABLES: Regex = Regex::new(r#"(?P<formula>\$formula_\d)"#).unwrap();
    pub static ref RE_FORMULA_ASSIGN: Regex = Regex::new(r#"^(?P<assign>(?P<name>\{[\s\w]+\})[\s\t]{0,}(?P<op>=|>|<|>=|<=)[\s\t]{0,}((?P<formula>\$formula_*\d*)|(?P<value>"*[\.\w\d\s]+"*)))"#).unwrap();
}

// achiever planet functions
pub const FORMULA_FUNCTIONS: [&str; 50] = [
    FUNCTION_CONCAT,
    FUNCTION_FORMAT,
    FUNCTION_JOINLIST,
    FUNCTION_LENGTH,
    FUNCTION_LOWER,
    FUNCTION_UPPER,
    FUNCTION_REPLACE,
    FUNCTION_DATE,
    FUNCTION_DATEFMT,
    FUNCTION_DAY,
    FUNCTION_DAYS,
    FUNCTION_HOUR,
    FUNCTION_MONTH,
    FUNCTION_NOW,
    FUNCTION_SECOND,
    FUNCTION_MINUTE,
    FUNCTION_TODAY,
    FUNCTION_WEEK,
    FUNCTION_WEEKDAY,
    FUNCTION_YEAR,
    FUNCTION_IF,
    FUNCTION_MID,
    FUNCTION_REPT,
    FUNCTION_SUBSTITUTE,
    FUNCTION_TRIM,
    FUNCTION_CEILING,
    FUNCTION_COUNT,
    FUNCTION_COUNTA,
    FUNCTION_COUNTALL,
    FUNCTION_EVEN,
    FUNCTION_EXP,
    FUNCTION_FLOOR,
    FUNCTION_INT,
    FUNCTION_LOG,
    FUNCTION_MAX,
    FUNCTION_MIN,
    FUNCTION_MOD,
    FUNCTION_POWER,
    FUNCTION_ROUND,
    FUNCTION_ROUNDDOWN,
    FUNCTION_ROUNDUP,
    FUNCTION_SQRT,
    FUNCTION_VALUE,
    FUNCTION_CREATED_TIME,
    FUNCTION_DATEADD,
    FUNCTION_DATEDIF,
    FUNCTION_LAST_MODIFIED_TIME,
    FUNCTION_RECORD_ID,
    FUNCTION_TRUE,
    FUNCTION_FALSE,
];

pub trait Function {
    fn validate(&self) -> bool;
    fn replace(&mut self, formula: String) -> String;
}

pub fn check_achiever_function(function_text: String) -> bool {
    let mut check = false;
    for function_item in FORMULA_FUNCTIONS {
        let function_name = get_function_name(function_text.clone());
        if function_item.to_lowercase() == function_name.to_string().to_lowercase() {
            check = true;
            break
        }
    }
    return check;
}

pub fn get_function_name(function_text: String) -> String {
    let function_name_pieces = function_text.split("(");
    let function_name_pieces: Vec<&str> = function_name_pieces.collect();
    let function_name = function_name_pieces[0].to_string();
    return function_name
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionAttribute{
    pub id: String,
    pub remove_quotes: Option<bool>,
    pub item_processed: Option<String>,
    pub skip_curl: Option<bool>,
}
impl FunctionAttribute {
    pub fn defaults(id: &String, remove_quotes: Option<bool>, skip_curl: Option<bool>) -> Self {
        let mut remove_quotes_value: bool = false;
        if remove_quotes.is_some() {
            remove_quotes_value = true;
        }
        let obj = Self{
            id: id.clone(),
            remove_quotes: Some(remove_quotes_value),
            item_processed: None,
            skip_curl: skip_curl,
        };
        return obj
    }
    pub fn replace(&self, data_map: &BTreeMap<String, Vec<BTreeMap<String, String>>>) -> Self {
        // data_map :: {id} => Value
        let mut item = self.id.clone();
        let remove_quotes = self.remove_quotes.unwrap();
        if remove_quotes == true {
            item = item.replace("\"", "");
        }
        let item_string: String;
        let skip_curl = self.skip_curl;
        let mut obj = self.clone();
        if skip_curl.is_none() {
            let item_find = item.find("{");
            if item_find.is_some() && item_find.unwrap() == 0 {
                item = item.replace("{", "").replace("}", "");
                let item_value = data_map.get(&item);
                if item_value.is_some() {
                    let item_value = item_value.unwrap().clone();
                    let item_value = get_value_list(&item_value);
                    if item_value.is_some() {
                        item_string = item_value.unwrap();
                        obj.item_processed = Some(item_string);
                    }
                }
            } else {
                item_string = item.to_string();
                obj.item_processed = Some(item_string);
            }
        } else {
            let item_value = data_map.get(&item);
            if item_value.is_some() {
                let item_value = item_value.unwrap().clone();
                let item_value = get_value_list(&item_value);
                if item_value.is_some() {
                    item_string = item_value.unwrap();
                    obj.item_processed = Some(item_string);
                }
            }
        }        
        return obj;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FormulaProcessMode {
    Validate,
    Execute
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AttributeType {
    Text,
    Number,
    Bool,
    Date,
    Null,
}

// {Field} = "pepito"
// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct AttributeAssign(
//     pub String, 
//     pub FormulaOperator, 
//     pub String
// );

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttributeAssign {
    pub name: String, 
    pub op: FormulaOperator, 
    pub value: String,
    pub assign_type: AttributeType,
}
impl AttributeAssign {
    pub fn defaults(name: &String, op: &FormulaOperator, value: &String, assign_type: &AttributeType) -> Self {
        let obj = AttributeAssign{
            name: name.clone(),
            value: value.clone(),
            op: op.clone(),
            assign_type: assign_type.clone(),
        };
        return obj
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FormulaQueryCompiled {
    pub function: Option<CompiledFunction>,
    pub assignment: Option<FunctionAttributeItem>,
}
impl FormulaQueryCompiled {
    pub fn defaults(function: Option<CompiledFunction>, assignment: Option<FunctionAttributeItem>) -> Self {
        let obj = Self{
            function: function,
            assignment: assignment,
        };
        return obj
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Formula {
    pub functions: Option<BTreeMap<String, CompiledFunction>>,
    pub assignment: Option<AttributeAssign>,
    pub formula: String,
    pub data: Option<BTreeMap<String, String>>,
}
impl Formula {
    pub fn defaults(
        formula: &String,
        formula_format: &String,
        table: Option<DbData>,
        properties_map: Option<HashMap<String, ColumnConfig>>,
        db_folder: Option<TreeFolder>,
        folder_name: Option<String>,
        is_assign_function: bool,
        field_config_map: Option<BTreeMap<String, ColumnConfig>>,
    ) -> Result<Self, PlanetError> {
        //eprintln!("Formula...");
        // If I have an error in compilation, then does not validate. Compilation uses validate of functions.
        // This function is the one does compilation from string formula to FormulaFieldCompiled
        let formula_origin = formula.clone();
        let properties_map = properties_map.clone();
        let field_config_map_ = field_config_map.clone();
        let mut field_config_map: BTreeMap<String, ColumnConfig> = BTreeMap::new();
        if field_config_map_.is_some() {
            field_config_map = field_config_map_.unwrap();
        }
        // let field_name_map_i = field_name_map.clone();
        let db_table_i = db_folder.clone();
        let table_name_i = folder_name.clone();
        // eprintln!("Formula :: formula_origin: {:?}", &formula_origin);
        // eprintln!("Formula :: formula_format: {:?}", &formula_format);
        let formula_map= compile_formula(formula_origin.clone()).unwrap();
        //eprintln!("Formula :: formula_map: {:?}", &formula_map);
        // let expr = &RE_FORMULA_FIELD_FUNCTIONS;
        let mut formula_processed = formula_origin.clone();
        let formula_format = formula_format.clone();
        let mut properties_map_: HashMap<String, ColumnConfig> = HashMap::new();
        if table.is_some() {
            let table = table.unwrap();
            let db_table = db_folder.unwrap();
            let table_name = folder_name.unwrap();
            let field_type_map_ = TreeFolder::get_column_type_map(&table)?;
            let field_name_map_ = TreeFolder::get_column_name_map(&db_table, &table_name)?;
            for (column_name, column_type) in field_type_map_.clone() {
                let mut column_config = ColumnConfig::defaults(None);
                column_config.column_type = Some(column_type);
                column_config.name = Some(column_name.clone());
                properties_map_.insert(column_name.clone(), column_config);
            }
            for (column_name, column_id) in field_name_map_.clone() {
                let mut column_config = properties_map_.get(&column_name).unwrap().clone();
                column_config.id = Some(column_id);
                properties_map_.insert(column_name.clone(), column_config);
            }
        } else if properties_map.is_some() {
            let properties_map = properties_map.unwrap();
            properties_map_ = properties_map;
        }
        // eprintln!("Formula :: file_name_map_: {:#?}", &field_name_map_);
        // eprintln!("Formula :: file_type_map_: {:#?}", &field_type_map_);
        let mut formula_compiled = Formula{
            functions: None,
            assignment: None,
            formula: String::from(""),
            data: None,
        };
        let mut compiled_functions_map: BTreeMap<String, CompiledFunction> = BTreeMap::new();
        let mut compiled_functions: Vec<CompiledFunction> = Vec::new();
        let expr = &RE_FORMULA_FUNCTION_VARIABLES;
        // let expr_chained = &RE_FORMULA_FUNCTION_PIECES;
        for (function_placeholder, function_text) in formula_map {
            let function_text = function_text.as_str();
            //eprintln!("Formula :: function_text: {}", function_text);
            //eprintln!("Formula :: function_placeholder: {}", function_placeholder);
            let function_list_ = expr.captures(function_text.clone());
            if function_list_.is_none() {
                //eprintln!("Formula :: have function list, compile function text");
                let mut main_function = compile_function_text(
                    function_text, 
                    &formula_format,
                    &properties_map_,
                    db_table_i.clone(),
                    table_name_i.clone(),
                    Some(field_config_map.clone())
                )?;
                if is_assign_function {
                    main_function.function_type = FunctionType::Assign;
                } else {
                    main_function.function_type = FunctionType::Attribute;
                }
                compiled_functions.push(main_function.clone());
                compiled_functions_map.insert(function_placeholder.clone(), main_function.clone());
                formula_processed = formula_processed.replace(function_text, function_placeholder.as_str());
                // eprintln!("Formula :: formula_processed: {:#?}", &formula_processed);
            }
        }
        // TODO: Apply also to the functions linked inside this function as attributes
        for (_, function) in &compiled_functions_map {
            // eprintln!("Formula :: function_key: {}", &function_key);
            let function = function.clone();
            let function_text = function.text.unwrap();
            let function_name = function.name;
            let mut function_parse = FunctionParse::defaults(&function_name);
            function_parse.text = Some(function_text.clone());
            let function_parse = process_function(
                &function_parse, 
                None,
                Some(field_config_map.clone())
            )?;
            let validate = function_parse.validate;
            // eprintln!("Formula :: validate: {}", &validate);
            if validate.unwrap() == false {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!("This function is not correctly formatted: {}", function_text.clone())),
                    )
                );
            }
        }
        formula_compiled.functions = Some(compiled_functions_map.clone());
        let formula_processed_string = formula_processed.clone();
        let formula_processed_str = formula_processed_string.as_str();
        formula_compiled.formula = formula_processed;

        // Compile assignment in case I do not have an attribute function
        let mut do_compile_assignment = false;
        for (_, function_) in formula_compiled.functions.clone().unwrap() {
            match function_.function_type {
                FunctionType::Assign => {
                    do_compile_assignment = true
                }
                _ => {}
            }
        }
        //eprintln!("Formula :: formula_processed_str: {}", formula_processed_str);
        //eprintln!("Formula :: do_compile_assignment: {}", &do_compile_assignment);
        if do_compile_assignment {
            // Since I have no functions, reference will be an assignment
            let assignment = compile_assignment(
                formula_processed_str, 
                formula_format,
                Some(compiled_functions_map.clone()), 
                properties_map_, 
                db_table_i.clone(),
                table_name_i.clone(),
                &field_config_map
            )?;
            if assignment.is_some() {
                let assignment = assignment.unwrap();
                formula_compiled.assignment = Some(assignment);
            }
        }
        eprintln!("Formula :: formula_compiled: {:#?}", &formula_compiled);
        return Ok(formula_compiled)
    }
}

pub fn compile_assignment(
    formula: &str,
    formula_format: String,
    functions_map: Option<BTreeMap<String, CompiledFunction>>,
    properties_map: HashMap<String, ColumnConfig>,
    db_table: Option<TreeFolder>,
    table_name: Option<String>,
    field_config_map: &BTreeMap<String, ColumnConfig>
) -> Result<Option<AttributeAssign>, PlanetError> {
    //eprintln!("compile_assignment...");
    //eprintln!("compile_assignment :: formula: {}", &formula);
    let field_config_map = field_config_map.clone();
    let field_config_map_wrap = Some(field_config_map);
    let formula = formula.clone();
    let formula_string = formula.to_string();
    let expr = &RE_FORMULA_ASSIGN;
    let capture_assignment = expr.captures(&formula);
    let assignment: Option<AttributeAssign>;
    if functions_map.is_none() {
        let formula_map= compile_formula(formula_string.clone()).unwrap();
        let mut compiled_functions_map: BTreeMap<String, CompiledFunction> = BTreeMap::new();
        for (function_placeholder, function_text) in formula_map {
            let function_text = function_text.as_str();
            let function_list_ = expr.captures(function_text);
            if function_list_.is_none() {
                // eprintln!("FormulaFieldCompiled :: function_text: {}", function_text);
                // eprintln!("FormulaFieldCompiled :: function_placeholder: {}", function_placeholder);
                let main_function = compile_function_text(
                    function_text, 
                    &formula_format,
                    &properties_map,
                    db_table.clone(),
                    table_name.clone(),
                    field_config_map_wrap.clone()
                )?;
                // compiled_functions.push(main_function.clone());
                compiled_functions_map.insert(function_placeholder.clone(), main_function.clone());
                // formula_processed = formula_processed.replace(function_text, function_placeholder.as_str());
                // eprintln!("FormulaFieldCompiled :: formula_processed: {:#?}", &formula_processed);
            }
        }
    }
    if capture_assignment.is_some() {
        let mut formula_assign = formula_string.clone();
        //eprintln!("compile_assignment: formula_assign: {}", &formula_assign);
        let capture_assignment = capture_assignment.unwrap();
        let assign = capture_assignment.name("assign").unwrap().as_str().to_string();
        // let name = capture_assignment.name("name").unwrap().as_str().to_string();
        // let op = capture_assignment.name("op").unwrap().as_str().to_string();
        // let value = capture_assignment.name("value").unwrap().as_str().to_string();
        let function = capture_assignment.name("function");
        // eprintln!("compile_assignment: {} {}{}{}", &assign, &name, &op, &value);
        if function.is_some() {
            if functions_map.is_some() {
                let functions_map = functions_map.unwrap();
                for (function_key, function) in functions_map {
                    formula_assign = formula_assign.replace(&function_key, &function.text.unwrap());
                }    
            }
        }
        //eprintln!("compile_assignment: assign: {} formula_assign: {}", &assign, &formula_assign);
        let (items, attribute_operator) = parse_assign_operator(
            &assign, &formula_assign
        )?;
        //eprintln!("compile_assignment: items: {:?} op: {:?}", &items, &attribute_operator);
        let (reference_name, items_new) = get_assignment_reference(
            &items, 
            properties_map.clone()
        )?;
        //eprintln!("compile_assignment: reference_name: {} items_new: {:?}", &reference_name, &items_new);
        let column_config = properties_map.get(&reference_name).unwrap().clone();
        let field_type = column_config.column_type;
        //eprintln!("compile_assignment: field_type: {:?}", &field_type);
        let mut attribute_type: AttributeType = AttributeType::Text;
        if field_type.is_some() {
            //eprintln!("compile_assignment: I have field_type...");
            let field_type = field_type.unwrap();
            //eprintln!("compile_assignment: field_type: {}", field_type);
            attribute_type = get_attribute_type(&field_type, None);
            //eprintln!("compile_assignment: attribute_type: {:?}", &attribute_type);
        }
        // {Counter} = 23
        // {My Column} = TRIM(" hola ")
        assignment = Some(
            AttributeAssign::defaults(
                &items_new[0].clone(), 
                &attribute_operator, 
                &items_new[1].clone(),
                &attribute_type
            )
        );

        return Ok(assignment)
    }
    return Ok(None)
}

pub fn formula_attr_collection(
    attrs_string: String
) -> Result<Vec<String>, PlanetError> {
    //eprintln!("formula_attr_collection :: attrs_string: {}", &attrs_string);
    let mut attributes: Vec<String> = Vec::new();
    let expr = &RE_FORMULA_FUNCTION_PIECES;
    let tries = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20];
    let mut count = 1;
    let mut formula_map_: BTreeMap<String, String> = BTreeMap::new();
    let mut formula_map: BTreeMap<String, String> = BTreeMap::new();
    let mut final_attrs = attrs_string.clone();
    for _ in tries {
        let final_attrs_ = final_attrs.clone();
        let final_attrs_ = final_attrs_.as_str();
        let attributes = expr.captures_iter(final_attrs_);
        let mut have_formulas = false;
        for attribute in attributes {
            have_formulas = true;
            let function_text = attribute.get(0).unwrap().as_str();
            //eprintln!("formula_attr_collection :: function_text: {}", function_text);
            let function_text_string = function_text.to_string();
            let function_placeholder = format!("$formula_{}", &count);
            final_attrs = final_attrs.replace(function_text, &function_placeholder);
            formula_map_.insert(function_placeholder.clone(), function_text_string);
            //eprintln!("formula_attr_collection :: [{}] formula_map: {:#?}", &count, &formula_map_);
            count += 1;
        }
        if !have_formulas {
            // I should have all function placeholders with $formula_XX, no functions left to process
            //eprintln!("formula_attr_collection :: I break");
            break
        }
    }
    //eprintln!("formula_attr_collection :: formula_map_: {:#?}", &formula_map_);
    let expr = &RE_FORMULA_VARIABLES;
    for (k, v) in formula_map_.clone() {
        let has_formula = v.clone().find("$formula_").is_some();
        if has_formula {
            let mut formula_value = v.clone();
            let formula_value_str = v.as_str();
            let formula_variables = expr.captures_iter(formula_value_str);
            for formula_variable in formula_variables {
                let formula_variable_text = formula_variable.get(0).unwrap().as_str();
                let formula_content = formula_map_.get(formula_variable_text);
                if formula_content.is_some() {
                    let formula_content = formula_content.unwrap().clone();
                    formula_value = formula_value.replace(
                        formula_variable_text, 
                        formula_content.as_str()
                    );
                    formula_map_.insert(k.clone(), formula_value.clone());
                }
            }
        }
    }
    //eprintln!("formula_attr_collection :: [2] formula_map_: {:#?}", &formula_map_);
    // Clean function_map for keys not final in the final formula
    let expr = &RE_FORMULA_VARIABLES;
    let final_attrs_ = final_attrs.clone();
    let final_attrs_ = final_attrs_.as_str();
    let formula_list_ = expr.captures_iter(final_attrs_);
    for formula_item in formula_list_ {
        let formula_item_text = formula_item.get(0).unwrap().as_str();
        let formula_item_text = formula_item_text.to_string();
        let formula_item_text_value = formula_map_.get(&formula_item_text);
        if formula_item_text_value.is_some() {
            let formula_item_text_value = formula_item_text_value.unwrap().clone();
            formula_map.insert(formula_item_text, formula_item_text_value);
        }
    }
    //eprintln!("formula_attr_collection :: formula_map: {:#?}", &formula_map);
    let final_attrs_items: Vec<&str> = final_attrs_.split(",").collect();
    for mut item in final_attrs_items {
        item = item.trim();
        let mut replaced_item = item.to_string();
        let formula_list_ = expr.captures_iter(item);
        for formula_item in formula_list_ {
            let formula_item_text = formula_item.get(0).unwrap().as_str();
            let formula_item_text = formula_item_text.to_string();    
            let formula_item_text_value = formula_map.get(&formula_item_text);
            if formula_item_text_value.is_some() {
                let formula_item_text_value = formula_item_text_value.unwrap().clone();
                replaced_item = item.replace(&formula_item_text, &formula_item_text_value);
            }
        }
        attributes.push(replaced_item);
    }
    //eprintln!("formula_attr_collection :: attributes: {:#?}", &attributes);
    return Ok(attributes)
}

pub fn compile_formula(
    formula: String
) -> Result<BTreeMap<String, String>, PlanetError> {
    let expr = &RE_FORMULA_FUNCTION_PIECES;
    // Number tries I repeat processing of functions left
    let tries = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20];
    let mut count = 1;
    let mut function_map_: BTreeMap<String, String> = BTreeMap::new();
    let mut final_formula = formula.clone();
    for _ in tries {
        let final_formula_ = final_formula.clone();
        let final_formula_ = final_formula_.as_str();
        let functions = expr.captures_iter(final_formula_);
        let mut have_functions = false;
        for function in functions {
            have_functions = true;
            let function_text = function.get(0).unwrap().as_str();
            let function_text_string = function_text.to_string();
            // TRIM(" hola ") ) or MINE(" hola 02 "), etc...
            let function_placeholder = format!("$func_{}", &count);
            final_formula = final_formula.replace(function_text, &function_placeholder);
            function_map_.insert(function_placeholder.clone(), function_text_string);
            count += 1;
        }
        // Check I have 
        if !have_functions {
            // I should have all function placeholders with $func_XX, no functions left to process
            break
        }
    }
    //eprintln!("compile_formula :: function_map_: {:#?}", &function_map_);
    // I return the final formula text and the function text map????
    // {"$func_1": "TRIM(\" hola \")", "$func_3": "TRIM(\" comino \")", "$func_4": "CONCAT( \"this-is-some-slug\", \" \", {My Field}, $func_1 )", "$func_5": "TRIM($func_2)", "$func_2": "MINE(\" hola 02 \")"}
    // post process function map
    let expr = &RE_FORMULA_FUNCTION_VARIABLES;
    for (k, v) in function_map_.clone() {
        let has_func = v.clone().find("$func_").is_some();
        if has_func {
            let mut function_value = v.clone();
            let function_value_str = v.as_str();
            let function_variables = expr.captures_iter(function_value_str);
            for function_variable in function_variables {
                let function_variable_text = function_variable.get(0).unwrap().as_str();
                let function_content = function_map_.get(function_variable_text);
                if function_content.is_some() {
                    let function_content = function_content.unwrap().clone();
                    function_value = function_value.replace(
                        function_variable_text, 
                        function_content.as_str()
                    );
                    function_map_.insert(k.clone(), function_value.clone());
                }
            }
        }
    }
    // Clean function_map for keys not final in the final formula
    let mut function_map: BTreeMap<String, String> = BTreeMap::new();
    let expr = &RE_FORMULA_FUNCTION_VARIABLES;
    let final_formula_ = final_formula.clone();
    let final_formula_ = final_formula_.as_str();
    // eprintln!("compile_formula :: final_formula_: {}", final_formula);
    let function_list_ = expr.captures_iter(final_formula_);
    for function_item in function_list_ {
        let function_item_text = function_item.get(0).unwrap().as_str();
        let function_item_text = function_item_text.to_string();
        let function_item_text_value = function_map_.get(&function_item_text);
        if function_item_text_value.is_some() {
            let mut function_item_text_value = function_item_text_value.unwrap().clone();
            let function_item_text_value_str = function_item_text_value.clone();
            let function_item_text_value_str = function_item_text_value_str.as_str();
            // This string might have placeholders still
            // eprintln!("compile_formula :: function_item_text_value: {}", &function_item_text_value);
            let function_list_ = expr.captures_iter(function_item_text_value_str);
            for function_item in function_list_ {
                let function_item_text = function_item.get(0).unwrap().as_str();
                // eprintln!("compile_formula :: function_item_text: {}", function_item_text);
                let function_item_text_value_ = function_map_.get(function_item_text);
                if function_item_text_value_.is_some() {
                    let function_item_text_value_ = function_item_text_value_.unwrap().clone();
                    function_item_text_value = function_item_text_value.replace(
                        function_item_text, 
                        &function_item_text_value_
                    );
                }
            }
            function_map.insert(function_item_text, function_item_text_value);
        }
    }

    return Ok(function_map)
}

pub fn compile_function_text(
    function_text: &str,
    formula_format: &String,
    properties_map: &HashMap<String, ColumnConfig>,
    db_table: Option<TreeFolder>,
    table_name: Option<String>,
    field_config_map: Option<BTreeMap<String, ColumnConfig>>,
) -> Result<CompiledFunction, PlanetError> {
    //eprintln!("compile_function_text :: function_text: {}", &function_text);
    let field_config_map_wrap = field_config_map.clone();
    let field_config_map: BTreeMap<String, ColumnConfig> = BTreeMap::new();
    let formula_format = formula_format.clone();
    let properties_map = properties_map.clone();
    // eprintln!("compile_function_text :: field_type_map: {:#?}", &field_type_map);
    // eprintln!("compile_function_text :: field_name_map: {:#?}", &field_name_map);
    let parts: Vec<&str> = function_text.split("(").collect();
    let function_name = parts[0];
    // eprintln!("compile_function_text :: parts: {:?}", &parts);
    //eprintln!("compile_function_text :: function_name: {}", function_name);
    let mut function_parse = FunctionParse::defaults(&function_name.to_string());
    function_parse.text = Some(function_text.to_string());
    let function_parse = process_function(
        &function_parse, 
        None,
        field_config_map_wrap.clone(),
    )?;
    // eprintln!("compile_function_text :: function_parse from coded function: {:#?}", &function_parse);
    let validate = function_parse.validate.unwrap();
    if validate == false {
        return Err(
            PlanetError::new(
                500, 
                Some(tr!("function {} not having the expected format, validation error", &function_text)),
            )
        );
    }
    // function_name: CONCAT for example
    let mut main_function: CompiledFunction = CompiledFunction::defaults(
        &function_name.to_string());
    main_function.text = Some(function_text.to_string());
    let mut main_function_attrs: Vec<FunctionAttributeItem> = Vec::new();
    let function_attributes = function_parse.attributes;
    if function_attributes.is_none() {
        return Err(
            PlanetError::new(
                500, 
                Some(tr!("function {} not having the expected format, attributes error", &function_text)),
            )
        );
    }
    let function_attributes = function_attributes.unwrap();
    // //eprintln!("compile_function_text :: captured_attrs: {:?}", &captured_attrs);
    for attr_ in function_attributes {
        let mut attr = attr_.as_str();
        attr = attr.trim();
        //eprintln!("compile_function_text :: attr: {}", &attr);
        let mut attribute_type: AttributeType = AttributeType::Text;
        let mut function_attribute = FunctionAttributeItem::defaults(
            None,
            None, 
            attribute_type
        );
        let replaced_text: String;
        // Here we resolve the attribute type, if reference, function, string, number, boolean through Regex
        let expr = &RE_ATTR_TYPE_RESOLVE;
        let attr_type_resolve = expr.captures(attr);
        if attr_type_resolve.is_none() {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("Attribute \"{}\" does not have correct format, regex error", &attr)),
                )
            );
        }
        let attr_type_resolve = attr_type_resolve.unwrap();
        //eprintln!("compile_function_text :: attr_type_resolve: {:?}", &attr_type_resolve);
        let attr_type_ref = attr_type_resolve.name("ref");
        let attr_type_formula = attr_type_resolve.name("formula");
        let attr_type_bool = attr_type_resolve.name("bool");
        let attr_type_string = attr_type_resolve.name("string");
        let attr_type_number = attr_type_resolve.name("number");
        let attr_type_assign = attr_type_resolve.name("assign");
        if attr_type_ref.is_some() && function_name != FUNCTION_FORMAT {
            // Reference
            // I have attribute name and also the field type -> attribute_type from table
            // eprintln!("compile_function_text :: [{}] is reference", &attr);
            let attr_string = attr.to_string();
            let attr_string = attr_string.
                replace("\"", "").
                replace("{", "").
                replace("}", "");
            let field_name = attr_string.clone();
            // let field_id = field_name_map.get(&field_name.clone());
            let column_config = properties_map.get(&field_name).unwrap().clone();
            let field_id = column_config.id;
            if field_id.is_some() {
                let field_id = field_id.unwrap().clone();
                function_attribute.id = Some(field_id);
            }
            function_attribute.name = Some(field_name);
            
            function_attribute.reference_value = Some(attr_string.clone());
            let field_type = column_config.column_type;
            if field_type.is_some() {
                let field_type = field_type.unwrap().clone();
                attribute_type = get_attribute_type(&field_type, Some(formula_format.clone()));
                function_attribute.attr_type = attribute_type;
                function_attribute.is_reference = true;
            }
        } else if attr_type_formula.is_some() {
            // formula
            //eprintln!("compile_function_text :: [{}] is a formula", &attr);
            let function_attribute_string = attr.to_string();
            let formula_compiled = Formula::defaults(
                &function_attribute_string.clone(),
                &formula_format,
                None,
                Some(properties_map.clone()),
                db_table.clone(),
                table_name.clone(),
                false,
                field_config_map_wrap.clone(),
            )?;
            function_attribute.formula = Some(formula_compiled);
            function_attribute.name = Some(function_attribute_string);
        } else if attr_type_bool.is_some() {
            // eprintln!("compile_function_text :: [{}] is boolean", &attr);
            function_attribute.attr_type = AttributeType::Bool;
            function_attribute.value = Some(attr.to_string());
        } else if attr_type_string.is_some() {
            // Could be text or date, time. How deal with it?
            // TODO: Have date strings to check to resolve if we have a date, time or text
            // eprintln!("compile_function_text :: [{}] is string, date, time", &attr);
            function_attribute.attr_type = AttributeType::Text;
            replaced_text = attr.replace("\"", "");
            attr = replaced_text.as_str();
            function_attribute.value = Some(attr.to_string());
        } else if attr_type_number.is_some() {
            // eprintln!("compile_function_text :: [{}] is number", &attr);
            function_attribute.attr_type = AttributeType::Number;
            function_attribute.value = Some(attr.to_string());
        } else if attr_type_assign.is_some() {
            // Process assign
            //eprintln!("compile_function_text :: assignment...");
            // In case I have assign into a formula, I need to parse it.
            let expr = &RE_FORMULA_FUNCTIONS;
            let have_assign_functions = expr.is_match(attr);
            let mut assign_attr = attr.clone().to_string();
            //eprintln!("compile_function_text :: have_assign_functions: {}", &have_assign_functions);
            // let mut formula_map: BTreeMap<String, String> = BTreeMap::new();
            if have_assign_functions {
                let captures = expr.captures_iter(attr);
                let capture = captures.last();
                if capture.is_some() {
                    let formula = capture.unwrap().get(0).unwrap().as_str().to_string();
                    //eprintln!("compile_function_text :: formula: {}", &formula);
                    assign_attr = assign_attr.replace(formula.clone().as_str(), "$formula");
                    //eprintln!("compile_function_text :: assign_attr: {}", &assign_attr);
                    let formula_obj = Formula::defaults(
                        &formula, 
                        &formula_format, 
                        None, 
                        Some(properties_map.clone()), 
                        db_table.clone(), 
                        table_name.clone(),
                        true,
                        field_config_map_wrap.clone()
                    )?;
                    function_attribute.formula = Some(formula_obj.clone());
                    //eprintln!("compile_function_text :: formula_obj: {:#?}", &formula_obj);
                }
            }
            //eprintln!("compile_function_text :: assign_attr: {}", &assign_attr);
            // eprintln!("compile_function_text :: formula_map: {:#?}", &formula_map);
            // compile all formulas linked in assignment
            let assignment = compile_assignment(
                assign_attr.as_str(), 
                formula_format.clone(),
                None, 
                properties_map.clone(), 
                db_table.clone(),
                table_name.clone(),
                &field_config_map
            )?;
            // let function_attrib = assignment.clone().unwrap();
            function_attribute.assignment = assignment.clone();
            let attr_type = assignment.clone().unwrap().assign_type;
            function_attribute.attr_type = attr_type;
            function_attribute.value = Some(attr.to_string());
            //eprintln!("compile_function_text :: function attribute & assignment: {:#?}", &function_attribute);
            // function_attribute.attr_type = AttributeType::Assign;
            // How I do attr_type?
            // function_attribute.assignment = assignment;
        }
        main_function_attrs.push(function_attribute);
    }
    main_function.attributes = Some(main_function_attrs);
    return Ok(main_function)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionResult {
    pub text: Option<String>,
    pub date: Option<String>,
    pub check: Option<bool>,
    pub number: Option<usize>,
}
impl FunctionResult {
    pub fn defaults() -> Self {
        let obj = FunctionResult{
            text: None,
            date: None,
            check: None,
            number: None,
        };
        return obj
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionParse {
    name: String,
    text: Option<String>,
    validate: Option<bool>,
    attributes: Option<Vec<String>>,
    compiled_attributes: Option<Vec<FunctionAttributeItem>>,
    result: Option<FunctionResult>,
    field_config_map: BTreeMap<String, ColumnConfig>,
}
impl FunctionParse {
    pub fn defaults(name: &String) -> Self {
        let field_config_map: BTreeMap<String, ColumnConfig> = BTreeMap::new();
        let obj = FunctionParse{
            name: name.clone(),
            text: None,
            validate: None,
            attributes: None,
            compiled_attributes: None,
            result: None,
            field_config_map: field_config_map,
        };
        return obj;
    }
}

pub fn prepare_function_parse(
    function_parse: &FunctionParse, 
    data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
) -> (
    Option<String>, 
    String, Vec<FunctionAttributeItem>, 
    FunctionResult, 
    BTreeMap<String, Vec<BTreeMap<String, String>>>
) {
    let mut function = function_parse.clone();
    let function_text_wrap = function.text;
    let mut function_text: String = String::from("");
    if function_text_wrap.is_some() {
        function_text = function_text_wrap.clone().unwrap();
    }
    let data_map_wrap = data_map;
    let mut data_map: BTreeMap<String, Vec<BTreeMap<String, String>>> = BTreeMap::new();
    if data_map_wrap.is_some() {
        data_map = data_map_wrap.unwrap();
    }
    let compiled_attrtibutes_wrap = function.compiled_attributes;
    let mut compiled_attributes: Vec<FunctionAttributeItem> = Vec::new();
    if compiled_attrtibutes_wrap.is_some() {
        compiled_attributes = compiled_attrtibutes_wrap.unwrap();
    }
    let function_result = FunctionResult::defaults();
    function.result = Some(function_result.clone());
    return (
        function_text_wrap,
        function_text.to_string(),
        compiled_attributes,
        function_result,
        data_map,
    )
}

pub fn process_function(
    function_parse: &FunctionParse, 
    data_map: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
    field_config_map: Option<BTreeMap<String, ColumnConfig>>,
) -> Result<FunctionParse, PlanetError> {
    // let list_items = Some(expr.captures_iter(function_text));
    // I need either check or list of attributes, so I have only one function to deal with Regex expr.
    let function = function_parse.clone();
    // eprintln!("process_function :: function: {:#?}", &function);
    let data_map_wrap = data_map;
    let function_name = function.name.as_str();
    let field_config_map = field_config_map.clone();
    let mut func = function.clone();
    let data = data_map_wrap.clone();
    let conf: BTreeMap<String, ColumnConfig>;
    if field_config_map.is_some() {
        conf = field_config_map.unwrap();
    } else {
        conf = BTreeMap::new();
    }
    match function_name {
        FUNCTION_CONCAT => {
            func = Concat::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_TRIM => {
            func = Trim::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_FORMAT => {
            func = Format::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_JOINLIST => {
            func = JoinList::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_LENGTH => {
            func = Length::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_LOWER => {
            func = Lower::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_UPPER => {
            func = Upper::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_REPLACE => {
            func = Replace::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_MID => {
            func = Mid::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_REPT => {
            func = Rept::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_SUBSTITUTE => {
            func = Substitute::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_CEILING => {
            func = Ceiling::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_FLOOR => {
            func = Floor::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_COUNT => {
            func = Count::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_COUNTA => {
            func = CountA::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_COUNTALL => {
            func = CountAll::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_EVEN => {
            func = Even::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_EXP => {
            func = Exp::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_INT => {
            func = Int::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_LOG => {
            func = Log::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_MOD => {
            func = Mod::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_POWER => {
            func = Power::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_ROUND => {
            func = Round::defaults(Some(func), data.clone(), &conf).handle(
                RoundOption::Basic)?;
        },
        FUNCTION_ROUNDUP => {
            func = Round::defaults(Some(func), data.clone(), &conf).handle(
                RoundOption::Up)?;
        },
        FUNCTION_ROUNDDOWN => {
            func = Round::defaults(Some(func), data.clone(), &conf).handle(
                RoundOption::Down)?;
        },
        FUNCTION_SQRT => {
            func = Sqrt::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_VALUE => {
            func = Value::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_TRUE => {
            func = Boolean::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_FALSE => {
            func = Boolean::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_DATE => {
            func = Date::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_SECOND => {
            func = DateTimeParse::defaults(Some(func), data.clone(), &conf).handle(
                DateTimeParseOption::Second)?;
        },
        FUNCTION_MINUTE => {
            func = DateTimeParse::defaults(Some(func), data.clone(), &conf).handle(
                    DateTimeParseOption::Minute)?;
        },
        FUNCTION_HOUR => {
            func = DateTimeParse::defaults(Some(func), data.clone(), &conf).handle(
                DateTimeParseOption::Hour)?;
        },
        FUNCTION_DAY => {
            func = DateParse::defaults(Some(func), data.clone(), &conf).handle(
                DateParseOption::Day)?;
        },
        FUNCTION_WEEK => {
            func = DateParse::defaults(Some(func), data.clone(), &conf).handle(
                DateParseOption::Week)?;
        },
        FUNCTION_WEEKDAY => {
            func = DateParse::defaults(Some(func), data.clone(), &conf).handle(
                DateParseOption::WeekDay)?;
        },
        FUNCTION_MONTH => {
            func = DateParse::defaults(Some(func), data.clone(), &conf).handle(
                DateParseOption::Month)?;
        },
        FUNCTION_NOW => {
            func = Now::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_TODAY => {
            func = Today::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_DAYS => {
            func = Days::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_DATEADD => {
            func = DateAddDiff::defaults(Some(func), data.clone(), &conf).handle(
                DateDeltaOperation::Add)?;
        },
        FUNCTION_DATEDIF => {
            func = DateAddDiff::defaults(Some(func), data.clone(), &conf).handle(
                DateDeltaOperation::Diff)?;
        },
        FUNCTION_DATEFMT => {
            func = DateFormatFunc::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_MIN => {
            func = Stats::defaults(Some(func), data.clone(), &conf).handle(
                StatOption::Min)?;
        },
        FUNCTION_MAX => {
            func = Stats::defaults(Some(func), data.clone(), &conf).handle(
                StatOption::Max)?;
        },
        FUNCTION_IF => {
            func = If::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_AND => {
            func = And::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_OR => {
            func = Or::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_NOT => {
            func = Not::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        FUNCTION_XOR => {
            func = Xor::defaults(Some(func), data.clone(), &conf).handle()?;
        },
        _ => {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("Function \"{}\" not supported", &function_name)),
                )
            );
        }
    }
    return Ok(func)
}

// Score for this is validate all functions in a text formula, but design  might change with compilation
// of formulas
pub fn function_validate_tuple(
    function_text: &String, 
    validate_tuple: (u32, Vec<String>),
    expr: &Regex,
    function_name: &str,
 ) -> (u32, Vec<String>) {
    let (number_fails, mut failed_functions) = validate_tuple;
    let expr = expr;
    let check = expr.is_match(&function_text);
    let mut number_fails = number_fails.clone();
    if check == false {
        number_fails += 1;
        failed_functions.push(String::from(function_name));
    }
    return (number_fails, failed_functions);
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FunctionType {
    Attribute,
    Assign,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompiledFunction {
    pub name: String,
    pub text: Option<String>,
    pub function_type: FunctionType,
    pub attributes: Option<Vec<FunctionAttributeItem>>,
    pub data: Option<BTreeMap<String, String>>,
}
impl CompiledFunction {
    pub fn defaults(name: &String) -> Self {
        let name = name.clone();
        let obj = Self{
            name: name,
            attributes: None,
            text: None,
            function_type: FunctionType::Attribute,
            data: None,
        };
        return obj;
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FormulaOperator {
    Eq,
    Greater,
    Smaller,
    GreaterOrEqual,
    SmallerOrEqual,
    Contains,
    IsEmpty,
}

// attributes:
// {Column}
// "value" or value
// (3 + 4)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionAttributeItem {
    pub is_reference: bool,
    pub reference_value: Option<String>,
    pub assignment: Option<AttributeAssign>,
    pub name: Option<String>,
    pub id: Option<String>,
    pub value: Option<String>,
    pub attr_type: AttributeType,
    pub formula: Option<Formula>,
    pub data: Option<BTreeMap<String, String>>,
}
impl FunctionAttributeItem {
    pub fn defaults(id: Option<String>, name: Option<String>, attr_type: AttributeType) -> Self {
        let attr_type = attr_type.clone();
        let obj = Self{
            is_reference: false,
            reference_value: None,
            assignment: None,
            name: name,
            id: id,
            value: None,
            attr_type: attr_type,
            formula: None,
            data: None,
        };
        return obj
    }
    pub fn get_value(
        &self, 
        data_map: &BTreeMap<String, Vec<BTreeMap<String, String>>>,
        field_config_map: &BTreeMap<String, ColumnConfig>
    ) -> Result<String, PlanetError> {
        let data_map = data_map.clone();
        let field_config_map = field_config_map.clone();
        let is_reference = self.is_reference;
        let formula = self.formula.clone();
        let attribute_id = self.id.clone().unwrap_or_default();
        let attribute_value = self.value.clone().unwrap_or_default();
        let has_assignment = self.assignment.is_some();
        //eprintln!("FunctionAttributeItem.get_value :: id: {}", &attribute_id);
        //eprintln!("FunctionAttributeItem: {:#?}", &self);
        let mut value: String;
        if is_reference && !has_assignment {
            //eprintln!("FunctionAttributeItem.get_value :: is_reference");
            // FUNC({Column}) : I get value from data_map, key Column
            let function_attr = FunctionAttribute::defaults(
                &attribute_id, Some(true), Some(true)
            );
            value = function_attr.replace(&data_map).item_processed.unwrap();
            //eprintln!("FunctionAttributeItem.get_value :: {}={}", &attribute_id, &value);
        } else if formula.is_some() {
            //eprintln!("FunctionAttributeItem.get_value :: formula");
            // I execute the formula and return value
            // execute_formula(formula: &Formula, data_map: &BTreeMap<String, String>)
            let formula = formula.unwrap();
            value = execute_formula(
                &formula, 
                &data_map,
                &field_config_map
            )?;
            //eprintln!("FunctionAttributeItem.get_value :: {}={}", &attribute_id, &value);
        } else if has_assignment {
            let check = self.check_assignment(&data_map);
            value = String::from("0");
            if check {
                value = String::from("1");
            }
        } else {
            //eprintln!("FunctionAttributeItem.get_value :: normal");
            // I get value and return it
            value = attribute_value;
            //eprintln!("FunctionAttributeItem.get_value :: {}={}", &attribute_id, &value);
        }
        return Ok(value)
    }
    pub fn check_assignment(&self, data_map: &BTreeMap<String, Vec<BTreeMap<String, String>>>) -> bool {
        let attr_type = self.attr_type.clone();
        let assignment = self.assignment.clone();
        let mut check = false;
        if assignment.is_some() {
            let assignment = assignment.unwrap();
            check = check_assignment(assignment, attr_type, data_map).unwrap();
        }
        return check
    }
}

pub fn execute_formula(
    formula: &Formula, 
    data_map: &BTreeMap<String, Vec<BTreeMap<String, String>>>,
    field_config_map: &BTreeMap<String, ColumnConfig>,
) -> Result<String, PlanetError> {
    // 23 + LOG(34)
    // FUNC(attr1, attr2, ...)
    // FUNC(FUNC(attr1, attr2, ...))
    // This needs to execute the formula for a field
    // The type will depend on the formula_format on what we return
    // 1. I execute the functions in the formula and substitute result by placeholder and call LIB
    let field_config_map = field_config_map.clone();
    let field_config_map_wrap = Some(field_config_map);
    let functions = formula.functions.clone();
    let mut formula_str = formula.formula.clone();
    if functions.is_some() {
        // $func1 => Function compiled
        let functions = functions.unwrap();
        for (function_key, function) in functions {
            let function = function.clone();
            let function_key = function_key.as_str();
            let mut function_parse = FunctionParse::defaults(&function.name);
            function_parse.text = function.text;
            function_parse.compiled_attributes = function.attributes;
            let function_parse = process_function(
                &function_parse, 
                Some(data_map.clone()),
                field_config_map_wrap.clone()
            )?;
            // eprintln!("execute_formula_field :: function_parse: {:#?}", &function_parse);
            // eprintln!("execute_formula_field :: function: {:#?}", function.clone());
            let function_result = function_parse.result.unwrap();
            let result_str = function_result.text;
            let result_number = function_result.number;
            let result_date = function_result.date;
            let result_bool = function_result.check;
            if result_str.is_some() {
                let result_str = result_str.unwrap();
                let replaced_str = result_str.as_str();
                formula_str = formula_str.replace(function_key, replaced_str);
                formula_str = format!("{}{}{}", String::from("\""), formula_str, String::from("\""));
            } else if result_number.is_some() {
                let result_number = result_number.unwrap();
                let replaced_str = result_number.to_string();
                let replaced_str = replaced_str.as_str();
                formula_str = formula_str.replace(function_key, replaced_str);
            } else if result_date.is_some() {
                let result_date = result_date.unwrap();
                let replaced_str = result_date.as_str();
                formula_str = formula_str.replace(function_key, replaced_str);
                formula_str = format!("{}{}{}", String::from("\""), formula_str, String::from("\""));
            } else if result_bool.is_some() {
                let result_bool = result_bool.unwrap();
                let replaced_str: &str;
                if result_bool == true {
                    replaced_str = "1";
                } else {
                    replaced_str = "0";
                }
                formula_str = formula_str.replace(function_key, replaced_str);
            }
        }
    }
    // execute formula_str with LIB to provide result, which output will depend on formula_format from config
    // Check how it is on Formula object
    formula_str = formula_str.replace("\"\"", "");
    let formula_string = format!("={}", &formula_str);
    // formula_string = String::from("=23 + -4 + 4");
    // eprintln!("execute_formula_field :: formula_string: {}", &formula_string);
    // let t_exec_1 = Instant::now();
    let formula_ = parse_formula::parse_string_to_formula(
        &formula_string, 
        None::<NoCustomFunction>
    );
    // eprintln!("execute_formula_field :: formula_: {:?}", &formula_);
    let result = calculate::calculate_formula(formula_, None::<NoReference>);
    // eprintln!("execute_formula_field :: calcuated formula_: {:?}", &result);
    let result = calculate::result_to_string(result);
    // eprintln!("execute_formula_field :: perf : exec: {} µs", &t_exec_1.elapsed().as_micros());
    let result = result.trim().to_string();
    return Ok(result)
}

pub fn fetch_logical_op(attribute: &str) -> &str {
    let mut log_op: &str = "";
    let has_equal = attribute.find("=").is_some();
    let has_greater = attribute.find(">").is_some();
    let has_smaller = attribute.find("<").is_some();
    let has_equal_or_greater = attribute.find("=>").is_some();
    let has_smaller_or_equal = attribute.find("<=").is_some();
    if has_equal == true {
        log_op = "=";
    } else {
        if has_greater == true && has_equal_or_greater == false {
            log_op = ">";
        } else if has_greater == true && has_equal_or_greater == true {
            log_op = "=>";
        } else if has_smaller == true && has_smaller_or_equal == false {
            log_op = "<";
        } else if has_smaller == true && has_smaller_or_equal == true {
            log_op = "<=";
        }
    }
    return log_op
}

pub fn get_attribute_type(field_type: &String, formula_format: Option<String>) -> AttributeType {
    let field_type = field_type.clone();
    let field_type = field_type.to_lowercase();
    let field_type = field_type.as_str();
    let attr_type: AttributeType;
    match field_type {
        "checkbox" => {attr_type = AttributeType::Bool},
        "small text" => {attr_type = AttributeType::Text},
        "long text" => {attr_type = AttributeType::Text},
        "select" => {attr_type = AttributeType::Text},
        "number" => {attr_type = AttributeType::Number},
        _ => {attr_type = AttributeType::Text},
    }
    if field_type == "formula" && formula_format.is_some() {
        let formula_format = formula_format.unwrap().to_lowercase();
        let formula_format = formula_format.as_str();
        return match formula_format {
            "text" => AttributeType::Text,
            "number" => AttributeType::Number,
            "date" => AttributeType::Date,
            "bool" => AttributeType::Bool,
            _ => AttributeType::Text,
        }
    }
    return attr_type
}

pub fn get_assignment_reference(
    items: &Vec<String>, 
    properties_map: HashMap<String, ColumnConfig>
) -> Result<(String, Vec<String>), PlanetError> {
    let mut reference_name: String = String::from("");
    let mut items_new: Vec<String> = Vec::new();
    let mut item_replaced: String;
    for (count, item) in items.iter().enumerate() {
        let mut item = item.clone();
        // |  {Status}  |
        // eprintln!("compile_formula_query :: item: *{}*", item);
        // let mut item_ = *item;
        let item_ = item.trim();
        item = item_.to_string();
        // eprintln!("compile_formula_query :: item: *{}*", item);
        if count == 0 {
            // {Column A} => $column_id
            // let mut item_string = item_.to_string();
            item = item.replace("{", "}").replace("}", "");
            reference_name = item.clone();
            // let column_id = &field_name_map.get(&item).unwrap();
            let column_config = properties_map.get(&item).unwrap().clone();
            let column_id = column_config.id.unwrap();
            let column_id = column_id.clone();
            //eprintln!("compile_formula_query :: column_id: {}", column_id);
            item = column_id.clone();
        }
        item_replaced = item.replace("\"", "");
        items_new.push(item_replaced);
    }
    return Ok((reference_name, items_new));
}

pub fn parse_assign_operator(
    attr_source: &str, 
    formula: &String
) -> Result<(Vec<String>, FormulaOperator), PlanetError> {
    let log_op = fetch_logical_op(attr_source);
    let items: Vec<&str>;
    let attribute_operator: FormulaOperator;
    match log_op {
        "=" => {
            items = attr_source.split(log_op).collect();
            attribute_operator = FormulaOperator::Eq;
        },
        "=>" => {
            items = attr_source.split(log_op).collect();
            attribute_operator = FormulaOperator::GreaterOrEqual;
        },
        "<=" => {
            items = attr_source.split(log_op).collect();
            attribute_operator = FormulaOperator::SmallerOrEqual;
        },
        ">" => {
            items = attr_source.split(log_op).collect();
            attribute_operator = FormulaOperator::Greater;
        },
        "<" => {
            items = attr_source.split(log_op).collect();
            attribute_operator = FormulaOperator::Smaller;
        },
        _ => {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("Could not validate formula. Formula: {}", &formula)),
                )
            );
        }
    }
    let mut items_string: Vec<String> = Vec::new();
    for item_str in items {
        let item_string: String = item_str.to_string();
        items_string.push(item_string);
    }    
    return Ok((items_string, attribute_operator))
}

pub fn check_assignment(
    attr_assignment: AttributeAssign,
    attr_type: AttributeType,
    db_data_map: &BTreeMap<String, Vec<BTreeMap<String, String>>>,
) -> Result<bool, PlanetError> {
    //eprintln!("check_assignment...");
    //eprintln!("check_assignment :: db_data_map: {:#?}", db_data_map);
    //eprintln!("check_assignment :: attr_assignment: {:#?}", &attr_assignment);
    //eprintln!("check_assignment :: attr_type: {:#?}", &attr_type);
    let column_id = attr_assignment.name;
    let column_id = column_id.as_str();
    let db_value = db_data_map.get(column_id).unwrap();
    let db_value = get_value_list(db_value);
    let mut check: bool = false;
    if db_value.is_some() {
        let db_value = db_value.unwrap();
        let op = attr_assignment.op;
        let mut value = attr_assignment.value;
        // We have case when we try to compare dates, but is not supported, functions would need to be used.
        // Greater and smaller is used for numbers
        // TODO: Match for bool {Column} = true. How???? TRUE() right?
        match attr_type {
            AttributeType::Text | AttributeType::Date => {
                match op {
                    FormulaOperator::Eq => {
                        value = value.replace("\"", "");
                        check = check_string_equal(&db_value, &value)?;
                    },
                    _ => {
                        return Err(
                            PlanetError::new(
                                500, 
                                Some(tr!("Assignment for string only support equal operator.")),
                            )
                        );
                    },
                }    
            },
            AttributeType::Number => {
                let value = value.as_str();
                let value: f64 = FromStr::from_str(value).unwrap();
                let db_value: f64 = FromStr::from_str(&db_value).unwrap();
                //eprintln!("check_assignment :: db_value: {}", &db_value);
                //eprintln!("check_assignment :: op: {:?}", &op);
                //eprintln!("check_assignment :: value: {:?}", &value);
                check = check_float_compare(&value, &db_value, op)?;
            },
            _ => {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!("Assignment type not supported. We only support 
                        Text, Number, Bool and Date")),
                    )
                );
            }
        }    
    }
    return Ok(check)
}

pub fn get_vector_regex_attributes(list_items: CaptureMatches) -> Vec<String> {
    let mut attributes: Vec<String> = Vec::new();
    for item in list_items {
        let attr = item.get(0).unwrap().as_str();
        let attr = attr.to_string();
        attributes.push(attr);
    }
    return attributes
}

pub fn prepare_string_attribute(attr: String) -> String {
    let mut attr = attr.clone();
    if attr.find("\"").is_none() {
        attr = format!("\"{}\"", &attr);
    }
    return attr
}