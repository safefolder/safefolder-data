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
    // We receive assignments, like
    // AND({Column}=23, {Column2}="pepito", {ColumnC}>6)
    // We receive attributes, since some attributes might not be assignments, text assign.
    // AND("3"="3")
    // AND({Column} = TRIM(" 234 "))
    // So I can have functions as well
    // So far simple version

    // 1. I first do this one with all execution fn's
    // AND({My Field}="pepito", {Status}="c4vhm0gsmpv7omu4aqg0")
    // tuple = ("MyField", enum::Eq, "pepito")
    // tuple = {name} {op} {value}

    // 2.
    // AND(
    //    OR({This Field}=78, {Other Field}="hola"),
    //    {This Way}=TRIM(" other ")
    // )
    let mut check_all = true;
    for attribute in attributes {
        let assignment = attribute.assignment.clone();
        let attr_type = attribute.attr_type.clone();
        match attr_type {
            AttributeType::Text => {
                if assignment.is_some() {
                    let assignment = assignment.unwrap();
                    // pub struct AttributeAssign(String, FormulaOperator, String);
                    let reference_id = assignment.0;
                    let name = data_map.get(&reference_id);
                    if name.is_some() {
                        let name = name.unwrap();
                        let op = assignment.1;
                        let value = assignment.2;
                        let check: bool;
                        match op {
                            FormulaOperator::Eq => {
                                // name is the data id
                                // I need to check name == value through function
                                check = check_string_equal(name, &value)?;
                                if check == false {
                                    check_all = false;
                                }
                            },
                            _ => {
                            }
                        }    
                    } else {
                        check_all = false;
                    }
                }        
            },
            _ => {
            }
        }
    }
    return Ok(check_all)
}

pub fn simple_assign(
    data_map: &HashMap<String, String>, 
    attribute: &FunctionAttributeItem
) -> Result<bool, PlanetError> {
    // {My Column} = "pepito"
    // {My Column} = 98.89
    // {My Column} = TRIM(" pepito ")
    // {My Column} > 98
    let mut check_all = true;
    let assignment = attribute.assignment.clone();
    let attr_type = attribute.attr_type.clone();
    match attr_type {
        AttributeType::Text => {
            if assignment.is_some() {
                let assignment = assignment.unwrap();
                // pub struct AttributeAssign(String, FormulaOperator, String);
                let reference_id = assignment.0;
                let name = data_map.get(&reference_id);
                if name.is_some() {
                    let name = name.unwrap();
                    let op = assignment.1;
                    let value = assignment.2;
                    let check: bool;
                    match op {
                        FormulaOperator::Eq => {
                            // name is the data id
                            // I need to check name == value through function
                            check = check_string_equal(name, &value)?;
                            if check == false {
                                check_all = false;
                            }
                        },
                        _ => {
                        }
                    }    
                } else {
                    check_all = false;
                }
            }        
        },
        _ => {
        }
    }
    return Ok(check_all)
}
