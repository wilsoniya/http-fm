use std::cmp::Ordering;

#[derive(Serialize)]
pub struct DirContext {
    pub dpath: String,
    pub items: Vec<DirItem>
}

#[derive(Serialize, Eq)]
pub struct DirItem {
    pub is_dir: bool,
    pub item: String
}

impl Ord for DirItem {
    fn cmp(&self, other: &DirItem) -> Ordering {
        self.item.cmp(&other.item)
    }
}

impl PartialEq for DirItem {
    fn eq(&self, other: &DirItem) -> bool {
        self.item == other.item
    }
}

impl PartialOrd for DirItem {
    fn partial_cmp(&self, other: &DirItem) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
