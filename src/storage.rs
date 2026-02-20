use std::fs::{self, File, TryLockError};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};

use crate::todo::{self, Frontmatter, Todo};
use crate::util::{generate_id, slugify};

pub struct Store {
    pub dir: PathBuf,
    _lock: File,
    cache: Option<Vec<Todo>>,
}

impl Store {
    /// Open a store, creating the directory if it doesn't exist.
    /// Acquires an exclusive lock to prevent concurrent access.
    pub fn open(dir: &Path) -> Result<Self> {
        fs::create_dir_all(dir)
            .with_context(|| format!("failed to create todo directory: {}", dir.display()))?;

        let lock_path = dir.join(".lock");
        let lock_file = File::create(&lock_path)
            .with_context(|| format!("failed to create lock file: {}", lock_path.display()))?;
        match lock_file.try_lock() {
            Ok(()) => {}
            Err(TryLockError::WouldBlock) => {
                bail!("another tdo instance is using {}", dir.display());
            }
            Err(TryLockError::Error(e)) => {
                return Err(e)
                    .with_context(|| format!("failed to acquire lock: {}", lock_path.display()));
            }
        }

        Ok(Store {
            dir: dir.to_path_buf(),
            _lock: lock_file,
            cache: None,
        })
    }

    /// Resolve the todo directory: use the override if provided, otherwise `.todo/` in cwd.
    pub fn resolve_dir(override_dir: Option<&Path>) -> PathBuf {
        match override_dir {
            Some(d) => d.to_path_buf(),
            None => PathBuf::from(".todo"),
        }
    }

    /// Ensure the in-memory cache is populated from disk.
    fn ensure_cache(&mut self) -> Result<()> {
        if self.cache.is_none() {
            self.cache = Some(self.load_from_disk()?);
        }
        Ok(())
    }

    /// Read all todos from the filesystem.
    fn load_from_disk(&self) -> Result<Vec<Todo>> {
        let mut todos = Vec::new();
        let entries = fs::read_dir(&self.dir)
            .with_context(|| format!("failed to read directory: {}", self.dir.display()))?;
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
                    }
                }
            }
        }
        todos.sort_by(|a, b| a.frontmatter.created.cmp(&b.frontmatter.created));
        Ok(todos)
    }

    /// Invalidate the cache, forcing a reload from disk on next access.
    pub fn invalidate(&mut self) {
        self.cache = None;
    }

    /// List all todos, sorted by created timestamp ascending.
    pub fn list_all(&mut self) -> Result<Vec<Todo>> {
        self.ensure_cache()?;
        Ok(self.cache.as_ref().unwrap().clone())
    }

    /// List only open todos.
    pub fn list_open(&mut self) -> Result<Vec<Todo>> {
        Ok(self
            .list_all()?
            .into_iter()
            .filter(|t| t.is_open())
            .collect())
    }

    /// Find a single todo by ID or unique prefix.
    pub fn find_by_id(&mut self, prefix: &str) -> Result<Todo> {
        self.ensure_cache()?;
        let cache = self.cache.as_ref().unwrap();
        let matches: Vec<_> = cache.iter().filter(|t| t.id.starts_with(prefix)).collect();
        match matches.len() {
            0 => Err(anyhow!("no todo with id prefix '{prefix}'")),
            1 => Ok(matches[0].clone()),
            n => {
                let ids: Vec<_> = matches.iter().map(|t| t.id.as_str()).collect();
                Err(anyhow!(
                    "ambiguous id prefix '{prefix}': {n} matches ({})",
                    ids.join(", ")
                ))
            }
        }
    }

    /// Create a new todo file, returning the assigned ID.
    pub fn create(&mut self, fm: &Frontmatter, body: Option<&str>) -> Result<String> {
        self.ensure_cache()?;
        let existing_ids: Vec<String> = self
            .cache
            .as_ref()
            .unwrap()
            .iter()
            .map(|t| t.id.clone())
            .collect();
        let id = generate_id(|candidate| existing_ids.contains(&candidate.to_string()));
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

        let todo = Todo {
            id: id.clone(),
            filename,
            frontmatter: fm.clone(),
            body: body.map(|s| s.to_string()),
        };
        if let Some(ref mut cache) = self.cache {
            cache.push(todo);
            cache.sort_by(|a, b| a.frontmatter.created.cmp(&b.frontmatter.created));
        }
        Ok(id)
    }

    /// Overwrite an existing todo file.
    pub fn save(&mut self, todo: &Todo) -> Result<()> {
        let content = todo::render_file(&todo.frontmatter, todo.body.as_deref())?;
        let path = self.dir.join(&todo.filename);
        fs::write(&path, &content)
            .with_context(|| format!("failed to write: {}", path.display()))?;
        if let Some(ref mut cache) = self.cache {
            if let Some(entry) = cache.iter_mut().find(|t| t.id == todo.id) {
                *entry = todo.clone();
            }
        }
        Ok(())
    }

    /// Delete a todo file by ID.
    pub fn delete(&mut self, id: &str) -> Result<()> {
        let todo = self.find_by_id(id)?;
        let path = self.dir.join(&todo.filename);
        let full_id = todo.id.clone();
        fs::remove_file(&path)
            .with_context(|| format!("failed to delete: {}", path.display()))?;
        if let Some(ref mut cache) = self.cache {
            cache.retain(|t| t.id != full_id);
        }
        Ok(())
    }

    /// Return the full path for a todo file.
    pub fn path_for(&self, todo: &Todo) -> PathBuf {
        self.dir.join(&todo.filename)
    }
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
