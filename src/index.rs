use std::collections::HashMap;
use std::path::Path;
use std::time::SystemTime;

pub struct Todo {
    author: String,
    body: String,
    severity: usize,
}

pub struct IndexedEntry {
    path: Box<Path>,
    todos: Vec<Todo>,
    last_indexed: SystemTime,
}

pub type Index = HashMap<Path, IndexedEntry>;

pub fn index_file(path: &Path) -> IndexedEntry {}
