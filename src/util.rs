use is_terminal::IsTerminal;
use rand::RngExt;

/// Generate a random 4-hex-character ID, retrying if `is_taken` returns true.
pub fn generate_id(mut is_taken: impl FnMut(&str) -> bool) -> String {
    loop {
        let n: u16 = rand::rng().random();
        let id = format!("{n:04x}");
        if !is_taken(&id) {
            return id;
        }
    }
}

/// Slugify a title into a filename-safe lowercase string.
///
/// Lowercase, replace non-alphanumeric runs with a single hyphen,
/// strip leading/trailing hyphens, truncate to ~50 chars.
pub fn slugify(title: &str) -> String {
    let lower = title.to_lowercase();
    let mut slug = String::new();
    let mut last_was_sep = true;
    for c in lower.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c);
            last_was_sep = false;
        } else if !last_was_sep {
            slug.push('-');
            last_was_sep = true;
        }
    }
    let slug = slug.trim_end_matches('-');
    let truncated = match slug.char_indices().nth(50) {
        Some((i, _)) => &slug[..i],
        None => slug,
    };
    truncated.to_string()
}

pub fn stdout_is_tty() -> bool {
    std::io::stdout().is_terminal()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_simple() {
        assert_eq!(slugify("fix the login bug"), "fix-the-login-bug");
    }

    #[test]
    fn slugify_special_chars() {
        assert_eq!(slugify("hello, world! #42"), "hello-world-42");
    }

    #[test]
    fn slugify_leading_trailing_special() {
        assert_eq!(slugify("--hello--"), "hello");
    }

    #[test]
    fn slugify_empty() {
        assert_eq!(slugify(""), "");
    }

    #[test]
    fn slugify_all_special() {
        assert_eq!(slugify("!@#$%"), "");
    }

    #[test]
    fn slugify_unicode() {
        // Non-ASCII is stripped since we only keep ascii_alphanumeric
        assert_eq!(slugify("café latte"), "caf-latte");
    }

    #[test]
    fn slugify_truncates_long_title() {
        let long = "a".repeat(100);
        let slug = slugify(&long);
        assert_eq!(slug.len(), 50);
    }

    #[test]
    fn slugify_consecutive_separators() {
        assert_eq!(slugify("hello   ---   world"), "hello-world");
    }

    #[test]
    fn generate_id_format() {
        let id = generate_id(|_| false);
        assert_eq!(id.len(), 4);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn generate_id_retries_on_collision() {
        let mut calls = 0;
        let id = generate_id(|_| {
            calls += 1;
            calls < 3 // reject first two attempts
        });
        assert_eq!(id.len(), 4);
        assert!(calls >= 3);
    }
}
