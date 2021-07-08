
pub trait TableField {
    fn is_valid(&self) -> bool;
    fn get_conf(&self) -> String;
}

pub struct SmallTextField<'a> {
    pub name: &'a str,
    pub conf: &'a str,
    pub id: &'a str,
    pub required: bool,
    pub indexed: bool,
}

impl<'a> TableField for SmallTextField<'a> {

    fn is_valid(&self) -> bool {
        // This method is used when wrting data for small text, to check that data to be writtem is valid
        return true;
    }
    fn get_conf(&self) -> String {
        return String::from("hola");
    }

}