
use lingua::{Language};
use lingua::Language::{Danish, German, Spanish, French, Italian, Nynorsk, Portuguese, Swedish, English};

pub const LANGUAGES: [Language; 9] = [English, Danish, German, Spanish, French, Italian, Nynorsk, Portuguese, Swedish];

pub const SERDE_ERROR_TYPE_INVALID_TYPE: &str = "invalid type";
pub const SERDE_ERROR_TYPE_INVALID_VALUE: &str = "invalid value";
pub const SERDE_ERROR_TYPE_INVALID_LENGTH: &str = "invalid length";
pub const SERDE_ERROR_TYPE_UNKOWN_VARIANT: &str = "unknown variant";
pub const SERDE_ERROR_TYPE_UNKNOWN_FIELD: &str = "unknown field";
pub const SERDE_ERROR_TYPE_MISSING_FIELD: &str = "missing field";
pub const SERDE_ERROR_TYPE_DUPLICATE_FIELD: &str = "duplicate field";

pub const NULL: &str = "null";
pub const ACCOUNT_ID: &str = "account_id";
pub const SPACE_ID: &str = "space_id";
pub const SITE_ID: &str = "site_id";
pub const BOX_ID: &str = "box_id";
pub const IPFS_CID : &str = "ipfs_cid";
pub const ID: &str = "id";
pub const VALUE: &str = "value";
pub const NAME: &str = "name";
pub const SLUG: &str = "slug";
pub const TRUE: &str = "true";
pub const FALSE: &str = "false";
pub const PARTITION: &str = "partition";
pub const ITEMS_PER_PARTITION: u16 = 1000;
pub const MAX_PARTITIONS: u16 = 1000;
pub const MAX_ITEMS_PER_PARTITION: u32 = 1000000;
pub const PARENT_ID: &str = "parent_id";
pub const SUB_FOLDERS: &str = "sub_folders";
pub const COLUMNS: &str = "columns";

// Languages
pub const LANGUAGE_SPANISH: &str = "spanish";
pub const LANGUAGE_ENGLISH: &str = "english";
pub const LANGUAGE_FRENCH: &str = "french";
pub const LANGUAGE_GERMAN: &str = "german";
pub const LANGUAGE_ITALIAN: &str = "italian";
pub const LANGUAGE_PORTUGUESE: &str = "portuguese";
pub const LANGUAGE_NORWEGIAN: &str = "norwegian";
pub const LANGUAGE_SWEDISH: &str = "swedish";
pub const LANGUAGE_DANISH: &str = "danish";
pub const LANGUAGE_CODE_SPANISH: &str = "es";
pub const LANGUAGE_CODE_ENGLISH: &str = "en";
pub const LANGUAGE_CODE_FRENCH: &str = "fr";
pub const LANGUAGE_CODE_GERMAN: &str = "de";
pub const LANGUAGE_CODE_ITALIAN: &str = "it";
pub const LANGUAGE_CODE_PORTUGUESE: &str = "pt";
pub const LANGUAGE_CODE_NORWEGIAN: &str = "no";
pub const LANGUAGE_CODE_SWEDISH: &str = "sw";
pub const LANGUAGE_CODE_DANISH: &str = "da";
pub const LANGUAGE_ITEMS: [&str; 9] = [
    LANGUAGE_SPANISH,
    LANGUAGE_ENGLISH,
    LANGUAGE_FRENCH,
    LANGUAGE_GERMAN,
    LANGUAGE_ITALIAN,
    LANGUAGE_PORTUGUESE,
    LANGUAGE_NORWEGIAN,
    LANGUAGE_SWEDISH,
    LANGUAGE_DANISH
    ];

// Tika Local Server
pub const TIKA_HOST: &str = "localhost";
pub const TIKA_PORT: &str = "9998";

// Paths
pub const HOME_DIR_FOLDER: &str = "achiever-planet";
