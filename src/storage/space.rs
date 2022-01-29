extern crate sled;

use tr::tr;

use crate::planet::{PlanetError};

#[derive(Debug, Clone)]
pub struct SpaceDatabase {
    pub database: sled::Db,
}

impl SpaceDatabase {
    pub fn defaults(
        site_id: Option<&str>, 
        space_id: &str, 
        home_dir: Option<&str>
    ) -> Result<Self, PlanetError> {
        let home_dir = home_dir.unwrap_or_default();
        let space_id = space_id;
        let path: String;
        if site_id.is_none() && space_id == "private" {
            path = format!("{home}/private/database.db", home=&home_dir);
        } else {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("Sites not yet supported, only private spaces")),
                )
            )
        }
        let config: sled::Config = sled::Config::default()
            .use_compression(true)
            .path(path);
        let result: Result<sled::Db, sled::Error> = config.open();
        if result.is_ok() {
            let database = result.unwrap();
            let obj = Self{
                database: database
            };
            return Ok(obj)
        } else {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("Error opening database.db on space \"{}\".", space_id)),
                )
            )
        }
    }
}
