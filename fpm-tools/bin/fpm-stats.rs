use std::collections::HashSet;
use std::path;
use std::fs;
use std::env;
use std::process::exit;

use fpm::flatpak_manifest::{FlatpakManifest, FlatpakModule, FlatpakModuleDescription};

fn main() {
    fpm::logger::init();
}
