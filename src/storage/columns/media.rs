extern crate rust_stemmers;
extern crate dirs;

use std::collections::{BTreeMap, HashMap};
use colored::Colorize;
use std::fs::File;
use mime_guess;
use json;
use reqwest::blocking::Client;
use serde_yaml;

use crate::planet::{PlanetError};
use crate::statements::folder::schema::*;
use crate::storage::constants::*;
use crate::storage::columns::*;
use crate::storage::folder::{DbFile, RoutingData, TreeFolderItem};
use crate::storage::generate_id;
use crate::storage::space::SpaceDatabase;

#[derive(Debug, Clone)]
pub struct FileColumn {
    pub config: ColumnConfig,
    pub db_folder_item: Option<TreeFolderItem>,
    pub space_database: Option<SpaceDatabase>,
}
impl FileColumn {
    pub fn defaults(
        column_config: &ColumnConfig,
        db_folder_item: Option<TreeFolderItem>,
        space_database: Option<SpaceDatabase>
    ) -> Self {
        let column_config = column_config.clone();
        let field_obj = Self{
            config: column_config,
            db_folder_item: db_folder_item,
            space_database: space_database
        };
        return field_obj
    }
    pub fn validate(
        &self, 
        paths: &Vec<String>,
        data: &BTreeMap<String, Vec<BTreeMap<String, String>>>,
        routing: Option<RoutingData>,
        home_dir: &String,
    ) -> Result<(
            Vec<String>,
            Vec<String>,
            BTreeMap<String, Vec<BTreeMap<String, String>>>
        ), PlanetError> {
        let mut db_folder_item = self.db_folder_item.clone().unwrap();
        let mut data = data.clone();
        let paths = paths.clone();
        let config = self.config.clone();
        let column_id = &config.id.unwrap();
        let content_types_wrap = config.content_types;
        let mut content_types: Vec<String> = Vec::new();
        if content_types_wrap.is_some() {
            content_types = content_types_wrap.unwrap();
        }
        let mut document_texts: Vec<String> = Vec::new();
        let mut file_ids: Vec<String> = Vec::new();
        for path in paths.clone() {
            let path_fields: Vec<&str> = path.split("/").collect();
            let file_name = path_fields.last().unwrap().clone();
            // 1. Check path exists, raise error if does not exist
            let file = File::open(path.clone());
            if file.is_err() {
                let error = file.unwrap_err();
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!(
                            "Could not open File at \"{}\". Error: \"{}\".", &path, &error
                        )),
                    )
                );
            }
            let file = file.unwrap();
            // 2. Validate file is allowed in config content types. Some crate??? Using file name????
            let mime_guess = mime_guess::from_path(file_name);
            let content_type_wrap = mime_guess.first();
            let mut content_type: String = String::from("");
            if content_type_wrap.is_some() {
                let content_type_ = content_type_wrap.unwrap();
                content_type = content_type_.to_string();
            }
            if content_types.len() > 0 {
                let check = content_types.contains(&content_type);
                if !check {
                    return Err(
                        PlanetError::new(
                            500, 
                            Some(tr!(
                                "File with path \"{}\" has content type not supported: \"{}\"", 
                                &path, &content_type
                            )),
                        )
                    );
                }
            }
            let file_type = get_file_type(&content_type.as_str());
            if file_type.is_err() {
                let error = file_type.unwrap_err();
                return Err(error);
            }
            let file_type = file_type.unwrap();
            let metadata = &file.metadata().unwrap();
            let file_size = metadata.len();
            if metadata.is_dir() {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!(
                            "Path \"\" is location of a directory instead of a file.", &path
                        )),
                    )
                );
            }
            let client = Client::new();
            let url = format!("http://{host}:{port}/rmeta/text", host=TIKA_HOST, port=TIKA_PORT);
            let res = client.put(&url)
            .body(file)
            .send();
            let mut file_id: Option<String> = None;
            if res.is_ok() {
                let response = res.unwrap();
                let response = response.text().unwrap();
                let json_document = json::parse(&response);
                let mut my_map: BTreeMap<String, String> = BTreeMap::new();
                if json_document.is_ok() {
                    let json_document = json_document.unwrap();
                    let main_document = &json_document[0];
                    let title = &main_document["dc:title"];
                    let created_time = &main_document["dcterms:created"];
                    let last_modified_time = &main_document["dcterms:modified"];
                    let content_type = &main_document["Content-Type"];
                    let creator = &main_document["dc:creator"];
                    let subject = &main_document["dc:subject"];
                    let description = &main_document["dc:description"];
                    let category = &main_document["cp:category"];
                    let image_width = &main_document["Image Width"];
                    let image_height = &main_document["Image Height"];
                    let text = &main_document["X-TIKA:content"];
                    let id = generate_id();
                    if id.is_some() {
                        let id = id.unwrap();
                        file_id = Some(id.clone());
                        my_map.insert(
                            ID.to_string(), 
                            id
                        );    
                    }
                    if !title.is_null() {
                        my_map.insert(
                            FILE_PROP_TITLE.to_string(), 
                            title.to_string()
                        );
                    }
                    my_map.insert(
                        FILE_PROP_FILE_NAME.to_string(), 
                        file_name.to_string()
                    );
                    my_map.insert(
                        FILE_PROP_SIZE.to_string(), 
                        file_size.to_string()
                    );
                    if !created_time.is_null() {
                        my_map.insert(
                            FILE_PROP_CREATED_TIME.to_string(), 
                            created_time.to_string()
                        );
                    }
                    if !last_modified_time.is_null() {
                        my_map.insert(
                            FILE_PROP_LAST_MODIFIED_TIME.to_string(), 
                            last_modified_time.to_string()
                        );
                    }
                    if !content_type.is_null() {
                        my_map.insert(
                            FILE_PROP_CONTENT_TYPE.to_string(), 
                            content_type.to_string()
                        );
                        my_map.insert(
                            FILE_PROP_FILE_TYPE.to_string(), 
                            file_type.clone()
                        );
                    }
                    if !creator.is_null() {
                        my_map.insert(
                            FILE_PROP_CREATOR.to_string(), 
                            creator.to_string()
                        );
                    }
                    if !category.is_null() {
                        my_map.insert(
                            FILE_PROP_CATEGORY.to_string(), 
                            category.to_string()
                        );
                    }
                    if !subject.is_null() {
                        if subject.is_string() {
                            my_map.insert(
                                FILE_PROP_SUBJECT.to_string(), 
                                subject.to_string()
                            );
                        } else {
                            my_map.insert(
                                FILE_PROP_SUBJECT.to_string(), 
                                subject[0].to_string()
                            );
                            let keywords = subject[1].to_string();
                            let check = keywords.contains(",");
                            if check {
                                let keyword_list: Vec<&str> = keywords.split(",").collect();
                                let mut list: Vec<BTreeMap<String, String>> = Vec::new();
                                for keyword in keyword_list {
                                    let keyword = keyword.trim();
                                    let mut my_map: BTreeMap<String, String> = BTreeMap::new();
                                    my_map.insert(VALUE.to_string(), keyword.to_string());
                                    list.push(my_map);
                                }
                                let key = format!("{}__tags", column_id);
                                data.insert(key, list);
                            } else {
                                let keyword = keywords;
                                let mut list: Vec<BTreeMap<String, String>> = Vec::new();
                                let mut my_map: BTreeMap<String, String> = BTreeMap::new();
                                my_map.insert(VALUE.to_string(), keyword.to_string());
                                list.push(my_map);
                                let key = format!("{}__tags", column_id);
                                data.insert(key, list);
                            }
                            my_map.insert(
                                FILE_PROP_SUBJECT.to_string(), 
                                subject[0].to_string()
                            );
                        }
                    }
                    if !image_width.is_null() {
                        let width_str = image_width.to_string();
                        let fields: Vec<&str> = width_str.split(" pixels").collect();
                        my_map.insert(
                            FILE_PROP_IMAGE_WIDTH.to_string(), 
                            fields[0].to_string()
                        );
                    }
                    if !image_height.is_null() {
                        let height_str = image_height.to_string();
                        let fields: Vec<&str> = height_str.split(" pixels").collect();
                        my_map.insert(
                            FILE_PROP_IMAGE_HEIGHT.to_string(), 
                            fields[0].to_string()
                        );
                    }
                    if !description.is_null() {
                        my_map.insert(
                            FILE_PROP_DESCRIPTION.to_string(), 
                            description.to_string()
                        );
                    }
                    if !text.is_null() {
                        let mut text = text.to_string();
                        text = text.replace("\n", "");
                        text = text.replace("\t", "");
                        text = text.replace("\r", "");
                        document_texts.push(text);
                    }
                    // let text = &main_document["X-TIKA:content"];
                    // main_document.remove("X-TIKA:content");
                    my_map.insert(
                        FILE_PROP_METADATA.to_string(), 
                        response
                    );
                    let mut my_list: Vec<BTreeMap<String, String>> = Vec::new();
                    my_list.push(my_map);
                    data.insert(column_id.clone(), my_list);
                } else {
                    return Err(
                        PlanetError::new(
                            500, 
                            Some(tr!(
                                "Error processing metadata response."
                            )),
                        )
                    );
                }
            } else {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!(
                            "Error processing file metadata."
                        )),
                    )
                );
            }
            // Write file into database or home OS dir
            if file_id.is_some() {
                let file_id = file_id.unwrap();
                let mut file = File::open(path.clone()).unwrap();
                let size = file.metadata().unwrap().len();
                let db_file = DbFile::defaults(
                    &file_id, 
                    &file_name.to_string(), 
                    &mut file,
                    &content_type,
                    &file_type,
                    routing.clone(),
                    home_dir
                );
                if db_file.is_err() {
                    return Err(
                        PlanetError::new(
                            500, 
                            Some(tr!(
                                "Error writing into file database."
                            )),
                        )
                    );                    
                }
                let mut db_file = db_file.unwrap();
                if size < MAX_FILE_DB {
                    let file_id = db_folder_item.write_file(&db_file);
                    if file_id.is_err() {
                        return Err(
                            PlanetError::new(
                                500, 
                                Some(tr!(
                                    "Error writing into file database."
                                )),
                            )
                        );
                    }
                    let file_id = file_id.unwrap();
                    file_ids.push(file_id.clone());
                } else {
                    // File into OS home directory for space
                    let file_id = db_file.write_file(&mut file, &db_folder_item);
                    if file_id.is_ok() {
                        let file_id = file_id.unwrap();
                        file_ids.push(file_id);
                    } else {
                        return Err(
                            PlanetError::new(
                                500, 
                                Some(tr!(
                                    "Error writing big file into file database."
                                )),
                            )
                        );        
                    }
                }
            } else {
                return Err(
                    PlanetError::new(
                        500, 
                        Some(tr!(
                            "Error generating file id."
                        )),
                    )
                );
            }
        }
        return Ok(
            (
                file_ids.clone(),
                document_texts.clone(),
                data.clone()
            )
        )
    }
}
impl StorageColumnBasic for FileColumn {
    fn create_config(
        &mut self, 
        column_config_map: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, PlanetError> {
        let mut column_config_map = column_config_map.clone();
        let config = self.config.clone();
        let many = config.many;
        if many.is_some() {
            let many = many.unwrap();
            column_config_map.insert(String::from(MANY), many.to_string());
        }
        let content_types = config.content_types;
        if content_types.is_some() {
            let content_types = content_types.unwrap();
            let content_types_serialized = serde_yaml::to_string(&content_types).unwrap();
            column_config_map.insert(String::from(CONTENT_TYPES), content_types_serialized);
        }
        return Ok(column_config_map)
    }
    fn get_config(
        &mut self, 
        column_config_map: &BTreeMap<String, String>,
    ) -> Result<ColumnConfig, PlanetError> {
        let mut config = self.config.clone();
        let column_config_map = column_config_map.clone();
        let many = column_config_map.get(MANY);
        let content_types = column_config_map.get(CONTENT_TYPES);
        if many.is_some() {
            let many = many.unwrap();
            let many = many.clone() == String::from(TRUE);
            config.many = Some(many);
        }
        if content_types.is_some() {
            let content_types = content_types.unwrap();
            let content_types: Vec<String> = serde_yaml::from_str(content_types).unwrap();
            config.content_types = Some(content_types);
        }
        return Ok(config)
    }
    fn get_yaml_out(&self, yaml_string: &String, value: &String) -> String {
        let column_config = self.config.clone();
        let column_name = column_config.name.unwrap();
        let mut yaml_string = yaml_string.clone();
        let field = &column_name.truecolor(
            YAML_COLOR_BLUE[0], YAML_COLOR_BLUE[1], YAML_COLOR_BLUE[2]
        );
        let value = format!("{}", 
            value.truecolor(YAML_COLOR_ORANGE[0], YAML_COLOR_ORANGE[1], YAML_COLOR_ORANGE[2]), 
        );
        yaml_string.push_str(format!("  {field}: {value}\n", field=field, value=value).as_str());
        return yaml_string;
    }
}

pub fn get_file_types() -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    map.insert(String::from("application/vnd.hzn-3d-crossword"), String::from("3D Crossword Plugin"));
    map.insert(String::from("video/3gpp"), String::from("3GP"));
    map.insert(String::from("video/3gpp2"), String::from("3GP2"));
    map.insert(String::from("application/vnd.mseq"), String::from("3GPP MSEQ File"));
    map.insert(String::from("application/vnd.3m.post-it-notes"), String::from("3M Post It Notes"));
    map.insert(String::from("application/vnd.3gpp.pic-bw-large"), String::from("3rd Generation Partnership Project - Pic Large"));
    map.insert(String::from("application/vnd.3gpp.pic-bw-small"), String::from("3rd Generation Partnership Project - Pic Small"));
    map.insert(String::from("application/vnd.3gpp.pic-bw-var"), String::from("3rd Generation Partnership Project - Pic Var"));
    map.insert(String::from("application/vnd.3gpp2.tcap"), String::from("3rd Generation Partnership Project - Transaction Capabilities Application Part"));
    map.insert(String::from("application/x-7z-compressed"), String::from("7-Zip"));
    map.insert(String::from("application/x-abiword"), String::from("AbiWord"));
    map.insert(String::from("application/x-ace-compressed"), String::from("Ace Archive"));
    map.insert(String::from("application/vnd.americandynamics.acc"), String::from("Active Content Compression"));
    map.insert(String::from("application/vnd.acucobol"), String::from("ACU Cobol"));
    map.insert(String::from("application/vnd.acucorp"), String::from("ACU Cobol"));
    map.insert(String::from("audio/adpcm"), String::from("Adaptive differential pulse-code modulation"));
    map.insert(String::from("application/x-authorware-bin"), String::from("Adobe (Macropedia) Authorware - Binary File"));
    map.insert(String::from("application/x-authorware-map"), String::from("Adobe (Macropedia) Authorware - Map"));
    map.insert(String::from("application/x-authorware-seg"), String::from("Adobe (Macropedia) Authorware - Segment File"));
    map.insert(String::from("application/vnd.adobe.air-application-installer-package+zip"), String::from("Adobe AIR Application"));
    map.insert(String::from("application/x-shockwave-flash"), String::from("Adobe Flash"));
    map.insert(String::from("application/vnd.adobe.fxp"), String::from("Adobe Flex Project"));
    map.insert(String::from("application/pdf"), String::from("Adobe Portable Document Format"));
    map.insert(String::from("application/vnd.cups-ppd"), String::from("Adobe PostScript Printer Description File Format"));
    map.insert(String::from("application/x-director"), String::from("Adobe Shockwave Player"));
    map.insert(String::from("application/vnd.adobe.xdp+xml"), String::from("Adobe XML Data Package"));
    map.insert(String::from("application/vnd.adobe.xfdf"), String::from("Adobe XML Forms Data Format"));
    map.insert(String::from("audio/x-aac"), String::from("Advanced Audio Coding (AAC)"));
    map.insert(String::from("application/vnd.ahead.space"), String::from("Ahead AIR Application"));
    map.insert(String::from("application/vnd.airzip.filesecure.azf"), String::from("AirZip FileSECURE"));
    map.insert(String::from("application/vnd.airzip.filesecure.azs"), String::from("AirZip FileSECURE"));
    map.insert(String::from("application/vnd.amazon.ebook"), String::from("Amazon Kindle eBook format"));
    map.insert(String::from("application/vnd.amiga.ami"), String::from("AmigaDE"));
    map.insert(String::from("application/andrew-inset"), String::from("Andrew Toolkit"));
    map.insert(String::from("application/vnd.android.package-archive"), String::from("Android Package Archive"));
    map.insert(String::from("application/vnd.anser-web-certificate-issue-initiation"), String::from("ANSER-WEB Terminal Client - Certificate Issue"));
    map.insert(String::from("application/vnd.anser-web-funds-transfer-initiation"), String::from("ANSER-WEB Terminal Client - Web Funds Transfer"));
    map.insert(String::from("application/vnd.antix.game-component"), String::from("Antix Game Player"));
    map.insert(String::from("application/x-apple-diskimage"), String::from("Apple Disk Image"));
    map.insert(String::from("application/vnd.apple.installer+xml"), String::from("Apple Installer Package"));
    map.insert(String::from("application/applixware"), String::from("Applixware"));
    map.insert(String::from("application/vnd.hhe.lesson-player"), String::from("Archipelago Lesson Player"));
    map.insert(String::from("application/vnd.aristanetworks.swi"), String::from("Arista Networks Software Image"));
    map.insert(String::from("text/x-asm"), String::from("Assembler Source File"));
    map.insert(String::from("application/atomcat+xml"), String::from("Atom Publishing Protocol"));
    map.insert(String::from("application/atomsvc+xml"), String::from("Atom Publishing Protocol Service Document"));
    map.insert(String::from("application/atom+xml"), String::from("Atom Syndication Format"));
    map.insert(String::from("application/pkix-attr-cert"), String::from("Attribute Certificate"));
    map.insert(String::from("audio/x-aiff"), String::from("Audio Interchange File Format"));
    map.insert(String::from("video/x-msvideo"), String::from("Audio Video Interleave (AVI)"));
    map.insert(String::from("application/vnd.audiograph"), String::from("Audiograph"));
    map.insert(String::from("image/vnd.dxf"), String::from("AutoCAD DXF"));
    map.insert(String::from("model/vnd.dwf"), String::from("Autodesk Design Web Format (DWF)"));
    map.insert(String::from("text/plain-bas"), String::from("BAS Partitur Format"));
    map.insert(String::from("application/x-bcpio"), String::from("Binary CPIO Archive"));
    map.insert(String::from("application/octet-stream"), String::from("Binary Data"));
    map.insert(String::from("image/bmp"), String::from("Bitmap Image File"));
    map.insert(String::from("application/x-bittorrent"), String::from("BitTorrent"));
    map.insert(String::from("application/vnd.rim.cod"), String::from("Blackberry COD File"));
    map.insert(String::from("application/vnd.blueice.multipass"), String::from("Blueice Research Multipass"));
    map.insert(String::from("application/vnd.bmi"), String::from("BMI Drawing Data Interchange"));
    map.insert(String::from("application/x-sh"), String::from("Bourne Shell Script"));
    map.insert(String::from("image/prs.btif"), String::from("BTIF"));
    map.insert(String::from("application/vnd.businessobjects"), String::from("BusinessObjects"));
    map.insert(String::from("application/x-bzip"), String::from("Bzip Archive"));
    map.insert(String::from("application/x-bzip2"), String::from("Bzip2 Archive"));
    map.insert(String::from("application/x-csh"), String::from("C Shell Script"));
    map.insert(String::from("text/x-c"), String::from("C Source File"));
    map.insert(String::from("application/vnd.chemdraw+xml"), String::from("CambridgeSoft Chem Draw"));
    map.insert(String::from("text/css"), String::from("Cascading Style Sheets (CSS)"));
    map.insert(String::from("chemical/x-cdx"), String::from("ChemDraw eXchange file"));
    map.insert(String::from("chemical/x-cml"), String::from("Chemical Markup Language"));
    map.insert(String::from("chemical/x-csml"), String::from("Chemical Style Markup Language"));
    map.insert(String::from("application/vnd.contact.cmsg"), String::from("CIM Database"));
    map.insert(String::from("application/vnd.claymore"), String::from("Claymore Data Files"));
    map.insert(String::from("application/vnd.clonk.c4group"), String::from("Clonk Game"));
    map.insert(String::from("image/vnd.dvb.subtitle"), String::from("Close Captioning - Subtitle"));
    map.insert(String::from("application/cdmi-capability"), String::from("Cloud Data Management Interface (CDMI) - Capability"));
    map.insert(String::from("application/cdmi-container"), String::from("Cloud Data Management Interface (CDMI) - Contaimer"));
    map.insert(String::from("application/cdmi-domain"), String::from("Cloud Data Management Interface (CDMI) - Domain"));
    map.insert(String::from("application/cdmi-object"), String::from("Cloud Data Management Interface (CDMI) - Object"));
    map.insert(String::from("application/cdmi-queue"), String::from("Cloud Data Management Interface (CDMI) - Queue"));
    map.insert(String::from("application/vnd.cluetrust.cartomobile-config"), String::from("ClueTrust CartoMobile - Config"));
    map.insert(String::from("application/vnd.cluetrust.cartomobile-config-pkg"), String::from("ClueTrust CartoMobile - Config Package"));
    map.insert(String::from("image/x-cmu-raster"), String::from("CMU Image"));
    map.insert(String::from("model/vnd.collada+xml"), String::from("COLLADA"));
    map.insert(String::from("text/csv"), String::from("Comma-Seperated Values"));
    map.insert(String::from("application/mac-compactpro"), String::from("Compact Pro"));
    map.insert(String::from("application/vnd.wap.wmlc"), String::from("Compiled Wireless Markup Language (WMLC)"));
    map.insert(String::from("image/cgm"), String::from("Computer Graphics Metafile"));
    map.insert(String::from("x-conference/x-cooltalk"), String::from("CoolTalk"));
    map.insert(String::from("image/x-cmx"), String::from("Corel Metafile Exchange (CMX)"));
    map.insert(String::from("application/vnd.xara"), String::from("CorelXARA"));
    map.insert(String::from("application/vnd.cosmocaller"), String::from("CosmoCaller"));
    map.insert(String::from("application/x-cpio"), String::from("CPIO Archive"));
    map.insert(String::from("application/vnd.crick.clicker"), String::from("CrickSoftware - Clicker"));
    map.insert(String::from("application/vnd.crick.clicker.keyboard"), String::from("CrickSoftware - Clicker - Keyboard"));
    map.insert(String::from("application/vnd.crick.clicker.palette"), String::from("CrickSoftware - Clicker - Palette"));
    map.insert(String::from("application/vnd.crick.clicker.template"), String::from("CrickSoftware - Clicker - Template"));
    map.insert(String::from("application/vnd.crick.clicker.wordbank"), String::from("CrickSoftware - Clicker - Wordbank"));
    map.insert(String::from("application/vnd.criticaltools.wbs+xml"), String::from("Critical Tools - PERT Chart EXPERT"));
    map.insert(String::from("application/vnd.rig.cryptonote"), String::from("CryptoNote"));
    map.insert(String::from("chemical/x-cif"), String::from("Crystallographic Interchange Format"));
    map.insert(String::from("chemical/x-cmdf"), String::from("CrystalMaker Data Format"));
    map.insert(String::from("application/cu-seeme"), String::from("CU-SeeMe"));
    map.insert(String::from("application/prs.cww"), String::from("CU-Writer"));
    map.insert(String::from("text/vnd.curl"), String::from("Curl - Applet"));
    map.insert(String::from("text/vnd.curl.dcurl"), String::from("Curl - Detached Applet"));
    map.insert(String::from("text/vnd.curl.mcurl"), String::from("Curl - Manifest File"));
    map.insert(String::from("text/vnd.curl.scurl"), String::from("Curl - Source Code"));
    map.insert(String::from("application/vnd.curl.car"), String::from("CURL Applet"));
    map.insert(String::from("application/vnd.curl.pcurl"), String::from("CURL Applet"));
    map.insert(String::from("application/vnd.yellowriver-custom-menu"), String::from("CustomMenu"));
    map.insert(String::from("application/dssc+der"), String::from("Data Structure for the Security Suitability of Cryptographic Algorithms"));
    map.insert(String::from("application/dssc+xml"), String::from("Data Structure for the Security Suitability of Cryptographic Algorithms"));
    map.insert(String::from("application/x-debian-package"), String::from("Debian Package"));
    map.insert(String::from("audio/vnd.dece.audio"), String::from("DECE Audio"));
    map.insert(String::from("image/vnd.dece.graphic"), String::from("DECE Graphic"));
    map.insert(String::from("video/vnd.dece.hd"), String::from("DECE High Definition Video"));
    map.insert(String::from("video/vnd.dece.mobile"), String::from("DECE Mobile Video"));
    map.insert(String::from("video/vnd.uvvu.mp4"), String::from("DECE MP4"));
    map.insert(String::from("video/vnd.dece.pd"), String::from("DECE PD Video"));
    map.insert(String::from("video/vnd.dece.sd"), String::from("DECE SD Video"));
    map.insert(String::from("video/vnd.dece.video"), String::from("DECE Video"));
    map.insert(String::from("application/x-dvi"), String::from("Device Independent File Format (DVI)"));
    map.insert(String::from("application/vnd.fdsn.seed"), String::from("Digital Siesmograph Networks - SEED Datafiles"));
    map.insert(String::from("application/x-dtbook+xml"), String::from("Digital Talking Book"));
    map.insert(String::from("application/x-dtbresource+xml"), String::from("Digital Talking Book - Resource File"));
    map.insert(String::from("application/vnd.dvb.ait"), String::from("Digital Video Broadcasting"));
    map.insert(String::from("application/vnd.dvb.service"), String::from("Digital Video Broadcasting"));
    map.insert(String::from("audio/vnd.digital-winds"), String::from("Digital Winds Music"));
    map.insert(String::from("image/vnd.djvu"), String::from("DjVu"));
    map.insert(String::from("application/xml-dtd"), String::from("Document Type Definition"));
    map.insert(String::from("application/vnd.dolby.mlp"), String::from("Dolby Meridian Lossless Packing"));
    map.insert(String::from("application/x-doom"), String::from("Doom Video Game"));
    map.insert(String::from("application/vnd.dpgraph"), String::from("DPGraph"));
    map.insert(String::from("audio/vnd.dra"), String::from("DRA Audio"));
    map.insert(String::from("application/vnd.dreamfactory"), String::from("DreamFactory"));
    map.insert(String::from("audio/vnd.dts"), String::from("DTS Audio"));
    map.insert(String::from("audio/vnd.dts.hd"), String::from("DTS High Definition Audio"));
    map.insert(String::from("image/vnd.dwg"), String::from("DWG Drawing"));
    map.insert(String::from("application/vnd.dynageo"), String::from("DynaGeo"));
    map.insert(String::from("application/ecmascript"), String::from("ECMAScript"));
    map.insert(String::from("application/vnd.ecowin.chart"), String::from("EcoWin Chart"));
    map.insert(String::from("image/vnd.fujixerox.edmics-mmr"), String::from("EDMICS 2000"));
    map.insert(String::from("image/vnd.fujixerox.edmics-rlc"), String::from("EDMICS 2000"));
    map.insert(String::from("application/exi"), String::from("Efficient XML Interchange"));
    map.insert(String::from("application/vnd.proteus.magazine"), String::from("EFI Proteus"));
    map.insert(String::from("application/epub+zip"), String::from("Electronic Publication"));
    map.insert(String::from("message/rfc822"), String::from("Email Message"));
    map.insert(String::from("application/vnd.enliven"), String::from("Enliven Viewer"));
    map.insert(String::from("application/vnd.is-xpr"), String::from("Express by Infoseek"));
    map.insert(String::from("image/vnd.xiff"), String::from("eXtended Image File Format (XIFF)"));
    map.insert(String::from("application/vnd.xfdl"), String::from("Extensible Forms Description Language"));
    map.insert(String::from("application/emma+xml"), String::from("Extensible MultiModal Annotation"));
    map.insert(String::from("application/vnd.ezpix-album"), String::from("EZPix Secure Photo Album"));
    map.insert(String::from("application/vnd.ezpix-package"), String::from("EZPix Secure Photo Album"));
    map.insert(String::from("image/vnd.fst"), String::from("FAST Search & Transfer ASA"));
    map.insert(String::from("video/vnd.fvt"), String::from("FAST Search & Transfer ASA"));
    map.insert(String::from("image/vnd.fastbidsheet"), String::from("FastBid Sheet"));
    map.insert(String::from("application/vnd.denovo.fcselayout-link"), String::from("FCS Express Layout Link"));
    map.insert(String::from("video/x-f4v"), String::from("Flash Video"));
    map.insert(String::from("video/x-flv"), String::from("Flash Video"));
    map.insert(String::from("image/vnd.fpx"), String::from("FlashPix"));
    map.insert(String::from("image/vnd.net-fpx"), String::from("FlashPix"));
    map.insert(String::from("text/vnd.fmi.flexstor"), String::from("FLEXSTOR"));
    map.insert(String::from("video/x-fli"), String::from("FLI/FLC Animation Format"));
    map.insert(String::from("application/vnd.fluxtime.clip"), String::from("FluxTime Clip"));
    map.insert(String::from("application/vnd.fdf"), String::from("Forms Data Format"));
    map.insert(String::from("text/x-fortran"), String::from("Fortran Source File"));
    map.insert(String::from("application/vnd.mif"), String::from("FrameMaker Interchange Format"));
    map.insert(String::from("application/vnd.framemaker"), String::from("FrameMaker Normal Format"));
    map.insert(String::from("image/x-freehand"), String::from("FreeHand MX"));
    map.insert(String::from("application/vnd.fsc.weblaunch"), String::from("Friendly Software Corporation"));
    map.insert(String::from("application/vnd.frogans.fnc"), String::from("Frogans Player"));
    map.insert(String::from("application/vnd.frogans.ltf"), String::from("Frogans Player"));
    map.insert(String::from("application/vnd.fujixerox.ddd"), String::from("Fujitsu - Xerox 2D CAD Data"));
    map.insert(String::from("application/vnd.fujixerox.docuworks"), String::from("Fujitsu - Xerox DocuWorks"));
    map.insert(String::from("application/vnd.fujixerox.docuworks.binder"), String::from("Fujitsu - Xerox DocuWorks Binder"));
    map.insert(String::from("application/vnd.fujitsu.oasys"), String::from("Fujitsu Oasys"));
    map.insert(String::from("application/vnd.fujitsu.oasys2"), String::from("Fujitsu Oasys"));
    map.insert(String::from("application/vnd.fujitsu.oasys3"), String::from("Fujitsu Oasys"));
    map.insert(String::from("application/vnd.fujitsu.oasysgp"), String::from("Fujitsu Oasys"));
    map.insert(String::from("application/vnd.fujitsu.oasysprs"), String::from("Fujitsu Oasys"));
    map.insert(String::from("application/x-futuresplash"), String::from("FutureSplash Animator"));
    map.insert(String::from("application/vnd.fuzzysheet"), String::from("FuzzySheet"));
    map.insert(String::from("image/g3fax"), String::from("G3 Fax Image"));
    map.insert(String::from("application/vnd.gmx"), String::from("GameMaker ActiveX"));
    map.insert(String::from("model/vnd.gtw"), String::from("Gen-Trix Studio"));
    map.insert(String::from("application/vnd.genomatix.tuxedo"), String::from("Genomatix Tuxedo Framework"));
    map.insert(String::from("application/vnd.geogebra.file"), String::from("GeoGebra"));
    map.insert(String::from("application/vnd.geogebra.tool"), String::from("GeoGebra"));
    map.insert(String::from("model/vnd.gdl"), String::from("Geometric Description Language (GDL)"));
    map.insert(String::from("application/vnd.geometry-explorer"), String::from("GeoMetry Explorer"));
    map.insert(String::from("application/vnd.geonext"), String::from("GEONExT and JSXGraph"));
    map.insert(String::from("application/vnd.geoplan"), String::from("GeoplanW"));
    map.insert(String::from("application/vnd.geospace"), String::from("GeospacW"));
    map.insert(String::from("application/x-font-ghostscript"), String::from("Ghostscript Font"));
    map.insert(String::from("application/x-font-bdf"), String::from("Glyph Bitmap Distribution Format"));
    map.insert(String::from("application/x-gtar"), String::from("GNU Tar Files"));
    map.insert(String::from("application/x-texinfo"), String::from("GNU Texinfo Document"));
    map.insert(String::from("application/x-gnumeric"), String::from("Gnumeric"));
    map.insert(String::from("application/vnd.google-earth.kml+xml"), String::from("Google Earth - KML"));
    map.insert(String::from("application/vnd.google-earth.kmz"), String::from("Google Earth - Zipped KML"));
    map.insert(String::from("application/gpx+xml"), String::from("GPS eXchange Format"));
    map.insert(String::from("application/vnd.grafeq"), String::from("GrafEq"));
    map.insert(String::from("image/gif"), String::from("Graphics Interchange Format"));
    map.insert(String::from("text/vnd.graphviz"), String::from("Graphviz"));
    map.insert(String::from("application/vnd.groove-account"), String::from("Groove - Account"));
    map.insert(String::from("application/vnd.groove-help"), String::from("Groove - Help"));
    map.insert(String::from("application/vnd.groove-identity-message"), String::from("Groove - Identity Message"));
    map.insert(String::from("application/vnd.groove-injector"), String::from("Groove - Injector"));
    map.insert(String::from("application/vnd.groove-tool-message"), String::from("Groove - Tool Message"));
    map.insert(String::from("application/vnd.groove-tool-template"), String::from("Groove - Tool Template"));
    map.insert(String::from("application/vnd.groove-vcard"), String::from("Groove - Vcard"));
    map.insert(String::from("video/h261"), String::from("H.261"));
    map.insert(String::from("video/h263"), String::from("H.263"));
    map.insert(String::from("video/h264"), String::from("H.264"));
    map.insert(String::from("application/vnd.hp-hpid"), String::from("Hewlett Packard Instant Delivery"));
    map.insert(String::from("application/vnd.hp-hps"), String::from("Hewlett-Packard's WebPrintSmart"));
    map.insert(String::from("application/x-hdf"), String::from("Hierarchical Data Format"));
    map.insert(String::from("audio/vnd.rip"), String::from("Hit'n'Mix"));
    map.insert(String::from("application/vnd.hbci"), String::from("Homebanking Computer Interface (HBCI)"));
    map.insert(String::from("application/vnd.hp-jlyt"), String::from("HP Indigo Digital Press - Job Layout Languate"));
    map.insert(String::from("application/vnd.hp-pcl"), String::from("HP Printer Command Language"));
    map.insert(String::from("application/vnd.hp-hpgl"), String::from("HP-GL/2 and HP RTL"));
    map.insert(String::from("application/vnd.yamaha.hv-script"), String::from("HV Script"));
    map.insert(String::from("application/vnd.yamaha.hv-dic"), String::from("HV Voice Dictionary"));
    map.insert(String::from("application/vnd.yamaha.hv-voice"), String::from("HV Voice Parameter"));
    map.insert(String::from("application/vnd.hydrostatix.sof-data"), String::from("Hydrostatix Master Suite"));
    map.insert(String::from("application/hyperstudio"), String::from("Hyperstudio"));
    map.insert(String::from("application/vnd.hal+xml"), String::from("Hypertext Application Language"));
    map.insert(String::from("text/html"), String::from("HyperText Markup Language (HTML)"));
    map.insert(String::from("application/vnd.ibm.rights-management"), String::from("IBM DB2 Rights Manager"));
    map.insert(String::from("application/vnd.ibm.secure-container"), String::from("IBM Electronic Media Management System - Secure Container"));
    map.insert(String::from("text/calendar"), String::from("iCalendar"));
    map.insert(String::from("application/vnd.iccprofile"), String::from("ICC profile"));
    map.insert(String::from("image/x-icon"), String::from("Icon Image"));
    map.insert(String::from("application/vnd.igloader"), String::from("igLoader"));
    map.insert(String::from("image/ief"), String::from("Image Exchange Format"));
    map.insert(String::from("application/vnd.immervision-ivp"), String::from("ImmerVision PURE Players"));
    map.insert(String::from("application/vnd.immervision-ivu"), String::from("ImmerVision PURE Players"));
    map.insert(String::from("application/reginfo+xml"), String::from("IMS Networks"));
    map.insert(String::from("text/vnd.in3d.3dml"), String::from("In3D - 3DML"));
    map.insert(String::from("text/vnd.in3d.spot"), String::from("In3D - 3DML"));
    map.insert(String::from("model/iges"), String::from("Initial Graphics Exchange Specification (IGES)"));
    map.insert(String::from("application/vnd.intergeo"), String::from("Interactive Geometry Software"));
    map.insert(String::from("application/vnd.cinderella"), String::from("Interactive Geometry Software Cinderella"));
    map.insert(String::from("application/vnd.intercon.formnet"), String::from("Intercon FormNet"));
    map.insert(String::from("application/vnd.isac.fcs"), String::from("International Society for Advancement of Cytometry"));
    map.insert(String::from("application/ipfix"), String::from("Internet Protocol Flow Information Export"));
    map.insert(String::from("application/pkix-cert"), String::from("Internet Public Key Infrastructure - Certificate"));
    map.insert(String::from("application/pkixcmp"), String::from("Internet Public Key Infrastructure - Certificate Management Protocole"));
    map.insert(String::from("application/pkix-crl"), String::from("Internet Public Key Infrastructure - Certificate Revocation Lists"));
    map.insert(String::from("application/pkix-pkipath"), String::from("Internet Public Key Infrastructure - Certification Path"));
    map.insert(String::from("application/vnd.insors.igm"), String::from("IOCOM Visimeet"));
    map.insert(String::from("application/vnd.ipunplugged.rcprofile"), String::from("IP Unplugged Roaming Client"));
    map.insert(String::from("application/vnd.irepository.package+xml"), String::from("iRepository / Lucidoc Editor"));
    map.insert(String::from("text/vnd.sun.j2me.app-descriptor"), String::from("J2ME App Descriptor"));
    map.insert(String::from("application/java-archive"), String::from("Java Archive"));
    map.insert(String::from("application/java-vm"), String::from("Java Bytecode File"));
    map.insert(String::from("application/x-java-jnlp-file"), String::from("Java Network Launching Protocol"));
    map.insert(String::from("application/java-serialized-object"), String::from("Java Serialized Object"));
    map.insert(String::from("text/x-java-source"), String::from("Java Source File"));
    map.insert(String::from("application/javascript"), String::from("JavaScript"));
    map.insert(String::from("application/json"), String::from("JavaScript Object Notation (JSON)"));
    map.insert(String::from("application/vnd.joost.joda-archive"), String::from("Joda Archive"));
    map.insert(String::from("video/jpm"), String::from("JPEG 2000 Compound Image File Format"));
    map.insert(String::from("image/jpeg"), String::from("JPEG Image"));
    map.insert(String::from("image/x-citrix-jpeg"), String::from("JPEG Image (Citrix client)"));
    map.insert(String::from("image/pjpeg"), String::from("JPEG Image (Progressive)"));
    map.insert(String::from("video/jpeg"), String::from("JPGVideo"));
    map.insert(String::from("application/vnd.kahootz"), String::from("Kahootz"));
    map.insert(String::from("application/vnd.chipnuts.karaoke-mmd"), String::from("Karaoke on Chipnuts Chipsets"));
    map.insert(String::from("application/vnd.kde.karbon"), String::from("KDE KOffice Office Suite - Karbon"));
    map.insert(String::from("application/vnd.kde.kchart"), String::from("KDE KOffice Office Suite - KChart"));
    map.insert(String::from("application/vnd.kde.kformula"), String::from("KDE KOffice Office Suite - Kformula"));
    map.insert(String::from("application/vnd.kde.kivio"), String::from("KDE KOffice Office Suite - Kivio"));
    map.insert(String::from("application/vnd.kde.kontour"), String::from("KDE KOffice Office Suite - Kontour"));
    map.insert(String::from("application/vnd.kde.kpresenter"), String::from("KDE KOffice Office Suite - Kpresenter"));
    map.insert(String::from("application/vnd.kde.kspread"), String::from("KDE KOffice Office Suite - Kspread"));
    map.insert(String::from("application/vnd.kde.kword"), String::from("KDE KOffice Office Suite - Kword"));
    map.insert(String::from("application/vnd.kenameaapp"), String::from("Kenamea App"));
    map.insert(String::from("application/vnd.kidspiration"), String::from("Kidspiration"));
    map.insert(String::from("application/vnd.kinar"), String::from("Kinar Applications"));
    map.insert(String::from("application/vnd.kodak-descriptor"), String::from("Kodak Storyshare"));
    map.insert(String::from("application/vnd.las.las+xml"), String::from("Laser App Enterprise"));
    map.insert(String::from("application/x-latex"), String::from("LaTeX"));
    map.insert(String::from("application/vnd.llamagraphics.life-balance.desktop"), String::from("Life Balance - Desktop Edition"));
    map.insert(String::from("application/vnd.llamagraphics.life-balance.exchange+xml"), String::from("Life Balance - Exchange Format"));
    map.insert(String::from("application/vnd.jam"), String::from("Lightspeed Audio Lab"));
    map.insert(String::from("application/vnd.lotus-1-2-3"), String::from("Lotus 1-2-3"));
    map.insert(String::from("application/vnd.lotus-approach"), String::from("Lotus Approach"));
    map.insert(String::from("application/vnd.lotus-freelance"), String::from("Lotus Freelance"));
    map.insert(String::from("application/vnd.lotus-notes"), String::from("Lotus Notes"));
    map.insert(String::from("application/vnd.lotus-organizer"), String::from("Lotus Organizer"));
    map.insert(String::from("application/vnd.lotus-screencam"), String::from("Lotus Screencam"));
    map.insert(String::from("application/vnd.lotus-wordpro"), String::from("Lotus Wordpro"));
    map.insert(String::from("audio/vnd.lucent.voice"), String::from("Lucent Voice"));
    map.insert(String::from("audio/x-mpegurl"), String::from("M3U (Multimedia Playlist)"));
    map.insert(String::from("video/x-m4v"), String::from("M4v"));
    map.insert(String::from("application/mac-binhex40"), String::from("Macintosh BinHex 4.0"));
    map.insert(String::from("application/vnd.macports.portpkg"), String::from("MacPorts Port System"));
    map.insert(String::from("application/vnd.osgeo.mapguide.package"), String::from("MapGuide DBXML"));
    map.insert(String::from("application/marc"), String::from("MARC Formats"));
    map.insert(String::from("application/marcxml+xml"), String::from("MARC21 XML Schema"));
    map.insert(String::from("application/mxf"), String::from("Material Exchange Format"));
    map.insert(String::from("application/vnd.wolfram.player"), String::from("Mathematica Notebook Player"));
    map.insert(String::from("application/mathematica"), String::from("Mathematica Notebooks"));
    map.insert(String::from("application/mathml+xml"), String::from("Mathematical Markup Language"));
    map.insert(String::from("application/mbox"), String::from("Mbox database files"));
    map.insert(String::from("application/vnd.medcalcdata"), String::from("MedCalc"));
    map.insert(String::from("application/mediaservercontrol+xml"), String::from("Media Server Control Markup Language"));
    map.insert(String::from("application/vnd.mediastation.cdkey"), String::from("MediaRemote"));
    map.insert(String::from("application/vnd.mfer"), String::from("Medical Waveform Encoding Format"));
    map.insert(String::from("application/vnd.mfmp"), String::from("Melody Format for Mobile Platform"));
    map.insert(String::from("model/mesh"), String::from("Mesh Data Type"));
    map.insert(String::from("application/mads+xml"), String::from("Metadata Authority Description Schema"));
    map.insert(String::from("application/mets+xml"), String::from("Metadata Encoding and Transmission Standard"));
    map.insert(String::from("application/mods+xml"), String::from("Metadata Object Description Schema"));
    map.insert(String::from("application/metalink4+xml"), String::from("Metalink"));
    map.insert(String::from("application/vnd.mcd"), String::from("Micro CADAM Helix D&D"));
    map.insert(String::from("application/vnd.micrografx.flo"), String::from("Micrografx"));
    map.insert(String::from("application/vnd.micrografx.igx"), String::from("Micrografx iGrafx Professional"));
    map.insert(String::from("application/vnd.eszigno3+xml"), String::from("MICROSEC e-Szignï¿½"));
    map.insert(String::from("application/x-msaccess"), String::from("Microsoft Access"));
    map.insert(String::from("video/x-ms-asf"), String::from("Microsoft Advanced Systems Format (ASF)"));
    map.insert(String::from("application/x-msdownload"), String::from("Microsoft Application"));
    map.insert(String::from("application/vnd.ms-artgalry"), String::from("Microsoft Artgalry"));
    map.insert(String::from("application/vnd.ms-cab-compressed"), String::from("Microsoft Cabinet File"));
    map.insert(String::from("application/vnd.ms-ims"), String::from("Microsoft Class Server"));
    map.insert(String::from("application/x-ms-application"), String::from("Microsoft ClickOnce"));
    map.insert(String::from("application/x-msclip"), String::from("Microsoft Clipboard Clip"));
    map.insert(String::from("image/vnd.ms-modi"), String::from("Microsoft Document Imaging Format"));
    map.insert(String::from("application/vnd.ms-fontobject"), String::from("Microsoft Embedded OpenType"));
    map.insert(String::from("application/vnd.ms-excel"), String::from("Microsoft Excel"));
    map.insert(String::from("application/vnd.ms-excel.addin.macroenabled.12"), String::from("Microsoft Excel - Add-In File"));
    map.insert(String::from("application/vnd.ms-excel.sheet.binary.macroenabled.12"), String::from("Microsoft Excel - Binary Workbook"));
    map.insert(String::from("application/vnd.ms-excel.template.macroenabled.12"), String::from("Microsoft Excel - Macro-Enabled Template File"));
    map.insert(String::from("application/vnd.ms-excel.sheet.macroenabled.12"), String::from("Microsoft Excel - Macro-Enabled Workbook"));
    map.insert(String::from("application/vnd.ms-htmlhelp"), String::from("Microsoft Html Help File"));
    map.insert(String::from("application/x-mscardfile"), String::from("Microsoft Information Card"));
    map.insert(String::from("application/vnd.ms-lrm"), String::from("Microsoft Learning Resource Module"));
    map.insert(String::from("application/x-msmediaview"), String::from("Microsoft MediaView"));
    map.insert(String::from("application/x-msmoney"), String::from("Microsoft Money"));
    map.insert(String::from("application/vnd.openxmlformats-officedocument.presentationml.presentation"), String::from("Microsoft Office - OOXML - Presentation"));
    map.insert(String::from("application/vnd.openxmlformats-officedocument.presentationml.slide"), String::from("Microsoft Office - OOXML - Presentation (Slide)"));
    map.insert(String::from("application/vnd.openxmlformats-officedocument.presentationml.slideshow"), String::from("Microsoft Office - OOXML - Presentation (Slideshow)"));
    map.insert(String::from("application/vnd.openxmlformats-officedocument.presentationml.template"), String::from("Microsoft Office - OOXML - Presentation Template"));
    map.insert(String::from("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"), String::from("Microsoft Office - OOXML - Spreadsheet"));
    map.insert(String::from("application/vnd.openxmlformats-officedocument.spreadsheetml.template"), String::from("Microsoft Office - OOXML - Spreadsheet Template"));
    map.insert(String::from("application/vnd.openxmlformats-officedocument.wordprocessingml.document"), String::from("Microsoft Office - OOXML - Word Document"));
    map.insert(String::from("application/vnd.openxmlformats-officedocument.wordprocessingml.template"), String::from("Microsoft Office - OOXML - Word Document Template"));
    map.insert(String::from("application/x-msbinder"), String::from("Microsoft Office Binder"));
    map.insert(String::from("application/vnd.ms-officetheme"), String::from("Microsoft Office System Release Theme"));
    map.insert(String::from("application/onenote"), String::from("Microsoft OneNote"));
    map.insert(String::from("audio/vnd.ms-playready.media.pya"), String::from("Microsoft PlayReady Ecosystem"));
    map.insert(String::from("video/vnd.ms-playready.media.pyv"), String::from("Microsoft PlayReady Ecosystem Video"));
    map.insert(String::from("application/vnd.ms-powerpoint"), String::from("Microsoft PowerPoint"));
    map.insert(String::from("application/vnd.ms-powerpoint.addin.macroenabled.12"), String::from("Microsoft PowerPoint - Add-in file"));
    map.insert(String::from("application/vnd.ms-powerpoint.slide.macroenabled.12"), String::from("Microsoft PowerPoint - Macro-Enabled Open XML Slide"));
    map.insert(String::from("application/vnd.ms-powerpoint.presentation.macroenabled.12"), String::from("Microsoft PowerPoint - Macro-Enabled Presentation File"));
    map.insert(String::from("application/vnd.ms-powerpoint.slideshow.macroenabled.12"), String::from("Microsoft PowerPoint - Macro-Enabled Slide Show File"));
    map.insert(String::from("application/vnd.ms-powerpoint.template.macroenabled.12"), String::from("Microsoft PowerPoint - Macro-Enabled Template File"));
    map.insert(String::from("application/vnd.ms-project"), String::from("Microsoft Project"));
    map.insert(String::from("application/x-mspublisher"), String::from("Microsoft Publisher"));
    map.insert(String::from("application/x-msschedule"), String::from("Microsoft Schedule+"));
    map.insert(String::from("application/x-silverlight-app"), String::from("Microsoft Silverlight"));
    map.insert(String::from("application/vnd.ms-pki.stl"), String::from("Microsoft Trust UI Provider - Certificate Trust Link"));
    map.insert(String::from("application/vnd.ms-pki.seccat"), String::from("Microsoft Trust UI Provider - Security Catalog"));
    map.insert(String::from("application/vnd.visio"), String::from("Microsoft Visio"));
    map.insert(String::from("application/vnd.visio2013"), String::from("Microsoft Visio 2013"));
    map.insert(String::from("video/x-ms-wm"), String::from("Microsoft Windows Media"));
    map.insert(String::from("audio/x-ms-wma"), String::from("Microsoft Windows Media Audio"));
    map.insert(String::from("audio/x-ms-wax"), String::from("Microsoft Windows Media Audio Redirector"));
    map.insert(String::from("video/x-ms-wmx"), String::from("Microsoft Windows Media Audio/Video Playlist"));
    map.insert(String::from("application/x-ms-wmd"), String::from("Microsoft Windows Media Player Download Package"));
    map.insert(String::from("application/vnd.ms-wpl"), String::from("Microsoft Windows Media Player Playlist"));
    map.insert(String::from("application/x-ms-wmz"), String::from("Microsoft Windows Media Player Skin Package"));
    map.insert(String::from("video/x-ms-wmv"), String::from("Microsoft Windows Media Video"));
    map.insert(String::from("video/x-ms-wvx"), String::from("Microsoft Windows Media Video Playlist"));
    map.insert(String::from("application/x-msmetafile"), String::from("Microsoft Windows Metafile"));
    map.insert(String::from("application/x-msterminal"), String::from("Microsoft Windows Terminal Services"));
    map.insert(String::from("application/msword"), String::from("Microsoft Word"));
    map.insert(String::from("application/vnd.ms-word.document.macroenabled.12"), String::from("Microsoft Word - Macro-Enabled Document"));
    map.insert(String::from("application/vnd.ms-word.template.macroenabled.12"), String::from("Microsoft Word - Macro-Enabled Template"));
    map.insert(String::from("application/x-mswrite"), String::from("Microsoft Wordpad"));
    map.insert(String::from("application/vnd.ms-works"), String::from("Microsoft Works"));
    map.insert(String::from("application/x-ms-xbap"), String::from("Microsoft XAML Browser Application"));
    map.insert(String::from("application/vnd.ms-xpsdocument"), String::from("Microsoft XML Paper Specification"));
    map.insert(String::from("audio/midi"), String::from("MIDI - Musical Instrument Digital Interface"));
    map.insert(String::from("application/vnd.ibm.minipay"), String::from("MiniPay"));
    map.insert(String::from("application/vnd.ibm.modcap"), String::from("MO:DCA-P"));
    map.insert(String::from("application/vnd.jcp.javame.midlet-rms"), String::from("Mobile Information Device Profile"));
    map.insert(String::from("application/vnd.tmobile-livetv"), String::from("MobileTV"));
    map.insert(String::from("application/x-mobipocket-ebook"), String::from("Mobipocket"));
    map.insert(String::from("application/vnd.mobius.mbk"), String::from("Mobius Management Systems - Basket file"));
    map.insert(String::from("application/vnd.mobius.dis"), String::from("Mobius Management Systems - Distribution Database"));
    map.insert(String::from("application/vnd.mobius.plc"), String::from("Mobius Management Systems - Policy Definition Language File"));
    map.insert(String::from("application/vnd.mobius.mqy"), String::from("Mobius Management Systems - Query File"));
    map.insert(String::from("application/vnd.mobius.msl"), String::from("Mobius Management Systems - Script Language"));
    map.insert(String::from("application/vnd.mobius.txf"), String::from("Mobius Management Systems - Topic Index File"));
    map.insert(String::from("application/vnd.mobius.daf"), String::from("Mobius Management Systems - UniversalArchive"));
    map.insert(String::from("text/vnd.fly"), String::from("mod_fly / fly.cgi"));
    map.insert(String::from("application/vnd.mophun.certificate"), String::from("Mophun Certificate"));
    map.insert(String::from("application/vnd.mophun.application"), String::from("Mophun VM"));
    map.insert(String::from("video/mj2"), String::from("Motion JPEG 2000"));
    map.insert(String::from("audio/mpeg"), String::from("MPEG Audio"));
    map.insert(String::from("video/vnd.mpegurl"), String::from("MPEG Url"));
    map.insert(String::from("video/mpeg"), String::from("MPEG Video"));
    map.insert(String::from("application/mp21"), String::from("MPEG-21"));
    map.insert(String::from("audio/mp4"), String::from("MPEG-4 Audio"));
    map.insert(String::from("video/mp4"), String::from("MPEG-4 Video"));
    map.insert(String::from("application/mp4"), String::from("MPEG4"));
    map.insert(String::from("application/vnd.apple.mpegurl"), String::from("Multimedia Playlist Unicode"));
    map.insert(String::from("application/vnd.musician"), String::from("MUsical Score Interpreted Code Invented for the ASCII designation of Notation"));
    map.insert(String::from("application/vnd.muvee.style"), String::from("Muvee Automatic Video Editing"));
    map.insert(String::from("application/xv+xml"), String::from("MXML"));
    map.insert(String::from("application/vnd.nokia.n-gage.data"), String::from("N-Gage Game Data"));
    map.insert(String::from("application/vnd.nokia.n-gage.symbian.install"), String::from("N-Gage Game Installer"));
    map.insert(String::from("application/x-dtbncx+xml"), String::from("Navigation Control file for XML (for ePub)"));
    map.insert(String::from("application/x-netcdf"), String::from("Network Common Data Form (NetCDF)"));
    map.insert(String::from("application/vnd.neurolanguage.nlu"), String::from("neuroLanguage"));
    map.insert(String::from("application/vnd.dna"), String::from("New Moon Liftoff/DNA"));
    map.insert(String::from("application/vnd.noblenet-directory"), String::from("NobleNet Directory"));
    map.insert(String::from("application/vnd.noblenet-sealer"), String::from("NobleNet Sealer"));
    map.insert(String::from("application/vnd.noblenet-web"), String::from("NobleNet Web"));
    map.insert(String::from("application/vnd.nokia.radio-preset"), String::from("Nokia Radio Application - Preset"));
    map.insert(String::from("application/vnd.nokia.radio-presets"), String::from("Nokia Radio Application - Preset"));
    map.insert(String::from("text/n3"), String::from("Notation3"));
    map.insert(String::from("application/vnd.novadigm.edm"), String::from("Novadigm's RADIA and EDM products"));
    map.insert(String::from("application/vnd.novadigm.edx"), String::from("Novadigm's RADIA and EDM products"));
    map.insert(String::from("application/vnd.novadigm.ext"), String::from("Novadigm's RADIA and EDM products"));
    map.insert(String::from("application/vnd.flographit"), String::from("NpGraphIt"));
    map.insert(String::from("audio/vnd.nuera.ecelp4800"), String::from("Nuera ECELP 4800"));
    map.insert(String::from("audio/vnd.nuera.ecelp7470"), String::from("Nuera ECELP 7470"));
    map.insert(String::from("audio/vnd.nuera.ecelp9600"), String::from("Nuera ECELP 9600"));
    map.insert(String::from("application/oda"), String::from("Office Document Architecture"));
    map.insert(String::from("application/ogg"), String::from("Ogg"));
    map.insert(String::from("audio/ogg"), String::from("Ogg Audio"));
    map.insert(String::from("video/ogg"), String::from("Ogg Video"));
    map.insert(String::from("application/vnd.oma.dd2+xml"), String::from("OMA Download Agents"));
    map.insert(String::from("application/vnd.oasis.opendocument.text-web"), String::from("Open Document Text Web"));
    map.insert(String::from("application/oebps-package+xml"), String::from("Open eBook Publication Structure"));
    map.insert(String::from("application/vnd.intu.qbo"), String::from("Open Financial Exchange"));
    map.insert(String::from("application/vnd.openofficeorg.extension"), String::from("Open Office Extension"));
    map.insert(String::from("application/vnd.yamaha.openscoreformat"), String::from("Open Score Format"));
    map.insert(String::from("audio/webm"), String::from("Open Web Media Project - Audio"));
    map.insert(String::from("video/webm"), String::from("Open Web Media Project - Video"));
    map.insert(String::from("application/vnd.oasis.opendocument.chart"), String::from("OpenDocument Chart"));
    map.insert(String::from("application/vnd.oasis.opendocument.chart-template"), String::from("OpenDocument Chart Template"));
    map.insert(String::from("application/vnd.oasis.opendocument.database"), String::from("OpenDocument Database"));
    map.insert(String::from("application/vnd.oasis.opendocument.formula"), String::from("OpenDocument Formula"));
    map.insert(String::from("application/vnd.oasis.opendocument.formula-template"), String::from("OpenDocument Formula Template"));
    map.insert(String::from("application/vnd.oasis.opendocument.graphics"), String::from("OpenDocument Graphics"));
    map.insert(String::from("application/vnd.oasis.opendocument.graphics-template"), String::from("OpenDocument Graphics Template"));
    map.insert(String::from("application/vnd.oasis.opendocument.image"), String::from("OpenDocument Image"));
    map.insert(String::from("application/vnd.oasis.opendocument.image-template"), String::from("OpenDocument Image Template"));
    map.insert(String::from("application/vnd.oasis.opendocument.presentation"), String::from("OpenDocument Presentation"));
    map.insert(String::from("application/vnd.oasis.opendocument.presentation-template"), String::from("OpenDocument Presentation Template"));
    map.insert(String::from("application/vnd.oasis.opendocument.spreadsheet"), String::from("OpenDocument Spreadsheet"));
    map.insert(String::from("application/vnd.oasis.opendocument.spreadsheet-template"), String::from("OpenDocument Spreadsheet Template"));
    map.insert(String::from("application/vnd.oasis.opendocument.text"), String::from("OpenDocument Text"));
    map.insert(String::from("application/vnd.oasis.opendocument.text-master"), String::from("OpenDocument Text Master"));
    map.insert(String::from("application/vnd.oasis.opendocument.text-template"), String::from("OpenDocument Text Template"));
    map.insert(String::from("image/ktx"), String::from("OpenGL Textures (KTX)"));
    map.insert(String::from("application/vnd.sun.xml.calc"), String::from("OpenOffice - Calc (Spreadsheet)"));
    map.insert(String::from("application/vnd.sun.xml.calc.template"), String::from("OpenOffice - Calc Template (Spreadsheet)"));
    map.insert(String::from("application/vnd.sun.xml.draw"), String::from("OpenOffice - Draw (Graphics)"));
    map.insert(String::from("application/vnd.sun.xml.draw.template"), String::from("OpenOffice - Draw Template (Graphics)"));
    map.insert(String::from("application/vnd.sun.xml.impress"), String::from("OpenOffice - Impress (Presentation)"));
    map.insert(String::from("application/vnd.sun.xml.impress.template"), String::from("OpenOffice - Impress Template (Presentation)"));
    map.insert(String::from("application/vnd.sun.xml.math"), String::from("OpenOffice - Math (Formula)"));
    map.insert(String::from("application/vnd.sun.xml.writer"), String::from("OpenOffice - Writer (Text - HTML)"));
    map.insert(String::from("application/vnd.sun.xml.writer.global"), String::from("OpenOffice - Writer (Text - HTML)"));
    map.insert(String::from("application/vnd.sun.xml.writer.template"), String::from("OpenOffice - Writer Template (Text - HTML)"));
    map.insert(String::from("application/x-font-otf"), String::from("OpenType Font File"));
    map.insert(String::from("application/vnd.yamaha.openscoreformat.osfpvg+xml"), String::from("OSFPVG"));
    map.insert(String::from("application/vnd.osgi.dp"), String::from("OSGi Deployment Package"));
    map.insert(String::from("application/vnd.palm"), String::from("PalmOS Data"));
    map.insert(String::from("text/x-pascal"), String::from("Pascal Source File"));
    map.insert(String::from("application/vnd.pawaafile"), String::from("PawaaFILE"));
    map.insert(String::from("application/vnd.hp-pclxl"), String::from("PCL 6 Enhanced (Formely PCL XL)"));
    map.insert(String::from("application/vnd.picsel"), String::from("Pcsel eFIF File"));
    map.insert(String::from("image/x-pcx"), String::from("PCX Image"));
    map.insert(String::from("image/vnd.adobe.photoshop"), String::from("Photoshop Document"));
    map.insert(String::from("application/pics-rules"), String::from("PICSRules"));
    map.insert(String::from("image/x-pict"), String::from("PICT Image"));
    map.insert(String::from("application/x-chat"), String::from("pIRCh"));
    map.insert(String::from("application/pkcs10"), String::from("PKCS #10 - Certification Request Standard"));
    map.insert(String::from("application/x-pkcs12"), String::from("PKCS #12 - Personal Information Exchange Syntax Standard"));
    map.insert(String::from("application/pkcs7-mime"), String::from("PKCS #7 - Cryptographic Message Syntax Standard"));
    map.insert(String::from("application/pkcs7-signature"), String::from("PKCS #7 - Cryptographic Message Syntax Standard"));
    map.insert(String::from("application/x-pkcs7-certreqresp"), String::from("PKCS #7 - Cryptographic Message Syntax Standard (Certificate Request Response)"));
    map.insert(String::from("application/x-pkcs7-certificates"), String::from("PKCS #7 - Cryptographic Message Syntax Standard (Certificates)"));
    map.insert(String::from("application/pkcs8"), String::from("PKCS #8 - Private-Key Information Syntax Standard"));
    map.insert(String::from("application/vnd.pocketlearn"), String::from("PocketLearn Viewers"));
    map.insert(String::from("image/x-portable-anymap"), String::from("Portable Anymap Image"));
    map.insert(String::from("image/x-portable-bitmap"), String::from("Portable Bitmap Format"));
    map.insert(String::from("application/x-font-pcf"), String::from("Portable Compiled Format"));
    map.insert(String::from("application/font-tdpfr"), String::from("Portable Font Resource"));
    map.insert(String::from("application/x-chess-pgn"), String::from("Portable Game Notation (Chess Games)"));
    map.insert(String::from("image/x-portable-graymap"), String::from("Portable Graymap Format"));
    map.insert(String::from("image/png"), String::from("Portable Network Graphics (PNG)"));
    map.insert(String::from("image/x-citrix-png"), String::from("Portable Network Graphics (PNG) (Citrix client)"));
    map.insert(String::from("image/x-png"), String::from("Portable Network Graphics (PNG) (x-token)"));
    map.insert(String::from("image/x-portable-pixmap"), String::from("Portable Pixmap Format"));
    map.insert(String::from("application/pskc+xml"), String::from("Portable Symmetric Key Container"));
    map.insert(String::from("application/vnd.ctc-posml"), String::from("PosML"));
    map.insert(String::from("application/postscript"), String::from("PostScript"));
    map.insert(String::from("application/x-font-type1"), String::from("PostScript Fonts"));
    map.insert(String::from("application/vnd.powerbuilder6"), String::from("PowerBuilder"));
    map.insert(String::from("application/pgp-encrypted"), String::from("Pretty Good Privacy"));
    map.insert(String::from("application/pgp-signature"), String::from("Pretty Good Privacy - Signature"));
    map.insert(String::from("application/vnd.previewsystems.box"), String::from("Preview Systems ZipLock/VBox"));
    map.insert(String::from("application/vnd.pvi.ptid1"), String::from("Princeton Video Image"));
    map.insert(String::from("application/pls+xml"), String::from("Pronunciation Lexicon Specification"));
    map.insert(String::from("application/vnd.pg.format"), String::from("Proprietary P&G Standard Reporting System"));
    map.insert(String::from("application/vnd.pg.osasli"), String::from("Proprietary P&G Standard Reporting System"));
    map.insert(String::from("text/prs.lines.tag"), String::from("PRS Lines Tag"));
    map.insert(String::from("application/x-font-linux-psf"), String::from("PSF Fonts"));
    map.insert(String::from("application/vnd.publishare-delta-tree"), String::from("PubliShare Objects"));
    map.insert(String::from("application/vnd.pmi.widget"), String::from("Qualcomm's Plaza Mobile Internet"));
    map.insert(String::from("application/vnd.quark.quarkxpress"), String::from("QuarkXpress"));
    map.insert(String::from("application/vnd.epson.esf"), String::from("QUASS Stream Player"));
    map.insert(String::from("application/vnd.epson.msf"), String::from("QUASS Stream Player"));
    map.insert(String::from("application/vnd.epson.ssf"), String::from("QUASS Stream Player"));
    map.insert(String::from("application/vnd.epson.quickanime"), String::from("QuickAnime Player"));
    map.insert(String::from("application/vnd.intu.qfx"), String::from("Quicken"));
    map.insert(String::from("video/quicktime"), String::from("Quicktime Video"));
    map.insert(String::from("application/x-rar-compressed"), String::from("RAR Archive"));
    map.insert(String::from("audio/x-pn-realaudio"), String::from("Real Audio Sound"));
    map.insert(String::from("audio/x-pn-realaudio-plugin"), String::from("Real Audio Sound"));
    map.insert(String::from("application/rsd+xml"), String::from("Really Simple Discovery"));
    map.insert(String::from("application/vnd.rn-realmedia"), String::from("RealMedia"));
    map.insert(String::from("application/vnd.realvnc.bed"), String::from("RealVNC"));
    map.insert(String::from("application/vnd.recordare.musicxml"), String::from("Recordare Applications"));
    map.insert(String::from("application/vnd.recordare.musicxml+xml"), String::from("Recordare Applications"));
    map.insert(String::from("application/relax-ng-compact-syntax"), String::from("Relax NG Compact Syntax"));
    map.insert(String::from("application/vnd.data-vision.rdz"), String::from("RemoteDocs R-Viewer"));
    map.insert(String::from("application/rdf+xml"), String::from("Resource Description Framework"));
    map.insert(String::from("application/vnd.cloanto.rp9"), String::from("RetroPlatform Player"));
    map.insert(String::from("application/vnd.jisp"), String::from("RhymBox"));
    map.insert(String::from("application/rtf"), String::from("Rich Text Format"));
    map.insert(String::from("text/richtext"), String::from("Rich Text Format (RTF)"));
    map.insert(String::from("application/vnd.route66.link66+xml"), String::from("ROUTE 66 Location Based Services"));
    map.insert(String::from("application/rss+xml"), String::from("RSS - Really Simple Syndication"));
    map.insert(String::from("application/shf+xml"), String::from("S Hexdump Format"));
    map.insert(String::from("application/vnd.sailingtracker.track"), String::from("SailingTracker"));
    map.insert(String::from("image/svg+xml"), String::from("Scalable Vector Graphics (SVG)"));
    map.insert(String::from("application/vnd.sus-calendar"), String::from("ScheduleUs"));
    map.insert(String::from("application/sru+xml"), String::from("Search/Retrieve via URL Response Format"));
    map.insert(String::from("application/set-payment-initiation"), String::from("Secure Electronic Transaction - Payment"));
    map.insert(String::from("application/set-registration-initiation"), String::from("Secure Electronic Transaction - Registration"));
    map.insert(String::from("application/vnd.sema"), String::from("Secured eMail"));
    map.insert(String::from("application/vnd.semd"), String::from("Secured eMail"));
    map.insert(String::from("application/vnd.semf"), String::from("Secured eMail"));
    map.insert(String::from("application/vnd.seemail"), String::from("SeeMail"));
    map.insert(String::from("application/x-font-snf"), String::from("Server Normal Format"));
    map.insert(String::from("application/scvp-vp-request"), String::from("Server-Based Certificate Validation Protocol - Validation Policies - Request"));
    map.insert(String::from("application/scvp-vp-response"), String::from("Server-Based Certificate Validation Protocol - Validation Policies - Response"));
    map.insert(String::from("application/scvp-cv-request"), String::from("Server-Based Certificate Validation Protocol - Validation Request"));
    map.insert(String::from("application/scvp-cv-response"), String::from("Server-Based Certificate Validation Protocol - Validation Response"));
    map.insert(String::from("application/sdp"), String::from("Session Description Protocol"));
    map.insert(String::from("text/x-setext"), String::from("Setext"));
    map.insert(String::from("video/x-sgi-movie"), String::from("SGI Movie"));
    map.insert(String::from("application/vnd.shana.informed.formdata"), String::from("Shana Informed Filler"));
    map.insert(String::from("application/vnd.shana.informed.formtemplate"), String::from("Shana Informed Filler"));
    map.insert(String::from("application/vnd.shana.informed.interchange"), String::from("Shana Informed Filler"));
    map.insert(String::from("application/vnd.shana.informed.package"), String::from("Shana Informed Filler"));
    map.insert(String::from("application/thraud+xml"), String::from("Sharing Transaction Fraud Data"));
    map.insert(String::from("application/x-shar"), String::from("Shell Archive"));
    map.insert(String::from("image/x-rgb"), String::from("Silicon Graphics RGB Bitmap"));
    map.insert(String::from("application/vnd.epson.salt"), String::from("SimpleAnimeLite Player"));
    map.insert(String::from("application/vnd.accpac.simply.aso"), String::from("Simply Accounting"));
    map.insert(String::from("application/vnd.accpac.simply.imp"), String::from("Simply Accounting - Data Import"));
    map.insert(String::from("application/vnd.simtech-mindmapper"), String::from("SimTech MindMapper"));
    map.insert(String::from("application/vnd.commonspace"), String::from("Sixth Floor Media - CommonSpace"));
    map.insert(String::from("application/vnd.yamaha.smaf-audio"), String::from("SMAF Audio"));
    map.insert(String::from("application/vnd.smaf"), String::from("SMAF File"));
    map.insert(String::from("application/vnd.yamaha.smaf-phrase"), String::from("SMAF Phrase"));
    map.insert(String::from("application/vnd.smart.teacher"), String::from("SMART Technologies Apps"));
    map.insert(String::from("application/vnd.svd"), String::from("SourceView Document"));
    map.insert(String::from("application/sparql-query"), String::from("SPARQL - Query"));
    map.insert(String::from("application/sparql-results+xml"), String::from("SPARQL - Results"));
    map.insert(String::from("application/srgs"), String::from("Speech Recognition Grammar Specification"));
    map.insert(String::from("application/srgs+xml"), String::from("Speech Recognition Grammar Specification - XML"));
    map.insert(String::from("application/ssml+xml"), String::from("Speech Synthesis Markup Language"));
    map.insert(String::from("application/vnd.koan"), String::from("SSEYO Koan Play File"));
    map.insert(String::from("text/sgml"), String::from("Standard Generalized Markup Language (SGML)"));
    map.insert(String::from("application/vnd.stardivision.calc"), String::from("StarOffice - Calc"));
    map.insert(String::from("application/vnd.stardivision.draw"), String::from("StarOffice - Draw"));
    map.insert(String::from("application/vnd.stardivision.impress"), String::from("StarOffice - Impress"));
    map.insert(String::from("application/vnd.stardivision.math"), String::from("StarOffice - Math"));
    map.insert(String::from("application/vnd.stardivision.writer"), String::from("StarOffice - Writer"));
    map.insert(String::from("application/vnd.stardivision.writer-global"), String::from("StarOffice - Writer (Global)"));
    map.insert(String::from("application/vnd.stepmania.stepchart"), String::from("StepMania"));
    map.insert(String::from("application/x-stuffit"), String::from("Stuffit Archive"));
    map.insert(String::from("application/x-stuffitx"), String::from("Stuffit Archive"));
    map.insert(String::from("application/vnd.solent.sdkm+xml"), String::from("SudokuMagic"));
    map.insert(String::from("application/vnd.olpc-sugar"), String::from("Sugar Linux Application Bundle"));
    map.insert(String::from("audio/basic"), String::from("Sun Audio - Au file format"));
    map.insert(String::from("application/vnd.wqd"), String::from("SundaHus WQ"));
    map.insert(String::from("application/vnd.symbian.install"), String::from("Symbian Install Package"));
    map.insert(String::from("application/smil+xml"), String::from("Synchronized Multimedia Integration Language"));
    map.insert(String::from("application/vnd.syncml+xml"), String::from("SyncML"));
    map.insert(String::from("application/vnd.syncml.dm+wbxml"), String::from("SyncML - Device Management"));
    map.insert(String::from("application/vnd.syncml.dm+xml"), String::from("SyncML - Device Management"));
    map.insert(String::from("application/x-sv4cpio"), String::from("System V Release 4 CPIO Archive"));
    map.insert(String::from("application/x-sv4crc"), String::from("System V Release 4 CPIO Checksum Data"));
    map.insert(String::from("application/sbml+xml"), String::from("Systems Biology Markup Language"));
    map.insert(String::from("text/tab-separated-values"), String::from("Tab Seperated Values"));
    map.insert(String::from("image/tiff"), String::from("Tagged Image File Format"));
    map.insert(String::from("application/vnd.tao.intent-module-archive"), String::from("Tao Intent"));
    map.insert(String::from("application/x-tar"), String::from("Tar File (Tape Archive)"));
    map.insert(String::from("application/x-tcl"), String::from("Tcl Script"));
    map.insert(String::from("application/x-tex"), String::from("TeX"));
    map.insert(String::from("application/x-tex-tfm"), String::from("TeX Font Metric"));
    map.insert(String::from("application/tei+xml"), String::from("Text Encoding and Interchange"));
    map.insert(String::from("text/plain"), String::from("Text File"));
    map.insert(String::from("application/vnd.spotfire.dxp"), String::from("TIBCO Spotfire"));
    map.insert(String::from("application/vnd.spotfire.sfs"), String::from("TIBCO Spotfire"));
    map.insert(String::from("application/timestamped-data"), String::from("Time Stamped Data Envelope"));
    map.insert(String::from("application/vnd.trid.tpt"), String::from("TRI Systems Config"));
    map.insert(String::from("application/vnd.triscape.mxs"), String::from("Triscape Map Explorer"));
    map.insert(String::from("text/troff"), String::from("troff"));
    map.insert(String::from("application/vnd.trueapp"), String::from("True BASIC"));
    map.insert(String::from("application/x-font-ttf"), String::from("TrueType Font"));
    map.insert(String::from("text/turtle"), String::from("Turtle (Terse RDF Triple Language)"));
    map.insert(String::from("application/vnd.umajin"), String::from("UMAJIN"));
    map.insert(String::from("application/vnd.uoml+xml"), String::from("Unique Object Markup Language"));
    map.insert(String::from("application/vnd.unity"), String::from("Unity 3d"));
    map.insert(String::from("application/vnd.ufdl"), String::from("Universal Forms Description Language"));
    map.insert(String::from("text/uri-list"), String::from("URI Resolution Services"));
    map.insert(String::from("application/vnd.uiq.theme"), String::from("User Interface Quartz - Theme (Symbian)"));
    map.insert(String::from("application/x-ustar"), String::from("Ustar (Uniform Standard Tape Archive)"));
    map.insert(String::from("text/x-uuencode"), String::from("UUEncode"));
    map.insert(String::from("text/x-vcalendar"), String::from("vCalendar"));
    map.insert(String::from("text/x-vcard"), String::from("vCard"));
    map.insert(String::from("application/x-cdlink"), String::from("Video CD"));
    map.insert(String::from("application/vnd.vsf"), String::from("Viewport+"));
    map.insert(String::from("model/vrml"), String::from("Virtual Reality Modeling Language"));
    map.insert(String::from("application/vnd.vcx"), String::from("VirtualCatalog"));
    map.insert(String::from("model/vnd.mts"), String::from("Virtue MTS"));
    map.insert(String::from("model/vnd.vtu"), String::from("Virtue VTU"));
    map.insert(String::from("application/vnd.visionary"), String::from("Visionary"));
    map.insert(String::from("video/vnd.vivo"), String::from("Vivo"));
    map.insert(String::from("application/ccxml+xml"), String::from("Voice Browser Call Control"));
    map.insert(String::from("application/voicexml+xml"), String::from("VoiceXML"));
    map.insert(String::from("application/x-wais-source"), String::from("WAIS Source"));
    map.insert(String::from("application/vnd.wap.wbxml"), String::from("WAP Binary XML (WBXML)"));
    map.insert(String::from("image/vnd.wap.wbmp"), String::from("WAP Bitamp (WBMP)"));
    map.insert(String::from("audio/x-wav"), String::from("Waveform Audio File Format (WAV)"));
    map.insert(String::from("application/davmount+xml"), String::from("Web Distributed Authoring and Versioning"));
    map.insert(String::from("application/x-font-woff"), String::from("Web Open Font Format"));
    map.insert(String::from("application/wspolicy+xml"), String::from("Web Services Policy"));
    map.insert(String::from("image/webp"), String::from("WebP Image"));
    map.insert(String::from("application/vnd.webturbo"), String::from("WebTurbo"));
    map.insert(String::from("application/widget"), String::from("Widget Packaging and XML Configuration"));
    map.insert(String::from("application/winhlp"), String::from("WinHelp"));
    map.insert(String::from("text/vnd.wap.wml"), String::from("Wireless Markup Language (WML)"));
    map.insert(String::from("text/vnd.wap.wmlscript"), String::from("Wireless Markup Language Script (WMLScript)"));
    map.insert(String::from("application/vnd.wap.wmlscriptc"), String::from("WMLScript"));
    map.insert(String::from("application/vnd.wordperfect"), String::from("Wordperfect"));
    map.insert(String::from("application/vnd.wt.stf"), String::from("Worldtalk"));
    map.insert(String::from("application/wsdl+xml"), String::from("WSDL - Web Services Description Language"));
    map.insert(String::from("image/x-xbitmap"), String::from("X BitMap"));
    map.insert(String::from("image/x-xpixmap"), String::from("X PixMap"));
    map.insert(String::from("image/x-xwindowdump"), String::from("X Window Dump"));
    map.insert(String::from("application/x-x509-ca-cert"), String::from("X.509 Certificate"));
    map.insert(String::from("application/x-xfig"), String::from("Xfig"));
    map.insert(String::from("application/xhtml+xml"), String::from("XHTML - The Extensible HyperText Markup Language"));
    map.insert(String::from("application/xml"), String::from("XML - Extensible Markup Language"));
    map.insert(String::from("application/xcap-diff+xml"), String::from("XML Configuration Access Protocol - XCAP Diff"));
    map.insert(String::from("application/xenc+xml"), String::from("XML Encryption Syntax and Processing"));
    map.insert(String::from("application/patch-ops-error+xml"), String::from("XML Patch Framework"));
    map.insert(String::from("application/resource-lists+xml"), String::from("XML Resource Lists"));
    map.insert(String::from("application/rls-services+xml"), String::from("XML Resource Lists"));
    map.insert(String::from("application/resource-lists-diff+xml"), String::from("XML Resource Lists Diff"));
    map.insert(String::from("application/xslt+xml"), String::from("XML Transformations"));
    map.insert(String::from("application/xop+xml"), String::from("XML-Binary Optimized Packaging"));
    map.insert(String::from("application/x-xpinstall"), String::from("XPInstall - Mozilla"));
    map.insert(String::from("application/xspf+xml"), String::from("XSPF - XML Shareable Playlist Format"));
    map.insert(String::from("application/vnd.mozilla.xul+xml"), String::from("XUL - XML User Interface Language"));
    map.insert(String::from("chemical/x-xyz"), String::from("XYZ File Format"));
    map.insert(String::from("text/yaml"), String::from("YAML Ain't Markup Language / Yet Another Markup Language"));
    map.insert(String::from("application/yang"), String::from("YANG Data Modeling Language"));
    map.insert(String::from("application/yin+xml"), String::from("YIN (YANG - XML)"));
    map.insert(String::from("application/vnd.zul"), String::from("Z.U.L. Geometry"));
    map.insert(String::from("application/zip"), String::from("Zip Archive"));
    map.insert(String::from("application/vnd.handheld-entertainment+xml"), String::from("ZVUE Media Manager"));
    map.insert(String::from("application/vnd.zzazz.deck+xml"), String::from("Zzazz Deck"));    
    return map
}

pub fn get_file_type(content_type: &str) -> Result<String, PlanetError> {
    let map = get_file_types();
    let result = map.get(content_type);
    if result.is_none() {
        return Err(
            PlanetError::new(
                500, 
                Some(tr!(
                    "Content type \"{}\" not supported.", content_type
                )),
            )
        );
    }
    let file_type = result.unwrap();
    return Ok(file_type.clone())
}
