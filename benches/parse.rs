use pciid_parser::Database;
use std::io::Cursor;

const DB_DATA: &[u8] = include_bytes!("../tests/pci.ids");

fn main() {
    divan::main();
}

#[divan::bench(sample_count = 500)]
fn parse_embedded() -> Database {
    let cursor = Cursor::new(DB_DATA);
    Database::parse_db(cursor).unwrap()
}
