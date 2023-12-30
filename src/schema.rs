use ahash::RandomState;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::Hash};

#[derive(Default, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DeviceInfo<'a> {
    pub vendor_name: Option<&'a str>,
    pub device_name: Option<&'a str>,
    pub subvendor_name: Option<&'a str>,
    pub subdevice_name: Option<&'a str>,
}

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Vendor {
    pub name: String,
    pub devices: HashMap<String, Device, RandomState>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Device {
    pub name: String,
    pub subdevices: HashMap<SubDeviceId, String, RandomState>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SubDeviceId {
    pub subvendor: String,
    pub subdevice: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Class {
    pub name: String,
    pub subclasses: HashMap<String, SubClass, RandomState>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SubClass {
    pub name: String,
    pub prog_ifs: HashMap<String, String, RandomState>,
}
