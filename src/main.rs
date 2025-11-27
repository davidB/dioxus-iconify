mod api;
mod generator;
mod naming;
mod svg;

use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use api::IconifyClient;
use generator::Generator;
use naming::IconIdentifier;

#[derive(Parser)]
#[command(name = "dioxus-iconify")]
#[command(about = "CLI tool for generating Iconify icons in Dioxus projects", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output directory for generated icons (default: src/icons)
    #[arg(short, long, global = true, default_value = "src/icons")]
    output: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Add one or more icons to your project
    #[command(visible_alias = "a")]
    Add {
        /// Icon identifiers, SVG file paths, or directory paths (e.g., mdi:home, ./logo.svg, ./icons/)
        #[arg(required = true)]
        icons: Vec<String>,

        /// Skip icons that already exist (don't overwrite)
        #[arg(long)]
        skip_existing: bool,
    },

    /// Initialize the icons directory (creates mod.rs)
    #[command(visible_alias = "i")]
    Init,

    /// List all generated icons
    #[command(visible_alias = "l")]
    List,

    /// Update all icons by re-fetching from API
    #[command(visible_alias = "u")]
    Update,
    // Future commands (not yet implemented)
    // /// Remove icons from your project
    // #[command(visible_alias = "r")]
    // Remove {
    //     icons: Vec<String>,
    // },
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("Error: {:#}", err);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();
    let generator = Generator::new(cli.output.clone());

    match cli.command {
        Commands::Add {
            icons,
            skip_existing,
        } => {
            add_icons(&generator, &icons, skip_existing).await?;
        }
        Commands::Init => {
            init_icons_dir(&generator)?;
        }
        Commands::List => {
            list_icons(&generator)?;
        }
        Commands::Update => {
            update_icons(&generator).await?;
        }
    }

    Ok(())
}

