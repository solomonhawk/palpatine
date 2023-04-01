use log::*;
use std::collections::HashMap;
use std::error;
use std::fs;
use std::fs::DirBuilder;
use std::fs::DirEntry;
use std::path::Path;
use std::process;
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

pub fn read_index(repo: &Repository) -> Result<Index, Box<dyn error::Error>> {
    let workdir = repo
        .workdir()
        .expect("ERROR: could not find workdir, is this a git directory or subdirectory?");

    match std::fs::read_to_string(Path::new(workdir).join(".palpatine/index.json")) {
        Ok(index_contents) => Ok(serde_json::from_str(&index_contents)?),
        Err(_) => Ok(HashMap::new()),
    }
}

pub fn write_index(index: &Index, repo: &Repository) -> Result<(), Box<dyn error::Error>> {
    let workdir = repo
        .workdir()
        .expect("ERROR: could not find workdir, is this a git directory or subdirectory?");
    let palpatine_dir = Path::new(workdir).join(".palpatine");

    if !palpatine_dir.is_dir() {
        DirBuilder::new().create(palpatine_dir)?;
    }

    std::fs::write(
        Path::new(workdir).join(".palpatine/index.json"),
        serde_json::to_string_pretty(&index).unwrap(),
    )?;

    Ok(())
}

pub fn report_index(index: &Index) {
    for entry in index.values() {
        if entry.todos.len() > 0 {
            println!(
                "Found {} Todos in {}",
                entry.todos.len(),
                entry.relative_path.display()
            );

            for todo in &entry.todos {
                println!(
                    "    TODO({author}): {body}",
                    author = todo.author,
                    body = todo.body
                );
            }
        }
    }
}

pub fn index_file(
    index: &mut Index,
    entry: &DirEntry,
    repo: &Repository,
    indexed_count: &mut usize,
) -> Result<(), Box<dyn error::Error>> {
    let mut todos: Vec<Todo> = vec![];

    let last_modified = entry
        .metadata()
        .map_err(|err| {
            error!(
                "could not get file metadata for {filename:?} ({err})",
                filename = entry.file_name()
            );
            process::exit(1);
        })
        .unwrap()
        .modified()
        .unwrap();

    if let Some(existing_entry) = index.get(entry.path().as_path()) {
        if existing_entry.last_indexed >= last_modified {
            debug!(
                "Skipping {path}, index is up to date",
                path = entry.path().display()
            );
            return Ok(());
        }
    }

    debug!("Indexing {path}", path = entry.path().display());

    let file_contents = fs::read_to_string(entry.path())?;
    let entry_path = entry.path();
    let relative_path = entry_path.strip_prefix(repo.workdir().unwrap()).unwrap();

    // TODO: better pattern matching for todos
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

    *indexed_count += 1;

    index.insert(
        entry.path().into(),
        IndexedEntry {
            path: entry.path().into(),
            relative_path: relative_path.into(),
            todos,
            last_indexed: SystemTime::now(),
        },
    );

    Ok(())
}

fn extract_todo(
    line: &str,
    row: usize,
    col: usize,
    path: &Path,
    repo: &Repository,
) -> Result<Option<Todo>, Box<dyn error::Error>> {
    let blame = repo.blame_file(path, None);

    // TODO: add log levels, check if error is due to file not being in the git index
    if blame.is_err() {
        warn!("Could not get blame info for {path:?}");
        return Ok(None);
    }

    let mut author = "Unknown".to_string();
    let mut body = None;

    if let Some(blame) = blame.unwrap().get_line(row) {
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

// TODO: better pattern matching for comments
fn is_comment(line: &str) -> bool {
    let trimmed = line.trim();

    trimmed.starts_with("#") || trimmed.starts_with("//")
}
