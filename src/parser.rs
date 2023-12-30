#![allow(clippy::inline_always)]
use crate::error::Error;
use atoi::FromRadix16;
use std::io::BufRead;
use wide::{i8x16, CmpEq};

const VENDOR_NEEDLE: [u8; 16] = *b"\0\0\0\0  \0\0\0\0\0\0\0\0\0\0";
const VENDOR_MASK: i32 = 0b00_0011 << 26;

const DEVICE_NEEDLE: [u8; 16] = *b"\t\0\0\0\0  \0\0\0\0\0\0\0\0\0";
const DEVICE_MASK: i32 = 0b100_0011 << 25;

const SUBDEVICE_NEEDLE: [u8; 16] = *b"\t\t\0\0\0\0 \0\0\0\0  \0\0\0";
const SUBDEVICE_MASK: i32 = 0b1_1000_0100_0011 << 19;

const CLASS_NEEDLE: [u8; 16] = *b"C \0\0  \0\0\0\0\0\0\0\0\0\0";
const CLASS_MASK: i32 = 0b10011 << 26;

const SUBCLASS_NEEDLE: [u8; 16] = *b"\t\0\0  \0\0\0\0\0\0\0\0\0\0\0";
const SUBCLASS_MASK: i32 = 0b1_00_11 << 27;

const PROG_IF_NEEDLE: [u8; 16] = *b"\t\t\0\0  \0\0\0\0\0\0\0\0\0\0";
const PROG_IF_MASK: i32 = 0b11_0011 << 26;

#[derive(Debug, PartialEq, Eq)]
pub enum Event<'a> {
    Vendor {
        id: u16,
        name: &'a str,
    },
    Device {
        id: u16,
        name: &'a str,
    },
    Subdevice {
        subvendor: u16,
        subdevice: u16,
        subsystem_name: &'a str,
    },
    Class {
        id: u16,
        name: &'a str,
    },
    SubClass {
        id: u16,
        name: &'a str,
    },
    ProgIf {
        id: u16,
        name: &'a str,
    },
}

pub struct Parser<R> {
    reader: R,
    buf: Vec<u8>,
    section: Section,
}

enum Section {
    Devices,
    Classes,
}

impl<R: BufRead> Parser<R> {
    pub(crate) fn new(reader: R) -> Self {
        Self {
            reader,
            buf: Vec::new(),
            section: Section::Devices,
        }
    }

    pub fn next_event(&mut self) -> Result<Option<Event>, Error> {
        self.buf.clear();

        while self.reader.read_until(b'\n', &mut self.buf)? != 0 {
            if self.buf.is_empty() || self.buf.starts_with(b"#") || self.buf == b"\n" {
                self.buf.clear();
                continue;
            }

            let buf = &self.buf[..self.buf.len() - 1];
            let vector = buf_to_vector(buf);

            let event = match self.section {
                Section::Devices => {
                    if matches_pattern(vector, CLASS_NEEDLE, CLASS_MASK) {
                        self.section = Section::Classes;

                        let id = parse_id(&buf[2..4])?;
                        let name = std::str::from_utf8(&buf[6..])?;
                        Event::Class { id, name }
                    } else if matches_pattern(vector, DEVICE_NEEDLE, DEVICE_MASK) {
                        let id = parse_id(&buf[1..5])?;
                        let name = std::str::from_utf8(&buf[7..])?;
                        Event::Device { id, name }
                    } else if matches_pattern(vector, VENDOR_NEEDLE, VENDOR_MASK) {
                        let id = parse_id(&buf[0..4])?;
                        let name = std::str::from_utf8(&buf[6..])?;
                        Event::Vendor { id, name }
                    } else if matches_pattern(vector, SUBDEVICE_NEEDLE, SUBDEVICE_MASK) {
                        let subvendor = parse_id(&buf[2..6])?;
                        let subdevice = parse_id(&buf[7..11])?;
                        let subsystem_name = std::str::from_utf8(&buf[13..])?;
                        Event::Subdevice {
                            subvendor,
                            subdevice,
                            subsystem_name,
                        }
                    } else {
                        return Err(Error::Parse(format!(
                            "Could not match device section line \"{}\"",
                            String::from_utf8(buf.to_vec())
                                .unwrap_or_else(|_| "Invalid UTF-8".to_owned())
                        )));
                    }
                }
                Section::Classes => {
                    if matches_pattern(vector, CLASS_NEEDLE, CLASS_MASK) {
                        let id = parse_id(&buf[2..4])?;
                        let name = std::str::from_utf8(&buf[6..])?;
                        Event::Class { id, name }
                    } else if matches_pattern(vector, SUBCLASS_NEEDLE, SUBCLASS_MASK) {
                        let id = parse_id(&buf[1..3])?;
                        let name = std::str::from_utf8(&buf[5..])?;
                        Event::SubClass { id, name }
                    } else if matches_pattern(vector, PROG_IF_NEEDLE, PROG_IF_MASK) {
                        let id = parse_id(&buf[2..4])?;
                        let name = std::str::from_utf8(&buf[6..])?;
                        Event::ProgIf { id, name }
                    } else {
                        return Err(Error::Parse(format!(
                            "Could not match class section line \"{}\"",
                            String::from_utf8(buf.to_vec())
                                .unwrap_or_else(|_| "Invalid UTF-8".to_owned())
                        )));
                    }
                }
            };
            return Ok(Some(event));
        }

        Ok(None)
    }
}

