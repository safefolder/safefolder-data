pub mod schema;
pub mod config;

use validator::{ValidationErrors};

pub trait Command {
    fn validate(&self) -> Result<(), ValidationErrors>;
    fn run(&self);
}
