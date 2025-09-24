use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::env;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(name = "idf-rs")]
#[command(about = "ESP-IDF CLI build management tool (Rust implementation)")]
struct Cli {
    /// Show IDF version and exit
    #[arg(long)]
    version: bool,

    /// Print list of supported targets and exit
    #[arg(long, alias = "list-targets")]
    list_targets: bool,

    /// Project directory
    #[arg(short = 'C', long = "project-dir")]
    project_dir: Option<PathBuf>,

    /// Build directory
    #[arg(short = 'B', long = "build-dir")]
    build_dir: Option<PathBuf>,

    /// Verbose build output
    #[arg(short, long)]
    verbose: bool,

    /// Enable IDF features that are still in preview
    #[arg(long)]
    preview: bool,

    /// Use ccache in build
    #[arg(long)]
    ccache: bool,

    /// Disable ccache in build
    #[arg(long = "no-ccache")]
    no_ccache: bool,

    /// CMake generator
    #[arg(short = 'G', long = "generator")]
    generator: Option<String>,

    /// Disable hints on how to resolve errors and logging
    #[arg(long = "no-hints")]
    no_hints: bool,

    /// Create a cmake cache entry
    #[arg(short = 'D', long = "define-cache-entry")]
    define_cache_entry: Option<String>,

    /// Serial port
    #[arg(short = 'p', long = "port")]
    port: Option<String>,

    /// Global baud rate
    #[arg(short = 'b', long = "baud")]
    baud: Option<u32>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the project
    #[command(alias = "all")]
    Build {
        /// Additional build arguments
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Build only the app
    App,
    /// Build only bootloader
    Bootloader,
    /// Delete build output files from the build directory
    Clean,
    /// Delete the entire build directory contents
    Fullclean,
    /// Flash the project
    Flash {
        /// Flash arguments
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Flash the app only
    AppFlash,
    /// Flash bootloader only
    BootloaderFlash,
    /// Display serial output
    Monitor {
        /// Monitor arguments
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run "menuconfig" project configuration tool
    Menuconfig,
    /// Set the chip target to build
    SetTarget {
        /// Target chip (e.g., esp32, esp32s3, etc.)
        target: String,
    },
    /// Erase entire flash chip
    EraseFlash,
    /// Print basic size information about the app
    Size,
    /// Print per-component size information
    SizeComponents,
    /// Print per-source-file size information
    SizeFiles,
    /// Re-run CMake
    Reconfigure,
    /// Create a new project
    CreateProject {
        /// Project name
        name: String,
        /// Project path
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    /// Print list of build system targets
    BuildSystemTargets,
}

mod commands;
mod config;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    // Handle the special case of "flash monitor" by checking raw args
    let args: Vec<String> = env::args().collect();
    let has_flash_monitor = args.windows(2).any(|window| {
        window[0] == "flash" && window[1] == "monitor"
    });
    
    let cli = Cli::parse();

    // Handle global flags first
    if cli.version {
        println!("ESP-IDF Rust CLI v{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if cli.list_targets {
        utils::list_targets();
        return Ok(());
    }

    // Execute the command
    match &cli.command {
        Some(Commands::Build { args }) => {
            commands::build::execute(&cli, args).await
        }
        Some(Commands::App) => {
            commands::build::execute_app(&cli).await
        }
        Some(Commands::Bootloader) => {
            commands::build::execute_bootloader(&cli).await
        }
        Some(Commands::Clean) => {
            commands::build::execute_clean(&cli).await
        }
        Some(Commands::Fullclean) => {
            commands::build::execute_fullclean(&cli).await
        }
        Some(Commands::Flash { args }) => {
            commands::flash::execute(&cli, args).await?;
            
            // If "flash monitor" was detected, start monitor after successful flash
            if has_flash_monitor {
                println!("Starting monitor after successful flash...");
                commands::monitor::execute(&cli, &[]).await
            } else {
                Ok(())
            }
        }
        Some(Commands::AppFlash) => {
            commands::flash::execute_app(&cli).await
        }
        Some(Commands::BootloaderFlash) => {
            commands::flash::execute_bootloader(&cli).await
        }
        Some(Commands::Monitor { args }) => {
            commands::monitor::execute(&cli, args).await
        }
        Some(Commands::Menuconfig) => {
            commands::config::execute_menuconfig(&cli).await
        }
        Some(Commands::SetTarget { target }) => {
            commands::config::execute_set_target(&cli, target).await
        }
        Some(Commands::EraseFlash) => {
            commands::flash::execute_erase(&cli).await
        }
        Some(Commands::Size) => {
            commands::size::execute(&cli).await
        }
        Some(Commands::SizeComponents) => {
            commands::size::execute_components(&cli).await
        }
        Some(Commands::SizeFiles) => {
            commands::size::execute_files(&cli).await
        }
        Some(Commands::Reconfigure) => {
            commands::build::execute_reconfigure(&cli).await
        }
        Some(Commands::CreateProject { name, path }) => {
            let path_ref = path.as_deref();
            commands::project::create_project(&cli, name, path_ref).await
        }
        Some(Commands::BuildSystemTargets) => {
            commands::build::list_build_targets(&cli).await
        }
        None => {
            // Default behavior - show help
            println!("No command specified. Use --help for available commands.");
            Ok(())
        }
    }
}
