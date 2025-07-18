# fsquirrel

[![Documentation](https://docs.rs/fsquirrel/badge.svg)](https://docs.rs/fsquirrel)
[![crates.io](https://img.shields.io/crates/v/fsquirrel.svg)](https://crates.io/crates/fsquirrel)
[![Build](https://github.com/finnbear/fsquirrel/actions/workflows/build.yml/badge.svg)](https://github.com/finnbear/fsquirrel/actions/workflows/build.yml) 

A small library for stashing custom [extended attribute](https://en.wikipedia.org/wiki/Extended_file_attributes) key-value pairs into file metadata, avoiding the need for [sidecar files](https://en.wikipedia.org/wiki/Sidecar_file).

These attributes are may be preserved when files are copied to a compatible file system, but are always discarded in cases such as uploading to the internet. More specifically:
- On Windows and MacOS, `fs::copy` can preserve them.
- On Linux, only `/usr/bin/cp --preserve=xattr` can preserve them.

## Platform support

On **Unix (Android, Linux, MacOS, FreeBSD, NetBSD)**, this is a wrapper around [xattr](https://crates.io/crates/xattr), hard-coded to the `user.` namespace.

On **Windows**, this uses [NTFS Alternate Data Streams](https://en.wikipedia.org/wiki/NTFS#Alternate_data_stream_(ADS)). It's not compatible with OS/2 exended attributes.

## Features

- [x] Get 🕳️🌰🐿️
- [x] Set 🐿️💨 🌰
- [x] Remove 🌰🐿️💨 🕳️
- [x] List 🥜🌰🥔

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.