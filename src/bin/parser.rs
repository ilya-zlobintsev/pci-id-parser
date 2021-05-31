use std::fs::{self, File};

use pciid_parser::{PciDatabase, PciDatabaseError};
use handlebars::Handlebars;
use chrono::prelude::*;

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
    
    let handlebars = Handlebars::new();
    
    let index_template = include_str!("index.html.hbs");
    
    let f = File::create("index.html").expect("Failed to create file");
    
    handlebars.render_template_to_write(index_template, &TemplateData {
        update: Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
    }, f).expect("Failed to render template");
    
    println!("Saved index.html");

    Ok(())
}

#[derive(serde::Serialize)]
struct TemplateData {
    pub update: String,
}