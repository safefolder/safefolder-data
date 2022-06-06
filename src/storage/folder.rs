extern crate sled;
extern crate slug;
extern crate rust_stemmers;

use std::fs;
use std::io::{Read, Write};
use std::str::FromStr;
use std::{thread};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::collections::{BTreeMap, HashMap};
use colored::Colorize;
use rust_decimal::prelude::ToPrimitive;
use validator::{Validate, ValidationErrors};
use serde::{Deserialize, Serialize};
use tr::tr;
use serde_encrypt::{
    serialize::impls::BincodeSerializer, shared_key::SharedKey, traits::SerdeEncryptSharedKey,
    AsSharedKey, EncryptedMessage,
};
use slug::slugify;
use sled::Tree;
use std::fs::{File, remove_file, create_dir_all};
use chacha20poly1305::{ChaCha20Poly1305}; // Or `XChaCha20Poly1305`
use chacha20poly1305::aead::{NewAead, stream};
use anyhow::{anyhow};

use crate::planet::constants::*;
use crate::storage::{generate_id};
use crate::planet::{PlanetError};
use crate::statements::folder::config::{DbFolderConfig};
use crate::storage::constants::*;
use crate::storage::columns::*;
use crate::storage::columns::text::{
    get_stop_words_by_language, get_stemmer_by_language, get_default_language_code
};


pub trait FolderSchema {
    fn defaults(
        connection_pool: HashMap<String, sled::Db>,
        home_dir: Option<&str>,
        account_id: Option<&str>,
        space_id: Option<&str>,
        site_id: Option<String>,
    ) -> Result<TreeFolder, PlanetError>;
    fn create(&self, db_data: &DbData) -> Result<DbData, PlanetError>;
    fn update(&self, db_data: &DbData) -> Result<DbData, PlanetError>;
    fn get(&self, id: &String) -> Result<DbData, PlanetError>;
    fn get_by_name(&self, folder_name: &str) -> Result<Option<DbData>, PlanetError>;
    fn list(&self) -> Result<Vec<DbData>, PlanetError>;
    fn delete(&self, id: &String) -> Result<DbData, PlanetError>;
}

pub trait FolderItem {
    fn defaults(
        connection_pool: HashMap<String, sled::Db>,
        home_dir: &str,
        account_id: &str,
        space_id: &str,
        site_id: Option<String>,
        folder_id: &str,
        tree_folder: &TreeFolder,
    ) -> Result<TreeFolderItem, PlanetError>;
    fn insert(&mut self, folder_name: &String, db_data_list: &Vec<DbData>) -> Result<Vec<DbData>, Vec<PlanetError>>;
    fn update(&mut self, db_data: &DbData) -> Result<DbData, PlanetError>;
    fn get(
        &mut self, 
        folder_name: &String, 
        by: GetItemOption, 
        columns: Option<Vec<String>>
    ) -> Result<DbData, PlanetError>;
    fn select(&mut self, 
        folder_name: &String, 
        r#where: Option<String>, 
        page: Option<usize>,
        number_items: Option<usize>,
        columns: Option<Vec<String>>,
    ) -> Result<SelectResult, PlanetError>;
    fn count(&mut self, 
        folder_name: &String, 
        r#where: Option<String>, 
    ) -> Result<SelectCountResult, PlanetError>;
    fn total_count(&mut self) -> Result<SelectCountResult, PlanetError>;
    fn index(&mut self, db_item: &DbData, text_map: &BTreeMap<String, String>) -> Result<DbData, PlanetError>;
}

// lifetimes: gb (global, for contexts), db, bs

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoutingData {
    pub account_id: Option<String>,
    pub space_id: Option<String>,
    pub site_id: Option<String>,
    pub ipfs_cid: Option<String>,
}

impl RoutingData {
    pub fn defaults(
        account_id: Option<String>, 
        site_id: Option<String>,
        space_id: &str, 
        ipfs_cid: Option<String>,
    ) -> Option<RoutingData> {
        let mut routing = RoutingData{
            account_id: account_id,
            site_id: None,
            space_id: None,
            ipfs_cid: ipfs_cid,
        };
        if site_id.is_some() {
            let site_id = site_id.unwrap().to_string();
            routing.site_id = Some(site_id);
        }
        routing.space_id = Some(space_id.to_string());
        let routing_wrap = Some(routing);
        return routing_wrap
    }
}

