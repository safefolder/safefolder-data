use yaml_validator::{
    yaml_rust::YamlLoader,
    Context, Validate,
};
use std::convert::TryFrom;
use std::{fs, env};

struct CommandValidator {
    schema_path: String,
    schema_name: String,
}

impl CommandValidator {

    fn validate(&self, yaml_config: &str) {
        let path_dir = env::current_dir().unwrap();
        let path_schema = format!("{}/src/{}", path_dir.display(), &self.schema_path);
        let schema_config = fs::read_to_string(&path_schema).expect("Could not open file");
        let schemas = vec![
            YamlLoader::load_from_str(&*schema_config).unwrap().remove(0)
        ];
        let context = Context::try_from(&schemas).unwrap();
        let document = YamlLoader::load_from_str(&yaml_config).unwrap().remove(0);
        return context.get_schema(&self.schema_name).unwrap().validate(&context, &document).unwrap();
    }
}

/// Create table for space
pub fn create_table(_account_id: &str, _space_id: &str, yaml_config: &str) {
    // 1. Validate the command to execute
    // 2. Execute the command
    // commands/data_tables/schemas/create_table_schema.yaml
    // create-table

    CommandValidator {
        schema_path: String::from("commands/data_tables/schemas/create_table_schema.yaml"),
        schema_name: String::from("create-table")
    }.validate(&yaml_config);

    // One way is having a struct that has "validate" and "execute", with info on Schema location for command.
    // Then on runner match, we "CommandValidator"?????
    // let path_dir = env::current_dir().unwrap();
    // let path_schema = format!("{}/src/commands/data_tables/schemas/create_table_schema.yaml", path_dir.display());
    // let schema_config = fs::read_to_string(&path_schema)
    //     .expect("Something went wrong reading the YAML schema file");
    // let schemas = vec![
    //     YamlLoader::load_from_str(&*schema_config).unwrap().remove(0)
    // ];
    // let context = Context::try_from(&schemas).unwrap();
    // let document = YamlLoader::load_from_str(&yaml_config).unwrap().remove(0);
    // context.get_schema("create-table").unwrap()
    //     .validate(&context, &document).unwrap();
}

pub trait Command {
    fn validate(&self, yaml_config: &str, schema_name: &str) {
        let schema_path = format!("commands/data_tables/schemas/{}.yaml", &schema_name);
        let path_dir = env::current_dir().unwrap();
        let path_schema = format!("{}/src/{}", path_dir.display(), &schema_path);
        let schema_config = fs::read_to_string(&path_schema).expect("Could not open file");
        let schemas = vec![
            YamlLoader::load_from_str(&*schema_config).unwrap().remove(0)
        ];
        let context = Context::try_from(&schemas).unwrap();
        let document = YamlLoader::load_from_str(&yaml_config).unwrap().remove(0);
        context.get_schema(&schema_name).unwrap().validate(&context, &document).unwrap();
    }
    fn run(&self);
}

pub struct CreateTable<'a> {
    pub schema_name: &'a str,
    pub account_id: &'a str,
    pub space_id: &'a str,
    pub yaml_config: &'a str,
}

impl<'a> Command for CreateTable<'a> {

    fn run(&self) {
        // Insert into account "tables" the config
        println!("I run create table....")
    }

}

pub trait Table {
    fn insert(&self);
    fn update(&self, id: &str);
    fn delete(&self, id: &str);
    fn read(&self, id: &str);
}

pub struct KeyValueTable<'a> {
    pub name: &'a str,
}

impl<'a> Table for KeyValueTable<'a> {

    fn insert(&self) {
        // Hola
    }
    fn update(&self, id: &str) {
        println!("id: {}", id)
    }
    fn delete(&self, id: &str) {
        println!("id: {}", id)
    }
    fn read(&self, id: &str) {
        println!("id: {id}", id=id)
    }

}
