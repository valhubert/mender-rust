mender-rust
-----------

mender-rust is a small command-line utility built in Rust to
interact with a Mender server.

Licensed under MIT.

This small project is mainly to play with Rust and may not
be suitable for real use cases.

### Features

Currently you can:

 * login;
 * deploy an update to a group of devices;
 * get the internal id of a device based on its 'SerialNumber' attribute;
 * get the info of a device based on its internal id;
 * count the number of devices per artifact.

### Building


mender-rust is written in Rust, so you'll need to have
[Rust](https://www.rust-lang.org/) installed in order to compile it.

To build:

```
$ git clone https://github.com/valhubert/mender-rust
$ cd mender-rust
$ cargo build --release
```