impl SerdeEncryptSharedKey for NameTree {
    type S = BincodeSerializer<Self>;  // you can specify serializer implementation (or implement it by yourself).
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct NameTree {
    name: String
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct SubFolderItem {
    #[validate(required)]
    pub id: Option<String>,
    pub name: Option<String>,
    pub is_reference: Option<bool>,
    pub data: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct DbFile {
    #[validate(required)]
    pub id: Option<String>,
    #[validate(required)]
    pub name: Option<String>,
    pub size: Option<u64>,
    pub content_type: Option<String>,
    pub file_type: Option<String>,
    pub routing: Option<BTreeMap<String, String>>,
    pub options: Option<BTreeMap<String, String>>,
    pub context: Option<BTreeMap<String, String>>,
    pub content: Option<Vec<u8>>,
    pub path: Option<String>,
}
impl DbFile {
    pub fn defaults(
        file_id: &String,
        name: &String, 
        file: &mut File,
        content_type: &String,
        file_type: &String,
        routing: Option<RoutingData>,
        home_dir: &String
        ) -> Result<Self, PlanetError> {
        let metadata = file.metadata();
        let size = metadata.unwrap().len();
        let mut content: Option<Vec<u8>> = None;
        let mut path: Option<String> = None;
        let mut routing_map: BTreeMap<String, String> = BTreeMap::new();
        if routing.is_some() {
            let routing = routing.unwrap();
            let account_id = routing.account_id.clone().clone();
            let site_id = routing.site_id.clone();
            let space_id = routing.space_id.clone();
            let ipfs_cid = routing.ipfs_cid.clone().unwrap_or_default();
            if account_id.is_some() {
                routing_map.insert(String::from(ACCOUNT_ID), account_id.unwrap().to_string());
            }
            if site_id.is_some() {
                routing_map.insert(String::from(SITE_ID), site_id.unwrap());
            }

            if space_id.is_some() {
                routing_map.insert(String::from(SPACE_ID), space_id.unwrap());
            }
            routing_map.insert(String::from(IPFS_CID), ipfs_cid);
        }
        if size > MAX_FILE_DB {
            // .achiever-planet/private/files/{file_id}.achieverenc
            // .achiever-planet/sites/{site_id}/spaces/{space_id}/files/{file_id}.achieverenc
            let space_id = routing_map.get(SPACE_ID);
            let site_id = routing_map.get(SITE_ID);
            if space_id.is_some() && site_id.is_some() {
                let space_id = space_id.unwrap();
                let site_id = site_id.unwrap();
                let path_string: String;
                if space_id == PRIVATE {
                    path_string = format!(
                        "{home}/{private}/files/{file_id}.achieverenc", 
                        home=&home_dir, 
                        file_id=file_id,
                        private=PRIVATE
                    );
                } else {
                    path_string = format!(
                        "{home}/sites/{site_id}/spaces/{space_id}/files/{file_id}.achieverenc", 
                        home=&home_dir, 
                        file_id=file_id,
                        site_id=site_id,
                        space_id=space_id
                    );
                }
                path = Some(path_string);
            }
        } else {
            // I place file in the file database
            let mut contents = vec![0; size as usize];
            let response = file.read(&mut contents);
            if response.is_err() {
                let error = response.unwrap_err();
                eprintln!("{:?}", error);
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!("Error reading file.")),
                    )
                );
            }
            content = Some(contents);
        }
        let obj = Self{
            id: Some(file_id.clone()),
            name: Some(name.clone()),
            routing: Some(routing_map),
            options: None,
            context: None,
            content: content,
            path: path,
            size: Some(size),
            content_type: Some(content_type.clone()),
            file_type: Some(file_type.clone()),
        };
        return Ok(obj)
    }
    pub fn write_file(
        &mut self,
        file: &mut File,
        db_folder_item: &TreeFolderItem
    ) -> Result<String, PlanetError> {
        let path = self.path.clone();
        let mut db_folder_item = db_folder_item.clone();
        if path.is_none() {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("No path defined for file.")),
                )
            );
        }
        let path = path.unwrap();
        let cipher = ChaCha20Poly1305::new(CHILD_PRIVATE_KEY_ARRAY.as_ref().into());
        let mut stream_encryptor = stream::EncryptorBE32::from_aead(
            cipher, CHILD_NONCE.as_ref().into()
        );
        const BUFFER_LEN: usize = 500;
        let mut buffer = [0u8; BUFFER_LEN];
        let mut enc_file = File::create(&path).unwrap();
        loop {
            let read_count = file.read(&mut buffer).unwrap();
            if read_count == BUFFER_LEN {
                let ciphertext = stream_encryptor
                    .encrypt_next(buffer.as_slice())
                    .map_err(|err| anyhow!("Encrypting large file: {}", err));
                if ciphertext.is_err() {
                    return Err(
                        PlanetError::new(
                            500, 
                            Some(tr!("Problem encrypting file.")),
                        )
                    );
                }
                let ciphertext = ciphertext.unwrap();
                enc_file.write(&ciphertext).unwrap();
            } else {
                let ciphertext = stream_encryptor
                    .encrypt_last(&buffer[..read_count])
                    .map_err(|err| anyhow!("Encrypting large file: {}", err));
                if ciphertext.is_err() {
                    return Err(
                        PlanetError::new(
                            500, 
                            Some(tr!("Problem encrypting file.")),
                        )
                    );
                }
                let ciphertext = ciphertext.unwrap();
                enc_file.write(&ciphertext).unwrap();
                break;
            }
        }
        // Write into file database the path to encrypted file
        let file_id = db_folder_item.write_file(&self);
        if file_id.is_ok() {
            let file_id = file_id.unwrap();
            return Ok(file_id.clone())
        } else {
            // remove file from path
            let _ = remove_file(&path);
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("Could not write file path into file database.")),
                )
            );
        }
    }
    pub fn get_home_path(&mut self) -> Result<String, PlanetError> {
        let routing = self.routing.clone();
        if routing.is_none() {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("Cannot export file if no routing is found.")),
                )
            );
        }
        // let _ = create_dir_all(&path);
        let routing = routing.unwrap();
        let space_id = routing.get(SPACE_ID).unwrap();
        let site_id = routing.get(SITE_ID);
        let box_id = routing.get(BOX_ID);
        let sys_home_dir = dirs::home_dir().unwrap();
        let home_dir = sys_home_dir.as_os_str().to_str().unwrap().to_string();
        let file_name = self.name.clone().unwrap();
        let path: String;
        if space_id.as_str() == PRIVATE {
            let path_dir = format!(
                "{home_dir}/{home_dir_folder}/{private}",
                home_dir=home_dir,
                home_dir_folder=HOME_DIR_FOLDER,
                private=PRIVATE,
            );
            let _ = create_dir_all(&path_dir);
            path = format!(
                "{path_dir}/{file_name}",
                path_dir=path_dir,
                file_name=&file_name,
            );
        } else {
            let site_id = site_id.unwrap();
            let box_id = box_id.unwrap();
            if box_id.as_str() == BASE {
                let path_dir = format!(
                    "{home_dir}/{home_dir_folder}/sites/{site_id}/spaces/{space_id}",
                    home_dir=home_dir,
                    home_dir_folder=HOME_DIR_FOLDER,
                    site_id=site_id,
                    space_id=space_id,
                );
                let _ = create_dir_all(&path_dir);
                path = format!(
                    "{path_dir}/{file_name}",
                    path_dir=path_dir,
                    file_name=&file_name,
                );
            } else {
                let path_dir = format!(
                    "{home_dir}/{home_dir_folder}/sites/{site_id}/boxes/{box_id}/spaces/{space_id}",
                    home_dir=home_dir,
                    home_dir_folder=HOME_DIR_FOLDER,
                    site_id=site_id,
                    space_id=space_id,
                    box_id = box_id,
                );
                let _ = create_dir_all(&path_dir);
                path = format!(
                    "{path_dir}/{file_name}",
                    path_dir=path_dir,
                    file_name=&file_name,
                );
            }
        }
        return Ok(path.clone())
    }
    pub fn export_file(
        &mut self,
    ) -> Result<(), PlanetError> {
        let path = self.get_home_path();
        if path.is_err() {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("Error obtainning path for exported file.")),
                )
            );
        }
        let path = path.unwrap();
        let path_encrypted = self.path.clone();
        if path_encrypted.is_none() {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("No path defined for file.")),
                )
            );
        }
        let path_encrypted = path_encrypted.unwrap();
        let cipher = ChaCha20Poly1305::new(CHILD_PRIVATE_KEY_ARRAY.as_ref().into());
        let mut stream_decryptor = stream::DecryptorBE32::from_aead(
            cipher, 
            CHILD_NONCE.as_ref().into()
        );
        const BUFFER_LEN: usize = 500 + 16;
        let mut buffer = [0u8; BUFFER_LEN];
        let encrypted_file = File::open(path_encrypted);
        if encrypted_file.is_err() {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("Could not open encrypted file.")),
                )
            );
        }
        let mut encrypted_file = encrypted_file.unwrap();
        let mut file = File::create(path).unwrap();
        loop {
            let read_count = encrypted_file.read(&mut buffer).unwrap();
    
            if read_count == BUFFER_LEN {
                let plaintext = stream_decryptor
                    .decrypt_next(buffer.as_slice())
                    .map_err(|err| anyhow!("Decrypting large file: {}", err));
                if plaintext.is_err() {
                    return Err(
                        PlanetError::new(
                            500, 
                            Some(tr!("Problem decrypting file.")),
                        )
                    );
                }
                let plaintext = plaintext.unwrap();
                let _ = file.write(&plaintext);
            } else if read_count == 0 {
                break;
            } else {
                let plaintext = stream_decryptor
                    .decrypt_last(&buffer[..read_count])
                    .map_err(|err| anyhow!("Decrypting large file: {}", err));
                if plaintext.is_err() {
                    return Err(
                        PlanetError::new(
                            500, 
                            Some(tr!("Problem decrypting file.")),
                        )
                    );
                }
                let plaintext = plaintext.unwrap();
                let _ = file.write(&plaintext);
                break;
            }
        }
        return Ok(
            ()
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct DbDataMini {
    #[validate(required)]
    pub id: Option<String>,
    #[validate(required)]
    pub slug: Option<String>,
    #[validate(required)]
    pub name: Option<String>,
    pub routing: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
pub struct DbData {
    #[validate(required)]
    pub id: Option<String>,
    #[validate(required)]
    pub slug: Option<String>,
    #[validate(required)]
    pub name: Option<String>,
    pub routing: Option<BTreeMap<String, String>>,
    pub options: Option<BTreeMap<String, String>>,
    pub context: Option<BTreeMap<String, String>>,
    pub data: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>,
    pub sub_folders: Option<Vec<SubFolderItem>>,
}
impl DbData {
    pub fn defaults(
        name: &String, 
        data: Option<BTreeMap<String, Vec<BTreeMap<String, String>>>>, 
        options: Option<BTreeMap<String, String>>, 
        routing: Option<RoutingData>, 
        context: Option<BTreeMap<String, String>>,
        sub_folders: Option<Vec<SubFolderItem>>,
    ) -> Result<DbData, PlanetError> {
        let slug = slugify(name);
        let mut routing_map_: Option<BTreeMap<String, String>> = None;
        if routing.is_some() {
            let mut routing_map: BTreeMap<String, String> = BTreeMap::new();
            let routing = routing.unwrap();
            let account_id = routing.account_id.clone().clone();
            let site_id = routing.site_id.clone();
            let space_id = routing.space_id.clone();
            let ipfs_cid = routing.ipfs_cid.clone().unwrap_or_default();
            if account_id.is_some() {
                routing_map.insert(String::from(ACCOUNT_ID), account_id.unwrap().to_string());
            }
            if site_id.is_some() {
                routing_map.insert(String::from(SITE_ID), site_id.unwrap());
            }

            if space_id.is_some() {
                routing_map.insert(String::from(SPACE_ID), space_id.unwrap());
            }
            routing_map.insert(String::from(IPFS_CID), ipfs_cid);
            routing_map_ = Some(routing_map);
        }
        let db_data = Self{
            id: generate_id(),
            slug: Some(slug),
            name: Some(format!("{}", name)),
            routing: routing_map_,
            options: options,
            data: data,
            context: context,
            sub_folders: sub_folders,
        };
        let validate: Result<(), ValidationErrors> = db_data.validate();
        match validate {
            Ok(_) => {
                return Ok(db_data);
            },
            Err(_) => {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!("Could not validate folder data")),
                    )
                );
            }
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SchemaData {
    pub id: Option<String>,
    pub routing: RoutingData,
    pub name: String,
    pub config: DbFolderConfig,
}

impl SchemaData {
    pub fn defaults(
        name: &String, 
        config: &DbFolderConfig, 
        account_id: Option<&str>, 
        site_id: Option<&str>, 
        space_id: Option<&str>, 
    ) -> SchemaData {
        let mut routing = RoutingData{
            account_id: None,
            site_id: None,
            space_id: None,
            ipfs_cid: None,
        };
        if account_id.is_some() {
            let account_id = account_id.unwrap().to_string();
            routing.account_id = Some(account_id);
        }
        if site_id.is_some() {
            let site_id = site_id.unwrap().to_string();
            routing.site_id = Some(site_id);
        }
        if space_id.is_some() {
            let space_id = space_id.unwrap().to_string();
            routing.space_id = Some(space_id);
        }
        let schema_data = SchemaData{
            id: generate_id(),
            routing: routing,
            name: format!("{}", name),
            config: config.clone(),
        };
        return schema_data
    }
}

impl SerdeEncryptSharedKey for DbData {
    type S = BincodeSerializer<Self>;  // you can specify serializer implementation (or implement it by yourself).
}

impl SerdeEncryptSharedKey for DbFile {
    type S = BincodeSerializer<Self>;  // you can specify serializer implementation (or implement it by yourself).
}

#[derive(Debug, Clone)]
pub struct TreeFolder {
    pub home_dir: Option<String>,
    pub account_id: Option<String>,
    pub space_id: Option<String>,
    pub site_id: Option<String>,
    pub box_id: Option<String>,
    pub tree: sled::Tree,
    pub database: sled::Db,
}

impl TreeFolder {
    pub fn has_sub_folder_id(folder: &DbData, sub_folder_id: &String) -> bool {
        let folder = folder.clone();
        let data = folder.data;
        if data.is_some() {
            let data = data.unwrap();
            let sub_folders = data.get(SUB_FOLDERS);
            if sub_folders.is_some() {
                let sub_folders = sub_folders.unwrap();
                for sub_folder in sub_folders {
                    let db_sub_folder_id = sub_folder.get(ID).unwrap().clone();
                    if db_sub_folder_id == *sub_folder_id {
                        return true
                    }
                }
            }
        }
        return false
    }
}

impl TreeFolder {
    pub fn has_column(
        &self,
        folder_name: &String,
        column_name: &String
    ) -> bool {
        let result = self.get_by_name(folder_name);
        if result.is_ok() {
            let folder = result.unwrap();
            if folder.is_some() {
                let folder = folder.unwrap();
                let folder_data = folder.data.clone();
                if folder_data.is_some() {
                    let folder_data = folder_data.unwrap();
                    let folder_columns = folder_data.get(COLUMNS);
                    if folder_columns.is_some() {
                        let folder_columns = folder_columns.unwrap();
                        for item in folder_columns {
                            let value = item.get(NAME);
                            if value.is_some() {
                                let value = value.unwrap();
                                if value.clone().to_lowercase() == column_name.clone().to_lowercase() {
                                    return true
                                }
                            }
                        }
                    }
                }
            }
        }
        return false
    }

    pub fn get_column_by_name(
        column_name: &String, 
        folder: &DbData
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let folder = folder.clone();
        let folder_data = folder.data;
        if folder_data.is_some() {
            let folder_data = folder_data.unwrap();
            let columns = folder_data.get(COLUMNS);
            if columns.is_some() {
                let columns = columns.unwrap();
                for column in columns {
                    let column_name_db = column.get(NAME);
                    let column_id = column.get(ID);
                    if column_name_db.is_none() || column_id.is_none() {
                        // Raise error
                        return Err(
                            PlanetError::new(
                                500, 
                                Some(tr!("Could not get column id for \"{}\".", column_name)),
                            )
                        );
                    }
                    let column_name_db = column_name_db.unwrap().clone();
                    if column_name_db.to_lowercase() == column_name.to_lowercase() {
                        return Ok(
                            column.clone()
                        )
                    }
                }
            }
        }
        return Err(
            PlanetError::new(
                500, 
                Some(tr!("Could not get column id for \"{}\".", column_name)),
            )
        );
    }

    pub fn get_column_by_id(
        column_id: &String, 
        folder: &DbData
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let folder = folder.clone();
        let folder_data = folder.data;
        if folder_data.is_some() {
            let folder_data = folder_data.unwrap();
            let columns = folder_data.get(COLUMNS);
            if columns.is_some() {
                let columns = columns.unwrap();
                for column in columns {
                    let column_id_db = column.get(ID);
                    if column_id_db.is_none() {
                        // Raise error
                        return Err(
                            PlanetError::new(
                                500, 
                                Some(tr!("Could not get column for column id \"{}\".", column_id)),
                            )
                        );
                    }
                    let column_id_db = column_id_db.unwrap().clone();
                    if column_id_db.to_lowercase() == column_id.to_lowercase() {
                        return Ok(
                            column.clone()
                        )
                    }
                }
            }
        }
        return Err(
            PlanetError::new(
                500, 
                Some(tr!("Could not get colum for column_id \"{}\".", column_id)),
            )
        );
    }
}

impl FolderSchema for TreeFolder {

    fn defaults(
        connection_pool: HashMap<String, sled::Db>,
        home_dir: Option<&str>,
        account_id: Option<&str>,
        space_id: Option<&str>,
        site_id: Option<String>,
    ) -> Result<TreeFolder, PlanetError> {
        let home_dir = home_dir.unwrap_or_default();
        let account_id = account_id.unwrap_or_default();
        let space_id = space_id.unwrap_or_default();
        // I don't have paths in OS disk, only db and open tree in db
        // folders.db -> db.open_tree("folders")
        // 0001.db -> db.open_tree("box_{}_folder_{}_partition_{}_{db or index}")
        // partition.db -> db.open_tree("box_{}_folder_{}_partitions")
        // folders
        // box/base/folder/c7c815is1s406kaf3j30/partitions
        // box/base/folder/c7c815is1s406kaf3j30/partition_0001.db
        // box/base_folder_c7c815is1s406kaf3j30/partition_0001.index
        // folders.db
        // If private space, I open workspace db, otherwise I open site db

        let database: sled::Db;
        if site_id.is_none() {
            // I have private space, get workspace.db connection
            let database_ = connection_pool.get(WORKSPACE);
            if database_.is_none() {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!("Could not open private space workspace database.")),
                    )
                )
            }
            database = database_.unwrap().clone();
        } else {
            // I have site, get site.db connection
            let site_id = site_id.clone().unwrap();
            let database_ = connection_pool.get(&site_id);
            if database_.is_none() {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!("Could not open site \"{}\" database.", site_id)),
                    )
                )
            }
            database = database_.unwrap().clone();
        }
        let path =  format!("folders.db");
        let site_id = site_id.clone().unwrap_or_default();
        eprintln!("DbFolder.defaults :: path: {}", &path);
        let result = database.open_tree(path);
        match result {
            Ok(_) => {
                let db_tree = result.unwrap();
                let db_folder: TreeFolder= TreeFolder{
                    database: database.clone(),
                    home_dir: Some(home_dir.to_string()),
                    account_id: Some(account_id.to_string()),
                    space_id: Some(space_id.to_string()),
                    site_id: Some(site_id.to_string()),
                    box_id: None,
                    tree: db_tree,
                };
                Ok(db_folder)
            },
            Err(err) => {
                eprintln!("{:?}", &err);
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!("Could not open folder database")),
                    )
                )
            }
        }
    }

    fn get_by_name(&self, folder_name: &str) -> Result<Option<DbData>, PlanetError> {
        // I travel folder for account_id if any, space id if any and folder name
        let folder_name = folder_name.trim();
        let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        let iter = self.tree.iter();
        let mut number_items = 0;
        let mut matched_item: Option<DbData> = None;
        let ctx_account_id = self.account_id.clone().unwrap_or_default();
        let ctx_account_id = ctx_account_id.as_str();
        let ctx_space_id = self.space_id.clone().unwrap_or_default();
        let ctx_space_id = ctx_space_id.as_str();
        for result in iter {
            let tuple = result.unwrap();
            let item_db = tuple.1.to_vec();
            let item_ = EncryptedMessage::deserialize(item_db).unwrap();
            let item_ = DbData::decrypt_owned(
                &item_, 
                &shared_key);
            let item = item_.unwrap();
            let item_source = item.clone();
            let item_name = &item.name.unwrap();
            // eprintln!("DbFolder.get_by_name :: name: *{}*", item_name);
            let matches_name_none = &item_name.to_lowercase() == &folder_name.to_lowercase();
            let mut check_account: bool = true;
            let routing_response = item.routing;
            if routing_response.is_some() {
                let routing = routing_response.unwrap();
                let account_id = routing.get(ACCOUNT_ID).unwrap();
                let space_id = routing.get(SPACE_ID).unwrap();
                if ctx_account_id != "" && ctx_space_id != "" {
                    if (account_id != "" && account_id != ctx_account_id) || 
                    (space_id != "" && space_id != ctx_space_id) {
                        check_account = false;
                    }
                }
            }
            if matches_name_none && check_account {
                number_items += 1;
                matched_item = Some(item_source);
            }
        }
        match number_items {
            0 => {
                return Ok(None)
            },
            1 => {
                return Ok(matched_item)
            },
            2 => {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!(
                            "Could not fetch item from database, more than 1 item matches folder name."
                        ))
                    )
                )
            },
            _ => {
                return Err(PlanetError::new(500, Some(tr!("Item not found in folders."))))
            }
        }
    }

    fn get(&self, id: &String) -> Result<DbData, PlanetError> {
        let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        let id_db = xid::Id::from_str(id).unwrap();
        let id_db = id_db.as_bytes();
        let item_db = self.tree.get(&id_db).unwrap().unwrap().to_vec();
        let item_ = EncryptedMessage::deserialize(item_db).unwrap();
        let item_ = DbData::decrypt_owned(
            &item_, 
            &shared_key);
        match item_ {
            Ok(_) => {
                let item = item_.unwrap();
                // println!("get :: item: {:?}", &item);
                Ok(item)
            },
            Err(_) => {
                Err(PlanetError::new(500, Some(tr!("Could not fetch item from database"))))
            }
        }
    }

    fn delete(&self, id: &String) -> Result<DbData, PlanetError> {
        let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        let id_db = xid::Id::from_str(id).unwrap();
        let id_db = id_db.as_bytes();
        let item_db = self.tree.remove(&id_db).unwrap().unwrap().to_vec();
        let item_ = EncryptedMessage::deserialize(item_db).unwrap();
        let item_ = DbData::decrypt_owned(
            &item_, 
            &shared_key);
        match item_ {
            Ok(_) => {
                let item = item_.unwrap();
                Ok(item)
            },
            Err(_) => {
                Err(PlanetError::new(500, Some(tr!("Could not fetch item from database"))))
            }
        }
    }

    fn create(&self, db_data: &DbData) -> Result<DbData, PlanetError> {
        let folder_name = db_data.name.clone().unwrap();
        let result_table_exists: Result<Option<DbData>, PlanetError> = self.get_by_name(
            &folder_name.as_str()
        );
        let table_exists_error = *&result_table_exists.clone().is_err();
        let table_exists = *&table_exists_error == false && *&result_table_exists.clone().unwrap().is_some();
        if table_exists == true {
            let table_name_str = format!("\"{}\"", &folder_name).magenta();
            return Err(PlanetError::new(
                500, 
                Some(tr!("Folder {} already exists", &table_name_str))));
        } else if *&table_exists_error == true {
            let error = result_table_exists.unwrap_err();
            return Err(PlanetError::new(
                500, 
                Some(tr!("Error checking folder \"{}\": \"{}\"", &folder_name, &error.message))));
        }
        let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        let encrypted_schema = db_data.encrypt(&shared_key).unwrap();
        let encoded: Vec<u8> = encrypted_schema.serialize();
        let id = db_data.id.clone().unwrap();
        let id_db = xid::Id::from_str(id.as_str()).unwrap();
        let id_db = id_db.as_bytes();
        let response = &self.tree.insert(id_db, encoded);
        match response {
            Ok(_) => {
                println!("DbFolder.create :: id: {:?} name: {}", &id, &folder_name);
                let _db_response = response.clone().unwrap();
                // eprintln!("DbFolder.create :: db_response : {:?}", &db_response);
                let item_ = self.get(&id);
                match item_ {
                    Ok(_) => {
                        let item = item_.unwrap();
                        Ok(item)
                    },
                    Err(error) => {
                        Err(error)
                    }
                }
            },
            Err(_) => {
                Err(PlanetError::new(500, Some(tr!(
                    "Could not write folder config into database, generic data error."
                ))))
            }
        }
    }
    fn update(&self, folder: &DbData) -> Result<DbData, PlanetError> {
        let mut folder = folder.clone();
        let folder_name = folder.name.clone().unwrap();
        let result_table_exists: Result<Option<DbData>, PlanetError> = self.get_by_name(
            &folder_name.as_str()
        );
        let table_exists_error = *&result_table_exists.clone().is_err();
        let table_exists = *&table_exists_error == false && *&result_table_exists.clone().unwrap().is_some();
        if table_exists == false {
            let table_name_str = format!("\"{}\"", &folder_name).magenta();
            return Err(PlanetError::new(
                500, 
                Some(tr!("Folder {} does not exists", &table_name_str))));
        } else if *&table_exists_error == true {
            let error = result_table_exists.unwrap_err();
            return Err(PlanetError::new(
                500, 
                Some(tr!("Error checking folder \"{}\": \"{}\"", &folder_name, &error.message))));
        }
        let folder_db = result_table_exists.unwrap().unwrap();
        folder.id = folder_db.id;
        let id_db = folder.clone().id.unwrap();
        let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        let encrypted_schema = folder.clone().encrypt(&shared_key).unwrap();
        let encoded: Vec<u8> = encrypted_schema.serialize();
        let id_db = xid::Id::from_str(id_db.as_str()).unwrap();
        let id_db = id_db.as_bytes();
        let response = &self.tree.insert(id_db, encoded);
        match response {
            Ok(_) => {
                let _db_response = response.clone().unwrap();
                // eprintln!("DbFolder.update :: folder_name: {} db_response : {:?}", 
                //     &folder_name, &db_response
                // );
                Ok(folder.clone())
            }
            Err(_) => {
                Err(PlanetError::new(500, Some(tr!("Could not write folder schema"))))
            }
        }
    }
    fn list(&self) -> Result<Vec<DbData>, PlanetError> {
        let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        let mut items: Vec<DbData> = Vec::new();
        let response = self.tree.iter();
        for result in response {
            let item = result.unwrap();
            let item_db = item.1.to_vec();
            let item_ = EncryptedMessage::deserialize(item_db).unwrap();
            let item_ = DbData::decrypt_owned(
                &item_, 
                &shared_key);
            match item_ {
                Ok(_) => {
                    let item = item_.unwrap();
                    items.push(item);
                },
                Err(_) => {
                    return Err(PlanetError::new(500, Some(tr!("Could not fetch item from database"))))
                }
            }
        }
        return Ok(items)
    }
}

