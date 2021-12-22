extern crate tr;
extern crate colored;
extern crate slug;

use std::collections::BTreeMap;
use std::time::Instant;

use tr::tr;
use colored::*;
use regex::Regex;
use slug::slugify;
use std::str::FromStr;


use crate::commands::table::config::*;
use crate::storage::constants::*;
use crate::commands::table::{Command};
use crate::commands::{CommandRunner};
use crate::planet::constants::{ID, NAME};
use crate::storage::table::{DbTable, DbRow, Row, Schema, DbData, GetItemOption};
use crate::storage::table::*;
use crate::storage::ConfigStorageField;
use crate::planet::{
    PlanetContext, 
    PlanetError,
    Context,
    validation::PlanetValidationError,
};
use crate::storage::fields::{text::*, StorageField};
use crate::storage::fields::number::*;
use crate::storage::fields::date::*;
use crate::storage::fields::formula::*;

pub struct InsertIntoTable<'gb> {
    pub planet_context: &'gb PlanetContext<'gb>,
    pub context: &'gb Context<'gb>,
    pub db_table: &'gb DbTable<'gb>,
    pub config: InsertIntoTableConfig,
}

impl<'gb> InsertIntoTable<'gb> {

    pub fn run(&self) -> Result<DbData, Vec<PlanetError>> {
        let t_1 = Instant::now();
        let command = self.config.command.clone().unwrap_or_default();
        let expr = Regex::new(r#"(INSERT INTO TABLE) "(?P<table_name>[a-zA-Z0-9_ ]+)"#).unwrap();
        let table_name_match = expr.captures(&command).unwrap();
        let table_name = &table_name_match["table_name"].to_string();
        let table_file = slugify(&table_name);
        let table_file = table_file.as_str().replace("-", "_");

        let result: Result<DbRow<'gb>, PlanetError> = DbRow::defaults(
            &table_file,
            self.db_table,
            self.planet_context,
            self.context,
        );
        let mut errors: Vec<PlanetError> = Vec::new();
        match result {
            Ok(_) => {
                // let data_config = self.config.data.clone();
                let db_row: DbRow<'gb> = result.unwrap();
                // I need to get SchemaData and schema for the table
                // I go through fields in order to build RowData
                let table = self.db_table.get_by_name(table_name);
                if table.is_err() {
                    let error = table.unwrap_err();
                    errors.push(error);
                    return Err(errors);
                }

                // routing
                let account_id = Some(self.context.account_id.unwrap_or_default().to_string());
                let space_id = Some(self.context.space_id.unwrap_or_default().to_string());
                let routing_wrap = RoutingData::defaults(
                    account_id, 
                    space_id, 
                    None
                );
                let table = table.unwrap();
                if *&table.is_none() {
                    errors.push(
                        PlanetError::new(
                            500, 
                            Some(tr!("Could not find table {}", &table_name)),
                        )
                    );
                    return Err(errors);
                }    

                let table = table.unwrap();
                let table_name = &table.clone().name.unwrap();
                // eprintln!("InsertIntoTable.run :: table: {:#?}", &table);

                // I need a way to get list of instance FieldConfig (fields)
                let config_fields = FieldConfig::parse_from_db(&table);
                if config_fields.is_err() {
                    let error = config_fields.unwrap_err();
                    errors.push(error);
                    return Err(errors);
                }
                let config_fields = config_fields.unwrap();
                // eprintln!("InsertIntoTable.run :: config_fields: {:#?}", &config_fields);

                let insert_data_map: BTreeMap<String, String> = self.config.data.clone().unwrap();
                // I need to have {id} -> Value
                let mut insert_id_data_map: BTreeMap<String, String> = BTreeMap::new();
                // eprintln!("InsertIntoTable.run :: table: {:#?}", &table);
                // table.data_objects
                let table_data = table.clone().data_objects.unwrap();
                for (name, value) in insert_data_map.clone() {
                    let id_map = table_data.get(&name);
                    if id_map.is_some() {
                        let id_map = id_map.unwrap();
                        let id = id_map.get(ID);
                        if id.is_some() {
                            let id = id.unwrap().clone();
                            insert_id_data_map.insert(id, value);
                        }
                    }
                }
                
                // let insert_data_collections_map = self.config.data_collections.clone().unwrap();
                // eprintln!("InsertIntoTable.run :: insert_data__collections_map: {:#?}", &insert_data_collections_map);
                // TODO: Change for the item name
                // We will use this when we have the Name field, which is required in all tables
                // eprintln!("InsertIntoTable.run :: routing_wrap: {:#?}", &routing_wrap);

                // Keep in mind on name attribute for DbData
                // 1. Can be small text or any other field, so we need to do validation and generation of data...
                // 2. Becaouse if formula is generated from other fields, is generated number or id is also generated
                // I also need a set of fields allowed to be name (Small Text, Formula), but this in future....
                // name on YAML not required, since can be generated
                // Check field type and attribute to validate
                // So far only take Small Text
                let name_field: FieldConfig = FieldConfig::get_name_field(&table).unwrap();
                let name_field_type = name_field.field_type.unwrap().clone();
                let insert_name = self.config.name.clone();
                // Only support so far Small Text and needs to be informed in YAML with name
                if name_field_type != FIELD_TYPE_SMALL_TEXT.to_string() || insert_name.is_none() {
                    errors.push(
                        PlanetError::new(
                            500, 
                            Some(tr!("You need to include name field when inserting data into database.
                             Only \"Small Text\" supported so far")),
                        )
                    );
                }
                let name = insert_name.unwrap();
                // Check name does not exist
                // eprintln!("InsertIntoTable.run :: name: {}", &name);
                let name_exists = self.check_name_exists(&table_name, &name, &db_row);
                // eprintln!("InsertIntoTable.run :: name_exists: {}", &name_exists);
                if name_exists {
                    errors.push(
                        PlanetError::new(
                            500, 
                            Some(tr!("A record with name \"{}\" already exists in database", &name)),
                        )
                    );
                }

                // Instantiate DbData and validate
                let mut db_context: BTreeMap<String, String> = BTreeMap::new();
                db_context.insert(TABLE_NAME.to_string(), table_name.clone());
                let db_data = DbData::defaults(
                    &name,
                    None,
                    None,
                    None,
                    None,
                    routing_wrap,
                    None,
                );
                if db_data.is_err() {
                    let error = db_data.unwrap_err();
                    errors.push(error);
                    return Err(errors)
                }
                let mut db_data = db_data.unwrap();
                let mut data: BTreeMap<String, String> = BTreeMap::new();
                let mut field_config_map: BTreeMap<String, FieldConfig> = BTreeMap::new();
                for field in config_fields.clone() {
                    let field_name = field.name.clone().unwrap();
                    field_config_map.insert(field_name, field.clone());
                }
                for field in config_fields {
                    let field_config = field.clone();
                    let field_type = field.field_type.unwrap_or_default();
                    let field_type = field_type.as_str();
                    // eprintln!("InsertIntoTable.run :: field_type: {}", field_type);
                    let field_id = field.id.unwrap_or_default();
                    let field_data = insert_id_data_map.get(&field_id);
                    if field_data.is_none() && 
                        (
                            field_type != FIELD_TYPE_FORMULA && 
                            field_type != FIELD_TYPE_CREATED_TIME && 
                            field_type != FIELD_TYPE_LAST_MODIFIED_TIME
                        ) {
                        continue
                    }
                    let field_data_: String;
                    if field_data.is_some() {
                        field_data_ = field_data.unwrap().clone();
                    } else {
                        field_data_ = String::from("");
                    }
                    let field_data = field_data_;
                    // let field_data = field_data.unwrap().clone();
                    // let field_data: String;
                    let mut field_data_wrap: Result<String, PlanetError> = Ok(String::from(""));
                    // eprintln!("InsertIntoTable.run :: field_name: {}", &field_name);
                    match field_type {
                        FIELD_TYPE_SMALL_TEXT => {
                            let obj = SmallTextField::defaults(&field_config);
                            field_data_wrap = obj.validate(&field_data);
                        },
                        FIELD_TYPE_LONG_TEXT => {
                            let obj = LongTextField::defaults(&field_config);
                            field_data_wrap = obj.validate(&field_data);
                        },
                        FIELD_TYPE_CHECKBOX => {
                            let obj = CheckBoxField::defaults(&field_config);
                            field_data_wrap = obj.validate(&field_data);
                        },
                        FIELD_TYPE_NUMBER => {
                            let obj = NumberField::defaults(&field_config);
                            field_data_wrap = obj.validate(&field_data);
                        },
                        FIELD_TYPE_SELECT => {
                            let obj = SelectField::defaults(&field_config, Some(&table));
                            field_data_wrap = obj.validate(&field_data);
                        },
                        FIELD_TYPE_FORMULA => {
                            let obj = FormulaField::defaults(&field_config);
                            field_data_wrap = obj.validate(&data, &field_config_map);
                        },
                        FIELD_TYPE_DATE => {
                            let obj = DateField::defaults(&field_config);
                            field_data_wrap = obj.validate(&field_data);
                        },
                        FIELD_TYPE_DURATION => {
                            let obj = DurationField::defaults(&field_config);
                            field_data_wrap = obj.validate(&field_data);
                        },
                        FIELD_TYPE_CREATED_TIME => {
                            let obj = AuditDateField::defaults(&field_config);
                            field_data_wrap = obj.validate(&field_data);
                        },
                        FIELD_TYPE_LAST_MODIFIED_TIME => {
                            let obj = AuditDateField::defaults(&field_config);
                            field_data_wrap = obj.validate(&field_data);
                        },
                        _ => {
                            errors.push(
                                PlanetError::new(
                                    500, 
                                    Some(tr!("Field \"{}\" not supported.", &field_type)),
                                )
                            );        
                        }
                    };
                    let tuple = handle_field_response(
                        &field_data_wrap, &errors, &field_id, &data
                    );
                    data = tuple.0;
                    errors = tuple.1;
                }
                if errors.len() > 0 {
                    return Err(errors)
                }
                db_data.data = Some(data);
                eprintln!("InsertIntoTable.run :: I will write: {:#?}", &db_data);
                let response= db_row.insert(&table_name, &db_data);
                if response.is_err() {
                    let error = response.unwrap_err();
                    errors.push(error);
                    return Err(errors)
                }
                let response = response.unwrap();
                eprintln!("InsertIntoTable.run :: time: {} µs", &t_1.elapsed().as_micros());
                return Ok(response);
            },
            Err(error) => {
                errors.push(error);
                return Err(errors);
            }
        }
    }

