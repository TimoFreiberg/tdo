use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use crate::todo::{self, Frontmatter, Todo};
use crate::util::{generate_id, slugify};

pub struct Store {
    pub dir: PathBuf,
}

impl Store {
    /// Open a store, creating the directory if it doesn't exist.
    pub fn open(dir: &Path) -> Result<Self> {
        fs::create_dir_all(dir)
            .with_context(|| format!("failed to create todo directory: {}", dir.display()))?;
        Ok(Store {
            dir: dir.to_path_buf(),
        })
    }

    /// Resolve the todo directory: use the override if provided, otherwise `.todo/` in cwd.
    pub fn resolve_dir(override_dir: Option<&Path>) -> PathBuf {
        match override_dir {
            Some(d) => d.to_path_buf(),
            None => PathBuf::from(".todo"),
        }
    }

    /// List all todos, sorted by created timestamp ascending.
    pub fn list_all(&self) -> Result<Vec<Todo>> {
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

    /// List only open todos.
    pub fn list_open(&self) -> Result<Vec<Todo>> {
        Ok(self.list_all()?.into_iter().filter(|t| t.is_open()).collect())
    }

    /// Find a single todo by its hex ID.
    pub fn find_by_id(&self, id: &str) -> Result<Todo> {
        let todos = self.list_all()?;
        let matches: Vec<_> = todos.into_iter().filter(|t| t.id == id).collect();
        match matches.len() {
            0 => Err(anyhow!("no todo found with id '{id}'")),
            1 => Ok(matches.into_iter().next().unwrap()),
            n => Err(anyhow!("ambiguous id '{id}': {n} matches")),
        }
    }

    /// Create a new todo file, returning the assigned ID.
    pub fn create(&self, fm: &Frontmatter, body: Option<&str>) -> Result<String> {
        let existing_ids = self.existing_ids()?;
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
        Ok(id)
    }

    /// Overwrite an existing todo file.
    pub fn save(&self, todo: &Todo) -> Result<()> {
        let content = todo::render_file(&todo.frontmatter, todo.body.as_deref())?;
        let path = self.dir.join(&todo.filename);
        fs::write(&path, &content)
            .with_context(|| format!("failed to write: {}", path.display()))?;
        Ok(())
    }

    /// Delete a todo file by ID.
    pub fn delete(&self, id: &str) -> Result<()> {
        let todo = self.find_by_id(id)?;
        let path = self.dir.join(&todo.filename);
        fs::remove_file(&path)
            .with_context(|| format!("failed to delete: {}", path.display()))?;
        Ok(())
    }

    /// Return the full path for a todo file.
    pub fn path_for(&self, todo: &Todo) -> PathBuf {
        self.dir.join(&todo.filename)
    }

    fn existing_ids(&self) -> Result<Vec<String>> {
        let entries = fs::read_dir(&self.dir)?;
        let mut ids = Vec::new();
        for entry in entries {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if let Some(id) = id_from_filename(&name) {
                ids.push(id.to_string());
            }
        }
        Ok(ids)
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