impl TreeFolder {
    pub fn get_column_id_map(
        tree_folder: &TreeFolder,
        folder_name: &String
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let folder = tree_folder.get_by_name(folder_name).unwrap().unwrap();
        let db_columns = folder.data.unwrap();
        let db_columns = db_columns.get(COLUMNS).unwrap();
        let mut column_id_map: BTreeMap<String, String> = BTreeMap::new();
        for db_column in db_columns {
            let column_id = db_column.get(ID).unwrap().clone();
            let column_name = db_column.get(NAME).unwrap().clone();
            column_id_map.insert(column_id, column_name);
        }
        Ok(column_id_map)
    }
    pub fn get_column_name_map(
        tree_folder: &TreeFolder,
        folder_name: &String
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let folder = tree_folder.get_by_name(folder_name)?;
        if folder.is_some() {
            let folder = folder.unwrap();
            let db_columns = folder.data.unwrap();
            let mut column_id_map: BTreeMap<String, String> = BTreeMap::new();
            for db_column in db_columns.keys() {
                let column_config = db_columns.get(db_column).unwrap();
                if column_config.len() == 1 {
                    let column_config = &column_config[0];
                    let column_id = column_config.get(ID);
                    if column_id.is_some() {
                        let column_id = column_id.unwrap().clone();
                        let column_name = db_column.clone();
                        column_id_map.insert(column_name, column_id);
                    }
                }
            }
            Ok(column_id_map)
        } else {
            return Err(PlanetError::new(
                500, 
                Some(tr!("Folder does not exist")),
            ));
        }
    }
    // Get field_name -> field_type map
    pub fn get_column_type_map(
        folder: &DbData,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let db_columns = folder.data.clone().unwrap();
        let mut column_type_map: BTreeMap<String, String> = BTreeMap::new();
        for db_column in db_columns.keys() {
            let column_name = db_column.clone();
            let column_map = db_columns.get(&column_name);
            if column_map.is_some() {
                let column_map = column_map.unwrap();
                if column_map.len() == 1 {
                    let column_map = &column_map[0];
                    let column_type = column_map.get(COLUMN_TYPE);
                    if column_type.is_some() {
                        let column_type = column_type.unwrap().clone();
                        column_type_map.insert(column_name, column_type);
                    }    
                }
            }
        }
        Ok(column_type_map)
    }
    pub fn get_item_data_by_column_names(
        item_data_id: Option<BTreeMap<String, String>>,
        column_id_map: BTreeMap<String, String>
    ) -> BTreeMap<String, String> {
        let mut item_data: BTreeMap<String, String> = BTreeMap::new();
        if item_data_id.is_some() {
            let item_data_id = item_data_id.unwrap();
            for column_id in item_data_id.keys() {
                let has_name = column_id_map.get(column_id);
                if has_name.is_some() {
                    let column_name = has_name.unwrap().clone();
                    let column_value = item_data_id.get(column_id).unwrap().clone();
                    item_data.insert(column_name, column_value);
                }
            }    
        }
        return item_data
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FolderItemElement(pub ColumnType);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FolderItemData {
    pub id: Option<String>,
    pub routing: RoutingData,
    pub data: Option<BTreeMap<String, FolderItemElement>>,
}
impl FolderItemData {
    pub fn defaults(
        account_id: Option<&String>, 
        site_id: Option<&String>,
        space_id: Option<&String>,
    ) -> Self {
        let mut routing = RoutingData{
            account_id: None,
            site_id: None,
            space_id: None,
            ipfs_cid: None,
        };
        if account_id.is_some() {
            let account_id = account_id.unwrap().clone();
            routing.account_id = Some(account_id);
        }
        if site_id.is_some() {
            let site_id = site_id.unwrap().clone();
            routing.site_id = Some(site_id);
        }
        if space_id.is_some() {
            let space_id = space_id.unwrap().clone();
            routing.space_id = Some(space_id);
        }
        return Self{
            id: generate_id(),
            routing: routing,
            data: None,
        };
    }
}

pub enum GetItemOption {
    ById(String),
    ByName(String),
}

#[derive(Debug, Clone)]
pub struct TreeFolderItem {
    pub database: sled::Db,
    pub tree_folder: TreeFolder,
    pub folder_id: Option<String>,
    pub home_dir: Option<String>,
    pub account_id: Option<String>,
    pub space_id: Option<String>,
    pub site_id: Option<String>,
    pub tree: Option<sled::Tree>,
    pub index: Option<sled::Tree>,
    pub files_db: Option<sled::Tree>,
    pub tree_partitions: Option<sled::Tree>,
}

impl TreeFolderItem {

    pub fn reindex_all(
        &mut self
    ) -> Result<(), PlanetError> {
        let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        let partitions = self.get_partitions();
        if partitions.is_err() {
            // Throw error returning, no need to restore data since no data was removed
            return Err(
                PlanetError::new(500, Some(tr!("Error getting database partitions.")))
            )
        }
        let partitions = partitions.unwrap();
        for partition in partitions {
            let result = self.open_partition(&partition);
            if result.is_ok() {
                let items = result.unwrap();
                self.tree = Some(items.0);
                self.index = Some(items.1);
                let tree = self.tree.clone().unwrap();
                for result in tree.iter() {
                    let tuple = result.unwrap();
                    let item_db = tuple.1.to_vec();
                    let item_ = EncryptedMessage::deserialize(item_db).unwrap();
                    let item_ = DbData::decrypt_owned(
                        &item_, 
                        &shared_key);
                    let item = item_.unwrap();
                    let data = item.clone().data;
                    if data.is_some() {
                        let data = data.unwrap();
                        let mut text_data: BTreeMap<String, String> = BTreeMap::new();
                        let my_text_data = data.get(TEXT);
                        if my_text_data.is_some() {
                            text_data = my_text_data.unwrap()[0].clone();
                        }
                        let result = self.index(
                            &item,
                            &text_data
                        );
                        if result.is_err() {
                            let error = result.unwrap_err();
                            return Err(error)
                        }
                    }
                }
            } else {
                return Err(
                    PlanetError::new(500, Some(tr!(
                        "Could not open partition \"{}\".", &partition
                    )))
                )
    
            }
        }
        return Ok(())
    }

    pub fn reindex_default_language(
        &mut self
    ) -> Result<(), PlanetError> {
        let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        let tree_folder = self.tree_folder.clone();
        let folder_id = self.folder_id.clone();
        let mut language_id: String = String::from("");
        if folder_id.is_some() {
            let folder_id = folder_id.unwrap();
            let folder = tree_folder.get(&folder_id);
            if folder.is_ok() {
                let folder = folder.unwrap();
                let data = folder.data;
                if data.is_some() {
                    let data = data.unwrap();
                    let column_list = data.get(COLUMNS);
                    if column_list.is_some() {
                        let column_list = column_list.unwrap();
                        for column_item in column_list {
                            let column_type = column_item.get(COLUMN_TYPE).unwrap().clone();
                            if column_type == COLUMN_TYPE_LANGUAGE.to_string() {
                                language_id = column_item.get(ID).unwrap().clone();
                            }
                        }
                    }
                }
            }
        }
        if language_id == String::from("") {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("Error getting language id from folder config.")),
                )
            )
        }
        let partitions = self.get_partitions();
        if partitions.is_err() {
            // Throw error returning, no need to restore data since no data was removed
            return Err(
                PlanetError::new(500, Some(tr!("Error getting database partitions.")))
            )
        }
        let partitions = partitions.unwrap();
        for partition in partitions {
            let result = self.open_partition(&partition);
            if result.is_ok() {
                let items = result.unwrap();
                self.tree = Some(items.0);
                self.index = Some(items.1);
                let tree = self.tree.clone().unwrap();
                for result in tree.iter() {
                    let tuple = result.unwrap();
                    let item_db = tuple.1.to_vec();
                    let item_ = EncryptedMessage::deserialize(item_db).unwrap();
                    let item_ = DbData::decrypt_owned(
                        &item_, 
                        &shared_key);
                    let item = item_.unwrap();
                    let data = item.clone().data;
                    if data.is_some() {
                        let data = data.unwrap();
                        let language = data.get(&language_id);
                        if language.is_some() {
                            let language = language.unwrap();
                            let language = get_value_list(language);
                            if language.is_some() {
                                let language = language.unwrap();
                                if language == String::from("") {
                                    let mut text_data: BTreeMap<String, String> = BTreeMap::new();
                                    let my_text_data = data.get(TEXT);
                                    if my_text_data.is_some() {
                                        text_data = my_text_data.unwrap()[0].clone();
                                    }
                                    let result = self.index(
                                        &item,
                                        &text_data
                                    );
                                    if result.is_err() {
                                        let error = result.unwrap_err();
                                        return Err(error)
                                    }
                                }
                            }
                        }
                    }
                }    
            }
        }
        return Ok(())
    }

    fn get_partition(
        &mut self,
        item_id: &str,
    ) -> Result<u16, PlanetError> {
        let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        let account_id = self.account_id.clone().unwrap_or_default();
        let account_id = account_id.as_str();
        let space_id = self.space_id.clone().unwrap_or_default();
        let space_id = space_id.as_str();
        let site_id = self.site_id.clone().unwrap_or_default();
        let site_id = site_id.as_str();
        let partition:u16;
        let tree = self.open_partitions()?;
        let id_db = xid::Id::from_str(item_id).unwrap();
        let id_db = id_db.as_bytes();
        let db_result = tree.get(&id_db);
        if db_result.is_err() {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("Could not open partition database")),
                )
            )
        }
        let db_result = db_result.unwrap();
        if db_result.is_none() {
            // I need to assign a parition for this item_id, since none has been assigned, and write to db. Used for insert calls.
            let number_parition_items = tree.len();
            let number_parition_items = number_parition_items.to_u16().unwrap();
            partition = number_parition_items/ITEMS_PER_PARTITION + 1;
            // write to db partition
            let mut data: BTreeMap<String, Vec<BTreeMap<String, String>>> = BTreeMap::new();
            let item_id_string = item_id.to_string();
            data.insert(PARTITION.to_string(), build_value_list(&partition.to_string()));
            let routing_wrap = RoutingData::defaults(
                Some(account_id.to_string()),
                Some(site_id.to_string()), 
                space_id, 
                None
            );
            let db_data = DbData::defaults(
                &item_id_string, 
                Some(data), 
                None, 
                routing_wrap, 
                None,
                None,
            )?;
            let encrypted_data = db_data.encrypt(&shared_key).unwrap();
            let encoded: Vec<u8> = encrypted_data.serialize();
            eprintln!("TreeFolderItem.get_partition :: db_data: {:#?}", &db_data);
            let response = tree.insert(id_db, encoded);
            match response {
                Ok(_) => {
                    eprintln!("DbFolderItem.get_partition :: [not exist] partition: {}", &partition);
                    return Ok(partition)
                },
                Err(_) => {
                    return Err(
                        PlanetError::new(
                            500, 
                            Some(tr!("Could not write partition")),
                        )
                    )
                }
            }
        }
        let db_result = db_result.unwrap();
        let item_db = db_result.to_vec();
        let item_ = EncryptedMessage::deserialize(item_db).unwrap();
        let item_ = DbData::decrypt_owned(
            &item_, 
            &shared_key);
        match item_ {
            Ok(_) => {
                let partition_db = item_.unwrap();
                let data = partition_db.data;
                if data.is_some() {
                    let data = data.unwrap();
                    let partition_wrap = data.get(PARTITION);
                    if partition_wrap.is_some() {
                        let partition_wrap = &partition_wrap.unwrap()[0];
                        let partition_str = partition_wrap.get(VALUE);
                        if partition_str.is_some() {
                            let partition_str = partition_str.unwrap();
                            let partition_str = partition_str.as_str();
                            partition = FromStr::from_str(partition_str).unwrap();
                            return Ok(partition)    
                        }
                    }
                }
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!("Could not open partition database, no data")),
                    )
                )
            },
            Err(_) => {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!("Could not open partition database, encrypt and serialize error")),
                    )
                )
            }
        }
    }

    fn open_partition_by_item(
        &mut self,
        item_id: &str,
    ) -> Result<(sled::Tree, sled::Tree), PlanetError> {
        let partition = self.get_partition(item_id);
        let partition = partition.unwrap();
        if partition > MAX_PARTITIONS {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("Maximum number of partitions reached. Maximum number of items in a folder is 1,000,000")),
                )
            )
        }
        let tree_tuple = self.open_partition(&partition)?;
        return Ok(tree_tuple)
    }

    pub fn drop_trees(
        &mut self
    ) -> Result<(), PlanetError> {
        let folder_id = self.folder_id.clone().unwrap_or_default();
        let folder_id = folder_id.as_str();
        // TODO: Export data prior to drop trees, etc... so I can restore in case of errors to the state when request
        // to drop folder was received. Restore through IPFS. TODO when we have IPFS integrated.

        // Drop all trees in database for tree items, related to db and index for all partitions
        let partitions = self.get_partitions();
        if partitions.is_err() {
            // Throw error returning, no need to restore data since no data was removed
            return Err(
                PlanetError::new(500, Some(tr!("Error getting database partitions.")))
            )
        }
        let partitions = partitions.unwrap();
        for partition in partitions {
            let partition_str = partition.to_string();
            let partition_str = format!("{:0>4}", partition_str);
            let paths = self.get_db_paths(&partition_str, folder_id);
            let path_db = paths.0;
            let path_index = paths.1;
            let db_result = self.database.drop_tree(path_db.clone());
            if db_result.is_err() {
                // Throw error returning, trigger restore????
                return Err(
                    PlanetError::new(500, Some(tr!("Error deleting database partition \"{}\".", &path_db)))
                )
            }
            let index_result = self.database.drop_tree(path_index.clone());
            if index_result.is_err() {
                // Throw error returning, trigger restore????
                return Err(
                    PlanetError::new(500, Some(tr!("Error deleting index partition \"{}\".", &path_index)))
                )
            }
        }
        return Ok(())
    }

    pub fn drop_files(
        &mut self
    ) -> Result<(), PlanetError> {
        // delete all files from OS
        let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        let folder_id = self.folder_id.clone().unwrap_or_default();
        let folder_id = folder_id.as_str();
        let path_db = format!(
            "folders/{folder_id}/files.db",
            folder_id=folder_id,
        );
        let tree: Tree;
        if self.files_db.is_some() {
            tree = self.files_db.clone().unwrap();
        } else {
            let tree_ = self.database.open_tree(path_db.clone());
            if tree_.is_err() {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!("Could not find language column.")),
                    )
                )
            }
            tree = tree_.unwrap();
        }
        let mut path_list: Vec<String> = Vec::new();
        for result in tree.iter() {
            let tuple = result.unwrap();
            let item_db = tuple.1.to_vec();
            let item_ = EncryptedMessage::deserialize(item_db).unwrap();
            let item_ = DbFile::decrypt_owned(
                &item_, 
                &shared_key);
            let item = item_.unwrap();
            let path = item.path;
            if path.is_some() {
                let path = path.unwrap();
                path_list.push(path);
            }
        }
        // Delete tree
        let result = self.database.drop_tree(path_db.clone());
        if result.is_err() {
            let _error = result.unwrap_err();
            return Err(
                PlanetError::new(500, Some(tr!(
                    "Error deleting folder files database"
                )))
            )
        }
        // Delete big files from files folder
        for path in path_list {
            let result = fs::remove_file(path.clone());
            if result.is_err() {
                return Err(
                    PlanetError::new(500, Some(tr!(
                        "Error deleting file \"{}\"", &path
                    )))
                )   
            }
        }
        return Ok(())
    }

    fn get_db_paths(
        &mut self,
        partition_str: &String,
        folder_id: &str,
    ) -> (String, String) {
        let path_db = format!(
            "folders/{folder_id}/{partition_str}.db",
            folder_id=folder_id,
            partition_str=partition_str
        );
        let path_index = format!(
            "folders/{folder_id}/{partition_str}.index",
            folder_id=folder_id,
            partition_str=partition_str
        );
        return (path_db, path_index)
    }

    pub fn get_partition_str(partition: &u16) -> String {
        let partition_str = partition.to_string();
        let partition_str = format!("{:0>4}", partition_str);
        return partition_str
    }

    pub fn open_partition(
        &mut self,
        partition: &u16,
    ) -> Result<(sled::Tree, sled::Tree), PlanetError> {
        let folder_id = self.folder_id.clone().unwrap_or_default();
        let folder_id = folder_id.as_str();
        // let home_dir = self.home_dir.clone().unwrap_or_default();
        // let home_dir = home_dir.as_str();
        let partition_str = partition.to_string();
        let partition_str = format!("{:0>4}", partition_str);
        // box/base/folder/c7c815is1s406kaf3j30/partitions
        // box/base/folder/c7c815is1s406kaf3j30/partition/0001.db
        // box_base_folder/c7c815is1s406kaf3j30/partition/0001.index
        if self.tree.is_some() {
            let tree = self.tree.clone().unwrap();
            let index = self.index.clone().unwrap();
            return Ok(
                (tree, index)
            )
        }
        let paths = self.get_db_paths(&partition_str, folder_id);
        let path_db = paths.0;
        let path_index = paths.1;
        eprintln!("DbFolderItem.open_partition :: path_db: {:?} path_index: {:?}", &path_db, &path_index);
        let tree = self.database.open_tree(path_db);
        let index = self.database.open_tree(path_index);
        if tree.is_ok() && index.is_ok() {
            let tree_ = tree.unwrap().clone();
            let index_ = index.unwrap().clone();
            self.tree = Some(tree_.clone());
            self.index = Some(index_.clone());
            return Ok(
                (tree_.clone(), index_.clone())
            )
        } else {
            return Err(
                PlanetError::new(500, Some(tr!("Could not open database partition")))
            )
        }
    }

    fn open_partitions(
        &mut self,
    ) -> Result<sled::Tree, PlanetError> {
        // let home_dir = self.home_dir.clone().unwrap_or_default();
        // let home_dir = home_dir.as_str();
        let folder_id = self.folder_id.clone().unwrap_or_default();
        let folder_id = folder_id.as_str();
        if self.tree_partitions.is_some() {
            let db = self.tree_partitions.clone().unwrap();
            return Ok(db)
        }
        // folders/c7c815is1s406kaf3j30/partitions
        // folders/c7c815is1s406kaf3j30/partition/0001.db
        // folders/c7c815is1s406kaf3j30/partition/0001.index
        let path = format!(
            "folders/{folder_id}/partitions",
            folder_id=folder_id
        );
        eprintln!("DbFolderItem.open_partitions :: path: {:?}", path);
        let result = self.database.open_tree(path);
        if result.is_err() {
            let error = result.unwrap_err();
            let message = error.to_string();
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("{}", message)),
                )
            )
        }
        let tree = result.unwrap();
        self.tree_partitions = Some(tree.clone());
        return Ok(tree)
    }

    pub fn get_partitions(&mut self) -> Result<Vec<u16>, PlanetError> {
        let mut list_partitions: Vec<u16> = Vec::new();
        let count = self.total_count()?;
        eprintln!("get_partitions :: count: {:?}", &count);
        let number_items = count.total.to_u16().unwrap();
        // 1200 items => 2 partitions
        // 1000 items => 1 partition
        // 1950 items => 2 partitions
        let mut number_partitions = number_items/ITEMS_PER_PARTITION;
        let rest = number_items%ITEMS_PER_PARTITION;
        if rest != 0 {
            number_partitions += 1;
        }
        for item in 1..number_partitions+1 {
            list_partitions.push(item);
        }
        return Ok(list_partitions)
    }
    fn get_language_code(&mut self, item_db: &DbData) -> Result<String, PlanetError> {
        let item_db = item_db.clone();
        let data = item_db.data;
        if data.is_none() {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("Item has no data")),
                )
            )
        }
        let data = data.unwrap();
        let db_folder = self.tree_folder.clone();
        let folder_id = self.folder_id.as_ref().unwrap();
        let folder = db_folder.get(folder_id);
        if folder.is_err() {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("Folder by id: \"{}\" not found.", folder_id)),
                )
            )
        }
        let folder = folder.unwrap();
        // eprintln!("DbFolderItem.get_language_code :: folder: {:#?}", &folder);
        let data_config = folder.clone().data.unwrap();
        let columns = data_config.get(COLUMNS);
        if columns.is_some() {
            let columns = columns.unwrap();
            // eprintln!("TreeFolderItem.get_language_code :: columns: {:#?}", columns);
            for column in columns {
                let column_type = column.get(COLUMN_TYPE);
                let column_id = column.get(ID);
                if column_type.is_some() {
                    let column_type = column_type.unwrap();
                    let column_id = column_id.unwrap();
                    if column_type == COLUMN_TYPE_LANGUAGE {
                        // eprintln!("TreeFolderItem.get_language_code :: column_id: {}", column_id);
                        let language_code = data.get(column_id);
                        // eprintln!("TreeFolderItem.get_language_code :: language_code: {:?}", &language_code);
                        if language_code.is_some() {
                            let language_code = language_code.unwrap();
                            let language_code = get_value_list(&language_code);
                            if language_code.is_some() {
                                let mut language_code = language_code.unwrap();
                                if language_code == String::from("") {
                                    let result = get_default_language_code(&folder);
                                    if result.is_ok() {
                                        language_code = result.unwrap();
                                    } else {
                                        return Err(
                                            PlanetError::new(
                                                500, 
                                                Some(tr!("Could not detect default language")),
                                            )
                                        )
                                    }
                                }
                                return Ok(language_code)
                            }
                        }
                    }
                }
            }
        }
        return Err(
            PlanetError::new(
                500, 
                Some(tr!("Could not find language column.")),
            )
        )
    }
    fn get_relevance_by_column_id(&mut self, column_id: &String) -> Result<u8, PlanetError> {
        let column_id = column_id.clone();
        let db_folder = self.tree_folder.clone();
        let folder_id = self.folder_id.as_ref().unwrap();
        let folder = db_folder.get(folder_id);
        if folder.is_err() {
            return Err(
                PlanetError::new(
                    500, 
                    Some(tr!("Folder by id: \"{}\" not found.", folder_id)),
                )
            )
        }
        let folder = folder.unwrap();
        let data = folder.data;
        let mut relevance_int: u8 = 1;
        if data.is_some() {
            let data = data.unwrap();
            let mut column_map_by_id: BTreeMap<String, String> = BTreeMap::new();
            // Make map of column_id -> column_name
            for (k, v) in data.clone() {
                if v.len() == 1 {
                    let v = &v[0];
                    let column_type = v.get(COLUMN_TYPE);
                    if !column_type.is_some() {
                        continue
                    }
                    let column_id = k;
                    let column_name = v.get(NAME).unwrap().clone();
                    column_map_by_id.insert(column_id, column_name);
                }
            }
            // Get column name for id
            let column_name = column_map_by_id.get(&column_id);
            if column_name.is_some() {
                let column_name = column_name.unwrap();
                let relevance_map = data.get(
                    TEXT_SEARCH_COLUMN_RELEVANCE
                );
                if relevance_map.is_some() {
                    let relevance_map = relevance_map.unwrap();
                    if relevance_map.len() == 1 {
                        let relevance_map = &relevance_map[0];
                        let relevance = relevance_map.get(column_name);
                        if relevance.is_some() {
                            let relevance = relevance.unwrap();
                            relevance_int = FromStr::from_str(relevance).unwrap();
                        }    
                    }
                }
            }
        }
        return Ok(relevance_int)
    }
    pub fn write_file(&mut self, db_file: &DbFile) -> Result<String, PlanetError> {
        // box/base/folder/c7c815is1s406kaf3j30/files.db
        let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        let db_file = db_file.clone();
        let encrypted_data = db_file.encrypt(&shared_key).unwrap();
        let encoded = encrypted_data.serialize();
        let id = db_file.id.clone().unwrap();
        let id_db = xid::Id::from_str(id.as_str()).unwrap();
        let id_db = id_db.as_bytes();
        let folder_id = self.folder_id.clone().unwrap_or_default();
        let folder_id = folder_id.as_str();
        let file_name = db_file.name.unwrap_or_default();
        let path_db = format!(
            "folders/{folder_id}/files.db",
            folder_id=folder_id,
        );
        let db: Tree;
        if self.files_db.is_some() {
            db = self.files_db.clone().unwrap();
        } else {
            let db_ = self.database.open_tree(path_db);
            if db_.is_err() {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!("Could not find language column.")),
                    )
                )
            }
            db = db_.unwrap();
            self.files_db = Some(db.clone());
        }
        let response = &db.insert(id_db, encoded);
        match response {
            Ok(_) => {
                eprintln!("TreeFolderItem.write_file :: Wrote OK!");
                return Ok(id.clone())
            },
            Err(_) => {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(
                            tr!(
                                "Could not write file \"{}\" into file database.", &file_name
                            )
                        )
                    )
                )
            }
        }    
    }
    pub fn export_file(&mut self, id: &String) -> Result<usize, PlanetError> {
        let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        let folder_id = self.folder_id.clone().unwrap_or_default();
        let folder_id = folder_id.as_str();
        let path_db = format!(
            "folders/{folder_id}/files.db",
            folder_id=folder_id,
        );
        let db: Tree;
        if self.files_db.is_some() {
            db = self.files_db.clone().unwrap();
        } else {
            let db_ = self.database.open_tree(path_db);
            if db_.is_err() {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!("Could not find language column.")),
                    )
                )
            }
            db = db_.unwrap();
            self.files_db = Some(db.clone());
        }
        let id_db = xid::Id::from_str(&id).unwrap();
        let id_db = id_db.as_bytes();
        let result = db.get(id_db);
        let file_size: usize;
        match result {
            Ok(_) => {
                let result = result.unwrap();
                if result.is_none() {
                    return Err(
                        PlanetError::new(
                            500, 
                            Some(tr!("Item not found in file database.")),
                        )
                    )
                }
                let item_db = result.unwrap().to_vec();
                let item_ = EncryptedMessage::deserialize(
                    item_db
                ).unwrap();
                let item_ = DbFile::decrypt_owned(
                    &item_, 
                    &shared_key
                );
                if item_.is_err() {
                    return Err(
                        PlanetError::new(
                            500, 
                            Some(tr!("Error decrypting file.")),
                        )
                    )
                }
                let mut file = item_.unwrap();
                let content = file.clone().content.unwrap();
                let path = file.get_home_path();
                let path = path.unwrap();
                let mut file = File::create(path).unwrap();
                let result = file.write(&content);
                if result.is_err() {
                    return Err(
                        PlanetError::new(
                            500, 
                            Some(tr!("Error writing into home directory.")),
                        )
                    )
                }
                file_size = result.unwrap();
            },
            Err(_) => {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!("Could not find item in file database.")),
                    )
                )
            }
        }
        return Ok(file_size)
    }

    pub fn get_value(column_id: &String, item: &DbData) -> Result<String, PlanetError> {
        let item = item.clone();
        let data_map = item.clone().data.unwrap();
        let column_id = column_id.clone();
        let item_values = data_map.get(&column_id);
        if item_values.is_some() {
            let item_values = item_values.unwrap();
            let value = get_value_list(item_values);
            if value.is_some() {
                let value = value.unwrap();
                return Ok(value)
            }
        }
        return Err(
            PlanetError::new(
                500, 
                Some(tr!("Could not find item in file database.")),
            )
        )
    }

}

