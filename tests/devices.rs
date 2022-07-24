use pciid_parser::Database;
use pretty_assertions::assert_eq;

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
    use pciid_parser::schema::DeviceInfo;

    let db = Database::get_online().unwrap();

    let device_info = db.get_device_info("C 0c", "03", "fe", "");
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