    pub fn runner(runner: &CommandRunner, path_yaml: &String) -> () {
        let t_1 = Instant::now();
        let config_ = InsertIntoTableConfig::defaults(None);
        let config: Result<InsertIntoTableConfig, Vec<PlanetValidationError>> = config_.import(
            runner.planet_context,
            &path_yaml
        );
        match config {
            Ok(_) => {
                let db_table= DbTable::defaults(
                    runner.planet_context,
                    runner.context,
                ).unwrap();
        
                let insert_into_table: InsertIntoTable = InsertIntoTable{
                    planet_context: runner.planet_context,
                    context: runner.context,
                    config: config.unwrap(),
                    db_table: &db_table,
                };
                let result: Result<_, Vec<PlanetError>> = insert_into_table.run();
                match result {
                    Ok(_) => {
                        println!();
                        println!("{}", String::from("[OK]").green());
                        eprint!("Time: {} ms", &t_1.elapsed().as_millis());
                    },
                    Err(errors) => {
                        let count = 1;
                        println!();
                        println!("{}", tr!("I found these errors").red().bold());
                        println!("{}", "--------------------".red());
                        println!();
                        for error in errors {
                            println!(
                                "{}{} {}", 
                                count.to_string().blue(),
                                String::from('.').blue(),
                                error.message
                            );
                        }
                    }
                }
            },
            Err(errors) => {
                println!();
                println!("{}", tr!("I found these errors").red().bold());
                println!("{}", "--------------------".red());
                println!();
                let mut count = 1;
                for error in errors {
                    println!(
                        "{}{} {}", 
                        count.to_string().blue(), 
                        String::from('.').blue(), 
                        error.message
                    );
                    count += 1;
                }
            }
        }
    }
}

