use std::path::PathBuf;

/// Returns an absolute path given `path`.
pub fn absolutize(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        let path_str = path.to_str().unwrap();
        PathBuf::from(String::from("/") + path_str)
    }
}

/// Determines whether `path` is a hidden file given `prefix`.
pub fn is_hidden(prefix: &PathBuf, path: &PathBuf) -> bool {
    let hidden_prefix = ".";
    match path.strip_prefix(prefix).map(|p| {
        let without_prefix = p.to_str().unwrap().to_owned().clone();
        without_prefix.starts_with(hidden_prefix)
    }) {
        Ok(result) => result,
        Err(_) => false
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use utils::is_hidden;

    #[test]
    fn test_is_hidden() {

        assert!(is_hidden(
                &PathBuf::from("/prefix"),
                &PathBuf::from("/prefix/.hiddenFile")));

        assert!(!is_hidden(
                &PathBuf::from("/prefix"),
                &PathBuf::from("/prefix/notHiddenFile")));

        assert!(!is_hidden(
                &PathBuf::from("/prefix"),
                &PathBuf::from("/tri.cky/prefix/notHiddenFile")));
    }
}