impl FolderItem for TreeFolderItem {

    fn defaults(
        connection_pool: HashMap<String, sled::Db>,
        home_dir: &str,
        account_id: &str,
        space_id: &str,
        site_id: Option<String>,
        folder_id: &str,
        tree_folder: &TreeFolder,
    ) -> Result<TreeFolderItem, PlanetError> {
        // database for items is at space_id, for private space and for site spaces
        let database_ = connection_pool.get(space_id);
        if database_.is_none() {
            return Err(PlanetError::new(500, 
                Some(tr!("Could not get space database connection."))))
        }
        let database = database_.unwrap().clone();
        let db_row: TreeFolderItem = TreeFolderItem{
            database: database,
            home_dir: Some(home_dir.to_string()),
            account_id: Some(account_id.to_string()),
            space_id: Some(space_id.to_string()),
            site_id: site_id,
            tree_folder: tree_folder.clone(),
            folder_id: Some(folder_id.to_string()),
            tree: None,
            index: None,
            tree_partitions: None,
            files_db: None,
        };
        Ok(db_row)
    }

    fn insert(&mut self, folder_name: &String, db_data_list: &Vec<DbData>) -> Result<Vec<DbData>, Vec<PlanetError>> {
        let mut errors: Vec<PlanetError> = Vec::new();
        let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        let mut response_list: Vec<DbData> = Vec::new();
        for db_data in db_data_list {
            let mut db_data = db_data.clone();
            let data = db_data.data.clone();
            let mut text_data: BTreeMap<String, String> = BTreeMap::new();
            if data.is_some() {
                let mut data = data.unwrap();
                let my_text_data = data.get(TEXT);
                if my_text_data.is_some() {
                    text_data = my_text_data.unwrap()[0].clone();
                }
                data.remove(TEXT);
                db_data.data = Some(data.clone());
            }
            eprintln!("TreeFolderItem.insert :: db_data: {:#?}", &db_data);
            let encrypted_data = db_data.encrypt(&shared_key).unwrap();
            let encoded: Vec<u8> = encrypted_data.serialize();
            let id = db_data.id.clone().unwrap();
            let id_db = xid::Id::from_str(id.as_str()).unwrap();
            let id_db = id_db.as_bytes();
            let items = self.open_partition_by_item(&id);
            if items.is_err() {
                let error = items.unwrap_err();
                errors.push(error);
                continue
            }
            let items = items.unwrap();
            let tree = items.0;
            let response = &tree.insert(id_db, encoded);
            match response {
                Ok(_) => {
                    // Get item
                    let item_ = self.get(
                        &folder_name, 
                        GetItemOption::ById(id), 
                        None
                    );
                    match item_ {
                        Ok(_) => {
                            let item = item_.unwrap();
                            let index_response = self.index(&item, &text_data);
                            if index_response.is_err() {
                                let error = index_response.unwrap_err();
                                errors.push(error);
                            } else {
                                response_list.push(item);
                            }
                        },
                        Err(error) => {
                            errors.push(error);
                        }
                    }
                },
                Err(_) => {
                    errors.push(PlanetError::new(500, Some(tr!("Could not insert data"))));
                }
            }
        }
        if errors.len() > 0 {
            return Err(errors)
        } else {
            return Ok(response_list)
        }
    }