async fn add_icons(generator: &Generator, inputs: &[String], skip_existing: bool) -> Result<()> {
    // Classify inputs into three categories
    let mut api_identifiers = Vec::new();
    let mut svg_files = Vec::new();
    let mut svg_directories = Vec::new();

    for input in inputs {
        let path = Path::new(input);

        if path.exists() {
            if path.is_dir() {
                svg_directories.push(path.to_path_buf());
            } else if path.extension().and_then(|s| s.to_str()) == Some("svg") {
                svg_files.push(path.to_path_buf());
            } else {
                return Err(anyhow!(
                    "Path exists but is not SVG file or directory: {}",
                    input
                ));
            }
        } else {
            // Not a filesystem path, treat as API identifier
            api_identifiers.push(input.clone());
        }
    }

    let client = IconifyClient::new()?;
    let mut icons_to_add = Vec::new();
    let mut collections = HashSet::new();
    let mut api_collections = HashSet::new(); // Track which collections came from API

    // Process API icons (existing logic)
    if !api_identifiers.is_empty() {
        println!(
            "📦 Fetching {} icon(s) from Iconify API...",
            api_identifiers.len()
        );

        for icon_id in &api_identifiers {
            // Parse icon identifier
            let identifier = IconIdentifier::parse(icon_id)
                .context(format!("Invalid icon identifier: {}", icon_id))?;

            // Track collections
            collections.insert(identifier.collection.clone());
            api_collections.insert(identifier.collection.clone());

            // Fetch icon from API
            print!("  Fetching {}... ", icon_id);
            let icon = client
                .fetch_icon(&identifier.collection, &identifier.icon_name)
                .await
                .context(format!("Failed to fetch icon: {}", icon_id))?;

            println!("✓");

            icons_to_add.push((identifier, icon));
        }
    }

    // Process local SVG files (NEW)
    if !svg_files.is_empty() {
        println!("\n📁 Processing {} local SVG file(s)...", svg_files.len());
        for svg_path in &svg_files {
            match process_single_svg(svg_path) {
                Ok((identifier, icon)) => {
                    println!("  {} ✓", identifier.full_name);
                    collections.insert(identifier.collection.clone());
                    icons_to_add.push((identifier, icon));
                }
                Err(e) => {
                    eprintln!("  ⚠ Skipping {}: {}", svg_path.display(), e);
                }
            }
        }
    }

    // Process SVG directories (NEW)
    if !svg_directories.is_empty() {
        println!(
            "\n📂 Scanning {} director(ies) for SVGs...",
            svg_directories.len()
        );
        for dir_path in &svg_directories {
            let collection = match svg::extract_collection_name(dir_path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("  ⚠ Skipping {}: {}", dir_path.display(), e);
                    continue;
                }
            };

            let svg_files = match svg::scan_svg_directory(dir_path) {
                Ok(files) => files,
                Err(e) => {
                    eprintln!("  ⚠ Error scanning {}: {}", dir_path.display(), e);
                    continue;
                }
            };

            if !svg_files.is_empty() {
                println!(
                    "  Found {} SVG(s) in {}",
                    svg_files.len(),
                    dir_path.display()
                );
            }

            for (svg_path, icon_name) in svg_files {
                let full_name = format!("{}:{}", collection, icon_name);

                match IconIdentifier::parse(&full_name) {
                    Ok(identifier) => match svg::parse_svg_file(&svg_path) {
                        Ok(icon) => {
                            collections.insert(collection.clone());
                            icons_to_add.push((identifier, icon));
                        }
                        Err(e) => {
                            eprintln!("  ⚠ Skipping {}: {}", svg_path.display(), e);
                        }
                    },
                    Err(e) => {
                        eprintln!("  ⚠ Invalid icon name {}: {}", full_name, e);
                    }
                }
            }
        }
    }

    if icons_to_add.is_empty() {
        println!("\n⚠ No icons to add");
        return Ok(());
    }

    // Handle skip-existing flag
    if skip_existing {
        let existing = generator.get_all_icon_identifiers()?;
        let existing_set: HashSet<_> = existing.iter().collect();

        let original_count = icons_to_add.len();
        icons_to_add.retain(|(id, _)| {
            let keep = !existing_set.contains(&id.full_name);
            if !keep {
                println!("  Skipping existing icon: {}", id.full_name);
            }
            keep
        });

        let skipped_count = original_count - icons_to_add.len();
        if skipped_count > 0 {
            println!("\n⏭ Skipped {} existing icon(s)", skipped_count);
        }

        if icons_to_add.is_empty() {
            println!("\n⚠ All icons already exist, nothing to add");
            return Ok(());
        }
    }

    // Fetch collection info only for API collections (not local SVGs)
    let mut collection_info = std::collections::HashMap::new();
    if !api_collections.is_empty() {
        println!("\n📚 Fetching collection metadata...");
        for collection in &api_collections {
            print!("  Fetching info for {}... ", collection);
            match client.fetch_collection_info(collection).await {
                Ok(info) => {
                    println!("✓");
                    collection_info.insert(collection.clone(), info);
                }
                Err(e) => {
                    println!("⚠ (skipped: {})", e);
                    // Continue without collection info - it's optional
                }
            }
        }
    }

    // Generate code
    println!("\n📝 Generating Rust code...");
    generator.add_icons(&icons_to_add, &collection_info)?;

    println!(
        "\n✨ Done! Added {} icon(s) to your project.",
        icons_to_add.len()
    );
    println!("\n💡 Usage:");
    println!("   use icons::Icon;");
    for (identifier, _) in &icons_to_add {
        println!(
            "   use icons::{}::{};",
            identifier.module_name(),
            identifier.to_const_name()
        );
    }
    println!(
        "\n   Icon {{ data: {}::{} }}",
        icons_to_add[0].0.module_name(),
        icons_to_add[0].0.to_const_name()
    );

    Ok(())
}

/// Helper function to process a single SVG file
fn process_single_svg(svg_path: &Path) -> Result<(IconIdentifier, api::IconifyIcon)> {
    let collection = svg::extract_collection_name(
        svg_path
            .parent()
            .ok_or_else(|| anyhow!("No parent directory for: {}", svg_path.display()))?,
    )?;

    let icon_name = svg_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("Invalid filename: {}", svg_path.display()))?
        .to_string();

    let full_name = format!("{}:{}", collection, icon_name);
    let identifier = IconIdentifier::parse(&full_name)?;
    let icon = svg::parse_svg_file(svg_path)?;

    Ok((identifier, icon))
}

