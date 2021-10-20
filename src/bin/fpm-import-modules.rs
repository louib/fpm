use std::env;
use std::path;

fn main() {
    fpm::logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Requires 1 argument: the path of the directory to use for the import.");
    }

    let path = &args[1];

    let file_paths = match fpm::utils::get_all_paths(path::Path::new(path)) {
        Ok(paths) => paths,
        Err(message) => {
            eprintln!("Could not get the file paths :sad: {}", message);
            return;
        }
    };
    for file_path in file_paths.iter() {

    }
}
