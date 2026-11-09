use std::{fs, process::Command};

fn main() {
    emit_commit();

    tauri_build::build();
}

/// Bakes the short commit hash into the binary.
fn emit_commit() {
    println!("cargo:rerun-if-changed=../.git/HEAD");

    if let Ok(head) = fs::read_to_string("../.git/HEAD") {
        if let Some(reference) = head.trim().strip_prefix("ref: ") {
            println!("cargo:rerun-if-changed=../.git/{reference}");
        }
    }

    println!("cargo:rustc-env=POPEYE_BUILD_COMMIT={}", commit());
}

fn commit() -> String {
    let output = Command::new("git")
        .args(["rev-parse", "--short=7", "HEAD"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if sha.is_empty() {
                "unknown".to_string()
            } else {
                sha
            }
        }
        _ => "unknown".to_string(),
    }
}
