extern crate tr;
extern crate colored;
extern crate slug;

use std::collections::{BTreeMap, HashMap};
use std::time::Instant;

use tr::tr;
use colored::*;
use regex::Regex;
use slug::slugify;
use std::str::FromStr;


use crate::commands::folder::config::*;
use crate::storage::constants::*;
use crate::commands::folder::{Command};
use crate::commands::{CommandRunner};
use crate::planet::constants::{ID, NAME, VALUE, FALSE, COLUMNS};
use crate::storage::folder::{TreeFolder, TreeFolderItem, FolderItem, FolderSchema, DbData, GetItemOption};
use crate::storage::folder::*;
use crate::storage::{ConfigStorageColumn, generate_id};
use crate::storage::space::{SpaceDatabase};
use crate::planet::{
    PlanetContext, 
    PlanetError,
    Context,
    validation::PlanetValidationError,
};
use crate::storage::columns::{text::*, StorageColumn, ObjectStorageColumn};
use crate::storage::columns::number::*;
use crate::storage::columns::date::*;
use crate::storage::columns::formula::*;
use crate::storage::columns::reference::*;
use crate::storage::columns::structure::*;
use crate::storage::columns::media::*;

pub struct InsertIntoFolder<'gb> {
    pub planet_context: &'gb PlanetContext<'gb>,
    pub context: &'gb Context<'gb>,
    pub db_folder: TreeFolder,
    pub space_database: SpaceDatabase,
    pub config: InsertIntoFolderConfig,
}

impl<'gb> InsertIntoFolder<'gb> {

    fn get_insert_id_data_map(
        &self,
        insert_data_map: &BTreeMap<String, String>,
        folder_data: &BTreeMap<String, Vec<BTreeMap<String, String>>>,
    ) -> (BTreeMap<String, String>, BTreeMap<String, String>) {
        let mut insert_id_data_map: BTreeMap<String, String> = BTreeMap::new();
        let mut insert_id_data_objects_map: BTreeMap<String, String> = BTreeMap::new();
        let mut folder_map_by_name: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
        let columns = folder_data.get(COLUMNS);
        if columns.is_some() {
            let columns = columns.unwrap();
            for column in columns {
                let column_name = column.get(NAME).unwrap();
                folder_map_by_name.insert(column_name.clone(), column.clone());
            }
        }
        for (name, value) in insert_data_map.clone() {
            let map = folder_map_by_name.get(&name);
            if map.is_some() {
                let map = map.unwrap();
                let id = map.get(ID).unwrap().clone();
                let column_type = map.get(COLUMN_TYPE);
                let column_type = column_type.unwrap().clone();
                if column_type == COLUMN_TYPE_LINK.to_string() {
                    // column_id => remote folder item id
                    insert_id_data_objects_map.insert(id, value);
                } else {
                    insert_id_data_map.insert(id, value);
                }
            }
        }
        return (insert_id_data_map, insert_id_data_objects_map)
    }

    fn get_insert_id_data_collections_map(
        &self,
        insert_data_collections_map: Option<BTreeMap<String, Vec<String>>>,
        folder_data: &BTreeMap<String, Vec<BTreeMap<String, String>>>,
    ) -> BTreeMap<String, Vec<String>> {
        let mut insert_id_data_collections_map: BTreeMap<String, Vec<String>> = BTreeMap::new();
        if insert_data_collections_map.is_some() {
            // I receive a map of list of ids
            let mut folder_map_by_name: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
            let columns = folder_data.get(COLUMNS);
            if columns.is_some() {
                let columns = columns.unwrap();
                for column in columns {
                    let column_name = column.get(NAME).unwrap();
                    folder_map_by_name.insert(column_name.clone(), column.clone());
                }
            }
            let insert_data_collections_map = insert_data_collections_map.unwrap();
            for (name, id_list) in insert_data_collections_map {
                let id_map = folder_map_by_name.get(&name);
                if id_map.is_some() {
                    let id_map = id_map.unwrap();
                    let id = id_map.get(ID);
                    if id.is_some() {
                        let id = id.unwrap().clone();
                        insert_id_data_collections_map.insert(id, id_list);
                    }
                }
            }
        }
        return insert_id_data_collections_map
    }

