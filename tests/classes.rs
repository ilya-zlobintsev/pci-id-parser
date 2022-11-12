use pciid_parser::Database;
use pretty_assertions::assert_eq;

#[test]
fn get_basic_class() {
    let db = Database::read().unwrap();
    let class = db.classes.get("00").unwrap();

    assert_eq!(class.name, "Unclassified device");
}

#[test]
fn get_infiniband_subclass() {
    let db = Database::read().unwrap();
    let subclass = db.classes.get("02").unwrap().subclasses.get("07").unwrap();

    assert_eq!(subclass.name, "Infiniband controller");
}

#[test]
fn get_vga_controller_if_prog() {
    let db = Database::read().unwrap();
    let prog_if_name = db
        .classes
        .get("03")
        .unwrap()
        .subclasses
        .get("00")
        .unwrap()
        .prog_ifs
        .get("00")
        .unwrap();

    assert_eq!(prog_if_name, "VGA controller");
}

#[cfg(feature = "online")]
#[test]
fn get_usb_device_if_prog() {
    let db = Database::get_online().unwrap();

    let class = db.classes.get("0c").unwrap();
    assert_eq!(class.name, "Serial bus controller");

    let subclass = class.subclasses.get("03").unwrap();
    assert_eq!(subclass.name, "USB controller");

    let prog_if = subclass.prog_ifs.get("fe").unwrap();
    assert_eq!(prog_if, "USB Device");
}