fn buf_to_vector(buf: &[u8]) -> i8x16 {
    let mut data = [0u8; 16];
    if buf.len() >= 16 {
        data.copy_from_slice(&buf[0..16]);
    } else {
        data[0..buf.len()].copy_from_slice(buf);
    }

    i8x16::new(unsafe { std::mem::transmute(data) })
}

fn matches_pattern(vector: i8x16, needle: [u8; 16], expected_mask: i32) -> bool {
    let needle = unsafe { std::mem::transmute(needle) };
    let needle_vector = i8x16::new(needle);
    // println!("Needle: {needle_vector:?}, expected mask {expected_mask:#032b}");
    // Assume little-endian
    // println!("Resulting mask: {resulting_mask:#032b}");
    vector.cmp_eq(needle_vector).move_mask().reverse_bits() & expected_mask == expected_mask
}

#[inline(always)]
fn parse_id<T: FromRadix16>(value: &[u8]) -> Result<T, Error> {
    let (id, offset) = T::from_radix_16(value);
    if offset == 0 {
        Err(Error::Parse(format!(
            "Could not parse integer from {:?}",
            String::from_utf8(value.to_vec())
        )))
    } else {
        Ok(id)
    }
}
#[cfg(test)]
mod tests {
    use super::Parser;
    use crate::parser::{Event, Section};
    use pretty_assertions::assert_eq;
    use std::{
        fs::File,
        io::{BufReader, Cursor},
    };

    #[test]
    fn first_events() {
        let file = File::open("./tests/pci.ids").unwrap();
        let mut parser = Parser::new(BufReader::new(file));

        assert_eq!(
            Event::Vendor {
                id: 0x0001,
                name: "SafeNet (wrong ID)"
            },
            parser.next_event().unwrap().unwrap()
        );
        assert_eq!(
            Event::Vendor {
                id: 0x0010,
                name: "Allied Telesis, Inc (Wrong ID)"
            },
            parser.next_event().unwrap().unwrap()
        );
        assert_eq!(
            Event::Device {
                id: 0x8139,
                name: "AT-2500TX V3 Ethernet"
            },
            parser.next_event().unwrap().unwrap()
        );
    }

    #[test]
    fn parse_class_line() {
        let mut parser = Parser::new(Cursor::new("C 00  Unclassified device\n"));
        assert_eq!(
            Event::Class {
                id: 0x00,
                name: "Unclassified device"
            },
            parser.next_event().unwrap().unwrap()
        );
    }

    #[test]
    fn parse_subclass_line() {
        let buf = "	01  IDE interface\n";
        let mut parser = Parser::new(Cursor::new(buf));
        parser.section = Section::Classes;
        assert_eq!(
            Event::SubClass {
                id: 0x01,
                name: "IDE interface"
            },
            parser.next_event().unwrap().unwrap()
        );
    }
    #[test]
    fn parse_subclass_line2() {
        let buf = "\t00  Non-VGA unclassified device\n";
        let mut parser = Parser::new(Cursor::new(buf));
        parser.section = Section::Classes;
        assert_eq!(
            Event::SubClass {
                id: 0x00,
                name: "Non-VGA unclassified device"
            },
            parser.next_event().unwrap().unwrap()
        );
    }

    #[test]
    fn parse_prog_if_line() {
        let buf = "		00  ISA Compatibility mode-only controller\n";
        let mut parser = Parser::new(Cursor::new(buf));
        parser.section = Section::Classes;
        assert_eq!(
            Event::ProgIf {
                id: 0x00,
                name: "ISA Compatibility mode-only controller"
            },
            parser.next_event().unwrap().unwrap()
        );
    }
}
