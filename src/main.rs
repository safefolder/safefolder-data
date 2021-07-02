pub mod commands;

use commands::data_tables::schema::create_table;

fn main() {
    println!("Hello, world!");
    create_table();
}
