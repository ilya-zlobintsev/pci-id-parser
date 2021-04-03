# PCI ID Parser

This is a tool/library that lets you use a PCI ID database, such as one shipped with Linux distros or from https://pci-ids.ucw.cz/.

It can be used standalone to generate a JSON file of the database, which is easier to use and more portable than the original format.
You can also use it as a library in Rust. This was originaly made for [LACT](https://github.com/ilyazzz/LACT/), and you can see how it's used [here](https://github.com/ilyazzz/LACT/blob/master/daemon/src/gpu_controller.rs).

If you just want a json, you can get one from https://pci.endpoint.ml/.
