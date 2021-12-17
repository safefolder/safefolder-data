
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

pub const LANGUAGE_CODES: &str = "language_codes";
pub const LANGUAGE_DEFAULT: &str = "language_default";
pub const FIELDS: &str = "fields";
pub const FIELD_IDS: &str = "field_ids";
pub const SELECT_OPTIONS: &str = "select_options";
pub const KEY: &str = "key";
pub const NAME_CAMEL: &str = "Name";
pub const TABLE_NAME: &str = "table_name";

// FileConfig fields
pub const FIELD_TYPE: &str = "field_type";
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

// Field Types
pub const FIELD_TYPE_SMALL_TEXT: &str = "Small Text";
pub const FIELD_TYPE_LONG_TEXT: &str = "Long Text";
pub const FIELD_TYPE_CHECKBOX: &str = "Checkbox";
pub const FIELD_TYPE_SELECT: &str = "Select";
pub const FIELD_TYPE_DATE: &str = "Date";
pub const FIELD_TYPE_FORMULA: &str = "Formula";

// Date Format
pub const DATE_FORMAT_FRIENDLY: &str = "Friendly";
pub const DATE_FORMAT_US: &str = "US";
pub const DATE_FORMAT_EUROPEAN: &str = "European";
pub const DATE_FORMAT_ISO: &str = "ISO";
