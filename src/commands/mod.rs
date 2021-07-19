pub mod table;

use serde::{Deserialize, Serialize};
use validator::{Validate};

use crate::planet::{PlanetContext, Context};

pub struct CommandRunner<'a> {
    pub planet_context: PlanetContext,
    pub context: &'a Context,
    pub command: &'a str,
    pub account_id: Option<&'a str>,
    pub space_id: Option<&'a str>,
    pub path_yaml: Option<&'a str>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct ConfigSettings {
    yaml_path: Option<String>,
    planet_context: Option<PlanetContext>,
}

impl ConfigSettings {

    pub fn defaults(yaml_path: String, planet_context: PlanetContext) -> ConfigSettings {
        let settings = ConfigSettings{
            yaml_path: Some(yaml_path),
            planet_context: Some(planet_context)
        };
        return settings;
    }

}
