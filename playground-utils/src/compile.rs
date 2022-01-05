use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::error::Error;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Type {
    Run,
    Check,
    Test,
    Clean,
}

impl Type {
    fn as_str(&self) -> &str {
        match self {
            Type::Run => "build",
            Type::Check => "build",
            Type::Test => "test",
            Type::Clean => "clean",
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
pub fn do_compile_task<P, F>(
    outdir: P,
    task: Task,
    mut std_err_callback: F,
) -> Result<CompilerResult, Error>
where
    P: AsRef<Path>,
    F: FnMut(&str),
{
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

    command.stderr(Stdio::piped());
    command.stdout(Stdio::piped());

    let mut child = command.spawn().map_err(Error::CompileFailed)?;
    let stderr = child.stderr.take().expect("piped stderr must exist");
    let mut linereader = BufReader::new(stderr);

    let mut line_buf = String::new();
    // we send stderr lines as they arrive, so the client
    // is more responsive & informative
    loop {
        match linereader.read_line(&mut line_buf) {
            Ok(0) => {
                println!("read len 0");
                break;
            }
            Ok(_) => {
                std_err_callback(&line_buf);
                line_buf.clear();
            }
            Err(e) => return Err(Error::CompileFailed(e)),
        }
    }

    let output = child.wait_with_output().map_err(Error::CompileFailed)?;
    let success = output.status.success();
    let executable = get_output_path(outdir, &task);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    Ok(CompilerResult { success, stdout, stderr, executable })
}

fn create_cargo_scaffold(path: &Path, code: &str) -> Result<(), Error> {
    let src_dir = path.join("src");
    fs::create_dir_all(&src_dir).map_err(|_| Error::CreateOutputFailed(src_dir.clone()))?;

    let main_rs = src_dir.join("main.rs");
    fs::write(&main_rs, code.as_bytes()).map_err(|_| Error::CreateOutputFailed(main_rs.clone()))?;

    let cargo_toml = path.join("Cargo.toml");
    let extra_deps = parse_dep_comments(code)?;

    let mut manifest = PLACEHOLDER_CARGO_TOML.to_string();
    for line in extra_deps {
        manifest.push_str(&line);
        manifest.push('\n');
    }

    fs::write(&cargo_toml, manifest.as_bytes())
        .map_err(|_| Error::CreateOutputFailed(cargo_toml))?;

    Ok(())
}

/// Hacky. We allow dependencies to be specified as comments with the form,
/// '//~ serde = "1.0"'. This finds those strings and returns them formatted
/// suitable for appending to the toml text.
fn parse_dep_comments(code: &str) -> Result<Vec<String>, Error> {
    code.lines().filter(|l| l.trim().starts_with("//~")).map(dep_for_comment_line).collect()
}

fn dep_for_comment_line(line: &str) -> Result<String, Error> {
    // we trim twice to get whitespace between thee comment markere and the first token
    let line = line.trim().trim_start_matches("//~").trim();
    let tokens = line.split_whitespace().collect::<Vec<_>>();
    match tokens.as_slice() {
        &["use", name] => {
            if name.chars().all(legal_in_crate_name) {
                Ok(format!("{} = \"*\"", name))
            } else {
                Err(Error::MalformedDependency(line.to_owned()))
            }
        }
        &["use", name, "=", version] => {
            let version = version.trim_matches('"');
            if name.chars().all(legal_in_crate_name) && version.chars().all(legal_in_version) {
                Ok(format!("{} = \"{}\"", name, version))
            } else {
                Err(Error::MalformedDependency(line.into()))
            }
        }
        _other => Err(Error::MalformedDependency(line.into())),
    }
}

fn legal_in_crate_name(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

fn legal_in_version(c: char) -> bool {
    matches!(c, '0'..='9' | '.')
}

fn activate_toolchain(path: &Path, toolchain: &str) -> Result<(), Error> {
    let result = Command::new("rustup")
        .current_dir(path)
        .args(&["override", "set", toolchain])
        .output()
        .map_err(Error::ToolchainSelectFailed)?;

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

[dependencies]
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
        let result = do_compile_task(&outdir, task, |_| {}).expect("compile task failed");

        assert_eq!(result.executable, Some(exp_exec_path));
    }

    #[test]
    fn hacky_dependencies() {
        assert_eq!(dep_for_comment_line("//~ use serde = 1.0").unwrap(), ("serde = \"1.0\""));
        assert_eq!(dep_for_comment_line("  //~ use ast").unwrap(), ("ast = \"*\""));
        // allow missing first space
        assert_eq!(dep_for_comment_line("//~use ast").unwrap(), ("ast = \"*\""));
        // ignore quotes
        assert_eq!(dep_for_comment_line("//~ use ast = \"5\"").unwrap(), ("ast = \"5\""));

        assert_eq!(dep_for_comment_line("//~ use ast = \"5.0.1\"").unwrap(), ("ast = \"5.0.1\""));

        // versions are numeric values
        assert!(dep_for_comment_line("//~ use ast = \"5a\"").is_err());

        // identifiers are alphanums
        assert!(dep_for_comment_line("//~ use jsoñ = \"5\"").is_err());
        assert!(dep_for_comment_line("//~ use jso.n = \"5\"").is_err());
    }
}
