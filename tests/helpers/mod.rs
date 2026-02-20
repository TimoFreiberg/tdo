use std::process::{Command, Output};

use tempfile::TempDir;

pub struct TdoTest {
    pub dir: TempDir,
}

impl TdoTest {
    pub fn new() -> Self {
        TdoTest {
            dir: TempDir::new().unwrap(),
        }
    }

    /// Run `tdo --dir <tmpdir> <args...>` and return the output.
    pub fn run(&self, args: &[&str]) -> Output {
        let bin = assert_cmd::cargo::cargo_bin!("tdo");
        Command::new(bin)
            .arg("--dir")
            .arg(self.dir.path())
            .args(args)
            .output()
            .expect("failed to execute tdo")
    }

    /// Convenience: run and return stdout as a trimmed string.
    pub fn run_ok(&self, args: &[&str]) -> String {
        let output = self.run(args);
        assert!(
            output.status.success(),
            "tdo {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_string()
    }

    /// Convenience: run and assert it fails.
    pub fn run_err(&self, args: &[&str]) -> String {
        let output = self.run(args);
        assert!(
            !output.status.success(),
            "tdo {:?} should have failed but succeeded",
            args
        );
        String::from_utf8(output.stderr).unwrap().trim().to_string()
    }

    /// List todo files (excluding hidden files like .lock) in the todo directory.
    pub fn files(&self) -> Vec<String> {
        std::fs::read_dir(self.dir.path())
            .unwrap()
            .filter_map(|e| {
                let e = e.ok()?;
                let name = e.file_name().to_string_lossy().to_string();
                if name.starts_with('.') {
                    return None;
                }
                Some(name)
            })
            .collect()
    }
}
