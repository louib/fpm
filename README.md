# fpm
fpm is a CLI tool for managing Flatpak modules and manifests.

![Tests status](https://github.com/louib/fpm/workflows/tests/badge.svg)
![Code formatting](https://github.com/louib/fpm/workflows/formatting/badge.svg)
[![dependency status](https://deps.rs/repo/github/louib/fpm/status.svg)](https://deps.rs/repo/github/louib/fpm)
[![License file](https://img.shields.io/github/license/louib/fpm)](https://github.com/louib/fpm/blob/master/LICENSE)

> **This repo is a work-in-progress and is not ready for general use.
  The command-line options, command names and file formats might change
  at any time until the project reaches version 1.0.0.**

`fpm` focuses on managing the modules described in Flatpak manifests. If you are
looking for a tool to use Flatpak manifests for local development, have a
look at [fenv](https://gitlab.gnome.org/ZanderBrown/fenv).

`fpm` uses the [`flatpak-rs`](https://github.com/louib/flatpak-rs) to parse
Flatpak manifests.

## Features
* `install` modules from a database of Flatpak modules
* `import` modules from other package managers (currently `cargo` and `vcpkg` are supported).
* `update` modules (using the `x-checker-data` field).

## Installing
`fpm` is currently not published on crates.io. You will need to install it locally with cargo.

### Building with Cargo
```
git clone git@github.com:louib/fpm.git
cd fpm/
cargo install --path .
```

## License
MIT
