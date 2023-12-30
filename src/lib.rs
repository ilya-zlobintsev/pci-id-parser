#![warn(clippy::pedantic)]
#![doc = include_str!("../README.md")]
mod error;
mod parser;
pub mod schema;

use crate::parser::Parser;
use ahash::{HashMapExt, RandomState};
use error::Error;
use parser::Event;
use schema::{Class, Device, DeviceInfo, SubClass, SubDeviceId, Vendor};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

const DB_PATHS: &[&str] = &[
    "/usr/share/hwdata/pci.ids",
    "/usr/share/misc/pci.ids",
    "@hwdata@/share/hwdata/pci.ids",
];
#[cfg(feature = "online")]
const URL: &str = "https://pci-ids.ucw.cz/v2.2/pci.ids";

#[derive(Debug)]
pub enum VendorDataError {
    MissingIdsFile,
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Database {
    pub vendors: HashMap<u16, Vendor, RandomState>,
    pub classes: HashMap<u16, Class, RandomState>,
}

impl Database {
    /// Attempt to read the database from a list of known file paths
    ///
    /// # Errors
    /// Returns an error when either no file could be found or the parsing fails.
    pub fn read() -> Result<Self, Error> {
        let file = Self::open_file()?;
        Self::parse_db(file)
    }

    /// Read the database from a given path
    ///
    /// # Errors
    /// Returns an error when the file can't be read or when parsing fails
    pub fn read_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let file = File::open(path)?;
        Self::parse_db(file)
    }

    /// Fetch a database from an online source
    ///
    /// # Errors
    /// Returns an error when the database either can't be fetched or parsed
    #[cfg(feature = "online")]
    pub fn get_online() -> Result<Self, Error> {
        let response = ureq::get(URL).call()?;

        Self::parse_db(response.into_reader())
    }

