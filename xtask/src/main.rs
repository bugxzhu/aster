mod layering;
mod policy;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Collects file paths relative to `base`. `dir` is the directory currently being read.
fn walk(base: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        let rel = path.strip_prefix(base).unwrap_or(&path).to_path_buf();
        let rel_str = rel.to_string_lossy();
        if rel_str == "target"
            || rel_str.starts_with("target/")
            || rel_str == ".git"
            || rel_str.starts_with(".git/")
        {
            continue;
        }
        if path.is_dir() {
            walk(base, &path, out)?;
        } else {
            out.push(rel);
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    let arg = std::env::args().nth(1).unwrap_or_default();
    match arg.as_str() {
        "check-english" => check_english(),
        "check-layering" => check_layering(),
        _ => {
            eprintln!("usage: cargo run -p xtask -- <check-english|check-layering>");
            ExitCode::from(2)
        }
    }
}

fn check_layering() -> ExitCode {
    let output = match std::process::Command::new("cargo")
        .args(["metadata", "--format-version", "1"])
        .output()
    {
        Ok(o) if o.status.success() => o.stdout,
        _ => {
            eprintln!("cargo metadata failed");
            return ExitCode::from(3);
        }
    };
    let meta: serde_json::Value = match serde_json::from_slice(&output) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("cannot parse cargo metadata: {e}");
            return ExitCode::from(3);
        }
    };
    let graph = match layering::graph_from_metadata(&meta) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("cannot build dependency graph: {e}");
            return ExitCode::from(3);
        }
    };
    let found = layering::violations(&graph, &layering::rules());
    if found.is_empty() {
        println!("layering check passed ({} rules)", layering::rules().len());
        return ExitCode::SUCCESS;
    }
    for v in &found {
        println!("LAYERING VIOLATION: {v}");
    }
    ExitCode::from(1)
}

fn check_english() -> ExitCode {
    let root = std::env::current_dir().expect("INVARIANT: current directory exists");
    let mut files = Vec::new();
    if let Err(e) = walk(&root, &root, &mut files) {
        eprintln!("failed to walk repository: {e}");
        return ExitCode::from(3);
    }
    let mut count = 0;
    for rel in files.iter().filter(|p| policy::should_scan(p)) {
        let Ok(text) = fs::read_to_string(root.join(rel)) else {
            continue;
        };
        for v in policy::scan_text(&rel.to_string_lossy(), &text) {
            println!("{}:{}: contains CJK text: {}", v.path, v.line, v.text);
            count += 1;
        }
    }
    if count > 0 {
        eprintln!("{count} violation(s) of the English-only policy (06 section 3.0)");
        return ExitCode::from(1);
    }
    println!("english-only check passed");
    ExitCode::SUCCESS
}
