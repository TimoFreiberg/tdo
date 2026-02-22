use std::collections::HashSet;
use std::fs::{self, File, TryLockError};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use crate::todo::{self, Frontmatter, Todo};
use crate::util::{generate_id, slugify};

pub struct Store {
    dir: PathBuf,
    _lock_file: File,
    cache: Vec<Todo>,
    /// Number of `.md` files that failed to parse during load.
    pub skipped: usize,
}

impl Store {
    /// Open a store, creating the directory if it doesn't exist.
    /// Acquires an advisory lock to prevent concurrent access.
    pub fn open(dir: &Path) -> Result<Self> {
        fs::create_dir_all(dir)
            .with_context(|| format!("failed to create todo directory: {}", dir.display()))?;

        let lock_path = dir.join(".lock");
        let lock_file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&lock_path)
            .with_context(|| format!("failed to create lock file: {}", lock_path.display()))?;
        lock_file.try_lock().map_err(|e| match e {
            TryLockError::WouldBlock => {
                anyhow!("another tdo process is using {}", dir.display())
            }
            TryLockError::Error(io_err) => {
                anyhow::Error::from(io_err).context("failed to acquire lock")
            }
        })?;

        let (cache, skipped) = load_all_todos(dir)?;

        Ok(Store {
            dir: dir.to_path_buf(),
            _lock_file: lock_file,
            cache,
            skipped,
        })
    }

    /// Resolve the todo directory: use the override if provided, otherwise `.todo/` in cwd.
    pub fn resolve_dir(override_dir: Option<&Path>) -> PathBuf {
        match override_dir {
            Some(d) => d.to_path_buf(),
            None => PathBuf::from(".todo"),
        }
    }

    /// All todos, sorted by created timestamp ascending.
    pub fn list_all(&self) -> &[Todo] {
        &self.cache
    }

    /// Only open todos (references into the cache).
    pub fn list_open(&self) -> Vec<&Todo> {
        self.cache.iter().filter(|t| t.is_open()).collect()
    }

    /// Find a single todo by ID or unique prefix.
    pub fn find_by_id(&self, id: &str) -> Result<Todo> {
        let idx = self.find_index(id)?;
        Ok(self.cache[idx].clone())
    }

    /// Create a new todo file, returning the assigned ID.
    pub fn create(&mut self, fm: &Frontmatter, body: Option<&str>) -> Result<String> {
        let existing_ids: HashSet<String> = self.cache.iter().map(|t| t.id.clone()).collect();
        let id = generate_id(|candidate| existing_ids.contains(candidate))?;
        let slug = slugify(&fm.title);
        let filename = if slug.is_empty() {
            format!("{id}.md")
        } else {
            format!("{id}-{slug}.md")
        };
        let content = todo::render_file(fm, body)?;
        let path = self.dir.join(&filename);
        fs::write(&path, &content)
            .with_context(|| format!("failed to write: {}", path.display()))?;

        self.cache.push(Todo {
            id: id.clone(),
            filename,
            frontmatter: fm.clone(),
            body: body.map(|s| s.to_string()),
        });
        self.cache
            .sort_by(|a, b| a.frontmatter.created.cmp(&b.frontmatter.created));

        Ok(id)
    }

    /// Overwrite an existing todo file and update the cache.
    pub fn save(&mut self, todo: &Todo) -> Result<()> {
        let idx = self
            .cache
            .iter()
            .position(|t| t.id == todo.id)
            .ok_or_else(|| anyhow!("no todo in store with id '{}'", todo.id))?;
        let content = todo::render_file(&todo.frontmatter, todo.body.as_deref())?;
        let path = self.dir.join(&todo.filename);
        fs::write(&path, &content)
            .with_context(|| format!("failed to write: {}", path.display()))?;
        self.cache[idx] = todo.clone();
        Ok(())
    }

    /// Delete a todo file by ID (or prefix). Returns the deleted todo.
    pub fn delete(&mut self, id: &str) -> Result<Todo> {
        let idx = self.find_index(id)?;
        let todo = self.cache.remove(idx);
        let path = self.dir.join(&todo.filename);
        fs::remove_file(&path)
            .with_context(|| format!("failed to delete: {}", path.display()))?;
        Ok(todo)
    }

    /// Return the full path for a todo file.
    pub fn path_for(&self, todo: &Todo) -> PathBuf {
        self.dir.join(&todo.filename)
    }

    /// Re-read a specific todo from disk and update the cache.
    pub fn refresh(&mut self, id: &str) -> Result<()> {
        let idx = self.find_index(id)?;
        let path = self.dir.join(&self.cache[idx].filename);
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("failed to read: {}", path.display()))?;
        let (fm, body) = todo::parse_file(&raw)?;
        let cached = &mut self.cache[idx];
        cached.frontmatter = fm;
        cached.body = body;
        Ok(())
    }

    fn find_index(&self, id: &str) -> Result<usize> {
        if id.is_empty() {
            return Err(anyhow!("todo id must not be empty"));
        }
        let matches: Vec<usize> = self
            .cache
            .iter()
            .enumerate()
            .filter(|(_, t)| t.id.starts_with(id))
            .map(|(i, _)| i)
            .collect();
        match matches.len() {
            0 => Err(anyhow!("no todo found with id '{id}'")),
            1 => Ok(matches[0]),
            _ => {
                let ids: Vec<&str> =
                    matches.iter().map(|&i| self.cache[i].id.as_str()).collect();
                Err(anyhow!("ambiguous id '{id}': matches {}", ids.join(", ")))
            }
        }
    }
}

/// Load all valid todos from `dir`. Returns the list and the count of
/// `.md` files that looked like todo files but failed to parse.
fn load_all_todos(dir: &Path) -> Result<(Vec<Todo>, usize)> {
    let mut todos = Vec::new();
    let mut skipped: usize = 0;
    let entries = fs::read_dir(dir)
        .with_context(|| format!("failed to read directory: {}", dir.display()))?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let filename = entry.file_name().to_string_lossy().to_string();
        if let Some(id) = id_from_filename(&filename) {
            let raw = fs::read_to_string(&path)
                .with_context(|| format!("failed to read: {}", path.display()))?;
            match todo::parse_file(&raw) {
                Ok((fm, body)) => {
                    todos.push(Todo {
                        id: id.to_string(),
                        filename,
                        frontmatter: fm,
                        body,
                    });
                }
                Err(e) => {
                    eprintln!("warning: skipping {}: {e}", path.display());
                    skipped += 1;
                }
            }
        }
    }
    todos.sort_by(|a, b| a.frontmatter.created.cmp(&b.frontmatter.created));
    Ok((todos, skipped))
}

/// Extract the hex ID from a filename like "a3f9-fix-the-login-bug.md".
fn id_from_filename(name: &str) -> Option<&str> {
    let stem = name.strip_suffix(".md")?;
    let id_part = stem.split('-').next()?;
    if id_part.len() == 4 && id_part.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(id_part)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_from_filename_with_slug() {
        assert_eq!(id_from_filename("a3f9-fix-the-bug.md"), Some("a3f9"));
    }

    #[test]
    fn id_from_filename_no_slug() {
        assert_eq!(id_from_filename("a3f9.md"), Some("a3f9"));
    }

    #[test]
    fn id_from_filename_invalid() {
        assert_eq!(id_from_filename("not-a-todo.md"), None);
        assert_eq!(id_from_filename("abc.md"), None); // too short
        assert_eq!(id_from_filename("abcde.md"), None); // too long
        assert_eq!(id_from_filename("zzzz.txt"), None); // wrong extension
    }
}
