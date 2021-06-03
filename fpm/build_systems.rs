pub fn get_build_system(file_path: String) -> Option<String> {
    if file_path.ends_with("Makefile") {
        return Some("make".to_string());
    }
    if file_path.ends_with("CMakeLists.txt") {
        return Some("cmake".to_string());
    }
    if file_path.ends_with("autogen.sh") || file_path.ends_with("autogen") {
        return Some("autotools".to_string());
    }
    if file_path.ends_with("bootstrap.sh") || file_path.ends_with("bootstrap") {
        return Some("autotools".to_string());
    }
    if file_path.ends_with(".pro") {
        return Some("qmake".to_string());
    }
    if file_path.ends_with("meson.build") {
        return Some("meson".to_string());
    }
    if file_path.ends_with("Cargo.toml") {
        return Some("cargo".to_string());
    }
    None
}
