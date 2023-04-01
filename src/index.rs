use std::collections::HashMap;
use std::error;
use std::fs;
use std::fs::DirBuilder;
use std::fs::DirEntry;
use std::fs::File;
use std::io::Write;
use std::io::{self, ErrorKind};
use std::path::Path;
use std::time::SystemTime;

use git2::Repository;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Todo {
    line: usize,
    col: usize,
    author: String,
    body: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexedEntry {
    pub path: Box<Path>,
    pub relative_path: Box<Path>,
    todos: Vec<Todo>,
    last_indexed: SystemTime,
}

#[derive(Serialize, Deserialize)]
pub struct IndexData {
    index: Index,
}

pub type Index = HashMap<Box<Path>, IndexedEntry>;

pub fn write_index(index: &Index, repo: &Repository) -> Result<(), Box<dyn error::Error>> {
    let workdir = repo
        .workdir()
        .expect("ERROR: could not find workdir, is this a git directory or subdirectory?");

    let index_str = serde_json::to_string(index).unwrap();
    let dir_builder = DirBuilder::new();

    dir_builder.create(Path::new(workdir).join(".palpatine"))?;

    std::fs::write(
        Path::new(workdir).join(".palpatine/index.json"),
        serde_json::to_string_pretty(&index_str).unwrap(),
    )?;

    Ok(())
}

pub fn index_file(
    entry: &DirEntry,
    repo: &Repository,
) -> Result<IndexedEntry, Box<dyn error::Error>> {
    let mut todos: Vec<Todo> = vec![];

    let last_modified = entry
        .metadata()
        .map_err(|err| {
            eprintln!(
                "ERROR: could not get file metadata for {filename:?} ({err})",
                filename = entry.file_name()
            );
        })
        .unwrap()
        .modified();

    println!("repo path: {:?}", repo.workdir());

    let file_contents = fs::read_to_string(entry.path())?;
    let entry_path = entry.path();
    let relative_path = entry_path.strip_prefix(repo.workdir().unwrap()).unwrap();

    for (row, line) in file_contents.lines().enumerate() {
        if let Some(col) = line.find("TODO:") {
            if !is_comment(&line) {
                continue;
            }

            extract_todo(line, row, col, &relative_path, &repo)?.map(|todo| {
                todos.push(todo);
            });
        }
    }

    Ok(IndexedEntry {
        path: entry.path().into(),
        relative_path: relative_path.into(),
        todos,
        last_indexed: SystemTime::now(),
    })
}

fn extract_todo(
    line: &str,
    row: usize,
    col: usize,
    path: &Path,
    repo: &Repository,
) -> Result<Option<Todo>, Box<dyn error::Error>> {
    let blame = repo.blame_file(path, None)?;

    assert!(
        blame.len() == 1,
        "ERROR: blame entry should only contain one hunk"
    );

    let mut author = "Unknown".to_string();
    let mut body = None;

    if let Some(blame) = blame.get_line(row) {
        blame
            .orig_signature()
            .name()
            .map(|name| author = name.to_string());
    }

    if let Some((_, todo_body)) = line.split_once("TODO:") {
        body = Some(todo_body.trim().to_string());
    }

    if body.is_none() {
        return Ok(None);
    }

    Ok(Some(Todo {
        line: row + 1,
        col,
        author,
        body: body.unwrap(),
    }))
}

fn is_comment(line: &str) -> bool {
    let trimmed = line.trim();

    trimmed.starts_with("#") || trimmed.starts_with("//")
}
