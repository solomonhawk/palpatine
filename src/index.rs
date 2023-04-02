use log::*;
use std::collections::HashMap;
use std::error;
use std::fs;
use std::fs::DirBuilder;
use std::fs::DirEntry;
use std::path::Path;
use std::process;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::config::Config;

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

pub fn read_index(config: &Config) -> Result<Index, Box<dyn error::Error>> {
    match std::fs::read_to_string(config.index_file_path()) {
        Ok(index_contents) => Ok(serde_json::from_str(&index_contents)?),
        Err(_) => Ok(HashMap::new()),
    }
}

pub fn write_index(index: &Index, config: &Config) -> Result<(), Box<dyn error::Error>> {
    if !config.index_dir().is_dir() {
        DirBuilder::new().create(config.index_dir())?;
    }

    std::fs::write(
        config.index_file_path(),
        serde_json::to_string_pretty(&index).unwrap(),
    )?;

    Ok(())
}

pub fn delete_index(config: &Config) -> Result<(), Box<dyn error::Error>> {
    std::fs::remove_dir_all(config.index_dir())?;

    Ok(())
}

pub fn report_index(index: &Index) {
    for entry in index.values() {
        if entry.todos.len() > 0 {
            println!("{} TODOs in {}", entry.todos.len(), entry.path.display());

            for todo in &entry.todos {
                println!(
                    "    {line}: TODO({author}): {body}",
                    line = todo.line,
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
    config: &Config,
    indexed_count: &mut usize,
) -> Result<(), Box<dyn error::Error>> {
    match should_index(index, entry, config) {
        IndexCommand::Skip(reason) => {
            if reason.is_some() {
                debug!(
                    "Skipping {path}, {reason}",
                    path = entry.path().display(),
                    reason = reason.unwrap()
                );
            }

            Ok(())
        }

        IndexCommand::Scan(file_contents) => {
            let mut todos: Vec<Todo> = vec![];

            debug!("Indexing {path}", path = entry.path().display());

            let entry_path = entry.path();
            let relative_path = entry_path.strip_prefix(config.root_dir()).unwrap();

            // TODO: better pattern matching for todos
            for (row, line) in file_contents.lines().enumerate() {
                if let Some(col) = line.find("TODO:") {
                    if !is_comment(&line) {
                        continue;
                    }

                    extract_todo(line, row, col, &relative_path, config)?.map(|todo| {
                        todos.push(todo);
                    });
                }
            }

            *indexed_count += 1;

            index.insert(
                relative_path.into(),
                IndexedEntry {
                    path: entry.path().into(),
                    relative_path: relative_path.into(),
                    todos,
                    last_indexed: SystemTime::now(),
                },
            );

            Ok(())
        }
    }
}

type Reason = Option<String>;
type FileContents = String;

enum IndexCommand {
    Skip(Reason),
    Scan(FileContents),
}

fn should_index(index: &Index, entry: &DirEntry, config: &Config) -> IndexCommand {
    if entry.path().ends_with(".palpatine/index.json") {
        return IndexCommand::Skip(None);
    }

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

    let entry_path = entry.path();
    let relative_path = entry_path.strip_prefix(config.root_dir()).unwrap();

    if let Some(existing_entry) = index.get(relative_path) {
        if existing_entry.last_indexed >= last_modified {
            debug!(
                "Skipping {path}, index is up to date",
                path = entry.path().display()
            );
            return IndexCommand::Skip(Some(String::from("already up to date")));
        }
    }

    let file = fs::read(entry.path());

    if file.is_err() {
        error!(
            "Failed to read file data for {path}",
            path = entry.path().display()
        );
        process::exit(1);
    }

    let file_data = file.unwrap();
    let file_contents = std::str::from_utf8(&file_data);

    if file_contents.is_err() {
        debug!(
            "Skipping {path}, contents are not UTF-8",
            path = entry.path().display()
        );
        return IndexCommand::Skip(Some(String::from("contents are not UTF-8")));
    }

    IndexCommand::Scan(file_contents.unwrap().into())
}

fn extract_todo(
    line: &str,
    row: usize,
    col: usize,
    path: &Path,
    config: &Config,
) -> Result<Option<Todo>, Box<dyn error::Error>> {
    let blame = config.repo.blame_file(path, None);

    // TODO: check if error is due to file not being in the git index
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

    // TODO: capture full body even for multi-line comments?
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
