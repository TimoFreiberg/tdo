use anyhow::{anyhow, Result};
use jiff::civil::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Open,
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frontmatter {
    pub title: String,
    pub created: DateTime,
    pub status: Status,
}

#[derive(Debug, Clone)]
pub struct Todo {
    pub id: String,
    pub filename: String,
    pub frontmatter: Frontmatter,
    pub body: Option<String>,
}

impl Todo {
    pub fn title(&self) -> &str {
        &self.frontmatter.title
    }

    pub fn is_open(&self) -> bool {
        self.frontmatter.status == Status::Open
    }
}

pub fn parse_file(raw: &str) -> Result<(Frontmatter, Option<String>)> {
    let rest = raw
        .strip_prefix("---\n")
        .ok_or_else(|| anyhow!("missing opening ---"))?;
    let (yaml_part, after) = rest
        .split_once("\n---\n")
        .ok_or_else(|| anyhow!("missing closing ---"))?;
    let fm: Frontmatter = serde_yml::from_str(yaml_part)?;
    let body = if after.trim().is_empty() {
        None
    } else {
        Some(after.to_string())
    };
    Ok((fm, body))
}

pub fn render_file(fm: &Frontmatter, body: Option<&str>) -> Result<String> {
    let yaml = serde_yml::to_string(fm)?;
    let mut out = format!("---\n{yaml}---\n");
    if let Some(b) = body {
        out.push('\n');
        out.push_str(b);
        if !b.ends_with('\n') {
            out.push('\n');
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_frontmatter() -> Frontmatter {
        Frontmatter {
            title: "fix the login bug".to_string(),
            created: "2026-02-20T14:30:52".parse().unwrap(),
            status: Status::Open,
        }
    }

    #[test]
    fn render_then_parse_roundtrip_no_body() {
        let fm = sample_frontmatter();
        let rendered = render_file(&fm, None).unwrap();
        let (parsed_fm, parsed_body) = parse_file(&rendered).unwrap();
        assert_eq!(parsed_fm.title, fm.title);
        assert_eq!(parsed_fm.created, fm.created);
        assert_eq!(parsed_fm.status, fm.status);
        assert!(parsed_body.is_none());
    }

    #[test]
    fn render_then_parse_roundtrip_with_body() {
        let fm = sample_frontmatter();
        let body = "Some notes about the bug.\nSecond line.";
        let rendered = render_file(&fm, Some(body)).unwrap();
        let (parsed_fm, parsed_body) = parse_file(&rendered).unwrap();
        assert_eq!(parsed_fm.title, fm.title);
        let b = parsed_body.unwrap();
        assert!(b.contains("Some notes about the bug."));
        assert!(b.contains("Second line."));
    }

    #[test]
    fn parse_missing_opening_delimiter() {
        let raw = "title: foo\n---\n";
        assert!(parse_file(raw).is_err());
    }

    #[test]
    fn parse_missing_closing_delimiter() {
        let raw = "---\ntitle: foo\n";
        assert!(parse_file(raw).is_err());
    }

    #[test]
    fn parse_done_status() {
        let raw = "---\ntitle: done task\ncreated: 2026-02-20T14:30:52\nstatus: done\n---\n";
        let (fm, _) = parse_file(raw).unwrap();
        assert_eq!(fm.status, Status::Done);
    }
}