fn init_icons_dir(generator: &Generator) -> Result<()> {
    println!("🔧 Initializing icons directory...");
    generator.init()?;
    println!("✨ Created icons directory with mod.rs");
    println!("\n💡 Next: Run `dioxus-iconify add <icon>` to add icons");
    println!("   Example: dioxus-iconify add mdi:home");
    Ok(())
}

fn list_icons(generator: &Generator) -> Result<()> {
    let icons_by_collection = generator.list_icons()?;

    if icons_by_collection.is_empty() {
        println!("No icons found.");
        println!("\n💡 Add icons with: dioxus-iconify add <icon>");
        println!("   Example: dioxus-iconify add mdi:home");
        return Ok(());
    }

    let total_icons: usize = icons_by_collection.values().map(|v| v.len()).sum();
    println!(
        "📦 Found {} icon(s) across {} collection(s):\n",
        total_icons,
        icons_by_collection.len()
    );

    for (collection, icons) in &icons_by_collection {
        println!(
            "{}/ ({} icon{})",
            collection,
            icons.len(),
            if icons.len() == 1 { "" } else { "s" }
        );
        for icon in icons {
            println!("  {}", icon);
        }
        println!();
    }

    Ok(())
}

async fn update_icons(generator: &Generator) -> Result<()> {
    println!("🔄 Updating all icons...");

    // Get all existing icon identifiers
    let icon_ids = generator.get_all_icon_identifiers()?;

    if icon_ids.is_empty() {
        println!("No icons to update.");
        println!("\n💡 Add icons first with: dioxus-iconify add <icon>");
        println!("   Example: dioxus-iconify add mdi:home");
        return Ok(());
    }

    println!("📦 Found {} icon(s) to update", icon_ids.len());
    println!("\n🌐 Fetching latest versions from Iconify API...");

    let client = IconifyClient::new()?;
    let mut icons_to_update = Vec::new();
    let mut failed_icons = Vec::new();
    let mut collections = std::collections::HashSet::new();

    for icon_id in &icon_ids {
        // Parse icon identifier
        let identifier = match IconIdentifier::parse(icon_id) {
            Ok(id) => id,
            Err(e) => {
                eprintln!("  ⚠ Skipping invalid icon identifier {}: {}", icon_id, e);
                failed_icons.push(icon_id.clone());
                continue;
            }
        };

        // Track collections
        collections.insert(identifier.collection.clone());

        // Fetch icon from API
        print!("  Fetching {}... ", icon_id);
        match client
            .fetch_icon(&identifier.collection, &identifier.icon_name)
            .await
        {
            Ok(icon) => {
                println!("✓");
                icons_to_update.push((identifier, icon));
            }
            Err(e) => {
                println!("✗");
                eprintln!("    Error: {}", e);
                failed_icons.push(icon_id.clone());
            }
        }
    }

    if icons_to_update.is_empty() {
        eprintln!("\n❌ Failed to fetch any icons");
        return Ok(());
    }

    // Fetch collection info for all unique collections
    println!("\n📚 Fetching collection metadata...");
    let mut collection_info = std::collections::HashMap::new();
    for collection in collections {
        print!("  Fetching info for {}... ", collection);
        match client.fetch_collection_info(&collection).await {
            Ok(info) => {
                println!("✓");
                collection_info.insert(collection, info);
            }
            Err(e) => {
                println!("⚠ (skipped: {})", e);
                // Continue without collection info - it's optional
            }
        }
    }

    // Regenerate code
    println!("\n📝 Regenerating Rust code...");
    generator.add_icons(&icons_to_update, &collection_info)?;

    // Force regenerate mod.rs to ensure Icon component is up to date
    generator.regenerate_mod_rs()?;

    println!(
        "\n✨ Updated {} icon(s) successfully!",
        icons_to_update.len()
    );

    if !failed_icons.is_empty() {
        println!("\n⚠ Failed to update {} icon(s):", failed_icons.len());
        for icon_id in &failed_icons {
            println!("  - {}", icon_id);
        }
    }

    Ok(())
}
