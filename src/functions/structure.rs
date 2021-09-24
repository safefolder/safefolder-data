use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;
use regex::{Regex};
use std::{collections::HashMap};

use crate::functions::constants::*;
use crate::functions::Formula;
use crate::storage::table::DbData;
use crate::functions::*;

lazy_static! {
    static ref RE_IF: Regex = Regex::new(r#"IF\([\n\s\t]{0,}(?P<condition>\{[\w\s]+\}[\s]{0,}(=|<|>|<=|>=)[\s]{0,}((\d+)|("[\w\s]+"))),[\s\n\t]{0,}(?P<expr_true>(\d+)|("[\w\s]+")),[\s\n\t]{0,}(?P<expr_false>(\d+)|("[\w\s]+"))[\s\n\t]{0,}\)|IF\([\s\n\t]{0,}(?P<log_condition>(AND\([\s\n\t]{0,}[\w\W\s\n\t]{1,}[\s\n\t]{0,}\)|OR\([\s\n\t]{0,}[\w\W\s\n\t]{1,}[\s\n\t]{0,}\)|NOT\([\s\n\t]{0,}[\w\W\s\n\t]{1,}[\s\n\t]{0,}\)|XOR\([\s\n\t]{0,}[\w\W\s\n\t]{1,}[\s\n\t]{0,}\))),[\s\n\t]{0,}(?P<log_expr_true>(\d+)|("[\w\s]+")),[\s\n\t]{0,}(?P<log_expr_false>(\d+)|("[\w\s]+"))[\s\n\t]{0,}\)"#).unwrap();
    static ref RE_IF_REPLACED: Regex = Regex::new(r#"IF\([\s\n\t]{0,}(?P<condition>([a-zA-Z0-9_<>=\s]+)|(AND\([\w\W\s\n\t]{1,}\))|(OR\([\w\W\s\n\t]{1,}\))|(NOT\([\w\W\s\n\t]{1,}\))|(XOR\([\w\W\s\n\t]{1,}\)))[\s\n\t]{0,},[\s\n\t]{0,}(?P<expr_true>("[\w\s]+")|(\d+))[\s\n\t]{0,},[\s\n\t]{0,}(?P<expr_false>("[\w\s]+")|(\d+))[\s\n\t]{0,}\)"#).unwrap();
}

// IF({Sales} > 50, "Win", "Loose")
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IfFunction {
    pub function_text: String,
    pub condition: Option<String>,
    pub expr_true: Option<String>,
    pub expr_false: Option<String>,
    pub data_map: Option<HashMap<String, String>>,
    pub table: Option<DbData>,
}
impl IfFunction {
    pub fn defaults(function_text: &String, data_map: Option<HashMap<String, String>>, table: Option<DbData>) -> IfFunction {
        // IF({My Column} = 3, "Mine", "Yours")
        // IF({My Column} > 3, "Mine", "Yours")
        // IF({My Column}>3, "Mine", "Yours")
        // IF({My Column}>3,"Mine","Yours")
        // IF({My Column} >= 3, "Mine", "Yours")
        // IF({My Column} = "pepito mio", "Mine", "Yours")
        // IF(AND({My Column} = 23, {My Other Column} > 4, {My Text Column} = "pepito"),"mine","yours")
        let matches = RE_IF.captures(function_text).unwrap();
        let condition = matches.name("condition");
        let expr_true = matches.name("expr_true");
        let expr_false = matches.name("expr_false");
        let log_condition = matches.name("log_condition");
        let log_expr_true = matches.name("log_expr_true");
        let log_expr_false = matches.name("log_expr_false");

        let mut condition_wrap: Option<String> = None;
        let mut expr_true_wrap: Option<String> = None;
        let mut expr_false_wrap: Option<String> = None;

        if condition.is_some() && expr_true.is_some() && expr_false.is_some() {
            condition_wrap = Some(condition.unwrap().as_str().to_string());
            expr_true_wrap = Some(expr_true.unwrap().as_str().to_string());
            expr_false_wrap = Some(expr_false.unwrap().as_str().to_string());
        }
        if log_condition.is_some() && log_expr_true.is_some() && log_expr_false.is_some() {
            condition_wrap = Some(log_condition.unwrap().as_str().to_string());
            expr_true_wrap = Some(log_expr_true.unwrap().as_str().to_string());
            expr_false_wrap = Some(log_expr_false.unwrap().as_str().to_string());
        }

        let obj = Self{
            function_text: function_text.clone(),
            condition: condition_wrap,
            expr_true: expr_true_wrap,
            expr_false: expr_false_wrap,
            data_map: data_map,
            table: table,
        };
        
        return obj
    }
    pub fn do_validate(
        function_text: &String, 
        validate_tuple: (u32, Vec<String>),
    ) -> (u32, Vec<String>) {
        let (number_fails, mut failed_functions) = validate_tuple;
        let concat_obj = IfFunction::defaults(
            &function_text, None, None
        );
        let check = concat_obj.validate();
        let mut number_fails = number_fails.clone();
        if check == false {
            number_fails += 1;
            failed_functions.push(String::from(FUNCTION_IF));
        }
        return (number_fails, failed_functions);
    }
    pub fn do_replace(
        function_text: &String, 
        mut formula: String,
        insert_data_map: HashMap<String, String>,
        table: &DbData,
    ) -> String {
        let table = table.clone();
        let mut concat_obj = IfFunction::defaults(
            &function_text, Some(insert_data_map), Some(table)
        );
        formula = concat_obj.replace(formula);
        return formula
    }
}
impl Function for IfFunction {
    fn validate(&self) -> bool {
        let expr = RE_IF.clone();
        let function_text = self.function_text.clone();
        let check = expr.is_match(&function_text);
        return check
    }
    fn replace(&mut self, formula: String) -> String {
        // let insert_data_map = insert_data_map.clone();
        let insert_data_map = self.data_map.clone().unwrap();
        let function_text = self.function_text.clone();
        let mut formula = formula.clone();
        // let table = table.clone();
        let table = self.table.clone().unwrap();

        let expr_true_wrap = self.expr_true.clone();
        let expr_false_wrap = self.expr_false.clone();
        let expr_true = expr_true_wrap.unwrap();
        let expr_false = expr_false_wrap.unwrap();

        let formula_obj = Formula::defaults(
            Some(insert_data_map),
            Some(table.clone()),
        );
        let formula_wrap = formula_obj.inyect_data_formula(&formula);
        let formula_if = formula_wrap.unwrap();
        let replacement_string: String;

        let matches = RE_IF_REPLACED.captures(&formula_if);
        let matches = matches.unwrap();
        let processed_condition = matches.name("condition").unwrap().as_str().to_string();
        let formula_obj = Formula::defaults(
            self.data_map.clone(),
            self.table.clone(),
        );
        let result = formula_obj.execute(&processed_condition).unwrap();

        if result == "TRUE" {
            let formula_obj = Formula::defaults(
                self.data_map.clone(), 
                self.table.clone()
            );
            let result = formula_obj.execute(&expr_true).unwrap();
            replacement_string = result;
        } else {
            let formula_obj = Formula::defaults(
                self.data_map.clone(), 
                self.table.clone()
            );
            let result = formula_obj.execute(&expr_false).unwrap();
            replacement_string = result;
        }

        formula = formula.replace(function_text.as_str(), replacement_string.as_str());
        formula = format!("\"{}\"", formula);
        return formula;
    }
}

pub fn and(
    data_map: &HashMap<String, String>, 
    attributes: &Vec<FunctionAttributeItem>
) -> Result<bool, PlanetError> {
    // AND({My Field}="pepito", {Status}="c4vhm0gsmpv7omu4aqg0")

    // 2.
    // AND(
    //    OR({This Field}=78, {Other Field}="hola"),
    //    {This Way}=TRIM(" other ")
    // )

    let mut check_all = true;
    for attribute in attributes {
        let assignment = attribute.assignment.clone();
        let attr_type = attribute.attr_type.clone();
        if assignment.is_some() {
            let assignment = assignment.unwrap();
            let check = check_assignment(assignment, attr_type, data_map)?;
            if check == false {
                check_all = false;
                break
            }
        }
    }
    return Ok(check_all)
}

pub fn or(
    data_map: &HashMap<String, String>, 
    attributes: &Vec<FunctionAttributeItem>
) -> Result<bool, PlanetError> {
    let mut check = false;
    for attribute in attributes {
        let assignment = attribute.assignment.clone();
        let attr_type = attribute.attr_type.clone();
        if assignment.is_some() {
            let assignment = assignment.unwrap();
            let check_item = check_assignment(assignment, attr_type, data_map)?;
            if check_item == true {
                check = true;
                break
            }
        }
    }
    return Ok(check)
}

pub fn not(
    data_map: &HashMap<String, String>, 
    attributes: &Vec<FunctionAttributeItem>
) -> Result<bool, PlanetError> {
    let mut check_all = true;
    for attribute in attributes {
        let assignment = attribute.assignment.clone();
        let attr_type = attribute.attr_type.clone();
        if assignment.is_some() {
            let assignment = assignment.unwrap();
            let check_item = check_assignment(assignment, attr_type, data_map)?;
            if check_item == true {
                check_all = false;
                break
            }
        }
    }
    return Ok(check_all)
}

pub fn xor(
    data_map: &HashMap<String, String>, 
    attributes: &Vec<FunctionAttributeItem>
) -> Result<bool, PlanetError> {
    // Only 1 of the items needs to be true
    // Many true, we return false, which is not the case on the OR
    let mut check = false;
    let mut count = 0;
    for attribute in attributes {
        let assignment = attribute.assignment.clone();
        let attr_type = attribute.attr_type.clone();
        if assignment.is_some() {
            let assignment = assignment.unwrap();
            let check_item = check_assignment(assignment, attr_type, data_map)?;
            if check_item == true {
                check = true;
                count += 1;
            }
        }
    }
    if count == 1 {
        check = true;
    }
    return Ok(check)
}
