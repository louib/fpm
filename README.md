# fpm
fpm is a CLI tool for managing Flatpak modules and manifests.

![Tests status](https://github.com/louib/fpm/workflows/tests/badge.svg)
![Code formatting](https://github.com/louib/fpm/workflows/rustfmt/badge.svg)
[![License file](https://img.shields.io/github/license/louib/fpm)](https://github.com/louib/fpm/blob/master/LICENSE)

> **This repo is a work-in-progress and is not ready for general use.
  The command-line options, command names and file formats might change
  at any time until the project reaches version 1.0.0.**

## Features
* Flatpak manifest linting and validation.
* Bootstrapping of Flatpak manifest.
* Development workspace management, based on `flatpak-builder`.
* Module management, based on an internal database of Flatpak modules and `apt` packages.

## Installing
### Building with Cargo
```
git clone git@github.com:louib/fpm.git
cd fpm/
cargo install --path .
```

## License
MIT