    fn index(&mut self, db_item: &DbData, text_map: &BTreeMap<String, String>) -> Result<DbData, PlanetError> {
        let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        let db_item = db_item.clone();
        let index_tree = self.index.clone();
        if index_tree.is_none() {
            return Err(
                PlanetError::new(500, Some(tr!("Index file is not open.")))
            )
        }
        let index_tree = index_tree.unwrap();
        let data = db_item.clone().data;
        if data.is_none() {
            return Err(
                PlanetError::new(500, Some(tr!("Could not insert data")))
            )
        }
        // let data = data.unwrap();
        // eprintln!("DbFolderItem.index :: data: {:#?}", &data);
        let mut my_list: Vec<BTreeMap<String, String>> = Vec::new();
        my_list.push(text_map.clone());
        let map = Some(my_list);
        // let map = data.get(&TEXT.to_string());
        let mut index_data: BTreeMap<String, Vec<BTreeMap<String, String>>> = BTreeMap::new();
        let language_code = self.get_language_code(&db_item);
        if language_code.is_err() {
            let error = language_code.unwrap_err();
            eprintln!("DbFolderItem.index :: error: {:#?}", &error);
            return Err(error)
        }
        let language_code = language_code.unwrap();
        // eprintln!("DbFolderItem.index :: language_code: {:#?}", &language_code);
        let stop_words = get_stop_words_by_language(&language_code);
        let stemmer = get_stemmer_by_language(&language_code);
        if map.is_some() {
            let map = map.unwrap();
            if map.len() == 1 {
                let map = &map[0];
                for (column_id, column_value) in map {
                    let mut processed_words: Vec<&str> = Vec::new();
                    // eprintln!("DbFolderItem.index :: column_id: {} column_value: {}", column_id, column_value);
                    let words: Vec<&str> = column_value.split(" ").collect();
                    // These words are not unique, can have n items for same word
                    for word in words {
                        // eprintln!("DbFolderItem.index :: word: {}", word);
                        let is_word_processed = processed_words.contains(&word);
                        if !is_word_processed {
                            let is_stop = stop_words.contains(&word.to_string());
                            // eprintln!("DbFolderItem.index :: is_stop: {}", &is_stop);
                            if !is_stop {
                                let stem = stemmer.stem(word);
                                let stem = stem.to_string();
                                let index_stem_ = index_data.get(&stem);

                                // {column_id}:{score},{column_id}:{score},...
                                let mut items: Vec<&str> = Vec::new();
                                let index_stem: String;
                                if index_stem_.is_some() {
                                    let index_stem_ = index_stem_.unwrap();
                                    let index_stem_ = get_value_list(&index_stem_);
                                    if index_stem_.is_some() {
                                        index_stem = index_stem_.unwrap();
                                        items = index_stem.split(",").collect();
                                    }
                                }
                                // Calculate score
                                // Get relevance from the column name and text_search_column_relevance
                                // Relevance is 1-5
                                let relevance = self.get_relevance_by_column_id(&column_id).unwrap();
                                let score = relevance.to_string();
                                let item= format!("{}:{}", column_id, &score);
                                let item= item.as_str();
                                items.push(item);
                                let items_string = items.join(",");
                                let items_list = build_value_list(&items_string);
                                index_data.insert(stem.clone(), items_list);
                                processed_words.push(word);
                            }
                        }
                    }
                }    
            }
        }
        if index_data.len() == 0 {
            return Err(
                PlanetError::new(500, Some(tr!("No data to be indexed.")))
            )
        }
        // eprintln!("DbFolderItem.index :: index_data: {:#?}", &index_data);
        // Write into disk the index data
        let mut site_id_wrap: Option<String> = None;
        let site_id = self.site_id.clone().unwrap_or_default();
        let space_id = self.space_id.clone().unwrap_or_default();
        if site_id != String::from("") {
            site_id_wrap = Some(site_id);
        }
        let routing_wrap = RoutingData::defaults(
            self.account_id.clone(),
            site_id_wrap, 
            &space_id, 
            None
        );
        let name = db_item.name.unwrap_or_default();
        let db_index_data = DbData::defaults(
            &name,
            Some(index_data),
            None,
            routing_wrap,
            None,
            None,
        );
        if db_index_data.is_err() {
            let error = db_index_data.unwrap_err();
            return Err(error)
        }
        let db_index_data = db_index_data.unwrap();
        eprintln!("DbFolderItem.index :: I will write into index: {:#?}", &db_index_data);
        let id = db_item.id.clone().unwrap();
        let id_db = xid::Id::from_str(id.as_str()).unwrap();
        let id_db = id_db.as_bytes();
        let encrypted_data = db_index_data.encrypt(&shared_key).unwrap();
        let encoded: Vec<u8> = encrypted_data.serialize();
        let response = index_tree.insert(id_db, encoded);
        if response.is_err() {
            return Err(
                PlanetError::new(500, Some(tr!("Error writing into index.")))
            )
        }
        // Get from index by id
        let response = index_tree.get(&id_db);
        let response = response.unwrap();
        let item_db = response.unwrap().to_vec();
        let item_ = EncryptedMessage::deserialize(item_db).unwrap();
        let item_ = DbData::decrypt_owned(
            &item_, 
            &shared_key);
        if item_.is_err() {
            return Err(
                PlanetError::new(500, Some(tr!("Error decrypting from indices.")))
            )
        }
        let item_ = item_.unwrap();
        // eprintln!("DbFolderItem.index :: I wrote into index: {:#?}", &item_);
        return Ok(item_.clone())
    }

