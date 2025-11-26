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
    // Future commands (not yet implemented)
    // /// Remove icons from your project
    // #[command(visible_alias = "r")]
    // Remove {
    //     icons: Vec<String>,
    // },
    //
    // /// List all generated icons
    // #[command(visible_alias = "l")]
    // List,
    //
    // /// Update all icons by re-fetching from API
    // #[command(visible_alias = "u")]
    // Update,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {:#}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let generator = Generator::new(cli.output.clone());

    match cli.command {
        Commands::Add { icons } => {
            add_icons(&generator, &icons)?;
        }
        Commands::Init => {
            init_icons_dir(&generator)?;
        }
    }

    Ok(())
}

fn add_icons(generator: &Generator, icon_ids: &[String]) -> Result<()> {
    println!("📦 Fetching {} icon(s) from Iconify API...", icon_ids.len());

    let client = IconifyClient::new()?;
    let mut icons_to_add = Vec::new();

    for icon_id in icon_ids {
        // Parse icon identifier
        let identifier = IconIdentifier::parse(icon_id)
            .context(format!("Invalid icon identifier: {}", icon_id))?;

        // Fetch icon from API
        print!("  Fetching {}... ", icon_id);
        let icon = client
            .fetch_icon(&identifier.collection, &identifier.icon_name)
            .context(format!("Failed to fetch icon: {}", icon_id))?;

        println!("✓");

        icons_to_add.push((identifier, icon));
    }

    // Generate code
    println!("\n📝 Generating Rust code...");
    generator.add_icons(&icons_to_add)?;

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
