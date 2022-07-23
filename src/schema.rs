use std::collections::HashMap;

#[derive(Default, Clone, Debug)]
pub struct DeviceInfo<'a> {
    pub vendor_name: Option<&'a str>,
    pub device_name: Option<&'a str>,
    pub subvendor_name: Option<&'a str>,
    pub subdevice_name: Option<&'a str>,
}

#[derive(Debug, PartialEq)]
pub struct Vendor {
    pub name: String,
    pub devices: HashMap<String, Device>,
}

impl Vendor {
    pub fn new(name: String) -> Self {
        Vendor {
            name,
            devices: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Device {
    pub name: String,
    pub subdevices: HashMap<SubDeviceId, String>,
}

impl Device {
    pub fn new(name: String) -> Self {
        Device {
            name,
            subdevices: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubDeviceId {
    pub subvendor: String,
    pub subdevice: String,
}