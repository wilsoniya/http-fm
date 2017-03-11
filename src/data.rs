use std::cmp::Ordering;
use std::path::PathBuf;


#[derive(Serialize)]
pub struct DirContext {
    pub dpath: String,
    pub items: Vec<DirItem>,
    pub code: String,
}

#[derive(Serialize, Eq)]
pub struct DirItem {
    pub is_dir: bool,
    pub name: String,
    pub path: String,

}

impl Ord for DirItem {
    fn cmp(&self, other: &DirItem) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialEq for DirItem {
    fn eq(&self, other: &DirItem) -> bool {
        self.name == other.name
    }
}

impl PartialOrd for DirItem {
    fn partial_cmp(&self, other: &DirItem) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug)]
pub struct CodePath {
    pub code: String,
    pub path: PathBuf,
    pub expiration: Option<i64>,
    pub hits: u64,
}
