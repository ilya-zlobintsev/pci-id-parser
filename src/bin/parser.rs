use std::fs;

use pciid_parser::{PciDatabase, PciDatabaseError};

#[derive(Debug)]
enum ParseError {
    DBMissing,
    JsonError(serde_json::Error),
    WriteError(std::io::Error),
}

impl From<PciDatabaseError> for ParseError {
    fn from(_: PciDatabaseError) -> Self {
        ParseError::DBMissing
    }
}

impl From<serde_json::Error> for ParseError {
    fn from(err: serde_json::Error) -> Self {
        ParseError::JsonError(err)
    }
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        ParseError::WriteError(err)
    }
}

fn main() -> Result<(), ParseError> {
    let db = PciDatabase::get_online()?;

    let json = serde_json::to_string_pretty(&db.vendors)?;
    fs::write("devices.json", json)?;
    println!("Saved to devices.json");

    Ok(())
}