    fn update(&mut self, db_data: &DbData) -> Result<DbData, PlanetError> {
        let db_data = db_data.clone();
        let id = db_data.id.clone().unwrap();
        let id_db = xid::Id::from_str(id.as_str()).unwrap();
        let id_db = id_db.as_bytes();
        let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        let encrypted_data = db_data.encrypt(&shared_key).unwrap();
        let encoded: Vec<u8> = encrypted_data.serialize();
        let (db, _) = self.open_partition_by_item(&id)?;
        let response = &db.insert(id_db, encoded);
        match response {
            Ok(_) => {
                let response = response.clone().unwrap();
                let item_db = response.unwrap().to_vec();
                let item = EncryptedMessage::deserialize(
                    item_db
                ).unwrap();
                let item = DbData::decrypt_owned(
                    &item, 
                    &shared_key
                );
                match item {
                    Ok(_) => {
                        let item = item.unwrap();
                        Ok(item)
                    },
                    Err(_) => {
                        let error = item.unwrap_err();
                        Err(PlanetError::new(
                            500, 
                            Some(tr!(
                                "Could not update data, encryption error: {}", error.to_string()
                            )
                        )))
                    }
                }
            },
            Err(_) => {
                Err(PlanetError::new(500, Some(tr!("Could not update data"))))
            }
        }
    }

