use std::path::PathBuf;
use rusqlite;

pub fn get_last_path_component<'a>(path: &'a PathBuf) -> Option<&'a str> {
    path.iter().last().and_then(|last| last.to_str())
}

/// Determines whether `path` is a hidden file given `prefix`.
pub fn is_hidden(path: &PathBuf) -> bool {
    match get_last_path_component(path) {
        Some(last_comp) => last_comp.starts_with("."),
        None => false
    }
}

pub enum HFMError {
    SQLError(rusqlite::Error),
}

impl From<rusqlite::Error> for HFMError {
    fn from(e: rusqlite::Error) -> HFMError {
        HFMError::SQLError(e)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use utils::is_hidden;

    #[test]
    fn test_is_hidden() {
        assert!(is_hidden(&PathBuf::from("/prefix/.hiddenFile")));

        assert!(!is_hidden(&PathBuf::from("/prefix/notHiddenFile")));

        assert!(!is_hidden(&PathBuf::from("/tri.cky/prefix/notHiddenFile")));
    }
}
