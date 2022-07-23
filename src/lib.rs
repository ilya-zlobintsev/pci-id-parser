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

        let mut vendors: HashMap<String, Vendor> = HashMap::with_capacity(2500);

        let mut current_vendor: Option<(String, Vendor)> = None;
        let mut current_device: Option<(String, Device)> = None;

        let mut current_class: Option<(String, Class)> = None;
        let mut current_subclass: Option<(String, SubClass)> = None;

        let mut buf = String::new();

        // Devices
        while reader.read_line(&mut buf)? != 0 {
            if buf.starts_with("C ") | buf.starts_with("c ") {
                // Device classes, they're at the end of file and not yet supported
                break;
            } else if !(buf.starts_with('#') | buf.is_empty() | (buf == "\n")) {
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

        vendors.shrink_to_fit();
        trace!("db len: {}", vendors.len());

        Ok(Self { vendors })
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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use tracing::Level;

    #[test]
    fn init() {
        tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            .init();
    }

    #[test]
    fn parse_polaris_local() {
        let db = Database::read().unwrap();
        parse_polaris(db);
    }

    #[cfg(feature = "online")]
    #[test]
    fn parse_polaris_online() {
        let db = Database::get_online().unwrap();
        parse_polaris(db);
    }

    fn parse_polaris(db: Database) {
        let data = db.get_device_info("1002", "67DF", "1DA2", "E387");

        assert_eq!(
            data.vendor_name,
            Some("Advanced Micro Devices, Inc. [AMD/ATI]"),
        );
        assert_eq!(
            data.device_name,
            Some("Ellesmere [Radeon RX 470/480/570/570X/580/580X/590]"),
        );
        assert_eq!(data.subvendor_name, Some("Sapphire Technology Limited"));
        // Depending on the pci.ids version shipped this may be different
        let card_model = data.subdevice_name.unwrap();
        assert!(card_model == "Radeon RX 570 Pulse 4GB" || card_model == "Radeon RX 580 Pulse 4GB");
    }

    #[test]
    fn parse_vega() {
        let db = Database::read().unwrap();
        let data = db.get_device_info("1002", "687F", "1043", "0555");

        assert_eq!(
            data.vendor_name,
            Some("Advanced Micro Devices, Inc. [AMD/ATI]")
        );
        assert_eq!(
            data.device_name,
            Some("Vega 10 XL/XT [Radeon RX Vega 56/64]")
        );
        assert_eq!(data.subvendor_name, Some("ASUSTeK Computer Inc."));
        assert_eq!(data.subdevice_name, None);
    }

    #[test]
    fn class_not_in_vendors() {
        let db = Database::read().unwrap();

        assert_eq!(db.vendors.get("c"), None);
        assert_eq!(db.vendors.get("c 09"), None);
    }

    #[cfg(feature = "online")]
    #[test]
    fn parse_incomplete() {
        let db = Database::get_online().unwrap();

        let device_info = db.get_device_info("C 0c", "03", "fe", "");
        trace!("{device_info:?}");
        let expected_info = DeviceInfo {
            vendor_name: Some("Serial bus controller"),
            device_name: Some("USB controller"),
            subvendor_name: None,
            subdevice_name: Some("USB Device"),
        };

        assert_eq!(device_info.vendor_name, expected_info.vendor_name);
        assert_eq!(device_info.device_name, expected_info.device_name);
        assert_eq!(device_info.subvendor_name, expected_info.subvendor_name);
        assert_eq!(device_info.subdevice_name, expected_info.subdevice_name);
    }
}