    // We can get items by id (not changing string), and name (we search for slugified name)
    fn get(
        &mut self, 
        folder_name: &String, 
        by: GetItemOption, 
        columns: Option<Vec<String>>
    ) -> Result<DbData, PlanetError> {
        let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        let item_db: Vec<u8>;
        match by {
            GetItemOption::ById(id) => {
                let id_db = xid::Id::from_str(&id).unwrap();
                let id_db = id_db.as_bytes();
                let (tree, _) = self.open_partition_by_item(&id)?;
                let db_result = tree.get(&id_db);
                if db_result.is_err() {
                    return Err(
                        PlanetError::new(
                            500, 
                            Some(tr!("Could not fetch item from database"))
                        )
                    );
                }
                let item_exsists = db_result.unwrap();
                if item_exsists.is_none() {
                    return Err(
                        PlanetError::new(
                            404, 
                            Some(tr!("Folder Item with id \"{}\" at folder \"{}\" does not exist.", &id, folder_name))
                        )
                    );
                }
                item_db = item_exsists.unwrap().to_vec();
                let item_ = EncryptedMessage::deserialize(item_db).unwrap();
                let item_ = DbData::decrypt_owned(
                    &item_, 
                    &shared_key);
                match item_ {
                    Ok(_) => {
                        let mut item = item_.unwrap();
                        if columns.is_none() {
                            Ok(item)    
                        } else {
                            // eprintln!("get :: item: {:#?}", &item);
                            // If columns is informed, then I need to remove from item.data columns not requested
                            // data: Some(
                            //     {
                            //         "c49qh6osmpv69nnrt33g": "pepito",
                            //         "c49qh6osmpv69nnrt35g": "c49qh6osmpv69nnrt370",
                            //         "c49qh6osmpv69nnrt34g": "true",
                            //         "c49qh6osmpv69nnrt350": "34",
                            //         "c49qh6osmpv69nnrt360": "c49qh6osmpv69nnrt380,c49qh6osmpv69nnrt390",
                            //         "c49qh6osmpv69nnrt340": "This is some description I want to include",
                            //     },
                            // ),
                            let columns = columns.unwrap();
                            item = self.filter_fields(&folder_name, &columns, &item)?;
                            Ok(item)
                        }
                    },
                    Err(_) => {
                        Err(PlanetError::new(500, Some(tr!("Could not fetch item from database"))))
                    }
                }
            },
            GetItemOption::ByName(name) => {
                eprintln!("DbFolderItem.get :: by name, name: {}", &name);
                let partitions = self.get_partitions()?;
                let this: Arc<Mutex<TreeFolderItem>> = Arc::new(Mutex::new(self.clone()));
                let mut handles= vec![];
                // Threads to fetch name from all partitions
                let wrap_db_data: Arc<Mutex<Option<DbData>>> = Arc::new(Mutex::new(None));
                let shared_key: Arc<Mutex<SharedKey>> = Arc::new(Mutex::new(shared_key));
                let name: Arc<Mutex<String>> = Arc::new(Mutex::new(name));
                for partition in partitions {
                    let this = Arc::clone(&this);
                    let wrap_db_data = Arc::clone(&wrap_db_data);
                    let shared_key = Arc::clone(&shared_key);
                    let name = Arc::clone(&name);
                    let handle = thread::spawn(move || {
                        // open partition and search for name
                        let mut this = this.lock().unwrap();
                        let shared_key = shared_key.lock().unwrap();
                        let name = name.lock().unwrap();
                        let (db, _) = this.open_partition(&partition).unwrap();
                        let mut my_wrap_db_data: Option<DbData> = None;
                        for db_result in db.iter() {
                            let (_, db_item) = db_result.unwrap();
                            let db_item = db_item.to_vec();
                            let item_ = EncryptedMessage::deserialize(
                                db_item
                            ).unwrap();
                            let item_ = DbData::decrypt_owned(
                                &item_, 
                                &shared_key
                            );
                            let item_db_data = item_.unwrap();
                            let item_db_data_ = item_db_data.clone();
                            let item_db_name = &item_db_data.name.unwrap();
                            if *item_db_name == *name {
                                my_wrap_db_data = Some(item_db_data_);
                                break;
                            }
                        }
                        if my_wrap_db_data.is_some() {
                            let mut wrap_db_data = wrap_db_data.lock().unwrap();
                            *wrap_db_data = my_wrap_db_data;
                        }
                    });
                    handles.push(handle);
                }
                for handle in handles {
                    handle.join().unwrap();
                }
                let wrap_db_data_ = wrap_db_data.lock().unwrap().clone();
                let found = wrap_db_data_.is_some();
            
                if found == false {
                    return Err(
                        PlanetError::new(
                            404, 
                            Some(tr!("Item does not exist"))
                        )
                    );
                }
                return Ok(wrap_db_data_.unwrap())
            }
        }
    }
    fn count(&mut self, 
        folder_name: &String, 
        r#where: Option<String>, 
    ) -> Result<SelectCountResult, PlanetError> {
        let select_result = self.select(
            folder_name,
            r#where,
            Some(1),
            Some(1),
            None
        )?;
        let result = SelectCountResult{
            time: select_result.time,
            total: select_result.total,
            data_count: select_result.data_count,
        };
        return Ok(result)
    }
    fn total_count(&mut self) -> Result<SelectCountResult, PlanetError> {
        let t_1 = Instant::now();
        let db = self.open_partitions()?;
        let total = db.len();
        let result = SelectCountResult{
            time: t_1.elapsed().as_millis() as usize,
            total: total,
            data_count: total,
        };
        let _ = result.time;
        let _ = result.total;
        let _ = result.data_count;
        return Ok(result);
    }
    fn select(&mut self, 
        _folder_name: &String, 
        r#_where: Option<String>, 
        _page: Option<usize>,
        _number_items_page: Option<usize>,
        _fields: Option<Vec<String>>,
    ) -> Result<SelectResult, PlanetError> {
        // let t_1 = Instant::now();
        // let shared_key: SharedKey = SharedKey::from_array(CHILD_PRIVATE_KEY_ARRAY);
        // let iter = self.db.iter();
        // let db_folder = self.db_folder.clone();
        // let folder = db_folder.get_by_name(folder_name)?.unwrap();
        // let field_config_map = ColumnConfig::get_Column_config_map(
        //     self.planet_context,
        //     self.context,
        //     &folder
        // ).unwrap();
        // let field_config_map_wrap = Some(field_config_map.clone());
        // // eprintln!("DbFolderItem.select :: folder: {:#?}", &folder);
        // let fields_wrap = columns.clone();
        // let has_fields = fields_wrap.is_some();
        // let mut columns: Vec<String> = Vec::new();
        // if fields_wrap.is_some() {
        //     columns = fields_wrap.unwrap();
        // }
        
        // let t_total_1 = Instant::now();
        // let result_total_count = self.total_count()?;
        // let total = result_total_count.total;
        // eprintln!("DbFolderItem.select :: get total: {} µs", &t_total_1.elapsed().as_micros());
        
        // let mut items: Vec<DbData> = Vec::new();
        // let page = page.unwrap();
        // let number_items_page = number_items_page.unwrap();
        // let where_formula = r#where.clone();
        // let where_formula = where_formula.unwrap_or_default();
        // let mut count = 1;
        // Think way to return total
        let total = 0;
        let page = 0;
        let select_result = SelectResult{
            total: total,
            time: 0,
            page: page,
            data: Vec::new(),
            data_count: 0,
        };
        let _ = select_result.data;
        let _ = select_result.total;
        let _ = select_result.time;
        let _ = select_result.page;
        let _ = select_result.data_count;
        // let t_header = &t_1.elapsed().as_micros();
        // eprintln!("DbFolderItem.select :: t_header: {} µs", &t_header);

        // let t_f_1 = Instant::now();
        // // Check where_formula is assign or not
        // let expr = &RE_FORMULA_ASSIGN;
        // let is_assign_function = expr.is_match(&where_formula);
        // eprintln!("DbFolderItem.select :: is_assign_function: {}", &is_assign_function);
        // let formula_query = Formula::defaults(
        //     &where_formula, 
        //     &String::from("bool"), 
        //     Some(folder), 
        //     None, 
        //     Some(db_folder), 
        //     Some(folder_name.clone()), 
        //     is_assign_function,
        //     field_config_map_wrap.clone()
        // )?;
        // // eprintln!("DbFolderItem.select :: original formula_query: {:#?}", &formula_query);
        // let t_f_2 = &t_f_1.elapsed().as_micros();
        // eprintln!("select :: Time compile formula: {} µs", &t_f_2);

        // for result in iter {
        //     let t_item_1 = Instant::now();
        //     let tuple = result.unwrap();
        //     let item_db = tuple.1.to_vec();
        //     let item_ = EncryptedMessage::deserialize(item_db).unwrap();
        //     let item_ = DbData::decrypt_owned(
        //         &item_, 
        //         &shared_key);
        //     eprintln!("DbFolderItem.select :: [{}] encrypt & deser: {} µs", &count, &t_item_1.elapsed().as_micros());
        //     // let t_item_2 = Instant::now();
        //     let item = item_.unwrap().clone();
        //     let routing_response = item.clone().routing;
        //     if routing_response.is_some() {
        //         let routing = routing_response.unwrap();
        //         let account_id = routing.get(ACCOUNT_ID).unwrap().as_str();
        //         let space_id = routing.get(SPACE_ID).unwrap().as_str();
        //         if account_id == "" && space_id == "" {
        //             continue
        //         }
        //     }

        //     let formula_matches: bool;
        //     let formula_result: String;
        //     let t_item_3 = Instant::now();
        //     let data_map = item.clone().data.unwrap();

        //     formula_result = execute_formula(&formula_query, &data_map, &field_config_map)?;
        //     if formula_result == String::from("1") {
        //         formula_matches = true;
        //     } else {
        //         formula_matches = false;
        //     }
        //     eprintln!("select :: formula_matches: {}", &formula_matches);
        //     eprintln!("DbFolderItem.select :: [{}] formula exec: {} µs", &count, &t_item_3.elapsed().as_micros());

        //     let count_float: f64 = FromStr::from_str(count.to_string().as_str()).unwrap();
        //     let number_items_page_float: f64 = FromStr::from_str(
        //         number_items_page.to_string().as_str()).unwrap();
        //     let page_target = (count_float / number_items_page_float).round() + 1.0;
        //     let page_target = page_target as usize;
        //     if page_target == page && formula_matches {
        //         if &has_fields == &true {
        //             let item_new = self.filter_fields(&folder_name, &columns, &item)?;
        //             items.push(item_new);
        //         } else {
        //             items.push(item);
        //         }
        //     } else {
        //         continue
        //     }
        //     // let number_items_page_ = number_items_page as usize;
        //     count += 1;
        //     let t_item = t_item_1.elapsed().as_micros();
        //     eprintln!("DbFolderItem.select :: item [{}] : {} µs", &count-1, &t_item);
        // }
        // select_result.data = items;
        // select_result.data_count = select_result.data.len();
        // select_result.time = t_1.elapsed().as_millis() as usize;
        // eprintln!("DbFolderItem.select :: total db time: {} µs", &t_1.elapsed().as_micros());
        return Ok(select_result);
    }
}

