use std::collections::BTreeMap;
use colored::Colorize;
use tr::tr;

use crate::planet::{PlanetError, PlanetContext, Context};
use crate::statements::folder::schema::*;
use crate::storage::constants::*;
use crate::planet::constants::*;
use crate::storage::columns::*;
use crate::storage::folder::*;
use crate::storage::space::*;
use crate::functions::Formula;

#[derive(Debug, Clone)]
pub struct LinkColumn<'gb> {
    pub config: ColumnConfig,
    pub planet_context: &'gb PlanetContext<'gb>,
    pub context: &'gb Context<'gb>,
    pub folder_name: &'gb String,
    pub db_folder: Option<TreeFolder>,
    pub space_database: Option<SpaceDatabase>,
}
impl<'gb> LinkColumn<'gb> {
    pub fn defaults(
        planet_context: &'gb PlanetContext, 
        context: &'gb Context, 
        column_config: &ColumnConfig,
        folder_name: &'gb String,
        db_folder: Option<TreeFolder>,
        space_database: Option<SpaceDatabase>
    ) -> Self {
        let column_config = column_config.clone();
        let field_obj = Self{
            config: column_config,
            planet_context: planet_context,
            context: context,
            db_folder: db_folder,
            folder_name: folder_name,
            space_database: space_database
        };
        return field_obj
    }
}
impl<'gb> ObjectStorageColumn<'gb> for LinkColumn<'gb> {
    fn create_config(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
        _: &HashMap<String, ColumnConfig>,
        _: &String,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let mut field_config_map = field_config_map.clone();
        let config = self.config.clone();
        let many = config.many;
        let linked_folder = config.linked_folder;
        let delete_on_link_drop = config.delete_on_link_drop;
        let folder_name = self.folder_name.clone();
        // linked folder id is required for a link Column
        if linked_folder.is_none() {
            let name = config.name.unwrap_or_default();
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("Column not configured for links: \"{}\". 
                    Linked folder not defined.", name)),
                )
            );
        }
        let linked_folder = linked_folder.unwrap();
        // Get folder config by folder name
        if folder_name.to_lowercase() != linked_folder.to_lowercase() {
            // Only get remote folder in case is not Self connection for Link
            let folder_result = self.db_folder.clone().unwrap().get_by_name(
                &linked_folder
            )?;
            if folder_result.is_none() {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!("Folder \"{}\" not found.", &linked_folder)),
                    )
                );
            }
        }
        field_config_map.insert(LINKED_FOLDER.to_string(), linked_folder);
        // These are options, not required
        if many.is_some() {
            let many = many.unwrap();
            let mut many_string = String::from(FALSE);
            if many {
                many_string = String::from(TRUE);
            }
            field_config_map.insert(MANY.to_string(), many_string);
        }
        if delete_on_link_drop.is_some() {
            let delete_on_link_drop = delete_on_link_drop.unwrap();
            let mut delete_on_link_drop_string = String::from(FALSE);
            if delete_on_link_drop {
                delete_on_link_drop_string = String::from(TRUE);
            }
            field_config_map.insert(DELETE_ON_LINK_DROP.to_string(), delete_on_link_drop_string);
        }
        return Ok(field_config_map)
    }
    fn get_config(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError> {
        let field_config_map = field_config_map.clone();
        let mut config = self.config.clone();
        let many = field_config_map.get(MANY);
        let linked_folder = field_config_map.get(LINKED_FOLDER);
        let delete_on_link_drop = field_config_map.get(DELETE_ON_LINK_DROP);
        if many.is_some() {
            let many = many.unwrap().clone().to_lowercase();
            if many == String::from("1") || many == String::from(TRUE) {
                config.many = Some(true);
            } else {
                config.many = Some(false);
            }
        }
        if linked_folder.is_some() {
            let linked_folder = linked_folder.unwrap().clone();
            config.linked_folder = Some(linked_folder);
        }
        if delete_on_link_drop.is_some() {
            let delete_on_link_drop = delete_on_link_drop.unwrap().clone();
            if delete_on_link_drop == String::from("1") || delete_on_link_drop == String::from(TRUE) {
                config.delete_on_link_drop = Some(true);
            } else {
                config.delete_on_link_drop = Some(false);
            }
        }
        return Ok(config)
    }
    fn validate(
        &self, 
        data: &Vec<String>, 
    ) -> Result<Vec<String>, Vec<PlanetError>> {
        //eprintln!("LinkColumn.validate :: data: {:?}", data);
        let data = data.clone();
        let config = self.config.clone();
        let linked_folder = config.linked_folder.unwrap();
        let db_folder = self.db_folder.clone().unwrap();
        // eprintln!("LinkColumn.validate  :: linked_folder: {}", &linked_folder);
        let folder = db_folder.get_by_name(&linked_folder);
        // let folder = db_folder.get(&linked_folder_id);
        if folder.is_err() {
            let error = folder.unwrap_err();
            let mut errors: Vec<PlanetError> = Vec::new();
            errors.push(error);
            return Err(errors)
        }
        let folder = folder.unwrap();
        if folder.is_none() {
            let error = PlanetError::new(
                500, 
                Some(tr!("Folder \"{}\" not found.", &linked_folder)),
            );
            let mut errors: Vec<PlanetError> = Vec::new();
            errors.push(error);
            return Err(errors);
        }
        let folder = folder.unwrap();
        let linked_folder_id = folder.id.unwrap_or_default();
        let folder_name = folder.name.unwrap();
        //eprintln!("LinkColumn.validate  :: folder_name: {}", &folder_name);
        let many = config.many.unwrap();
        //eprintln!("LinkColumn.validate  :: many: {}", &many);
        if many == false && data.len() > 1 {
            let error = PlanetError::new(
                500, 
                Some(tr!("Link is not configured for many items. Length items sent: {}", &data.len())),
            );
            let mut errors: Vec<PlanetError> = Vec::new();
            errors.push(error);
            return Err(errors);
        }
        let home_dir = self.planet_context.home_path.clone();
        let account_id = self.context.account_id.clone().unwrap_or_default();
        let space_id = self.context.space_id;
        let site_id = self.context.site_id.clone();
        let space_database = self.space_database.clone();
        let space_database = space_database.unwrap();
        let result: Result<TreeFolderItem, PlanetError> = TreeFolderItem::defaults(
            space_database.connection_pool.clone(),
            home_dir.unwrap_or_default().as_str(),
            &account_id,
            space_id,
            Some(site_id.unwrap().to_string()),
            &linked_folder_id,
            &db_folder,
        );
        if result.is_err() {
            let error = PlanetError::new(
                500, 
                Some(tr!("Folder by id: \"{}\" not found", &linked_folder_id)),
            );
            let mut errors: Vec<PlanetError> = Vec::new();
            errors.push(error);
            return Err(errors);
        }
        let mut db_folder_item = result.unwrap();
        // I will check I am able to fetch the link remote by id and fetch name
        for item_id in data.clone() {
            //eprintln!("LinkColumn.validate  :: item_id: {}", &item_id);
            let item = db_folder_item.get(
                &folder_name, 
                GetItemOption::ById(item_id), 
                None
            );
            if item.is_err() {
                let error = item.unwrap_err();
                let mut errors: Vec<PlanetError> = Vec::new();
                errors.push(error);
                return Err(errors)
            }
        }
        return Ok(data);
    }
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String {
        let column_config = self.config.clone();
        let field_name = column_config.name.unwrap();
        let mut yaml_string = yaml_string.clone();
        let field = &field_name.truecolor(
            YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
        );
        let value = format!("{}", value.to_string().truecolor(
            YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
        ));
        let value = format!("{}", value.to_string().truecolor(
            YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
        ));
        yaml_string.push_str(format!("  {field}: {value}\n", field=field, value=value).as_str());
        return yaml_string;
    }
}

