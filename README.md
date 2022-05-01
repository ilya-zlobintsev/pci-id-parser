# PCI ID Parser

[![Crates.io](https://img.shields.io/crates/v/pciid-parser)](https://crates.io/crates/pciid-parser)
[![Docs.rs](https://docs.rs/pciid-parser/badge.svg)](https://docs.rs/pciid-parser/)

This is a library that lets you use a PCI ID database, such as one shipped with Linux distros or from https://pci-ids.ucw.cz/.
It can either read the locally installed file or fetch one from the website.

## Usage

```rust
use pciid_parser::Database;

let db = Database::get_online().unwrap();
```
Alternatively, you can read the local DB:
```rust
let db = Database::read().unwrap();
```
Get full device and subdevice info:
```rust
let info = db.get_device_info("1002", "67DF", "1DA2", "E387");
```
