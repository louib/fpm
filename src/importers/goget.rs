pub struct GoGet {}

impl GoGet {
    pub fn file_path_matches(path: &str) -> bool {
        false
    }

    pub fn parse(manifest_path: &str, manifest_content: &str) -> Result<GoGet, String> {
        Err("Not implemented yet.".to_string())
    }
}

#[cfg(test)]
mod manifest_tests {
    use super::*;

    #[test]
    pub fn test_parse_manifest() {}
}