#[derive(Debug, Clone)]
pub struct ReferenceColumn<'gb> {
    pub config: ColumnConfig,
    pub planet_context: &'gb PlanetContext<'gb>,
    pub context: &'gb Context<'gb>,
    pub db_folder: Option<TreeFolder>,
}
impl<'gb> ReferenceColumn<'gb> {
    pub fn defaults(
        planet_context: &'gb PlanetContext, 
        context: &'gb Context, 
        column_config: &ColumnConfig,
        db_folder: Option<TreeFolder>,
    ) -> Self {
        let column_config = column_config.clone();
        let field_obj = Self{
            config: column_config,
            planet_context: planet_context,
            context: context,
            db_folder: db_folder,
        };
        return field_obj
    }
}
impl<'gb> ObjectStorageColumn<'gb> for ReferenceColumn<'gb> {
    fn create_config(
        &mut self, 
        field_config_map: &BTreeMap<String, String>,
        properties_map: &HashMap<String, ColumnConfig>,
        folder_name: &String,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let db_folder = self.db_folder.clone().unwrap();
        let mut field_config_map = field_config_map.clone();
        let config = self.config.clone();
        let link_column = config.link_column;
        let remote_column = config.remote_column;
        if link_column.is_none() {
            let name = config.name.unwrap_or_default();
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("Column not configured for reference: \"{}\". \"LinkColumn\" is required.", name)),
                )
            );
        }
        let link_column = link_column.unwrap();
        let link_column_obj = properties_map.get(&link_column);
        if link_column_obj.is_none() {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("Link Column \"{}\" not found.", &link_column)),
                )
            );
        }
        let link = properties_map.get(&link_column);
        let mut linked_folder = String::from("");
        if link.is_some() {
            let link = link.unwrap().clone();
            let many = link.many;
            linked_folder = link.linked_folder.unwrap();
            if many.is_none() {
                field_config_map.insert(String::from(MANY), String::from(FALSE));
            } else {
                let many = many.unwrap();
                if many {
                    field_config_map.insert(String::from(MANY), String::from(TRUE));
                } else {
                    field_config_map.insert(String::from(MANY), String::from(FALSE));
                }
            }
        }
        field_config_map.insert(LINK_COLUMN.to_string(), link_column);
        // Remote column
        if remote_column.is_some() {
            let remote_column = remote_column.unwrap();
            let has_column = db_folder.has_column(&linked_folder, &remote_column);
            if !has_column {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!("Column \"{}\" does not exist at \"{}\" folder.", 
                            &remote_column, &linked_folder
                    )),
                    )
                );
            }
            field_config_map.insert(String::from(REMOTE_COLUMN), remote_column);
        }
        // Formula
        let formula = config.formula;
        if formula.is_some() {
            let formula = formula.unwrap();
            let formula_format = config.formula_format.unwrap();
            // let field_type_map = field_type_map.clone();
            // let field_name_map = field_name_map.clone();

            let folder_name = folder_name.clone();
            let formula_compiled = Formula::defaults(
                &formula,
                &formula_format,
                None,
                Some(properties_map.clone()),
                Some(db_folder.clone()),
                Some(folder_name),
                false,
                None
            )?;
            field_config_map.insert(String::from(FORMULA), formula);
            field_config_map.insert(String::from(FORMULA_FORMAT), formula_format);
            let formula_serialized = serde_yaml::to_string(&formula_compiled).unwrap();
            field_config_map.insert(String::from(FORMULA_COMPILED), formula_serialized);
        }
        //eprintln!("Reference.create_config :: end field_config_map: {:#?}", &field_config_map);
        return Ok(field_config_map)
    }
    fn get_config(
        &mut self, 
        _: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError> {
        let config = self.config.clone();
        return Ok(config)
    }
    fn validate(
        &self, 
        data: &Vec<String>, 
    ) -> Result<Vec<String>, Vec<PlanetError>> {
        let data = data.clone();
        return Ok(data);
    }
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String {
        let column_config = self.config.clone();
        let field_name = column_config.name.unwrap();
        let mut yaml_string = yaml_string.clone();
        let field = &field_name.truecolor(
            YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
        );
        let value = format!("{}", value.to_string().truecolor(
            YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
        ));
        let value = format!("{}", value.to_string().truecolor(
            YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
        ));
        yaml_string.push_str(format!("  {field}: {value}\n", field=field, value=value).as_str());
        return yaml_string;
    }
}