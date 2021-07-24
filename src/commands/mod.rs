pub mod table;

use crate::planet::{PlanetContext, Context};

#[derive(Debug, Clone)]
pub struct CommandRunner<'ctl, 'gb> {
    pub planet_context: &'gb PlanetContext<'gb>,
    pub context: &'gb Context<'gb>,
    pub command: &'ctl String,
    pub path_yaml: Option<&'ctl String>,
}
