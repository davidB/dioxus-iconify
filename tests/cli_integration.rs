use anyhow::Result;
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Helper to create a test command
fn cmd() -> Command {
    assert_cmd::cargo_bin_cmd!("dioxus-iconify")
}

#[test]
fn test_cli_init_creates_mod_file() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let output_dir = temp_dir.path().join("icons");

    cmd()
        .arg("init")
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success();

    // Verify mod.rs was created
    let mod_file = output_dir.join("mod.rs");
    assert!(mod_file.exists(), "mod.rs should be created");

    let content = fs::read_to_string(&mod_file)?;
    assert!(
        content.contains("pub struct IconData"),
        "mod.rs should contain IconData struct"
    );
    assert!(
        content.contains("pub fn Icon("),
        "mod.rs should contain Icon component"
    );

    Ok(())
}

#[test]
fn test_cli_add_local_svg_file() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let output_dir = temp_dir.path().join("icons");

    // Get the path to our test SVG file
    let test_svg =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/test-icons/simple.svg");

    cmd()
        .arg("add")
        .arg(&test_svg)
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Done!"));

    // Verify the generated file exists (collection name is derived from parent directory: test-icons)
    let test_icons_file = output_dir.join("test_icons.rs");
    assert!(test_icons_file.exists(), "test_icons.rs should be created");

    let content = fs::read_to_string(&test_icons_file)?;
    assert!(
        content.contains("pub const Simple: IconData"),
        "Should generate Simple constant"
    );
    assert!(
        content.contains(r#"name: "test-icons:simple""#),
        "Should have correct icon name"
    );

    Ok(())
}

#[test]
fn test_cli_add_directory_of_svgs() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let output_dir = temp_dir.path().join("icons");

    // Get the path to our test directory with SVG files
    let test_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/test-icons/arrows");

    cmd()
        .arg("add")
        .arg(&test_dir)
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Done!"));

    // Verify the generated file exists (collection name is "arrows" from directory name)
    let arrows_file = output_dir.join("arrows.rs");
    assert!(arrows_file.exists(), "arrows.rs should be created");

    let content = fs::read_to_string(&arrows_file)?;
    assert!(
        content.contains("pub const Left: IconData"),
        "Should generate Left constant"
    );
    assert!(
        content.contains("pub const Right: IconData"),
        "Should generate Right constant"
    );

    Ok(())
}

#[test]
#[ignore] // Requires internet connection to fetch icons from API
fn test_cli_add_icon_from_api() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let output_dir = temp_dir.path().join("icons");

    cmd()
        .arg("add")
        .arg("mdi:home")
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Done!"))
        .stdout(predicate::str::contains("mdi:home"));

    // Verify the generated file exists
    let mdi_file = output_dir.join("mdi.rs");
    assert!(mdi_file.exists(), "mdi.rs should be created");

    let content = fs::read_to_string(&mdi_file)?;
    assert!(
        content.contains("pub const Home: IconData"),
        "Should generate Home constant"
    );
    assert!(
        content.contains(r#"name: "mdi:home""#),
        "Should have correct icon name"
    );

    // Verify mod.rs was also created
    let mod_file = output_dir.join("mod.rs");
    assert!(mod_file.exists(), "mod.rs should be created");

    Ok(())
}

#[test]
#[ignore] // Requires internet connection
fn test_cli_add_multiple_icons_from_api() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let output_dir = temp_dir.path().join("icons");

    cmd()
        .arg("add")
        .arg("mdi:home")
        .arg("heroicons:arrow-left")
        .arg("lucide:settings")
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success();

    // Verify collection files were created
    assert!(output_dir.join("mdi.rs").exists(), "mdi.rs should exist");
    assert!(
        output_dir.join("heroicons.rs").exists(),
        "heroicons.rs should exist"
    );
    assert!(
        output_dir.join("lucide.rs").exists(),
        "lucide.rs should exist"
    );

    Ok(())
}

#[test]
fn test_cli_list_icons() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let output_dir = temp_dir.path().join("icons");

    // First add some local icons
    let test_svg =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/test-icons/simple.svg");

    cmd()
        .arg("add")
        .arg(&test_svg)
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success();

    // Now list them
    cmd()
        .arg("list")
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("test-icons:"))
        .stdout(predicate::str::contains("simple"));

    Ok(())
}

#[test]
fn test_cli_skip_existing_flag() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let output_dir = temp_dir.path().join("icons");

    let test_svg =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/test-icons/simple.svg");

    // Add icon first time
    cmd()
        .arg("add")
        .arg(&test_svg)
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success();

    // Try to add again with --skip-existing
    cmd()
        .arg("add")
        .arg(&test_svg)
        .arg("--skip-existing")
        .arg("--output")
        .arg(&output_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Skipped"));

    Ok(())
}

#[test]
#[ignore] // Requires internet connection and takes time to compile
fn test_generated_code_compiles() -> Result<()> {
    // Create a temporary directory for our test project
    let temp_dir = TempDir::new()?;
    let project_dir = temp_dir.path();
    let icons_dir = project_dir.join("src/icons");

    // Add some test icons using the CLI
    cmd()
        .arg("add")
        .arg("mdi:home")
        .arg("heroicons:arrow-left")
        .arg("lucide:settings")
        .arg("--output")
        .arg(&icons_dir)
        .assert()
        .success();

    // Create a minimal Cargo.toml for the test project
    let cargo_toml = r#"[package]
name = "icon-test"
version = "0.1.0"
edition = "2021"

[dependencies]
dioxus = "0.7"
"#;
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;

    // Create a main.rs that uses the generated icons
    let main_rs = r#"#![deny(warnings)]

mod icons;

use dioxus::prelude::*;
use icons::Icon;
use icons::{heroicons, lucide, mdi};

fn main() {
    // Use the icons to avoid dead_code warnings
    let _home = mdi::Home;
    let _arrow = heroicons::ArrowLeft;
    let _settings = lucide::Settings;

    println!("Icons loaded successfully");
}

#[component]
fn App() -> Element {
    rsx! {
        div {
            Icon { data: mdi::Home }
            Icon { data: heroicons::ArrowLeft }
            Icon { data: lucide::Settings, width: "32", height: "32" }
            Icon { data: mdi::Home, size: "24" }
            Icon { data: heroicons::ArrowLeft, size: 32.to_string() }
            Icon { data: lucide::Settings, size: "2em" }
        }
    }
}
"#;
    fs::write(project_dir.join("src/main.rs"), main_rs)?;

    // Run cargo build and check it succeeds without warnings
    let output = std::process::Command::new("cargo")
        .args(["build", "--quiet"])
        .current_dir(project_dir)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        eprintln!("Build failed!");
        eprintln!("STDOUT:\n{}", stdout);
        eprintln!("STDERR:\n{}", stderr);
        panic!("Build failed");
    }

    // Check for warnings
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let full_output = format!("{}\n{}", stdout, stderr);

    assert!(
        !full_output.contains("warning:"),
        "Generated code should not produce warnings"
    );

    Ok(())
}

#[test]
fn test_cli_invalid_icon_format() {
    cmd().arg("add").arg("invalid-format").assert().failure();
}

#[test]
fn test_cli_help() {
    cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "CLI tool for generating Iconify icons",
        ));
}

#[test]
fn test_cli_version() {
    cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}