#[derive(Debug, Clone)]
pub struct SelectResult {
    total: usize,
    time: usize,
    page: usize,
    data_count: usize,
    data: Vec<DbData>,
}

#[derive(Debug, Clone)]
pub struct SelectCountResult {
    total: usize,
    time: usize,
    data_count: usize,
}

impl TreeFolderItem {

    pub fn filter_fields(
        &self, 
        folder_name: &String, 
        columns: &Vec<String>, 
        item: &DbData
    ) -> Result<DbData, PlanetError> {
        let columns = columns.clone();
        let mut item = item.clone();
        // column_id => column_name
        let column_id_map= TreeFolder::get_column_id_map(
            &self.tree_folder,
            folder_name
        )?;
        // data
        let mut data_new: BTreeMap<String, Vec<BTreeMap<String, String>>> = BTreeMap::new();
        let db_data = item.data;
        if db_data.is_some() {
            let db_data = db_data.unwrap();
            for (column_db_id, column_value) in db_data {
                let column_db_name = &column_id_map.get(&column_db_id).unwrap().clone();
                for column in &columns {
                    if column.to_lowercase() == column_db_name.to_lowercase() {
                        data_new.insert(column_db_id.clone(), column_value.clone());
                    }
                }
            }
            item.data = Some(data_new);
        } else {
            item.data = None;
        }
        // data_collections
        // let mut data_collections_new: BTreeMap<String, Vec<BTreeMap<String, String>>> = BTreeMap::new();
        // let db_data_collections = item.data_collections;
        // if db_data_collections.is_some() {
        //     for (field_db_id, items) in db_data_collections.unwrap() {
        //         let field_db_name = &field_id_map.get(&field_db_id).unwrap().clone();
        //         for column in &columns {
        //             if column.to_lowercase() == field_db_name.to_lowercase() {
        //                 data_collections_new.insert(field_db_id.clone(), items.clone());
        //             }
        //         }
        //     }
        //     item.data_collections = Some(data_collections_new);
        // } else {
        //     item.data_collections = None;
        // }
        // // data_objects
        // let mut data_objects_new: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
        // let db_data_objects = item.data_objects;
        // if db_data_objects.is_some() {
        //     for (field_db_id, map) in db_data_objects.unwrap() {
        //         let field_db_name = &field_id_map.get(&field_db_id).unwrap().clone();
        //         for column in &columns {
        //             if column.to_lowercase() == field_db_name.to_lowercase() {
        //                 data_objects_new.insert(field_db_id.clone(), map.clone());
        //             }
        //         }
        //     }
        //     item.data_objects = Some(data_objects_new);
        // } else {
        //     item.data_objects = None;
        // }
        return Ok(item);
    }
}

pub struct DbQuery {
    pub sql_where: String,
    pub page: u8,
    pub number_items: u8,
}

pub fn build_value_list(value: &String) -> Vec<BTreeMap<String, String>> {
    let mut my_map: BTreeMap<String, String> = BTreeMap::new();
    my_map.insert(VALUE.to_string(), value.clone());
    let mut my_list: Vec<BTreeMap<String, String>> = Vec::new();
    my_list.push(my_map);
    return my_list
}

pub fn get_value_list(list: &Vec<BTreeMap<String, String>>) -> Option<String> {
    if list.len() == 1 {
        let map = &list[0];
        let value = map.get(VALUE);
        if value.is_some() {
            let value = value.unwrap();
            return Some(value.clone())
        }
    }
    return None
}
