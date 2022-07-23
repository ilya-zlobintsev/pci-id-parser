mod error;
mod parser;
pub mod schema;

use error::Error;
use parser::{drain_id_and_name, parse_subdevice_id};
use schema::{Class, Device, DeviceInfo, SubClass, SubDeviceId, Vendor};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
};
use tracing::trace;

use crate::parser::{parse_class, parse_prog_if, parse_subclass};

const DB_PATHS: &[&str] = &["/usr/share/hwdata/pci.ids", "/usr/share/misc/pci.ids"];
#[cfg(feature = "online")]
const URL: &str = "https://pci-ids.ucw.cz/v2.2/pci.ids";

#[derive(Debug)]
pub enum VendorDataError {
    MissingIdsFile,
}

#[derive(Debug)]
pub struct Database {
    pub vendors: HashMap<String, Vendor>,
    pub classes: HashMap<String, Class>,
}

impl Database {
    pub fn read() -> Result<Self, Error> {
        let file = Self::open_file()?;
        Self::parse_db(file)
    }

    #[cfg(feature = "online")]
    pub fn get_online() -> Result<Self, Error> {
        let response = ureq::get(URL).call()?;

        Self::parse_db(response.into_reader())
    }

    pub fn parse_db<R: Read>(reader: R) -> Result<Self, Error> {
        let mut reader = BufReader::new(reader);
        let mut buf = String::new();

        let mut current_vendor: Option<(String, Vendor)> = None;
        let mut current_device: Option<(String, Device)> = None;

        let mut current_class: Option<(String, Class)> = None;
        let mut current_subclass: Option<(String, SubClass)> = None;

        let mut vendors: HashMap<String, Vendor> = HashMap::with_capacity(2500);

        while reader.read_line(&mut buf)? != 0 {
            if buf.starts_with("C ") || buf.starts_with("c ") {
                // Proceed to parse classes
                current_class = Some(parse_class(&mut buf)?);
                break;
            } else if !(buf.starts_with('#') || buf.is_empty() || (buf == "\n")) {
                // Subdevice
                if buf.starts_with("\t\t") {
                    let (_, current_device) = current_device
                        .as_mut()
                        .ok_or_else(Error::no_current_device)?;

                    let (name, subdevice_id) = parse_subdevice_id(&mut buf)?;

                    current_device.subdevices.insert(subdevice_id, name);

                // Device
                } else if buf.starts_with('\t') {
                    // Device section is over, write to vendor
                    if let Some((device_id, device)) = current_device {
                        let (_, current_vendor) = current_vendor
                            .as_mut()
                            .ok_or_else(Error::no_current_vendor)?;

                        current_vendor.devices.insert(device_id, device);
                    }

                    let (id, name) = drain_id_and_name(&mut buf)?;

                    let device = Device {
                        name,
                        subdevices: HashMap::new(),
                    };

                    current_device = Some((id, device));
                // Vendor
                } else {
                    // The vendor section is complete so it needs to be pushed to the main list
                    if let Some((device_id, device)) = current_device {
                        let (_, vendor) = current_vendor
                            .as_mut()
                            .ok_or_else(Error::no_current_vendor)?;
                        vendor.devices.insert(device_id, device);
                    }
                    if let Some((vendor_id, vendor)) = current_vendor {
                        vendors.insert(vendor_id, vendor);
                    }

                    let (vendor_id, name) = drain_id_and_name(&mut buf)?;

                    let vendor = Vendor {
                        name,
                        devices: HashMap::new(),
                    };
                    current_vendor = Some((vendor_id, vendor));
                    current_device = None;
                }
                debug_assert!(buf.trim().is_empty());
            }
            buf.clear();
        }
        if let Some((device_id, device)) = current_device {
            let (_, vendor) = current_vendor
                .as_mut()
                .ok_or_else(Error::no_current_vendor)?;
            vendor.devices.insert(device_id, device);
        }
        if let Some((vendor_id, vendor)) = current_vendor {
            vendors.insert(vendor_id, vendor);
        }
        buf.clear();

        vendors.shrink_to_fit();
        trace!("Parsed {} vendors", vendors.len());

        let mut classes: HashMap<String, Class> = HashMap::with_capacity(200);

        while reader.read_line(&mut buf)? != 0 {
            if buf.starts_with("C ") || buf.starts_with("c ") {
                if let Some((subclass_id, subclass)) = current_subclass {
                    let (_, class) = current_class
                        .as_mut()
                        .ok_or_else(|| Error::no_current_class())?;

                    class.subclasses.insert(subclass_id, subclass);
                }
                if let Some((class_id, class)) = current_class {
                    classes.insert(class_id, class);
                }

                current_class = Some(parse_class(&mut buf)?);
                current_subclass = None;
            } else if buf.starts_with("\t\t") {
                // Prog-if
                let (id, name) = parse_prog_if(&mut buf)?;
                let (_, subclass) = current_subclass
                    .as_mut()
                    .ok_or_else(|| Error::no_current_subclass())?;

                subclass.prog_ifs.insert(id, name);
            } else if buf.starts_with("\t") {
                // Subclass
                // Flush previous subclass
                if let Some((subclass_id, subclass)) = current_subclass {
                    let (_, class) = current_class
                        .as_mut()
                        .ok_or_else(|| Error::no_current_class())?;

                    class.subclasses.insert(subclass_id, subclass);
                }
                current_subclass = Some(parse_subclass(&mut buf)?);
            }
            buf.clear();
        }
        classes.shrink_to_fit();

        trace!("Parsed {} classes", classes.len());

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

    pub fn get_device_info<'a>(
        &'a self,
        vendor_id: &str,
        model_id: &str,
        subsys_vendor_id: &str,
        subsys_model_id: &str,
    ) -> DeviceInfo<'a> {
        let vendor_id = vendor_id.to_lowercase();
        let model_id = model_id.to_lowercase();
        let subsys_vendor_id = subsys_vendor_id.to_lowercase();
        let subsys_model_id = subsys_model_id.to_lowercase();

        let mut vendor_name = None;
        let mut device_name = None;
        let mut subvendor_name = None;
        let mut subdevice_name = None;

        trace!("Searching vendor {}", vendor_id);
        if let Some(vendor) = self.vendors.get(&vendor_id) {
            trace!("Found vendor {}", vendor.name);
            vendor_name = Some(vendor.name.as_str());

            trace!("Searching device {}", model_id);
            if let Some(device) = vendor.devices.get(&model_id) {
                trace!("Found device {}", device.name);
                device_name = Some(device.name.as_str());

                trace!(
                    "Searching subdevice {} {}",
                    subsys_vendor_id,
                    subsys_model_id
                );
                if let Some(subvendor) = self.vendors.get(&subsys_vendor_id) {
                    trace!("Found subvendor {}", subvendor.name);
                    subvendor_name = Some(subvendor.name.as_str());
                }

                let subdevice_id = SubDeviceId {
                    subvendor: subsys_vendor_id.to_owned(),
                    subdevice: subsys_model_id,
                };

                subdevice_name = device.subdevices.get(&subdevice_id).map(|s| s.as_str());
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
