use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use dioxus_iconify::api::IconifyClient;
use dioxus_iconify::generator::Generator;
use dioxus_iconify::naming::IconIdentifier;

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
        /// Icon identifiers in the format collection:icon-name (e.g., mdi:home, heroicons:arrow-left)
        #[arg(required = true)]
        icons: Vec<String>,
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
        Commands::Add { icons } => {
            add_icons(&generator, &icons).await?;
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

async fn add_icons(generator: &Generator, icon_ids: &[String]) -> Result<()> {
    println!("📦 Fetching {} icon(s) from Iconify API...", icon_ids.len());

    let client = IconifyClient::new()?;
    let mut icons_to_add = Vec::new();
    let mut collections = std::collections::HashSet::new();

    for icon_id in icon_ids {
        // Parse icon identifier
        let identifier = IconIdentifier::parse(icon_id)
            .context(format!("Invalid icon identifier: {}", icon_id))?;

        // Track collections
        collections.insert(identifier.collection.clone());

        // Fetch icon from API
        print!("  Fetching {}... ", icon_id);
        let icon = client
            .fetch_icon(&identifier.collection, &identifier.icon_name)
            .await
            .context(format!("Failed to fetch icon: {}", icon_id))?;

        println!("✓");

        icons_to_add.push((identifier, icon));
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

    // Generate code
    println!("\n📝 Generating Rust code...");
    generator.add_icons(&icons_to_add, &collection_info)?;

    println!(
        "\n✨ Done! Added {} icon(s) to your project.",
        icon_ids.len()
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
