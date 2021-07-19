pub mod schema;
pub mod config;

// use syn::{parse_quote, spanned::Spanned, GenericParam, Lifetime, LifetimeDef, Type};
use validator::{ValidationErrors};
use tr::tr;
use std::fs;
// use proc_macro_error::{abort, proc_macro_error};


use crate::commands::CommandRunner;


pub trait ImportConfig {
    fn import_new(&self) -> ();
}

impl<T: ImportConfig> ImportConfig for &T {
    fn import_new(&self) -> () {
        T::import_new(*self)
    }
}


// #[proc_macro_derive(ImportConfig)]
// #[proc_macro_error]
// pub fn derive_import(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
//     let ast = syn::parse(input).unwrap();
//     // impl_validate(&ast).into()
//     println!("derive_import....");
// }

pub trait Command {
    // fn validate(&self) -> Result<(), ValidationErrors>;
    fn run(&self);
    fn runner(runner: CommandRunner, path_yaml: String) -> ();
}


pub trait CommandConfig {
    fn is_valid(&self) -> Result<(), ValidationErrors>;
    fn import(&self, yaml_config: String);
}

pub fn fetch_yaml_config(yaml_path: &String) -> String {
    let yaml_config = fs::read_to_string(&yaml_path)
    .expect(&*tr!("Something went wrong reading the YAML file"));
    return yaml_config
}