impl<'gb> InsertIntoTable<'gb> {
    pub fn check_name_exists(&self, table_name: &String, name: &String, db_row: &DbRow) -> bool {
        let check: bool;
        let name = name.clone();
        let result = db_row.get(&table_name, GetItemOption::ByName(name), None);
        eprintln!("InsertIntoTable.check_name_exists :: get response: {:#?}", &result);
        match result {
            Ok(_) => {
                check = true
            },
            Err(_) => {
                check = false
            }
        }
        return check
    }
}

pub struct GetFromTable<'gb> {
    pub planet_context: &'gb PlanetContext<'gb>,
    pub context: &'gb Context<'gb>,
    pub db_table: &'gb DbTable<'gb>,
    pub config: GetFromTableConfig,
}

impl<'gb> Command<String> for GetFromTable<'gb> {

    fn run(&self) -> Result<String, PlanetError> {
        let command = self.config.command.clone().unwrap_or_default();
        let expr = Regex::new(r#"(GET FROM TABLE) "(?P<table_name>[a-zA-Z0-9_ ]+)""#).unwrap();
        let table_name_match = expr.captures(&command).unwrap();
        let table_name = &table_name_match["table_name"].to_string();
        let table_file = slugify(&table_name);
        let table_file = table_file.as_str().replace("-", "_");

        let result: Result<DbRow<'gb>, PlanetError> = DbRow::defaults(
            &table_file,
            self.db_table,
            self.planet_context,
            self.context,
        );
        match result {
            Ok(_) => {
                // let data_config = self.config.data.clone();
                let db_row: DbRow<'gb> = result.unwrap();
                // I need to get SchemaData and schema for the table
                // I go through fields in order to build RowData                
                let table = self.db_table.get_by_name(table_name)?;
                if *&table.is_none() {
                    return Err(
                        PlanetError::new(
                            500, 
                            Some(tr!("Could not find table {}", &table_name)),
                        )
                    );
                }
                let table = table.unwrap();
                let data_collections = table.clone().data_collections;
                let field_ids = data_collections.unwrap().get(FIELD_IDS).unwrap().clone();
                let config_fields = FieldConfig::parse_from_db(&table)?;
                let field_id_map: BTreeMap<String, FieldConfig> = FieldConfig::get_field_id_map(&config_fields)?;
                let fields = self.config.data.clone().unwrap().fields;
                eprintln!("GetFromTable.run :: fields: {:?}", &fields);
                let item_id = self.config.data.clone().unwrap().id.unwrap();
                // Get item from database
                let db_data = db_row.get(&table_name, GetItemOption::ById(item_id), fields)?;
                // data and basic fields
                let data = db_data.data;
                let mut yaml_out_str = String::from("---\n");
                // id
                let id_yaml_value = self.config.data.clone().unwrap().id.unwrap().truecolor(
                    YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
                );
                let id_yaml = format!("{}", 
                    id_yaml_value.truecolor(YAML_COLOR_ORANGE[0], YAML_COLOR_ORANGE[1], YAML_COLOR_ORANGE[2]), 
                );
                yaml_out_str.push_str(format!("{field}: {value}\n", 
                    field=String::from(ID).truecolor(
                        YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
                    ), 
                    value=&id_yaml
                ).as_str());
                // name
                let name_yaml_value = &db_data.name.unwrap().clone();
                let name_yaml = format!("{}", 
                    name_yaml_value.truecolor(YAML_COLOR_ORANGE[0], YAML_COLOR_ORANGE[1], YAML_COLOR_ORANGE[2]), 
                );
                yaml_out_str.push_str(format!("{field}: {value}\n", 
                    field=String::from(NAME).truecolor(
                        YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
                    ), 
                    value=&name_yaml
                ).as_str());
                yaml_out_str.push_str(format!("{}\n", 
                    String::from("data:").truecolor(YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]),
                ).as_str());
                if data.is_some() {
                    // field_id -> string value
                    let data = data.unwrap();
                    // I need to go through in same order as fields were registered in FieldConfig when creating schema
                    for field_id_data in field_ids {
                        let field_id = field_id_data.get(ID).unwrap();
                        let field_config = field_id_map.get(field_id).unwrap().clone();
                        let field_config_ = field_config.clone();
                        let field_type = field_config.field_type.unwrap();
                        let field_type =field_type.as_str();
                        let value = data.get(field_id);
                        if value.is_none() {
                            continue
                        }
                        let value = value.unwrap();
                        // Get will return YAML document for the data
                        match field_type {
                            FIELD_TYPE_SMALL_TEXT => {
                                let obj = SmallTextField::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            FIELD_TYPE_LONG_TEXT => {
                                let obj = LongTextField::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            FIELD_TYPE_CHECKBOX => {
                                let obj = CheckBoxField::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            FIELD_TYPE_NUMBER => {
                                let obj = NumberField::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            FIELD_TYPE_SELECT => {
                                let obj = SelectField::defaults(&field_config_, Some(&table));
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            FIELD_TYPE_FORMULA => {
                                let obj = FormulaField::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            FIELD_TYPE_DATE => {
                                let obj = DateField::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            FIELD_TYPE_DURATION => {
                                let obj = DurationField::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },                            
                            _ => {
                                yaml_out_str = yaml_out_str;
                            }
                        }
                    }
                }
                eprintln!("{}", yaml_out_str);
                return Ok(yaml_out_str);
            },
            Err(error) => {
                return Err(error);
            }
        }
    }

    fn runner(runner: &CommandRunner, path_yaml: &String) -> () {
        let config_ = GetFromTableConfig::defaults(
            String::from("")
        );
        let config: Result<GetFromTableConfig, Vec<PlanetValidationError>> = config_.import(
            runner.planet_context,
            &path_yaml
        );
        match config {
            Ok(_) => {
                let db_table= DbTable::defaults(
                    runner.planet_context,
                    runner.context,
                ).unwrap();

                let insert_into_table: GetFromTable = GetFromTable{
                    planet_context: runner.planet_context,
                    context: runner.context,
                    config: config.unwrap(),
                    db_table: &db_table,
                };
                let result: Result<_, PlanetError> = insert_into_table.run();
                match result {
                    Ok(_) => {
                        println!();
                        println!("{}", String::from("[OK]").green());
                    },
                    Err(error) => {
                        let count = 1;
                        println!();
                        println!("{}", tr!("I found these errors").red().bold());
                        println!("{}", "--------------------".red());
                        println!();
                        println!(
                            "{}{} {}", 
                            count.to_string().blue(),
                            String::from('.').blue(),
                            error.message
                        );
                    }
                }
            },
            Err(errors) => {
                println!();
                println!("{}", tr!("I found these errors").red().bold());
                println!("{}", "--------------------".red());
                println!();
                let mut count = 1;
                for error in errors {
                    println!(
                        "{}{} {}", 
                        count.to_string().blue(), 
                        String::from('.').blue(), 
                        error.message
                    );
                    count += 1;
                }
            }
        }
    }
}

pub struct SelectFromTable<'gb> {
    pub planet_context: &'gb PlanetContext<'gb>,
    pub context: &'gb Context<'gb>,
    pub db_table: &'gb DbTable<'gb>,
    pub config: SelectFromTableConfig,
}

impl<'gb> Command<String> for SelectFromTable<'gb> {

    fn run(&self) -> Result<String, PlanetError> {
        let command = self.config.command.clone().unwrap_or_default();
        let expr = Regex::new(r#"(SELECT FROM TABLE) "(?P<table_name>[a-zA-Z0-9_ ]+)""#).unwrap();
        let table_name_match = expr.captures(&command).unwrap();
        let table_name = &table_name_match["table_name"].to_string();
        let table_file = slugify(&table_name);
        let table_file = table_file.as_str().replace("-", "_");
        eprintln!("SelectFromTable.run :: table_file: {}", &table_file);

        let result: Result<DbRow<'gb>, PlanetError> = DbRow::defaults(
            &table_file,
            self.db_table,
            self.planet_context,
            self.context,
        );
        match result {
            Ok(_) => {
                let db_row: DbRow<'gb> = result.unwrap();
                let config = self.config.clone();
                let r#where = config.r#where;
                let page = config.page;
                let number_items = config.number_items;
                let fields = config.fields;
                let mut page_wrap: Option<usize> = None;
                let mut number_items_wrap: Option<usize> = None;
                if page.is_some() {
                    let page_string = page.unwrap();
                    let page_number: usize = FromStr::from_str(page_string.as_str()).unwrap();
                    page_wrap = Some(page_number)
                }
                if number_items.is_some() {
                    let number_items_string = number_items.unwrap();
                    let number_items: usize = FromStr::from_str(number_items_string.as_str()).unwrap();
                    number_items_wrap = Some(number_items)
                }
                let result = db_row.select(
                    table_name, 
                    r#where, 
                    page_wrap, 
                    number_items_wrap, 
                    fields,
                )?;
                eprintln!("SelectFromTable :: result: {:#?}", &result);
                // Later on, I do pretty print
            },
            Err(error) => {
                return Err(error);
            }
        }

        return Ok(String::from(""));
    }
    fn runner(runner: &CommandRunner, path_yaml: &String) -> () {
        let config_ = SelectFromTableConfig::defaults(
            None,
            None,
            None
        );
        let config: Result<SelectFromTableConfig, Vec<PlanetValidationError>> = config_.import(
            runner.planet_context,
            &path_yaml
        );
        match config {
            Ok(_) => {
                let db_table= DbTable::defaults(
                    runner.planet_context,
                    runner.context,
                ).unwrap();

                let select_from_table: SelectFromTable = SelectFromTable{
                    planet_context: runner.planet_context,
                    context: runner.context,
                    config: config.unwrap(),
                    db_table: &db_table,
                };
                let result: Result<_, PlanetError> = select_from_table.run();
                match result {
                    Ok(_) => {
                        println!();
                        println!("{}", String::from("[OK]").green());
                    },
                    Err(error) => {
                        let count = 1;
                        println!();
                        println!("{}", tr!("I found these errors").red().bold());
                        println!("{}", "--------------------".red());
                        println!();
                        println!(
                            "{}{} {}", 
                            count.to_string().blue(),
                            String::from('.').blue(),
                            error.message
                        );
                    }
                }
            },
            Err(errors) => {
                println!();
                println!("{}", tr!("I found these errors").red().bold());
                println!("{}", "--------------------".red());
                println!();
                let mut count = 1;
                for error in errors {
                    println!(
                        "{}{} {}", 
                        count.to_string().blue(), 
                        String::from('.').blue(), 
                        error.message
                    );
                    count += 1;
                }
            }
        }
    }
}

fn handle_field_response(
    field_data: &Result<String, PlanetError>, 
    errors: &Vec<PlanetError>, 
    field_id: &String,
    data: &BTreeMap<String, String>
) -> (BTreeMap<String, String>, Vec<PlanetError>) {
    let field_data = field_data.clone();
    let mut errors = errors.clone();
    let mut data = data.clone();
    let field_id = field_id.clone();
    if field_data.is_err() {
        let err = field_data.unwrap_err();
        errors.push(err);
    } else {
        let field_data = field_data.unwrap();
        data.insert(field_id, field_data);
    }
    return (data, errors)
}