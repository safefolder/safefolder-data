pub mod table;

use serde::{Deserialize, Serialize};
use validator::{Validate};

use crate::planet::{PlanetContext, Context};

#[derive(Debug, Clone)]
pub struct CommandRunner {
    pub planet_context: PlanetContext,
    pub context: Context,
    pub command: String,
    pub path_yaml: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigSettings {
    yaml_path: Option<String>,
    planet_context: Option<PlanetContext>,
}

impl ConfigSettings {

    pub fn defaults(yaml_path: &String, planet_context: &PlanetContext) -> ConfigSettings{
        let settings: ConfigSettings = ConfigSettings{
            yaml_path: Some(yaml_path.clone()),
            planet_context: Some(planet_context.clone())
        };
        return settings;
    }

}
