use anyhow::{anyhow, Result};

pub fn validate_safe_path(path: &str) -> Result<()> {
    if path.is_empty() {
        return Err(anyhow!("Empty path is not allowed"));
    }
    let p = std::path::Path::new(path);
    for component in p.components() {
        if let std::path::Component::ParentDir = component {
            return Err(anyhow!("Path traversal detected in path: {}", path));
        }
    }
    Ok(())
}

pub fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|ch| if ch == '^' || ch == '-' { '_' } else { ch })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_safe_path() {
        assert!(validate_safe_path("safe/path/to/file.txt").is_ok());
        assert!(validate_safe_path("normal.txt").is_ok());
        assert!(validate_safe_path("../traversal.txt").is_err());
        assert!(validate_safe_path("path/../traversal.txt").is_err());
        assert!(validate_safe_path("").is_err());
    }

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("^GSPC"), "_GSPC");
        assert_eq!(sanitize_name("btc-usd"), "btc_usd");
        assert_eq!(sanitize_name("normal_ticker"), "normal_ticker");
    }
}
