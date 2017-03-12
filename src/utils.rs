use std::env::current_dir;
use std::path::PathBuf;

use rand::distributions::IndependentSample;
use rand::distributions::range::Range;
use rand::thread_rng;

use rusqlite;

static CODE_LEN: u64 = 10;

pub fn absolutize(path: PathBuf) -> Option<PathBuf> {
    if path.is_absolute() {
        Some(path)
    } else {
        current_dir().ok()
        .map(|cwd| cwd.join(path))
    }
}

/// Returns a sequence by uniformly sampling `alphabet` `num` times.
///
/// ## Parameters
/// * **alphabet** -- universe of items from which to sample
/// * **num** -- number of times to sample from `alphabet`
///
/// ## Return
/// A sequence of `num` items uniformly sampled from `alphabet`
pub fn sample_with_replacement<T: Copy>(alphabet: Vec<T>, num: usize) -> Vec<T> {
    assert!(alphabet.len() > 0);
    let idxs = Range::new(0, alphabet.len());
    let mut rng = thread_rng();

    (0..num)
    .map(|_| idxs.ind_sample(&mut rng))
    .map(|idx| alphabet[idx])
    .collect()
}


pub fn generate_code() -> String {
    let alphabet_str = "abcdefghijklmnopqrstuvwxyz0123456789";
    let alphabet = alphabet_str.chars().collect::<Vec<char>>();

    sample_with_replacement(alphabet, 8)
    .iter()
    .collect()
}

pub fn get_last_path_component<'a>(path: &'a PathBuf) -> Option<&'a str> {
    path.iter().last().and_then(|last| last.to_str())
}

/// Determines whether `path` is a hidden file.
///
/// ## Parameters
/// * **path** -- filesystem path
///
/// ## Return
/// `true` when `path` corresponds to a hidden file.
pub fn is_hidden(path: &PathBuf) -> bool {
    match get_last_path_component(path) {
        Some(last_comp) => last_comp.starts_with("."),
        None => false
    }
}

#[derive(Debug)]
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
    use utils::{is_hidden, generate_code};

    #[test]
    fn test_is_hidden() {
        assert!(is_hidden(&PathBuf::from("/prefix/.hiddenFile")));

        assert!(!is_hidden(&PathBuf::from("/prefix/notHiddenFile")));

        assert!(!is_hidden(&PathBuf::from("/tri.cky/prefix/notHiddenFile")));
    }

    #[test]
    fn test_generate_code() {
        assert!(generate_code() != generate_code())
    }
}
