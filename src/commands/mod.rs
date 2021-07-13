pub mod table;

use crate::planet::{PlanetContext, Context};

pub struct CommandRunner<'a> {
    pub planet_context: &'a PlanetContext,
    pub context: &'a Context,
    pub command: &'a str,
    pub account_id: Option<&'a str>,
    pub space_id: Option<&'a str>,
    pub path_yaml: Option<&'a str>,
}