
pub const FIELD_VERSION: &str = "v1";
pub const FIELD_API_VERSION: &str = "07/01/2021";
pub const CHILD_PRIVATE_KEY_STR: &str = "xprv9zcbcQgMpVAa742DEAUFDLW7KxE5xRgqKVgQ9zpf4diT73KnEjWUZZ2bs42aX4RgvbXLC91TTc2t7caNcfkg5i1GFg2KY57RNrGeMkZHJdi";
pub const CHILD_PUBLIC_KEY_STR: &str = "xpub6Dbx1vDFerisKY6gLC1FaUSqsz4aMtQggibzxPEGcyFRyqevnGpj7MM5iKoduqYcV9QHiLQafLcr9QYZ8M9Sb1XhkZ2xs73skyCTzsQ7Q7M";
pub const CHILD_PRIVATE_KEY_ARRAY: [u8; 32] = [5, 184, 20, 127, 179, 211, 242, 67, 135, 53, 134, 91, 9, 210, 7, 179, 187, 140, 129, 89, 76, 7, 225, 55, 201, 108, 208, 42, 131, 42, 104, 170];

pub const FORMULA_FORMAT_TEXT: &str = "Text";
pub const FORMULA_FORMAT_NUMBER: &str = "Number";
pub const FORMULA_FORMAT_CHECK: &str = "Check";
pub const FORMULA_FORMAT_DATE: &str = "Date";

// value.truecolor(197, 147, 122),

pub const YAML_COLOR_ORANGE: [u8; 3] = [197, 147, 122];
pub const YAML_COLOR_BLUE: [u8; 3] = [100, 153, 208];
pub const YAML_COLOR_YELLOW: [u8; 3] = [175, 193, 161];

pub const SELECT_DEFAULT_PAGE: u32 = 1;
pub const SELECT_DEFAULT_NUMBER_ITEMS: u32 = 20;

pub const INDEX_PROFILE_IDX: &str = "idx";
pub const INDEX_PROFILE_RAW: &str = "raw";

pub const DB: &str = "db";
pub const INDEX: &str = "index";
pub const PRIVATE: &str = "private";
pub const WORKSPACE: &str = "workspace";
pub const SITE: &str = "site";
pub const TEXT: &str = "text";
pub const COLUMN_ID: &str = "column_id";
pub const SCORE: &str = "score";
pub const LANGUAGE_CODES: &str = "language_codes";
pub const LANGUAGE_DEFAULT: &str = "language_default";
pub const TEXT_SEARCH_COLUMN_RELEVANCE: &str = "text_search_column_relevance";

pub const PROPERTIES: &str = "properties";
pub const COLUMN_IDS: &str = "column_ids";
pub const SELECT_OPTIONS: &str = "select_options";
pub const KEY: &str = "key";
pub const NAME_CAMEL: &str = "Name";
pub const FOLDER_NAME: &str = "folder_name";

// FileConfig fields
pub const COLUMN_TYPE: &str = "column_type";
pub const DEFAULT: &str = "default";
pub const VERSION: &str = "version";
pub const REQUIRED: &str = "required";
pub const API_VERSION: &str = "api_version";
pub const INDEXED: &str = "indexed";
pub const MANY: &str = "many";
pub const OPTIONS: &str = "options";
pub const FORMULA: &str = "formula";
pub const FORMULA_FORMAT: &str = "formula_format";
pub const FORMULA_COMPILED: &str = "formula_compiled";
pub const DATE_FORMAT: &str = "date_format";
pub const TIME_FORMAT: &str = "time_format";
pub const NUMBER_DECIMALS: &str = "number_decimals";
pub const CURRENCY_SYMBOL: &str = "currency_symbol";
pub const LINKED_FOLDER_ID: &str = "linked_folder_id";
pub const DELETE_ON_LINK_DROP: &str = "delete_on_link_drop";
pub const RELATED_COLUMN: &str = "related_column";
pub const LANGUAGE_COLUMN: &str = "Language";
pub const TEXT_COLUMN: &str = "Text";
pub const WHERE: &str = "where";
pub const SEQUENCE: &str = "sequence";

// Column Types
pub const COLUMN_TYPE_SMALL_TEXT: &str = "Small Text";
pub const COLUMN_TYPE_LONG_TEXT: &str = "Long Text";
pub const COLUMN_TYPE_CHECKBOX: &str = "Checkbox";
pub const COLUMN_TYPE_SELECT: &str = "Select";
pub const COLUMN_TYPE_NUMBER: &str = "Number";
pub const COLUMN_TYPE_DATE: &str = "Date";
pub const COLUMN_TYPE_FORMULA: &str = "Formula";
pub const COLUMN_TYPE_DURATION: &str = "Duration";
pub const COLUMN_TYPE_CREATED_TIME: &str = "Created Time";
pub const COLUMN_TYPE_LAST_MODIFIED_TIME: &str = "Last Modified Time";
pub const COLUMN_TYPE_CREATED_BY: &str = "Created By";
pub const COLUMN_TYPE_LAST_MODIFIED_BY: &str = "Last Modified By";
pub const COLUMN_TYPE_CURRENCY: &str = "Currency";
pub const COLUMN_TYPE_PERCENTAGE: &str = "Percentage";
pub const COLUMN_TYPE_LINK: &str = "Link";
pub const COLUMN_TYPE_REFERENCE: &str = "Reference";
pub const COLUMN_TYPE_LANGUAGE: &str = "Language";
pub const COLUMN_TYPE_TEXT: &str = "Text";
pub const COLUMN_TYPE_GENERATE_ID: &str = "Generate Id";
pub const COLUMN_TYPE_GENERATE_NUMBER: &str = "Generate Number";

// Date Format
pub const DATE_FORMAT_FRIENDLY: &str = "Friendly";
pub const DATE_FORMAT_US: &str = "US";
pub const DATE_FORMAT_EUROPEAN: &str = "European";
pub const DATE_FORMAT_ISO: &str = "ISO";
