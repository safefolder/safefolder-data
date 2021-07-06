// This is the module with the functions that will create schema for the tables, with field definition that comes
// From the work already done in December 2020.
// 

// Each db is having a table

/// This is the documentation for the function
pub fn create_table(account_id: &str, space_id: &str, yaml_config: &str) {
    println!("schema.rs :: create table command... {}:{}", account_id, space_id);
    println!("{}", yaml_config);
    // 1. validate the yaml config file
    // 2. If no account, then it is applied into global planet
    // 3. Write schema content into tables db file in source field
}
