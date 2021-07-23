pub mod table;

use serde::{Deserialize, Serialize};
use validator::{Validate};

use crate::planet::{PlanetContext, Context};
use crate::commands::table::config::DbTableConfig;
use crate::storage::generate_id;

#[derive(Debug, Clone)]
pub struct CommandRunner<'ctl, 'gb> {
    pub planet_context: &'gb PlanetContext<'gb>,
    pub context: &'gb Context<'gb>,
    pub command: &'ctl String,
    pub path_yaml: Option<&'ctl String>,
}

// #[derive(Debug, Serialize, Deserialize, Clone)]
// pub struct ConfigSettings<'bs, 'gb> {
//     yaml_path: Option<&'bs String>,
//     planet_context: Option<&'gb PlanetContext<'gb>>,
// }

// impl<'bs, 'gb> ConfigSettings<'bs, 'gb> {

//     pub fn defaults(yaml_path: String, planet_context: &PlanetContext) -> ConfigSettings<'bs, 'gb>{
//         let settings: ConfigSettings = ConfigSettings{
//             yaml_path: Some(yaml_path),
//             planet_context: Some(planet_context)
//         };
//         return settings;
//     }

// }
