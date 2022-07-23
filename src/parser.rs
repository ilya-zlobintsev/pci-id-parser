use std::collections::HashMap;

use crate::{
    error::Error,
    schema::{Class, SubClass, SubDeviceId},
};

pub fn parse_subdevice_id(buf: &mut String) -> Result<(String, SubDeviceId), Error> {
    let (mut sub, name) = drain_id_and_name(buf)?;

    let sub_offset = sub.find(' ').unwrap_or(sub.len());
    let start = get_actual_buf_start(&sub);
    let subvendor = sub.drain(start..sub_offset).collect();
    let start = get_actual_buf_start(&sub);
    let subdevice = sub.drain(start..).collect();

    Ok((
        name,
        SubDeviceId {
            subvendor,
            subdevice,
        },
    ))
}

pub fn parse_class(buf: &mut String) -> Result<(String, Class), Error> {
    let mut drain = buf.drain(2..);

    let mut id = String::new();
    while let Some(c) = drain.next() {
        if c.is_ascii_whitespace() {
            // Skip second space
            drain
                .next()
                .ok_or_else(|| Error::Parse("Unexpected end of class line".to_owned()))?;
            break;
        }

        id.push(c);
    }

    let mut name = String::new();

    while let Some(c) = drain.next() {
        if c == '\n' {
            break;
        }
        name.push(c);
    }

    let class = Class {
        name,
        subclasses: HashMap::new(),
    };

    Ok((id, class))
}

pub fn parse_subclass(buf: &mut String) -> Result<(String, SubClass), Error> {
    let (id, name) = drain_id_and_name(buf)?;

    let subclass = SubClass {
        name,
        prog_ifs: HashMap::new(),
    };

    Ok((id, subclass))
}

pub fn parse_prog_if(buf: &mut String) -> Result<(String, String), Error> {
    drain_id_and_name(buf)
}

const SPLIT: &str = "  ";

pub fn drain_id_and_name(buf: &mut String) -> Result<(String, String), Error> {
    let start = get_actual_buf_start(buf);
    let split_offset = buf.find(SPLIT).ok_or_else(|| {
        Error::Parse(format!(
            "missing delimiter between vendor id and name in line {buf}"
        ))
    })?;
    let mut id: String = buf.drain(start..split_offset).collect();
    id.make_ascii_lowercase();

    let start = get_actual_buf_start(buf);
    let end = buf.find('\n').unwrap_or(buf.len());
    let name = buf.drain(start..end).collect();

    Ok((id, name))
}

fn get_actual_buf_start(buf: &str) -> usize {
    for (i, c) in buf.chars().enumerate() {
        if !c.is_whitespace() {
            return i;
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_prog_if;

    use super::{parse_class, parse_subclass};
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_class_line() {
        let mut buf = "C 00  Unclassified device".to_owned();

        let (id, class) = parse_class(&mut buf).unwrap();

        assert_eq!(id, "00");
        assert_eq!(class.name, "Unclassified device")
    }

    #[test]
    fn parse_subclass_line() {
        let mut buf = "	01  IDE interface".to_owned();

        let (id, subclass) = parse_subclass(&mut buf).unwrap();

        assert_eq!(id, "01");
        assert_eq!(subclass.name, "IDE interface");
    }

    #[test]
    fn parse_prog_if_line() {
        let mut buf = "		00  ISA Compatibility mode-only controller".to_owned();

        let (id, name) = parse_prog_if(&mut buf).unwrap();

        assert_eq!(id, "00");
        assert_eq!(name, "ISA Compatibility mode-only controller");
    }
}