    pub fn run(&self) -> Result<DbData, Vec<PlanetError>> {
        let t_1 = Instant::now();
        let command = self.config.command.clone().unwrap_or_default();
        let expr = Regex::new(r#"(INSERT INTO FOLDER) "(?P<folder_name>[a-zA-Z0-9_ ]+)"#).unwrap();
        let folder_name_match = expr.captures(&command).unwrap();
        let folder_name = &folder_name_match["folder_name"].to_string();
        // eprintln!("InsertIntoFolder.run :: folder_name: {}", folder_name);

        // folder
        let mut errors: Vec<PlanetError> = Vec::new();
        let folder = self.db_folder.get_by_name(folder_name);
        if folder.is_err() {
            let error = folder.unwrap_err();
            errors.push(error);
            return Err(errors);
        }
        let folder = folder.unwrap();
        if *&folder.is_none() {
            errors.push(
                PlanetError::new(
                    500, 
                    Some(tr!("Could not find folder {}", &folder_name)),
                )
            );
            return Err(errors);
        }    

        let folder = folder.unwrap();
        let folder_name = &folder.clone().name.unwrap();
        // eprintln!("InsertIntoFolder.run :: Got folder! folder_name: {}", folder_name);
        let folder_id = folder.clone().id.unwrap();

        let home_dir = self.planet_context.home_path.unwrap_or_default();
        let account_id = self.context.account_id.unwrap_or_default();
        let space_id = self.context.space_id.unwrap_or_default();
        let site_id = self.context.site_id.unwrap_or_default();
        let box_id = self.context.box_id.unwrap_or_default();
        let space_database = self.space_database.clone();

        let result: Result<TreeFolderItem, PlanetError> = TreeFolderItem::defaults(
            space_database.connection_pool.clone(),
            home_dir,
            account_id,
            space_id,
            site_id,
            box_id,
            folder_id.as_str(),
            &self.db_folder,
        );
        match result {
            Ok(_) => {
                let mut db_row: TreeFolderItem = result.unwrap();

                // routing
                let routing_wrap = RoutingData::defaults(
                    Some(account_id.to_string()),
                    Some(site_id), 
                    Some(space_id), 
                    Some(box_id),
                    None
                );
                
                // eprintln!("InsertIntoFolder.run :: folder: {:#?}", &folder);

                // I need a way to get list of instance ColumnConfig (columns)
                let config_columns = ColumnConfig::get_config(
                    self.planet_context,
                    self.context,
                    &folder
                );
                if config_columns.is_err() {
                    let error = config_columns.unwrap_err();
                    errors.push(error);
                    return Err(errors);
                }
                let config_columns = config_columns.unwrap();
                // eprintln!("InsertIntoFolder.run :: config_columns: {:#?}", &config_columns);

                let insert_data_map: BTreeMap<String, String> = self.config.data.clone().unwrap();
                let insert_data_collections_map = self.config.data_collections.clone();
                // I need to have {id} -> Value
                let folder_data = folder.clone().data_collections.unwrap();

                // Validate sub_folder id exists in config for the folder and attach to DbData
                let sub_folders_config = self.config.clone().sub_folders;
                let mut sub_folders_wrap: Option<Vec<SubFolderItem>> = None;
                if sub_folders_config.is_some() {
                    let sub_folders_config = sub_folders_config.unwrap();
                    let mut sub_folders: Vec<SubFolderItem> = Vec::new();
                    for item in sub_folders_config {
                        let item_id = item.id.unwrap();
                        let check = TreeFolder::has_sub_folder_id(
                            &folder.clone(), 
                            &item_id
                        );
                        eprintln!("InsertIntoFolder.run :: item_id: {} check: {}", &item_id, &check);
                        if check {
                            let sub_folder = SubFolderItem{
                                id: Some(item_id),
                                is_reference: item.is_reference,
                                data: None,
                            };
                            sub_folders.push(sub_folder);    
                        } else {
                            errors.push(
                                PlanetError::new(
                                    500, 
                                    Some(tr!(
                                        "Sub folder id \"{}\" does not exist in folder.", &item_id
                                    )),
                                )
                            );
                        }
                    }
                    if sub_folders.len() > 0 {
                        sub_folders_wrap = Some(sub_folders);
                    }
                }

                // get id => value for data, data_objects and data_collections
                let (
                    insert_id_data_map, 
                    insert_id_data_objects_map
                ) = self.get_insert_id_data_map(
                    &insert_data_map, &folder_data
                );
                let insert_id_data_collections_map = self.get_insert_id_data_collections_map(
                    insert_data_collections_map, &folder_data
                );
                
                // process insert config data_collections
                // User authentication
                // TODO: Complete when implement the permission system exchange token by user_id
                let user_id_string = generate_id().unwrap();
                let mut user_id: Vec<String> = Vec::new();
                user_id.push(user_id_string);
                
                // let insert_data_collections_map = self.config.data_collections.clone().unwrap();
                // eprintln!("InsertIntoFolder.run :: insert_data__collections_map: {:#?}", &insert_data_collections_map);
                // TODO: Change for the item name
                // We will use this when we have the Name column, which is required in all tables
                // eprintln!("InsertIntoFolder.run :: routing_wrap: {:#?}", &routing_wrap);

                // Keep in mind on name attribute for DbData
                // 1. Can be small text or any other column, so we need to do validation and generation of data...
                // 2. Becaouse if formula is generated from other columns, is generated number or id is also generated
                // I also need a set of columns allowed to be name (Small Text, Formula), but this in future....
                // name on YAML not required, since can be generated
                // Check column type and attribute to validate
                // So far only take Small Text
                let name_column: ColumnConfig = ColumnConfig::get_name_column(&folder).unwrap();
                let name_column_type = name_column.column_type.unwrap().clone();
                let insert_name = self.config.name.clone();
                // Only support so far Small Text and needs to be informed in YAML with name
                if name_column_type != COLUMN_TYPE_SMALL_TEXT.to_string() || insert_name.is_none() {
                    errors.push(
                        PlanetError::new(
                            500, 
                            Some(tr!("You need to include name column when inserting data into database.
                             Only \"Small Text\" supported so far")),
                        )
                    );
                }
                let name = insert_name.unwrap();
                // Check name does not exist
                // eprintln!("InsertIntoFolder.run :: name: {}", &name);
                let name_exists = self.check_name_exists(&folder_name, &name, &mut db_row);
                // eprintln!("InsertIntoFolder.run :: record name_exists: {}", &name_exists);
                if name_exists {
                    errors.push(
                        PlanetError::new(
                            500, 
                            Some(tr!("A record with name \"{}\" already exists in database", &name)),
                        )
                    );
                }

                // Instantiate DbData and validate
                let mut db_context: BTreeMap<String, String> = BTreeMap::new();
                db_context.insert(FOLDER_NAME.to_string(), folder_name.clone());
                let db_data = DbData::defaults(
                    &name,
                    None,
                    None,
                    None,
                    None,
                    routing_wrap,
                    None,
                    sub_folders_wrap,
                );
                if db_data.is_err() {
                    let error = db_data.unwrap_err();
                    errors.push(error);
                    return Err(errors)
                }
                let mut db_data = db_data.unwrap();
                let mut data: BTreeMap<String, String> = BTreeMap::new();
                let mut data_objects: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
                let mut data_collections: BTreeMap<String, Vec<BTreeMap<String, String>>> = BTreeMap::new();
                let mut column_config_map: BTreeMap<String, ColumnConfig> = BTreeMap::new();
                for column in config_columns.clone() {
                    let column_name = column.name.clone().unwrap();
                    column_config_map.insert(column_name, column.clone());
                }
                let mut links_map: HashMap<String, Vec<ColumnConfig>> = HashMap::new();
                let mut links_data_map: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();
                // eprintln!("InsertIntoFolder.run :: insert_id_data_map: {:#?}", &insert_id_data_map);
                for column in config_columns.clone() {
                    let mut column_data: Option<Vec<String>> = None;
                    let column_config = column.clone();
                    let column_type = column.column_type.unwrap_or_default();
                    let column_type = column_type.as_str();
                    let mut is_set: String = FALSE.to_string();
                    let is_set_wrap = column_config.clone().is_set;
                    if is_set_wrap.is_some() {
                        is_set = is_set_wrap.unwrap();
                    }                    
                    eprintln!("InsertIntoFolder.run :: column_type: {} is_set: {}", column_type, &is_set);
                    let column_name = column.name.unwrap();
                    // I always have a column id
                    let column_id = column.id.unwrap_or_default();
                    
                    let data_item = insert_id_data_map.get(&column_id);
                    if data_item.is_some() {
                        let data_item = data_item.unwrap().clone();
                        let mut items = Vec::new();
                        items.push(data_item);
                        column_data = Some(items);
                    }
                    let data_objects_item = insert_id_data_objects_map.get(
                        &column_id
                    );
                    if data_objects_item.is_some() && column_data.is_none() {
                        let mut items = Vec::new();
                        items.push(data_objects_item.unwrap().clone());
                        column_data = Some(items);
                    }
                    let data_collections_item = insert_id_data_collections_map.get(
                        &column_id
                    );
                    if data_collections_item.is_some() && column_data.is_none() {
                        let data_collections_item = data_collections_item.unwrap().clone();
                        column_data = Some(data_collections_item);
                    }
                    // eprintln!("InsertIntoFolder.run :: column_data: {:?}", &column_data);
                    // In case we don't have any value and is system generated we skip
                    if column_data.is_none() &&
                        (
                            column_type != COLUMN_TYPE_FORMULA && 
                            column_type != COLUMN_TYPE_CREATED_TIME && 
                            column_type != COLUMN_TYPE_LAST_MODIFIED_TIME && 
                            column_type != COLUMN_TYPE_GENERATE_ID && 
                            column_type != COLUMN_TYPE_GENERATE_NUMBER
                        ) {
                        continue
                    }
                    let column_data_: Vec<String>;
                    if column_data.is_some() {
                        column_data_ = column_data.clone().unwrap().clone();
                    } else {
                        let mut items = Vec::new();
                        items.push(String::from(""));
                        column_data_ = items;
                    }
                    let column_data = column_data_;
                    let mut column_data_wrap: Result<Vec<String>, PlanetError> = Ok(Vec::new());
                    let mut skip_data_assign = false;
                    match column_type {
                        COLUMN_TYPE_SMALL_TEXT => {
                            let obj = SmallTextColumn::defaults(&column_config);
                            column_data_wrap = obj.validate(&column_data);
                        },
                        COLUMN_TYPE_LONG_TEXT => {
                            let obj = LongTextColumn::defaults(&column_config);
                            column_data_wrap = obj.validate(&column_data);
                        },
                        COLUMN_TYPE_CHECKBOX => {
                            let obj = CheckBoxColumn::defaults(&column_config);
                            column_data_wrap = obj.validate(&column_data);
                        },
                        COLUMN_TYPE_NUMBER => {
                            let obj = NumberColumn::defaults(&column_config);
                            column_data_wrap = obj.validate(&column_data);
                        },
                        COLUMN_TYPE_SELECT => {
                            let obj = SelectColumn::defaults(
                                &column_config, 
                                Some(&folder)
                            );
                            column_data_wrap = obj.validate(&column_data);
                        },
                        COLUMN_TYPE_FORMULA => {
                            let obj = FormulaColumn::defaults(&column_config);
                            column_data_wrap = obj.validate(&data, &column_config_map);
                        },
                        COLUMN_TYPE_DATE => {
                            let obj = DateColumn::defaults(&column_config);
                            column_data_wrap = obj.validate(&column_data);
                        },
                        COLUMN_TYPE_DURATION => {
                            let obj = DurationColumn::defaults(&column_config);
                            column_data_wrap = obj.validate(&column_data);
                        },
                        COLUMN_TYPE_CREATED_TIME => {
                            let obj = AuditDateColumn::defaults(&column_config);
                            column_data_wrap = obj.validate(&column_data);
                        },
                        COLUMN_TYPE_LAST_MODIFIED_TIME => {
                            let obj = AuditDateColumn::defaults(&column_config);
                            column_data_wrap = obj.validate(&column_data);
                        },
                        COLUMN_TYPE_CREATED_BY => {
                            let obj = AuditByColumn::defaults(&column_config);
                            column_data_wrap = obj.validate(&user_id);
                        },
                        COLUMN_TYPE_LAST_MODIFIED_BY => {
                            let obj = AuditByColumn::defaults(&column_config);
                            column_data_wrap = obj.validate(&user_id);
                        },
                        COLUMN_TYPE_CURRENCY => {
                            let obj = CurrencyColumn::defaults(&column_config);
                            column_data_wrap = obj.validate(&column_data);
                        },
                        COLUMN_TYPE_PERCENTAGE => {
                            let obj = PercentageColumn::defaults(&column_config);
                            column_data_wrap = obj.validate(&column_data);
                        },
                        COLUMN_TYPE_LINK => {
                            let obj = LinkColumn::defaults(
                                self.planet_context,
                                self.context,
                                &column_config,
                                Some(self.db_folder.clone()),
                                Some(self.space_database.clone())
                            );
                            let result = obj.validate(&column_data);
                            if result.is_err() {
                                let error = result.clone().err().unwrap();
                                errors.push(error);
                            }
                            let id_list = result.unwrap();
                            let many = column_config.many.unwrap();
                            if many {
                                let mut items: Vec<BTreeMap<String, String>> = Vec::new();
                                for item_id in id_list.clone() {
                                    let mut map: BTreeMap<String, String> = BTreeMap::new();
                                    map.insert(ID.to_string(), item_id);
                                    items.push(map);
                                }
                                data_collections.insert(column_id.clone(), items);
                            } else {
                                let mut map: BTreeMap<String, String> = BTreeMap::new();
                                let value = id_list[0].clone();
                                map.insert(ID.to_string(), value);
                                data_objects.insert(column_id.clone(), map);
                            }
                            skip_data_assign = true;
                            // links_map
                            let linked_folder_id = column_config.clone().linked_folder_id.unwrap();
                            let map_item = links_map.get(
                                &linked_folder_id
                            );
                            if map_item.is_some() {
                                let mut array = map_item.unwrap().clone();
                                array.push(column_config);
                                links_map.insert(column_id.clone(), array.clone());
                            } else {
                                let mut array: Vec<ColumnConfig> = Vec::new();
                                array.push(column_config);
                                links_map.insert(column_id.clone(), array);
                            }
                            // links_data_map
                            // address folder id => {"Home Addresses" => [jdskdsj], "Work Addresses": [djdks8dsjk]}
                            let map_item_data = links_data_map.get(&linked_folder_id);
                            if map_item_data.is_some() {
                                let mut my_map = map_item_data.unwrap().clone();
                                let my_list_wrap = my_map.get(&column_name.clone());
                                let mut my_list: Vec<String>;
                                if my_list_wrap.is_some() {
                                    my_list = my_list_wrap.unwrap().clone();
                                } else {
                                    my_list = Vec::new();
                                }
                                for item_id in id_list.clone() {
                                    my_list.push(item_id);
                                }
                                my_map.insert(column_name.clone(), my_list);
                                links_data_map.insert(column_id.clone(), my_map);
                            } else {
                                let mut my_map: HashMap<String, Vec<String>> = HashMap::new();
                                let mut my_list: Vec<String> = Vec::new();
                                for item_id in id_list.clone() {
                                    my_list.push(item_id);
                                }
                                my_map.insert(column_name.clone(), my_list);
                                links_data_map.insert(column_id.clone(), my_map);
                            }
                        },
                        COLUMN_TYPE_GENERATE_ID => {
                            let obj = GenerateIdColumn::defaults(&column_config);
                            column_data_wrap = obj.validate(&column_data);
                        },
                        COLUMN_TYPE_GENERATE_NUMBER => {
                            let obj = GenerateNumberColumn::defaults(
                                &column_config,
                                Some(folder.clone()),
                                Some(self.db_folder.clone()),
                            );
                            column_data_wrap = obj.validate(&column_data);
                        },
                        COLUMN_TYPE_PHONE => {
                            let obj = PhoneColumn::defaults(&column_config);
                            column_data_wrap = obj.validate(&column_data);
                        },
                        COLUMN_TYPE_EMAIL => {
                            let obj = EmailColumn::defaults(&column_config);
                            column_data_wrap = obj.validate(&column_data);
                        },
                        COLUMN_TYPE_URL => {
                            let obj = UrlColumn::defaults(&column_config);
                            column_data_wrap = obj.validate(&column_data);
                        },
                        COLUMN_TYPE_RATING => {
                            let obj = RatingColumn::defaults(&column_config);
                            column_data_wrap = obj.validate(&column_data);
                        },
                        COLUMN_TYPE_OBJECT => {
                            let obj = ObjectColumn::defaults(&column_config);
                            column_data_wrap = obj.validate(&column_data);
                        },
                        COLUMN_TYPE_FILE => {
                            let obj = FileColumn::defaults(&column_config);
                            let fields = obj.validate(
                                &column_data,
                                &data_objects,
                                &data_collections
                            );
                            if fields.is_ok() {
                                let fields = fields.unwrap();
                                column_data_wrap = Ok(fields.0);
                                data_objects = fields.1;
                                data_collections = fields.2;
                            }
                            // skip_data_assign = true;
                        },
                        _ => {
                            errors.push(
                                PlanetError::new(
                                    500, 
                                    Some(tr!("Field \"{}\" not supported.", &column_type)),
                                )
                            );
                        }
                    };
                    // eprintln!("InsertIntoFolder.run :: \"{}\" skip_data_assign: {} data: {} objects: {} collections: {}", 
                    //     &column_name,
                    //     &skip_data_assign,
                    //     &column_data_wrap.is_ok(),
                    //     &data_objects.len(),
                    //     &data_collections.len(),
                    // );
                    if skip_data_assign == false {
                        let tuple = handle_field_response(
                            &column_data_wrap, &errors, &column_id, &data, &data_collections, &is_set
                        );
                        data = tuple.0;
                        data_collections = tuple.1;
                        errors = tuple.2;
                    }
                }
                // text and language
                let mut text_map: BTreeMap<String, String> = BTreeMap::new();
                let mut text_column_id: String = String::from("");
                for column_config in config_columns {
                    let column_config_ = column_config.clone();
                    let column_type = &column_config.column_type.unwrap();
                    let column_type = column_type.as_str();
                    let column_id = &column_config.id.unwrap();
                    if column_type == COLUMN_TYPE_TEXT {
                        let obj = TextColumn::defaults(
                            &column_config_,
                            Some(column_config_map.clone()),
                        );
                        text_column_id = column_id.clone();
                        let result_text = obj.validate(
                            &data, 
                            &folder,
                            &text_column_id
                        );
                        if result_text.is_err() {
                            let error_message = result_text.clone().unwrap_err().message;
                            errors.push(
                                PlanetError::new(
                                    500, 
                                    Some(tr!("Error capturing text for folder item: {}", &error_message)),
                                )
                            );
                        }
                        text_map = result_text.unwrap();
                        data_objects.insert(TEXT.to_string(), text_map.clone());
                    } else if column_type == COLUMN_TYPE_LANGUAGE {
                        let obj = LanguageColumn::defaults(
                            &column_config_,
                        );
                        let text_map = text_map.clone();
                        let text = text_map.get(&text_column_id).unwrap();
                        let result_lang = obj.validate(text, &folder);
                        if result_lang.is_err() {
                            let error_message = result_lang.clone().unwrap_err().message;
                            errors.push(
                                PlanetError::new(
                                    500, 
                                    Some(tr!("Error capturing language for folder item: {}", &error_message)),
                                )
                            );
                        }
                        let language_code = result_lang.unwrap();
                        data.insert(column_id.clone(), language_code);
                    }
                }
                if errors.len() > 0 {
                    return Err(errors)
                }
                db_data.data = Some(data);
                db_data.data_objects = None;
                db_data.data_collections = None;
                if data_objects.keys().len() != 0 {
                    db_data.data_objects = Some(data_objects);
                }
                if data_collections.keys().len() != 0 {
                    db_data.data_collections = Some(data_collections);
                }
                eprintln!("InsertIntoFolder.run :: I will write: {:#?}", &db_data);
                let response= db_row.insert(&folder_name, &db_data);
                if response.is_err() {
                    let error = response.unwrap_err();
                    errors.push(error);
                    return Err(errors)
                }
                let response = response.unwrap();
                let id_record = response.clone().id.unwrap();
                
                // links
                for (column_id, config_column_list) in links_map {
                    // Get db item for this link
                    for config in config_column_list {
                        let many = config.many.unwrap();
                        let remote_folder_id = config.linked_folder_id;
                        if remote_folder_id.is_none() {
                            continue
                        }
                        let remote_folder_id = remote_folder_id.unwrap();
                        let folder = self.db_folder.get(&remote_folder_id).unwrap();
                        let folder_name = folder.name.unwrap();
                        let main_data_map = links_data_map.get(&column_id);
                        if main_data_map.is_some() {
                            let main_data_map = main_data_map.unwrap();
                            for (_column_name, id_list) in main_data_map {
                                for item_id in id_list {
                                    let result: Result<TreeFolderItem, PlanetError> = TreeFolderItem::defaults(
                                        space_database.connection_pool.clone(),
                                        home_dir,
                                        account_id,
                                        space_id,
                                        site_id,
                                        box_id,
                                        remote_folder_id.as_str(),
                                        &self.db_folder,
                                    );
                                    if result.is_err() {
                                        // Return error about database problem
                                    }
                                    let mut db_row = result.unwrap();
                                    let linked_item = db_row.get(
                                        &folder_name, 
                                        GetItemOption::ById(item_id.clone()), 
                                        None
                                    );
                                    if linked_item.is_ok() {
                                        let mut linked_item = linked_item.unwrap();
                                        // I may need to update to data_objects or data_collections
                                        if many {
                                            let data_collections_wrap = linked_item.data_collections;
                                            let mut data_collections: BTreeMap<String, Vec<BTreeMap<String, String>>>;
                                            if data_collections_wrap.is_some() {
                                                data_collections = data_collections_wrap.unwrap();
                                            } else {
                                                data_collections = BTreeMap::new();
                                            }
                                            let list_wrap = data_collections.get(&column_id);
                                            let mut list: Vec<BTreeMap<String, String>>;
                                            if list_wrap.is_some() {
                                                list = list_wrap.unwrap().clone();
                                                let mut item_object: BTreeMap<String, String> = BTreeMap::new();
                                                item_object.insert(ID.to_string(), id_record.clone());
                                                list.push(item_object);
                                            } else {
                                                list = Vec::new();
                                                let mut item_object: BTreeMap<String, String> = BTreeMap::new();
                                                item_object.insert(ID.to_string(), id_record.clone());
                                                list.push(item_object);
                                            }
                                            data_collections.insert(column_id.clone(), list);
                                            linked_item.data_collections = Some(data_collections);
                                            let _linked_item = db_row.update(&linked_item);
                                        } else {
                                            let data_objects_wrap = linked_item.data_objects;
                                            let mut data_objects: BTreeMap<String, BTreeMap<String, String>>;
                                            if data_objects_wrap.is_some() {
                                                data_objects = data_objects_wrap.unwrap();
                                            } else {
                                                data_objects = BTreeMap::new();
                                            }
                                            let mut item_object: BTreeMap<String, String> = BTreeMap::new();
                                            item_object.insert(ID.to_string(), id_record.clone());
                                            data_objects.insert(column_id.clone(), item_object);
                                            linked_item.data_objects = Some(data_objects);
                                            let _linked_item = db_row.update(&linked_item);
                                        }
                                    } else {
                                        let error = linked_item.unwrap_err();
                                        eprintln!("InsertIntoFolder.run :: I have error on get linked_item: {}", &error.message);
                                    }
                                }
                            }
                        }
                    }
                }
                eprintln!("InsertIntoFolder.run :: time: {} ms", &t_1.elapsed().as_millis());
                return Ok(response);
            },
            Err(error) => {
                errors.push(error);
                return Err(errors);
            }
        }
    }

    pub fn runner(runner: &CommandRunner, path_yaml: &String) -> () {
        let t_1 = Instant::now();
        let config_ = InsertIntoFolderConfig::defaults(None);
        let config: Result<InsertIntoFolderConfig, Vec<PlanetValidationError>> = config_.import(
            runner.planet_context,
            &path_yaml
        );
        match config {
            Ok(_) => {
                let home_dir = runner.planet_context.home_path.unwrap_or_default();
                let account_id = runner.context.account_id.unwrap_or_default();
                let space_id = runner.context.space_id.unwrap_or_default();
                let site_id = runner.context.site_id;
                let result = SpaceDatabase::defaults(
                    site_id, 
                    space_id, 
                    Some(home_dir),
                    Some(false)
                );
                if result.clone().is_err() {
                    let error = result.clone().unwrap_err();
                    println!();
                    println!("{}", tr!("I found these errors").red().bold());
                    println!("{}", "--------------------".red());
                    println!();
                    println!(
                        "{} {}", 
                        String::from('.').blue(),
                        error.message
                    );
                }
                let space_database = result.unwrap();
        
                let db_folder= TreeFolder::defaults(
                    space_database.connection_pool.clone(),
                    Some(home_dir),
                    Some(account_id),
                    Some(space_id),
                    site_id,
                ).unwrap();
        
                let insert_into_table: InsertIntoFolder = InsertIntoFolder{
                    planet_context: runner.planet_context,
                    context: runner.context,
                    config: config.unwrap(),
                    db_folder: db_folder.clone(),
                    space_database: space_database.clone()
                };
                let result: Result<_, Vec<PlanetError>> = insert_into_table.run();
                match result {
                    Ok(_) => {
                        println!();
                        println!("{}", String::from("[OK]").green());
                        eprint!("Time: {} ms", &t_1.elapsed().as_millis());
                    },
                    Err(errors) => {
                        let count = 1;
                        println!();
                        println!("{}", tr!("I found these errors").red().bold());
                        println!("{}", "--------------------".red());
                        println!();
                        for error in errors {
                            println!(
                                "{}{} {}", 
                                count.to_string().blue(),
                                String::from('.').blue(),
                                error.message
                            );
                        }
                    }
                }
            },
            Err(errors) => {
                println!();
                println!("{}", tr!("I found these errors").red().bold());
                println!("{}", "--------------------".red());
                println!();
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
}

impl<'gb> InsertIntoFolder<'gb> {
    pub fn check_name_exists(&self, folder_name: &String, name: &String, db_row: &mut TreeFolderItem) -> bool {
        let check: bool;
        let name = name.clone();
        let result = db_row.get(&folder_name, GetItemOption::ByName(name), None);
        eprintln!("InsertIntoFolder.check_name_exists :: get response: {:#?}", &result);
        match result {
            Ok(_) => {
                check = true
            },
            Err(_) => {
                check = false
            }
        }
        return check
    }
}

pub struct GetFromFolder<'gb> {
    pub planet_context: &'gb PlanetContext<'gb>,
    pub context: &'gb Context<'gb>,
    pub db_folder: TreeFolder,
    pub space_database: SpaceDatabase,
    pub config: GetFromFolderConfig,
}

impl<'gb> Command<String> for GetFromFolder<'gb> {

    fn run(&self) -> Result<String, PlanetError> {
        let command = self.config.command.clone().unwrap_or_default();
        let expr = Regex::new(r#"(GET FROM TABLE) "(?P<folder_name>[a-zA-Z0-9_ ]+)""#).unwrap();
        let table_name_match = expr.captures(&command).unwrap();
        let folder_name = &table_name_match["folder_name"].to_string();
        let folder_file = slugify(&folder_name);
        let folder_file = folder_file.as_str().replace("-", "_");

        let home_dir = self.planet_context.home_path.unwrap_or_default();
        let account_id = self.context.account_id.unwrap_or_default();
        let space_id = self.context.space_id.unwrap_or_default();
        let site_id = self.context.site_id.unwrap_or_default();
        let box_id = self.context.box_id.unwrap_or_default();
        let space_database = self.space_database.clone();
        let result: Result<TreeFolderItem, PlanetError> = TreeFolderItem::defaults(
            space_database.connection_pool,
            home_dir,
            account_id,
            space_id,
            site_id,
            box_id,
            folder_file.as_str(),
            &self.db_folder,
        );
        match result {
            Ok(_) => {
                // let data_config = self.config.data.clone();
                let mut db_row: TreeFolderItem = result.unwrap();
                // I need to get SchemaData and schema for the folder
                // I go through columns in order to build RowData                
                let folder = self.db_folder.get_by_name(folder_name)?;
                if *&folder.is_none() {
                    return Err(
                        PlanetError::new(
                            500, 
                            Some(tr!("Could not find folder {}", &folder_name)),
                        )
                    );
                }
                let folder = folder.unwrap();
                let data_collections = folder.clone().data_collections;
                let field_ids = data_collections.unwrap().get(COLUMN_IDS).unwrap().clone();
                let config_columns = ColumnConfig::get_config(
                    self.planet_context,
                    self.context,
                    &folder
                )?;
                let field_id_map: BTreeMap<String, ColumnConfig> = ColumnConfig::get_column_id_map(&config_columns)?;
                let columns = self.config.data.clone().unwrap().columns;
                eprintln!("GetFromFolder.run :: columns: {:?}", &columns);
                let item_id = self.config.data.clone().unwrap().id.unwrap();
                // Get item from database
                let db_data = db_row.get(&folder_name, GetItemOption::ById(item_id), columns)?;
                // data and basic columns
                let data = db_data.data;
                let mut yaml_out_str = String::from("---\n");
                // id
                let id_yaml_value = self.config.data.clone().unwrap().id.unwrap().truecolor(
                    YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
                );
                let id_yaml = format!("{}", 
                    id_yaml_value.truecolor(YAML_COLOR_ORANGE[0], YAML_COLOR_ORANGE[1], YAML_COLOR_ORANGE[2]), 
                );
                yaml_out_str.push_str(format!("{column}: {value}\n", 
                    column=String::from(ID).truecolor(
                        YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
                    ), 
                    value=&id_yaml
                ).as_str());
                // name
                let name_yaml_value = &db_data.name.unwrap().clone();
                let name_yaml = format!("{}", 
                    name_yaml_value.truecolor(YAML_COLOR_ORANGE[0], YAML_COLOR_ORANGE[1], YAML_COLOR_ORANGE[2]), 
                );
                yaml_out_str.push_str(format!("{column}: {value}\n", 
                    column=String::from(NAME).truecolor(
                        YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
                    ), 
                    value=&name_yaml
                ).as_str());
                yaml_out_str.push_str(format!("{}\n", 
                    String::from("data:").truecolor(YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]),
                ).as_str());
                if data.is_some() {
                    // column_id -> string value
                    let data = data.unwrap();
                    // I need to go through in same order as columns were registered in ColumnConfig when creating schema
                    for field_id_data in field_ids {
                        let column_id = field_id_data.get(ID).unwrap();
                        let column_config = field_id_map.get(column_id).unwrap().clone();
                        let field_config_ = column_config.clone();
                        let column_type = column_config.column_type.unwrap();
                        let column_type = column_type.as_str();
                        let value = data.get(column_id);
                        if value.is_none() {
                            continue
                        }
                        let value = value.unwrap();
                        // Get will return YAML document for the data
                        match column_type {
                            COLUMN_TYPE_SMALL_TEXT => {
                                let obj = SmallTextColumn::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            COLUMN_TYPE_LONG_TEXT => {
                                let obj = LongTextColumn::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            COLUMN_TYPE_CHECKBOX => {
                                let obj = CheckBoxColumn::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            COLUMN_TYPE_NUMBER => {
                                let obj = NumberColumn::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            COLUMN_TYPE_SELECT => {
                                let obj = SelectColumn::defaults(&field_config_, Some(&folder));
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            COLUMN_TYPE_FORMULA => {
                                let obj = FormulaColumn::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            COLUMN_TYPE_DATE => {
                                let obj = DateColumn::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            COLUMN_TYPE_DURATION => {
                                let obj = DurationColumn::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },                            
                            COLUMN_TYPE_CREATED_TIME => {
                                let obj = AuditDateColumn::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            COLUMN_TYPE_LAST_MODIFIED_TIME => {
                                let obj = AuditDateColumn::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            COLUMN_TYPE_CREATED_BY => {
                                let obj = AuditByColumn::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            COLUMN_TYPE_LAST_MODIFIED_BY => {
                                let obj = AuditByColumn::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            COLUMN_TYPE_CURRENCY => {
                                let obj = CurrencyColumn::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            COLUMN_TYPE_PERCENTAGE => {
                                let obj = PercentageColumn::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            COLUMN_TYPE_GENERATE_ID => {
                                let obj = GenerateIdColumn::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            COLUMN_TYPE_GENERATE_NUMBER => {
                                let obj = GenerateNumberColumn::defaults(
                                    &field_config_,
                                    Some(folder.clone()),
                                    Some(self.db_folder.clone()),
                                );
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            COLUMN_TYPE_PHONE => {
                                let obj = PhoneColumn::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            COLUMN_TYPE_EMAIL => {
                                let obj = EmailColumn::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            COLUMN_TYPE_URL => {
                                let obj = UrlColumn::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            COLUMN_TYPE_RATING => {
                                let obj = RatingColumn::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            COLUMN_TYPE_OBJECT => {
                                let obj = ObjectColumn::defaults(&field_config_);
                                yaml_out_str = obj.get_yaml_out(&yaml_out_str, value);
                            },
                            _ => {
                                yaml_out_str = yaml_out_str;
                            }
                        }
                    }
                }
                eprintln!("{}", yaml_out_str);
                return Ok(yaml_out_str);
            },
            Err(error) => {
                return Err(error);
            }
        }
    }

    fn runner(runner: &CommandRunner, path_yaml: &String) -> () {
        let config_ = GetFromFolderConfig::defaults(
            String::from("")
        );
        let config: Result<GetFromFolderConfig, Vec<PlanetValidationError>> = config_.import(
            runner.planet_context,
            &path_yaml
        );
        match config {
            Ok(_) => {
                let home_dir = runner.planet_context.home_path.unwrap_or_default();
                let account_id = runner.context.account_id.unwrap_or_default();
                let space_id = runner.context.space_id.unwrap_or_default();
                let site_id = runner.context.site_id.unwrap_or_default();
                let result = SpaceDatabase::defaults(
                    Some(site_id), 
                    space_id, 
                    Some(home_dir),
                    Some(false)
                );
                if result.is_err() {
                    let error = result.clone().unwrap_err();
                    println!();
                    println!("{}", tr!("I found these errors").red().bold());
                    println!("{}", "--------------------".red());
                    println!();
                    println!(
                        "{} {}", 
                        String::from('.').blue(),
                        error.message
                    );
                }
                let space_database = result.unwrap();
                let db_folder= TreeFolder::defaults(
                    space_database.connection_pool.clone(),
                    Some(home_dir),
                    Some(account_id),
                    Some(space_id),
                    Some(site_id),
                ).unwrap();

                let insert_into_table: GetFromFolder = GetFromFolder{
                    planet_context: runner.planet_context,
                    context: runner.context,
                    config: config.unwrap(),
                    space_database: space_database.clone(),
                    db_folder: db_folder.clone(),
                };
                let result: Result<_, PlanetError> = insert_into_table.run();
                match result {
                    Ok(_) => {
                        println!();
                        println!("{}", String::from("[OK]").green());
                    },
                    Err(error) => {
                        let count = 1;
                        println!();
                        println!("{}", tr!("I found these errors").red().bold());
                        println!("{}", "--------------------".red());
                        println!();
                        println!(
                            "{}{} {}", 
                            count.to_string().blue(),
                            String::from('.').blue(),
                            error.message
                        );
                    }
                }
            },
            Err(errors) => {
                println!();
                println!("{}", tr!("I found these errors").red().bold());
                println!("{}", "--------------------".red());
                println!();
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
}

pub struct SelectFromFolder<'gb> {
    pub planet_context: &'gb PlanetContext<'gb>,
    pub context: &'gb Context<'gb>,
    pub db_folder: TreeFolder,
    pub space_database: SpaceDatabase,
    pub config: SelectFromFolderConfig,
}

impl<'gb> Command<String> for SelectFromFolder<'gb> {

    fn run(&self) -> Result<String, PlanetError> {
        let command = self.config.command.clone().unwrap_or_default();
        let expr = Regex::new(r#"(SELECT FROM TABLE) "(?P<folder_name>[a-zA-Z0-9_ ]+)""#).unwrap();
        let table_name_match = expr.captures(&command).unwrap();
        let folder_name = &table_name_match["folder_name"].to_string();
        let folder_file = slugify(&folder_name);
        let folder_file = folder_file.as_str().replace("-", "_");
        eprintln!("SelectFromFolder.run :: folder_file: {}", &folder_file);

        let home_dir = self.planet_context.home_path.unwrap_or_default();
        let account_id = self.context.account_id.unwrap_or_default();
        let space_id = self.context.space_id.unwrap_or_default();
        let site_id = self.context.site_id.unwrap_or_default();
        let box_id = self.context.box_id.unwrap_or_default();
        let space_database = self.space_database.clone();
        let result: Result<TreeFolderItem, PlanetError> = TreeFolderItem::defaults(
            space_database.connection_pool,
            home_dir,
            account_id,
            space_id,
            site_id,
            box_id,
            folder_file.as_str(),
            &self.db_folder,
        );
        match result {
            Ok(_) => {
                let mut db_row: TreeFolderItem = result.unwrap();
                let config = self.config.clone();
                let r#where = config.r#where;
                let page = config.page;
                let number_items = config.number_items;
                let columns = config.columns;
                let mut page_wrap: Option<usize> = None;
                let mut number_items_wrap: Option<usize> = None;
                if page.is_some() {
                    let page_string = page.unwrap();
                    let page_number: usize = FromStr::from_str(page_string.as_str()).unwrap();
                    page_wrap = Some(page_number)
                }
                if number_items.is_some() {
                    let number_items_string = number_items.unwrap();
                    let number_items: usize = FromStr::from_str(number_items_string.as_str()).unwrap();
                    number_items_wrap = Some(number_items)
                }
                let result = db_row.select(
                    folder_name, 
                    r#where, 
                    page_wrap, 
                    number_items_wrap, 
                    columns,
                )?;
                eprintln!("SelectFromFolder :: result: {:#?}", &result);
                // Later on, I do pretty print
            },
            Err(error) => {
                return Err(error);
            }
        }

        return Ok(String::from(""));
    }
    fn runner(runner: &CommandRunner, path_yaml: &String) -> () {
        let config_ = SelectFromFolderConfig::defaults(
            None,
            None,
            None
        );
        let config: Result<SelectFromFolderConfig, Vec<PlanetValidationError>> = config_.import(
            runner.planet_context,
            &path_yaml
        );
        match config {
            Ok(_) => {
                let home_dir = runner.planet_context.home_path.unwrap_or_default();
                let account_id = runner.context.account_id.unwrap_or_default();
                let space_id = runner.context.space_id.unwrap_or_default();
                let site_id = runner.context.site_id.unwrap_or_default();
                let result = SpaceDatabase::defaults(
                    Some(site_id), 
                    space_id, 
                    Some(home_dir),
                    Some(false)
                );
                if result.is_err() {
                    let error = result.clone().unwrap_err();
                    println!();
                    println!("{}", tr!("I found these errors").red().bold());
                    println!("{}", "--------------------".red());
                    println!();
                    println!(
                        "{} {}", 
                        String::from('.').blue(),
                        error.message
                    );
                }
                let space_database = result.unwrap();
                let db_folder= TreeFolder::defaults(
                    space_database.connection_pool.clone(),
                    Some(home_dir),
                    Some(account_id),
                    Some(space_id),
                    Some(site_id),
                ).unwrap();

                let select_from_table: SelectFromFolder = SelectFromFolder{
                    planet_context: runner.planet_context,
                    context: runner.context,
                    config: config.unwrap(),
                    db_folder: db_folder.clone(),
                    space_database: space_database.clone()
                };
                let result: Result<_, PlanetError> = select_from_table.run();
                match result {
                    Ok(_) => {
                        println!();
                        println!("{}", String::from("[OK]").green());
                    },
                    Err(error) => {
                        let count = 1;
                        println!();
                        println!("{}", tr!("I found these errors").red().bold());
                        println!("{}", "--------------------".red());
                        println!();
                        println!(
                            "{}{} {}", 
                            count.to_string().blue(),
                            String::from('.').blue(),
                            error.message
                        );
                    }
                }
            },
            Err(errors) => {
                println!();
                println!("{}", tr!("I found these errors").red().bold());
                println!("{}", "--------------------".red());
                println!();
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
}

fn handle_field_response(
    column_data: &Result<Vec<String>, PlanetError>, 
    errors: &Vec<PlanetError>, 
    column_id: &String,
    data: &BTreeMap<String, String>,
    data_collections: &BTreeMap<String, Vec<BTreeMap<String, String>>>,
    is_set: &String
) -> (
    BTreeMap<String, String>, 
    BTreeMap<String, Vec<BTreeMap<String, String>>>,
    Vec<PlanetError>
) {
    let column_data = column_data.clone();
    // eprintln!("handle_field_response :: column_data: {:?}", column_data);
    let mut errors = errors.clone();
    let mut data = data.clone();
    let mut data_collections = data_collections.clone();
    let column_id = column_id.clone();
    let is_set = is_set.clone();
    if column_data.is_err() {
        let err = column_data.unwrap_err();
        errors.push(err);
    } else {
        let column_data = column_data.unwrap().clone();
        if is_set == FALSE.to_string() {
            // into data
            if column_data.clone().len() == 0 {
                let error = PlanetError::new(
                    500, 
                    Some(tr!("Content is empty, no data."))
                );
                errors.push(error);
            } else {
                let column_value = column_data[0].clone();
                data.insert(column_id.clone(), column_value);    
            }
        } else {
            // into data_collections, I have a set
            let mut list: Vec<BTreeMap<String, String>> = Vec::new();
            for item in column_data {
                let mut map: BTreeMap<String, String> = BTreeMap::new();
                map.insert(VALUE.to_string(), item);
                list.push(map);
            }
            data_collections.insert(column_id.clone(), list);
        }
    }
    return (data, data_collections, errors)
}
