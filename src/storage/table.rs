

pub trait Table {
    fn insert(&self);
    fn update(&self, id: &str);
    fn delete(&self, id: &str);
    fn read(&self, id: &str);
}

pub struct KeyValueTable<'a> {
    pub name: &'a str,
}

impl<'a> Table for KeyValueTable<'a> {

    fn insert(&self) {
        // Hola
    }
    fn update(&self, id: &str) {
        println!("id: {}", id)
    }
    fn delete(&self, id: &str) {
        println!("id: {}", id)
    }
    fn read(&self, id: &str) {
        println!("id: {id}", id=id)
    }

}
