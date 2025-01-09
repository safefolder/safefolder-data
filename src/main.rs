extern crate argparse;
extern crate xid;
extern crate serde_yaml;
extern crate colored;
extern crate lazy_static;
extern crate tokio;
use safefolder_data::storage::constants::PRIVATE;
use colored::Colorize;
use lingua::{LanguageDetector, LanguageDetectorBuilder};

pub mod statements;
pub mod storage;
pub mod planet;
pub mod functions;

use argparse::{ArgumentParser, StoreTrue, Store};
use tr::tr;

use crate::planet::{PlanetContext, Context, ContextSource, Environment, PlanetContextSource};
use planet::constants::*;
use crate::statements::*;

// #[tokio::main]
fn main() {
    let mut verbose = false;
    let mut account_id: String = String::from("");
    let mut space_id: String = String::from(PRIVATE);
    let mut site_id: String = String::from("");
    let mut statement = String::from("");
    let mut op = String::from("run");
    let mut scope = String::from("");
    // println!("account_id: {}", hex::encode_upper(account_id));
    let _: LanguageDetector = LanguageDetectorBuilder::from_languages(&LANGUAGES).with_preloaded_language_models().build();

    { // this block limits scope of borrows by ap.refer() method
        let mut ap = ArgumentParser::new();
        ap.set_description("Safefolder Data Client Tool");
        ap.refer(&mut verbose).add_option(
            &["-v", "--verbose"], 
            StoreTrue,
            "Be verbose");
        ap.refer(&mut account_id).add_option(
            &["-a", "--accountid"], Store,
            "Account Id");
        ap.refer(&mut site_id).add_option(
            &["-s", "--siteid"], Store,
            "Site Id");
        ap.refer(&mut space_id).add_option(
            &["-s", "--spaceid"], Store,
            "Space Id");
        ap.refer(&mut statement).add_option(
            &["-b", "--statement"], Store,
            "Statement");
        ap.refer(&mut op).add_argument(
            "op", 
            Store, 
            "Operation: run, etc..."
        );
        ap.refer(&mut scope).add_argument(
            "scope", 
            Store, 
            "Scope: statement"
        );
        ap.parse_args_or_exit();
    }

    let planet_context_source = PlanetContextSource::import_context().unwrap();
    let planet_context = PlanetContext::import(&planet_context_source);
    let context_source = ContextSource::defaults(space_id, site_id);
    let context = Context::defaults(&context_source);
    //eprintln!("main.rs :: context: {:#?}", &context);

    if op.to_lowercase() == "run" && &scope.to_lowercase() == "statement" {
        eprintln!("main.rs :: run statement...");
        eprintln!("main.rs :: run statement :: statement: {}", &statement);
        let env = Environment{
            context: &context,
            planet_context: &planet_context
        };
        let statement_runner = StatementRunner{
            response_format: StatementResponseFormat::YAML
        };
        let result = statement_runner.call(
            &env, 
            None, 
            &statement,
            None,
            &StatementCallMode::Run
        );
        if result.is_ok() {
            let result = result.unwrap();
            eprintln!("{}", String::from("[OK]").green());
            eprintln!("{}", &result);
        } else {
            let errors = result.unwrap_err();
            eprintln!("{}", tr!("I found these errors").red().bold());
            eprintln!("{}", "--------------------".red());
            eprintln!();
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
