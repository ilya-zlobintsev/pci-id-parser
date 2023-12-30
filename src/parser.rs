#![allow(clippy::inline_always)]
use crate::error::Error;
use atoi::FromRadix16;
use std::io::BufRead;

const SPLIT: &[u8] = b"  ";

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

            let event = if let Some(buf) = buf.strip_prefix(b"C ") {
                self.section = Section::Classes;

                let (id, name) = parse_split(buf)?;
                let id = parse_id(id)?;
                Event::Class { id, name }
            } else if let Some(buf) = buf.strip_prefix(b"\t\t") {
                // Subdevice
                let (prefix, name) = parse_split(buf)?;

                if let Some((subvendor, subdevice)) = split_slice_once(prefix, b" ") {
                    let subvendor = parse_id(subvendor)?;
                    let subdevice = parse_id(subdevice)?;
                    Event::Subdevice {
                        subvendor,
                        subdevice,
                        subsystem_name: name,
                    }
                } else {
                    let id = parse_id(prefix)?;
                    Event::ProgIf { id, name }
                }
            } else if let Some(buf) = buf.strip_prefix(b"\t") {
                let (id, name) = parse_split(buf)?;
                let id = parse_id(id)?;

                match self.section {
                    Section::Devices => Event::Device { id, name },
                    Section::Classes => Event::SubClass { id, name },
                }
            } else {
                let (id, name) = parse_split(buf)?;
                let id = parse_id(id)?;
                Event::Vendor { id, name }
            };
            return Ok(Some(event));
        }

        Ok(None)
    }
}

fn parse_split(buf: &[u8]) -> Result<(&[u8], &str), Error> {
    let (id, raw_name) = split_slice_once(buf, SPLIT).ok_or_else(|| {
        Error::Parse(format!(
            "missing delimiter in line {:?}",
            String::from_utf8(buf.to_vec())
        ))
    })?;

    let name = std::str::from_utf8(raw_name)?;
    Ok((id, name))
}

#[inline(always)]
fn split_slice_once<'a>(buf: &'a [u8], separator: &[u8]) -> Option<(&'a [u8], &'a [u8])> {
    buf.windows(separator.len())
        .position(|window| window == separator)
        .map(|split_index| (&buf[0..split_index], &buf[split_index + separator.len()..]))
}

#[inline(always)]
fn parse_id(value: &[u8]) -> Result<u16, Error> {
    let (id, offset) = u16::from_radix_16(value);
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
        parser.section = Section::Classes;
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
