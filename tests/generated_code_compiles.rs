use anyhow::{Context, Result};
use dioxus_iconify::api::IconifyClient;
use dioxus_iconify::generator::Generator;
use dioxus_iconify::naming::IconIdentifier;
use rstest::rstest;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
#[ignore] // Requires internet connection to fetch icons
fn test_generated_code_compiles_without_warnings() -> Result<()> {
    // Create a temporary directory for our test project
    let temp_dir = TempDir::new()?;
    let project_dir = temp_dir.path();
    let icons_dir = project_dir.join("src/icons");

    println!("Test project directory: {:?}", project_dir);

    // Generate some icons using our generator
    let generator = Generator::new(icons_dir.clone());
    let client = IconifyClient::new()?;

    let test_icons = vec!["mdi:home", "heroicons:arrow-left", "lucide:settings"];
    let mut icons_to_add = Vec::new();

    for icon_id in &test_icons {
        let identifier = IconIdentifier::parse(icon_id)?;
        let icon = client.fetch_icon(&identifier.collection, &identifier.icon_name)?;
        icons_to_add.push((identifier, icon));
    }

    generator.add_icons(&icons_to_add)?;

    // Create a minimal Cargo.toml for the test project
    let cargo_toml = r#"[package]
name = "icon-test"
version = "0.1.0"
edition = "2021"

[dependencies]
dioxus = "0.7"
"#;
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;

    // Create src directory
    fs::create_dir_all(project_dir.join("src"))?;

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
        }
    }
}
"#;
    fs::write(project_dir.join("src/main.rs"), main_rs)?;

    // Run cargo build and capture output
    let output = Command::new("cargo")
        .args(["build", "--quiet"])
        .current_dir(project_dir)
        .output()
        .context("Failed to run cargo build")?;

    // Check if build succeeded
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        eprintln!("Build failed!");
        eprintln!("STDOUT:\n{}", stdout);
        eprintln!("STDERR:\n{}", stderr);
        panic!("Build failed with exit code: {:?}", output.status.code());
    }

    // Check for warnings in the output
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let full_output = format!("{}\n{}", stdout, stderr);

    // Look for warning patterns
    if full_output.contains("warning:") {
        eprintln!("Build output contained warnings:");
        eprintln!("{}", full_output);
        panic!("Generated code produced compilation warnings");
    }

    println!("✓ Generated code compiles without warnings");
    Ok(())
}

#[rstest]
#[case(
    "test:sample-icon",
    "SampleIcon",
    r#"<path d="M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z"/>"#
)]
#[case(
    "my-collection:custom-icon",
    "CustomIcon",
    r#"<circle cx="12" cy="12" r="10"/>"#
)]
fn test_generated_icons_are_valid_rust(
    #[case] icon_id: &str,
    #[case] expected_const_name: &str,
    #[case] body: &str,
) -> Result<()> {
    // Create a temporary directory for icons
    let temp_dir = TempDir::new()?;
    let icons_dir = temp_dir.path().join("icons");

    // Generate a simple icon without fetching from API
    let generator = Generator::new(icons_dir.clone());

    // Manually create an IconData to avoid network dependency
    use dioxus_iconify::api::IconifyIcon;

    let test_icon = IconifyIcon {
        body: body.to_string(),
        width: Some(24),
        height: Some(24),
        view_box: Some("0 0 24 24".to_string()),
    };

    let identifier = IconIdentifier::parse(icon_id)?;
    let collection = identifier.collection.clone().replace('-', "_");
    generator.add_icons(&[(identifier, test_icon)])?;

    // Read the generated file and verify it contains valid Rust syntax markers
    let generated_file = icons_dir.join(format!("{}.rs", collection));
    assert!(generated_file.exists(), "Generated file should exist");

    let content = fs::read_to_string(&generated_file)?;

    // Check for expected patterns in generated code
    assert!(
        content.contains("use super::IconData;"),
        "Should import IconData"
    );
    assert!(
        content.contains(&format!("pub const {}: IconData", expected_const_name)),
        "Should define icon constant"
    );
    assert!(
        content.contains(&format!(r#"name: "{}""#, icon_id)),
        "Should include full icon name"
    );
    assert!(
        content.contains(r#"view_box: "0 0 24 24""#),
        "Should include viewBox"
    );

    // Verify mod.rs was created
    let mod_file = icons_dir.join("mod.rs");
    assert!(mod_file.exists(), "mod.rs should be created");

    let mod_content = fs::read_to_string(&mod_file)?;
    assert!(
        mod_content.contains(&format!("pub mod {};", collection)),
        "Should declare collection module"
    );
    assert!(
        mod_content.contains("pub struct IconData"),
        "Should define IconData struct"
    );
    assert!(
        mod_content.contains("pub fn Icon("),
        "Should define Icon component"
    );

    Ok(())
}
