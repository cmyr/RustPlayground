use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::Error;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Type {
    Run,
    Check,
    Test,
}

impl Type {
    fn as_str(&self) -> &str {
        match self {
            Type::Run => "build".as_ref(),
            Type::Check => "build".as_ref(),
            Type::Test => "test".as_ref(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Task {
    toolchain: String,
    code: String,
    task_type: Type,
    backtrace: bool,
    release: bool,
}

/// The result of a rustc run.
#[derive(Debug, Default, Clone, Serialize)]
pub struct CompilerResult {
    success: bool,
    stdout: String,
    stderr: String,
    /// The path to the produced binary, if any.
    executable: Option<PathBuf>,
}

/// Attempts to run the given task in the supplied directory, which will
/// be created if it does not exist.
pub fn do_compile_task<P: AsRef<Path>>(outdir: P, task: Task) -> Result<CompilerResult, Error> {
    let outdir = outdir.as_ref();
    create_cargo_scaffold(&outdir, &task.code)?;
    activate_toolchain(outdir, &task.toolchain)?;
    let mut command = Command::new("cargo");
    command.current_dir(outdir).arg(task.task_type.as_str());

    if task.backtrace {
        command.env("RUST_BACKTRACE", "1");
    }

    if task.release {
        command.arg("--release");
    }

    let output = command.output().map_err(|e| Error::CommandFailed(e))?;
    let success = output.status.success();
    let executable = get_output_path(outdir, &task);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    Ok(CompilerResult { success, executable, stdout, stderr })
}

fn create_cargo_scaffold(path: &Path, code: &str) -> Result<(), Error> {
    let src_dir = path.join("src");
    fs::create_dir_all(&src_dir).map_err(|_| Error::CreateOutputFailed(src_dir.clone()))?;

    let main_rs = src_dir.join("main.rs");
    fs::write(&main_rs, code.as_bytes()).map_err(|_| Error::CreateOutputFailed(main_rs.clone()))?;

    let cargo_toml = path.join("Cargo.toml");
    fs::write(&cargo_toml, PLACEHOLDER_CARGO_TOML.as_bytes())
        .map_err(|_| Error::CreateOutputFailed(cargo_toml))?;

    Ok(())
}

fn activate_toolchain(path: &Path, toolchain: &str) -> Result<(), Error> {
    let result = Command::new("rustup")
        .current_dir(path)
        .args(&["override", "set", toolchain])
        .output()
        .map_err(|e| Error::CommandFailed(e))?;

    if result.status.success() {
        Ok(())
    } else {
        Err(Error::bad_output("Failed to set toolchain", &result))
    }
}

fn get_output_path(path: &Path, task: &Task) -> Option<PathBuf> {
    let path = path.join("target");
    let path = if task.release { path.join("release") } else { path.join("debug") };
    let path = path.join(BIN_TARGET_NAME);
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

const BIN_TARGET_NAME: &str = "playground";

static PLACEHOLDER_CARGO_TOML: &str = r#"
[package]
name = "playground"
version = "0.0.0"
authors = ["The Intrepid User <jane.doe@example.com>"]
edition = "2018"
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn test_hello_world() {
        let tempdir = TempDir::new("hello_word_project").expect("failed to create temp dir");
        let outdir = tempdir.path().join("my_project");
        let task = Task {
            toolchain: "stable".into(),
            code: "fn main() {\n     println!(\"hello world!\");\n}".into(),
            task_type: Type::Run,
            backtrace: true,
            release: false,
        };

        let exp_exec_path = outdir.join("target").join("debug").join(BIN_TARGET_NAME);
        let result = do_compile_task(&outdir, task).expect("compile task failed");

        assert_eq!(result.executable, Some(exp_exec_path));
    }
}