    /// Parse a database from the given reader
    ///
    /// # Errors
    /// Returns an error whenever there's a parsing error
    #[allow(clippy::too_many_lines)] // todo
    pub fn parse_db<R: Read>(reader: R) -> Result<Self, Error> {
        let reader = BufReader::new(reader);
        let mut parser = Parser::new(reader);

        let mut current_vendor: Option<(u16, Vendor)> = None;
        let mut current_device: Option<(u16, Device)> = None;

        let mut current_class: Option<(u16, Class)> = None;
        let mut current_subclass: Option<(u16, SubClass)> = None;

        let mut vendors: HashMap<u16, Vendor, RandomState> =
            HashMap::<_, _, RandomState>::with_capacity(2500);
        let mut classes: HashMap<u16, Class, RandomState> =
            HashMap::<_, _, RandomState>::with_capacity(200);

        while let Some(event) = parser.next_event()? {
            match event {
                Event::Vendor { id, name } => {
                    // The vendor section is complete so it needs to be pushed to the main list
                    if let Some((device_id, device)) = current_device.take() {
                        let (_, vendor) = current_vendor
                            .as_mut()
                            .ok_or_else(Error::no_current_vendor)?;
                        vendor.devices.insert(device_id, device);
                    }
                    if let Some((vendor_id, vendor)) = current_vendor.take() {
                        vendors.insert(vendor_id, vendor);
                    }

                    let vendor = Vendor {
                        name: name.to_owned(),
                        devices: HashMap::default(),
                    };
                    current_vendor = Some((id, vendor));
                }
                Event::Device { id, name } => {
                    // Device section is over, write to vendor
                    if let Some((device_id, device)) = current_device.take() {
                        let (_, current_vendor) = current_vendor
                            .as_mut()
                            .ok_or_else(Error::no_current_vendor)?;

                        current_vendor.devices.insert(device_id, device);
                    }

                    let device = Device {
                        name: name.to_owned(),
                        subdevices: HashMap::default(),
                    };
                    current_device = Some((id, device));
                }
                Event::Subdevice {
                    subvendor,
                    subdevice,
                    subsystem_name,
                } => {
                    let (_, current_device) = current_device
                        .as_mut()
                        .ok_or_else(Error::no_current_device)?;

                    let subdevice_id = SubDeviceId {
                        subvendor,
                        subdevice,
                    };
                    current_device
                        .subdevices
                        .insert(subdevice_id, subsystem_name.to_owned());
                }
                Event::Class { id, name } => {
                    if let Some((subclass_id, subclass)) = current_subclass.take() {
                        let (_, class) =
                            current_class.as_mut().ok_or_else(Error::no_current_class)?;

                        class.subclasses.insert(subclass_id, subclass);
                    }
                    if let Some((class_id, class)) = current_class.take() {
                        classes.insert(class_id, class);
                    }

                    let class = Class {
                        name: name.to_owned(),
                        subclasses: HashMap::default(),
                    };
                    current_class = Some((id, class));
                }
                Event::SubClass { id, name } => {
                    if let Some((subclass_id, subclass)) = current_subclass {
                        let (_, class) =
                            current_class.as_mut().ok_or_else(Error::no_current_class)?;

                        class.subclasses.insert(subclass_id, subclass);
                    }

                    let subclass = SubClass {
                        name: name.to_owned(),
                        prog_ifs: HashMap::default(),
                    };
                    current_subclass = Some((id, subclass));
                }
                Event::ProgIf { id, name } => {
                    let (_, subclass) = current_subclass
                        .as_mut()
                        .ok_or_else(Error::no_current_subclass)?;

                    subclass.prog_ifs.insert(id.to_owned(), name.to_owned());
                }
            }
        }
        // Finish writing the last vendor and class
        if let Some((device_id, device)) = current_device.take() {
            let (_, vendor) = current_vendor
                .as_mut()
                .ok_or_else(Error::no_current_vendor)?;
            vendor.devices.insert(device_id, device);
        }
        if let Some((vendor_id, vendor)) = current_vendor.take() {
            vendors.insert(vendor_id, vendor);
        }

        if let Some((subclass_id, subclass)) = current_subclass.take() {
            let (_, class) = current_class.as_mut().ok_or_else(Error::no_current_class)?;

            class.subclasses.insert(subclass_id, subclass);
        }
        if let Some((class_id, class)) = current_class.take() {
            classes.insert(class_id, class);
        }

        vendors.shrink_to_fit();
        classes.shrink_to_fit();

        Ok(Self { vendors, classes })
    }

    fn open_file() -> Result<File, Error> {
        if let Some(path) = DB_PATHS
            .iter()
            .find(|path| Path::exists(&PathBuf::from(path)))
        {
            Ok(File::open(path)?)
        } else {
            Err(Error::FileNotFound)
        }
    }

    #[must_use]
    pub fn get_device_info<'a>(
        &'a self,
        vendor_id: &str,
        model_id: &str,
        subsys_vendor_id: &str,
        subsys_model_id: &str,
    ) -> DeviceInfo<'a> {
        let vendor_id = u16::from_str_radix(vendor_id, 16).unwrap_or_default();
        let model_id = u16::from_str_radix(model_id, 16).unwrap_or_default();
        let subsys_vendor_id = u16::from_str_radix(subsys_vendor_id, 16).unwrap_or_default();
        let subsys_model_id = u16::from_str_radix(subsys_model_id, 16).unwrap_or_default();

        let mut vendor_name = None;
        let mut device_name = None;
        let mut subvendor_name = None;
        let mut subdevice_name = None;

        if let Some(vendor) = self.vendors.get(&vendor_id) {
            vendor_name = Some(vendor.name.as_str());

            if let Some(device) = vendor.devices.get(&model_id) {
                device_name = Some(device.name.as_str());

                if let Some(subvendor) = self.vendors.get(&subsys_vendor_id) {
                    subvendor_name = Some(subvendor.name.as_str());
                }

                let subdevice_id = SubDeviceId {
                    subvendor: subsys_vendor_id,
                    subdevice: subsys_model_id,
                };

                subdevice_name = device.subdevices.get(&subdevice_id).map(String::as_str);
            }
        }

        DeviceInfo {
            vendor_name,
            device_name,
            subvendor_name,
            subdevice_name,
        }
    }
}
