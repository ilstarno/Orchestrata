use std::{env, error::Error, fs, path::Path, process::Command};

use orchestrata_facade_check::violations;

fn main() -> Result<(), Box<dyn Error>> {
    let root = env::args_os().nth(1).unwrap_or_else(|| ".".into());
    let output = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["rev-parse", "--show-toplevel"])
        .output()?;
    if !output.status.success() {
        return Err("cannot locate repository root".into());
    }
    let repository = String::from_utf8(output.stdout)?;
    let root = Path::new(repository.trim_end());
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "-z", "--", "*.rs"])
        .output()?;
    if !output.status.success() {
        return Err("cannot enumerate tracked Rust files".into());
    }
    let mut checked = 0;
    let mut failed = false;
    for raw in output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|p| !p.is_empty())
    {
        let relative = std::str::from_utf8(raw)?;
        let path = Path::new(relative);
        if !matches!(
            path.file_name().and_then(|name| name.to_str()),
            Some("lib.rs" | "mod.rs" | "module.rs")
        ) {
            continue;
        }
        checked += 1;
        let source = fs::read_to_string(root.join(path))?;
        match violations(&source) {
            Ok(problems) if problems.is_empty() => println!("OK {relative}"),
            Ok(problems) => {
                failed = true;
                for problem in problems {
                    eprintln!("{relative}: {problem}");
                }
            }
            Err(error) => {
                failed = true;
                eprintln!("{relative}: cannot parse Rust: {error}");
            }
        }
    }
    if checked == 0 {
        return Err("no tracked Rust facades found".into());
    }
    if failed {
        return Err("Rust facade check failed".into());
    }
    println!("Checked {checked} tracked Rust facades");
    Ok(())
}
