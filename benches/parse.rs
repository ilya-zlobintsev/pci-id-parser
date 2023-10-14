use pciid_parser::Database;
use std::io::Cursor;

const DB_DATA: &[u8] = include_bytes!("./pci.ids");

fn main() {
    divan::main();
}

#[divan::bench]
fn parse_embedded() -> Database {
    let cursor = Cursor::new(DB_DATA);
    Database::parse_db(cursor).unwrap()
}

#[divan::bench]
fn parse_from_file() -> Database {
    Database::read_from_file("./benches/pci.ids").unwrap()
}
