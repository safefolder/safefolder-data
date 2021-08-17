pub mod text;
pub mod constants;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::functions::constants::*;
use crate::functions::text::*;

// achiever planet functions
pub const FORMULA_FUNCTIONS: [&str; 1] = [
    FUNCTION_CONCAT,
];

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
pub struct FunctionsHanler {
    pub function_name: String,
    pub function_text: String,
    pub data_map: HashMap<String, String>,
}
impl FunctionsHanler{
    pub fn do_functions(&self, mut formula: String) -> String {
        let function_name = self.function_name.as_str();
        // Match all achiever functions here
        match function_name {
            FUNCTION_CONCAT => {
                formula = ConcatenateFunction::init_do(
                    &self.function_text, self.data_map.clone(), formula);
            },
            _ => {
            }
        }
        return formula
    }
}

