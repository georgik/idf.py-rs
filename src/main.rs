use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
#[command(name = "idf-rs")]
#[command(about = "ESP-IDF CLI build management tool (Rust implementation)")]
struct Cli {
    /// Show IDF version and exit
    #[arg(long = "idf-version")]
    idf_version: bool,

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

#[derive(Subcommand, Debug, Clone)]
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
        /// Extra arguments to pass to esptool
        #[arg(long = "extra-args")]
        extra_args: Option<String>,
        /// Force write, skip security and compatibility checks
        #[arg(long)]
        force: bool,
        /// Enable trace-level output of flasher tool interactions
        #[arg(long)]
        trace: bool,
        /// Flash arguments
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Flash the app only
    AppFlash {
        /// Extra arguments to pass to esptool
        #[arg(long = "extra-args")]
        extra_args: Option<String>,
        /// Force write, skip security and compatibility checks
        #[arg(long)]
        force: bool,
        /// Enable trace-level output of flasher tool interactions
        #[arg(long)]
        trace: bool,
    },
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
    /// Install idf-rs as idf.py replacement (creates symlink)
    InstallAlias {
        /// Force installation even if backup exists
        #[arg(long)]
        force: bool,
    },
    /// Uninstall idf-rs alias and restore original idf.py
    UninstallAlias,
}

mod build_systems;
mod commands;
mod config;
mod utils;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct EimIdfConfig {
    #[serde(rename = "gitPath")]
    git_path: String,
    #[serde(rename = "idfInstalled")]
    idf_installed: Vec<EimIdfInstallation>,
    #[serde(rename = "idfSelectedId")]
    idf_selected_id: String,
    #[serde(rename = "eimPath")]
    eim_path: String,
    version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct EimIdfInstallation {
    #[serde(rename = "activationScript")]
    activation_script: String,
    id: String,
    #[serde(rename = "idfToolsPath")]
    idf_tools_path: String,
    name: String,
    path: String,
    python: String,
}

#[derive(Debug, Clone)]
struct ParsedCommand {
    name: String,
    args: Vec<String>,
}

#[derive(Debug, Clone)]
struct MultipleCommands {
    global_args: Cli,
    commands: Vec<ParsedCommand>,
}

/// Parse command line arguments to detect multiple commands
fn parse_multiple_commands(args: &[String]) -> Result<MultipleCommands> {
    // List of known commands that can be chained
    let known_commands = [
        "build",
        "all",
        "app",
        "bootloader",
        "clean",
        "fullclean",
        "flash",
        "app-flash",
        "bootloader-flash",
        "monitor",
        "menuconfig",
        "set-target",
        "erase-flash",
        "size",
        "size-components",
        "size-files",
        "reconfigure",
        "create-project",
        "build-system-targets",
        "install-alias",
        "uninstall-alias",
    ];

    if args.len() < 2 {
        return Err(anyhow::anyhow!("No commands provided"));
    }

    let mut commands = Vec::new();
    let mut global_args = Vec::new();
    let mut current_command: Option<String> = None;
    let mut current_args = Vec::new();
    let mut found_multiple_commands = false;

    // Skip program name
    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];

        // Check if this is a known command
        if known_commands.contains(&arg.as_str()) {
            // Save previous command if exists
            if let Some(cmd) = current_command.take() {
                commands.push(ParsedCommand {
                    name: cmd,
                    args: current_args.clone(),
                });
                current_args.clear();
                found_multiple_commands = true;
            } else if !commands.is_empty() {
                found_multiple_commands = true;
            }

            current_command = Some(arg.clone());
        } else if current_command.is_some() {
            // This is an argument for the current command
            current_args.push(arg.clone());
        } else {
            // This is a global argument (before any commands)
            global_args.push(arg.clone());
        }

        i += 1;
    }

    // Save the last command
    if let Some(cmd) = current_command {
        commands.push(ParsedCommand {
            name: cmd,
            args: current_args,
        });
    }

    // Only return Ok if we found multiple commands or no commands at all
    if commands.len() > 1 || (commands.len() == 1 && found_multiple_commands) {
        // Parse global arguments - create a minimal CLI with defaults
        let cli = Cli {
            idf_version: false,
            list_targets: false,
            project_dir: None,
            build_dir: None,
            verbose: global_args.contains(&"-v".to_string())
                || global_args.contains(&"--verbose".to_string()),
            preview: global_args.contains(&"--preview".to_string()),
            ccache: global_args.contains(&"--ccache".to_string()),
            no_ccache: global_args.contains(&"--no-ccache".to_string()),
            generator: None, // TODO: parse -G
            no_hints: global_args.contains(&"--no-hints".to_string()),
            define_cache_entry: None, // TODO: parse -D
            port: None,               // TODO: parse -p
            baud: None,               // TODO: parse -b
            command: None,
        };

        Ok(MultipleCommands {
            global_args: cli,
            commands,
        })
    } else {
        Err(anyhow::anyhow!(
            "Single command detected, use normal parsing"
        ))
    }
}

/// Execute multiple commands in sequence
async fn execute_multiple_commands(parsed: MultipleCommands) -> Result<()> {
    println!(
        "Executing {} commands in sequence...",
        parsed.commands.len()
    );

    for (i, cmd) in parsed.commands.iter().enumerate() {
        println!(
            "[{}/{}] Executing command: {}",
            i + 1,
            parsed.commands.len(),
            cmd.name
        );

        // Execute each command
        match execute_single_command(&parsed.global_args, cmd).await {
            Ok(()) => {
                println!(
                    "[{}/{}] Command '{}' completed successfully",
                    i + 1,
                    parsed.commands.len(),
                    cmd.name
                );
            }
            Err(e) => {
                eprintln!(
                    "[{}/{}] Command '{}' failed: {}",
                    i + 1,
                    parsed.commands.len(),
                    cmd.name,
                    e
                );
                return Err(e);
            }
        }
    }

    println!("All commands completed successfully!");
    Ok(())
}

/// Execute a single parsed command
async fn execute_single_command(cli: &Cli, cmd: &ParsedCommand) -> Result<()> {
    match cmd.name.as_str() {
        "build" | "all" => commands::build::execute(cli, &cmd.args).await,
        "app" => commands::build::execute_app(cli).await,
        "bootloader" => commands::build::execute_bootloader(cli).await,
        "clean" => commands::build::execute_clean(cli).await,
        "fullclean" => commands::build::execute_fullclean(cli).await,
        "flash" => {
            // Parse flash-specific arguments
            commands::flash::execute(cli, &cmd.args, None, false, false).await
        }
        "app-flash" => {
            // Parse app-flash-specific arguments
            commands::flash::execute_app(cli, None, false, false).await
        }
        "bootloader-flash" => commands::flash::execute_bootloader(cli).await,
        "monitor" => commands::monitor::execute(cli, &cmd.args).await,
        "menuconfig" => commands::config::execute_menuconfig(cli).await,
        "set-target" => {
            if let Some(target) = cmd.args.first() {
                commands::config::execute_set_target(cli, target).await
            } else {
                Err(anyhow::anyhow!("set-target requires a target argument"))
            }
        }
        "erase-flash" => commands::flash::execute_erase(cli).await,
        "size" => commands::size::execute(cli).await,
        "size-components" => commands::size::execute_components(cli).await,
        "size-files" => commands::size::execute_files(cli).await,
        "reconfigure" => commands::build::execute_reconfigure(cli).await,
        "create-project" => {
            if let Some(name) = cmd.args.first() {
                commands::project::create_project(cli, name, None).await
            } else {
                Err(anyhow::anyhow!("create-project requires a project name"))
            }
        }
        "build-system-targets" => commands::build::list_build_targets(cli).await,
        "install-alias" => execute_install_alias(false).await,
        "uninstall-alias" => execute_uninstall_alias().await,
        _ => Err(anyhow::anyhow!("Unknown command: {}", cmd.name)),
    }
}

/// Install idf-rs as idf.py replacement
async fn execute_install_alias(force: bool) -> Result<()> {
    println!("Installing idf-rs as idf.py replacement...");

    #[cfg(windows)]
    {
        execute_install_alias_windows(force).await
    }

    #[cfg(not(windows))]
    {
        execute_install_alias_unix(force).await
    }
}

/// Windows-specific install-alias implementation using EIM
#[cfg(windows)]
async fn execute_install_alias_windows(force: bool) -> Result<()> {
    use std::path::Path;

    // Read EIM configuration
    let eim_config_path = Path::new("C:\\Espressif\\tools\\eim_idf.json");
    if !eim_config_path.exists() {
        return Err(anyhow::anyhow!(
            "EIM configuration not found at {}. Please ensure ESP-IDF is installed via EIM (Espressif Installation Manager).",
            eim_config_path.display()
        ));
    }

    let config_content = std::fs::read_to_string(eim_config_path)
        .map_err(|e| anyhow::anyhow!("Failed to read EIM configuration: {}", e))?;

    let config: EimIdfConfig = serde_json::from_str(&config_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse EIM configuration: {}", e))?;

    // Find the current ESP-IDF installation's tools path
    let current_installation = config
        .idf_installed
        .iter()
        .find(|install| install.id == config.idf_selected_id)
        .ok_or_else(|| anyhow::anyhow!("Current ESP-IDF installation not found in EIM config"))?;

    println!(
        "Found ESP-IDF installation: {} at {}",
        current_installation.name, current_installation.path
    );
    println!("Tools path: {}", current_installation.idf_tools_path);

    // The idf-exe directory structure
    let idf_exe_dir = Path::new(&current_installation.idf_tools_path).join("idf-exe");
    if !idf_exe_dir.exists() {
        return Err(anyhow::anyhow!(
            "idf-exe directory not found at {}. This might not be a complete EIM installation.",
            idf_exe_dir.display()
        ));
    }

    // Find the idf.py.exe version directory (should be something like "1.0.3")
    let version_dirs: Vec<_> = std::fs::read_dir(&idf_exe_dir)
        .map_err(|e| anyhow::anyhow!("Failed to read idf-exe directory: {}", e))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_dir() {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    if version_dirs.is_empty() {
        return Err(anyhow::anyhow!(
            "No version directories found in {}",
            idf_exe_dir.display()
        ));
    }

    // Use the first (and typically only) version directory
    let version_dir = &version_dirs[0];
    let original_idf_exe = version_dir.join("idf.py.exe");

    if !original_idf_exe.exists() {
        return Err(anyhow::anyhow!(
            "idf.py.exe not found at {}",
            original_idf_exe.display()
        ));
    }

    println!(
        "Found original idf.py.exe at: {}",
        original_idf_exe.display()
    );

    // Create backup
    let backup_path = version_dir.join("idf.py.exe.backup");
    if backup_path.exists() {
        if !force {
            return Err(anyhow::anyhow!(
                "Backup already exists at {}. Use --force to overwrite.",
                backup_path.display()
            ));
        } else {
            println!("Removing existing backup: {}", backup_path.display());
            std::fs::remove_file(&backup_path)
                .map_err(|e| anyhow::anyhow!("Failed to remove existing backup: {}", e))?;
        }
    }

    // Check if we're already installed
    if original_idf_exe.metadata().map(|m| m.len()).unwrap_or(0)
        == std::env::current_exe().unwrap().metadata().unwrap().len()
    {
        println!("idf-rs appears to already be installed as idf.py.exe");
        return Ok(());
    }

    // Create backup of original
    println!(
        "Creating backup: {} -> {}",
        original_idf_exe.display(),
        backup_path.display()
    );
    std::fs::copy(&original_idf_exe, &backup_path)
        .map_err(|e| anyhow::anyhow!("Failed to create backup: {}", e))?;

    // Get current executable path
    let current_exe = std::env::current_exe()
        .map_err(|e| anyhow::anyhow!("Failed to get current executable path: {}", e))?;

    // Replace the original idf.py.exe with our binary
    println!(
        "Replacing idf.py.exe: {} -> {}",
        current_exe.display(),
        original_idf_exe.display()
    );
    std::fs::copy(&current_exe, &original_idf_exe).map_err(|e| {
        // Try to restore backup if copy fails
        let _ = std::fs::copy(&backup_path, &original_idf_exe);
        let _ = std::fs::remove_file(&backup_path);
        anyhow::anyhow!("Failed to replace idf.py.exe: {}", e)
    })?;

    println!("✅ Successfully installed idf-rs as idf.py replacement!");
    println!(
        "   Original idf.py.exe backed up to: {}",
        backup_path.display()
    );
    println!("   idf.py.exe now points to idf-rs");
    println!("");
    println!("You can now use 'idf.py' commands and they will use the fast Rust implementation.");
    println!("To restore the original, run: idf-rs uninstall-alias");

    Ok(())
}

/// Unix-specific install-alias implementation using symlinks
#[cfg(not(windows))]
async fn execute_install_alias_unix(force: bool) -> Result<()> {
    use std::path::Path;
    use std::process::Command;

    // Find the current idf.py location
    let idf_py_output = Command::new("which")
        .arg("idf.py")
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to locate idf.py: {}", e))?;

    if !idf_py_output.status.success() {
        return Err(anyhow::anyhow!(
            "idf.py not found in PATH. Please ensure ESP-IDF is properly installed."
        ));
    }

    let idf_py_path = String::from_utf8(idf_py_output.stdout)
        .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in idf.py path: {}", e))?
        .trim()
        .to_string();

    let idf_py_path = Path::new(&idf_py_path);

    // Find the current idf-rs location
    let idf_rs_output = Command::new("which")
        .arg("idf-rs")
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to locate idf-rs: {}", e))?;

    if !idf_rs_output.status.success() {
        return Err(anyhow::anyhow!(
            "idf-rs not found in PATH. Please install idf-rs first."
        ));
    }

    let idf_rs_path = String::from_utf8(idf_rs_output.stdout)
        .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in idf-rs path: {}", e))?
        .trim()
        .to_string();

    println!("Found idf.py at: {}", idf_py_path.display());
    println!("Found idf-rs at: {}", idf_rs_path);

    // Create backup path
    let backup_path = idf_py_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine parent directory of idf.py"))?
        .join("idf-old.py");

    // Check if backup already exists
    if backup_path.exists() {
        if !force {
            return Err(anyhow::anyhow!(
                "Backup already exists at {}. Use --force to overwrite.",
                backup_path.display()
            ));
        } else {
            println!("Removing existing backup: {}", backup_path.display());
            std::fs::remove_file(&backup_path)
                .map_err(|e| anyhow::anyhow!("Failed to remove existing backup: {}", e))?;
        }
    }

    // Check if idf.py is already a symlink to idf-rs
    if idf_py_path.is_symlink() {
        let target = std::fs::read_link(&idf_py_path)
            .map_err(|e| anyhow::anyhow!("Failed to read symlink target: {}", e))?;

        if target.to_string_lossy().contains("idf-rs") {
            println!("idf.py is already linked to idf-rs: {}", target.display());
            return Ok(());
        }
    }

    // Step 1: Rename idf.py to idf-old.py
    println!(
        "Creating backup: {} -> {}",
        idf_py_path.display(),
        backup_path.display()
    );
    std::fs::rename(&idf_py_path, &backup_path)
        .map_err(|e| anyhow::anyhow!("Failed to create backup: {}", e))?;

    // Step 2: Create symlink from idf.py to idf-rs
    println!(
        "Creating symlink: {} -> {}",
        idf_py_path.display(),
        idf_rs_path
    );

    std::os::unix::fs::symlink(&idf_rs_path, &idf_py_path).map_err(|e| {
        // Try to restore backup if symlink creation fails
        let _ = std::fs::rename(&backup_path, &idf_py_path);
        anyhow::anyhow!("Failed to create symlink: {}", e)
    })?;

    println!("✅ Successfully installed idf-rs as idf.py replacement!");
    println!("   Original idf.py backed up to: {}", backup_path.display());
    println!("   idf.py now points to: {}", idf_rs_path);
    println!("");
    println!("You can now use 'idf.py' commands and they will use the fast Rust implementation.");
    println!("To restore the original, run: idf-rs uninstall-alias");

    Ok(())
}

/// Uninstall idf-rs alias and restore original idf.py
async fn execute_uninstall_alias() -> Result<()> {
    println!("Uninstalling idf-rs alias and restoring original idf.py...");

    #[cfg(windows)]
    {
        execute_uninstall_alias_windows().await
    }

    #[cfg(not(windows))]
    {
        execute_uninstall_alias_unix().await
    }
}

/// Windows-specific uninstall-alias implementation
#[cfg(windows)]
async fn execute_uninstall_alias_windows() -> Result<()> {
    use std::path::Path;

    // Read EIM configuration
    let eim_config_path = Path::new("C:\\Espressif\\tools\\eim_idf.json");
    if !eim_config_path.exists() {
        return Err(anyhow::anyhow!(
            "EIM configuration not found at {}. Please ensure ESP-IDF is installed via EIM.",
            eim_config_path.display()
        ));
    }

    let config_content = std::fs::read_to_string(eim_config_path)
        .map_err(|e| anyhow::anyhow!("Failed to read EIM configuration: {}", e))?;

    let config: EimIdfConfig = serde_json::from_str(&config_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse EIM configuration: {}", e))?;

    // Find the current ESP-IDF installation's tools path
    let current_installation = config
        .idf_installed
        .iter()
        .find(|install| install.id == config.idf_selected_id)
        .ok_or_else(|| anyhow::anyhow!("Current ESP-IDF installation not found in EIM config"))?;

    // The idf-exe directory structure
    let idf_exe_dir = Path::new(&current_installation.idf_tools_path).join("idf-exe");
    if !idf_exe_dir.exists() {
        return Err(anyhow::anyhow!(
            "idf-exe directory not found at {}.",
            idf_exe_dir.display()
        ));
    }

    // Find the version directory
    let version_dirs: Vec<_> = std::fs::read_dir(&idf_exe_dir)
        .map_err(|e| anyhow::anyhow!("Failed to read idf-exe directory: {}", e))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_dir() {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    if version_dirs.is_empty() {
        return Err(anyhow::anyhow!(
            "No version directories found in {}",
            idf_exe_dir.display()
        ));
    }

    let version_dir = &version_dirs[0];
    let current_idf_exe = version_dir.join("idf.py.exe");
    let backup_path = version_dir.join("idf.py.exe.backup");

    // Check if backup exists
    if !backup_path.exists() {
        return Err(anyhow::anyhow!(
            "No backup found at {}. Cannot restore original idf.py.exe.",
            backup_path.display()
        ));
    }

    // Check if current idf.py.exe exists
    if !current_idf_exe.exists() {
        return Err(anyhow::anyhow!(
            "Current idf.py.exe not found at {}.",
            current_idf_exe.display()
        ));
    }

    println!("Found backup at: {}", backup_path.display());
    println!("Restoring to: {}", current_idf_exe.display());

    // Remove current idf.py.exe
    std::fs::remove_file(&current_idf_exe)
        .map_err(|e| anyhow::anyhow!("Failed to remove current idf.py.exe: {}", e))?;

    // Restore from backup
    println!(
        "Restoring backup: {} -> {}",
        backup_path.display(),
        current_idf_exe.display()
    );
    std::fs::copy(&backup_path, &current_idf_exe)
        .map_err(|e| anyhow::anyhow!("Failed to restore backup: {}", e))?;

    // Remove backup file
    std::fs::remove_file(&backup_path)
        .map_err(|e| anyhow::anyhow!("Failed to remove backup file: {}", e))?;

    println!("✅ Successfully restored original idf.py.exe!");
    println!("   idf.py.exe now points to the original ESP-IDF Python implementation.");

    Ok(())
}

/// Unix-specific uninstall-alias implementation
#[cfg(not(windows))]
async fn execute_uninstall_alias_unix() -> Result<()> {
    use std::path::Path;
    use std::process::Command;

    // Find the current idf.py location
    let idf_py_output = Command::new("which")
        .arg("idf.py")
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to locate idf.py: {}", e))?;

    if !idf_py_output.status.success() {
        return Err(anyhow::anyhow!("idf.py not found in PATH."));
    }

    let idf_py_path = String::from_utf8(idf_py_output.stdout)
        .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in idf.py path: {}", e))?
        .trim()
        .to_string();

    let idf_py_path = Path::new(&idf_py_path);

    // Create backup path
    let backup_path = idf_py_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine parent directory of idf.py"))?
        .join("idf-old.py");

    // Check if backup exists
    if !backup_path.exists() {
        return Err(anyhow::anyhow!(
            "No backup found at {}. Cannot restore original idf.py.",
            backup_path.display()
        ));
    }

    // Check if current idf.py is our symlink
    if !idf_py_path.is_symlink() {
        return Err(anyhow::anyhow!(
            "Current idf.py at {} is not a symlink. Manual intervention required.",
            idf_py_path.display()
        ));
    }

    // Remove the symlink
    println!("Removing symlink: {}", idf_py_path.display());
    std::fs::remove_file(&idf_py_path)
        .map_err(|e| anyhow::anyhow!("Failed to remove symlink: {}", e))?;

    // Restore the backup
    println!(
        "Restoring backup: {} -> {}",
        backup_path.display(),
        idf_py_path.display()
    );
    std::fs::rename(&backup_path, &idf_py_path)
        .map_err(|e| anyhow::anyhow!("Failed to restore backup: {}", e))?;

    println!("✅ Successfully restored original idf.py!");
    println!("   idf.py now points to the original ESP-IDF Python implementation.");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // Parse raw arguments to detect multiple commands
    let args: Vec<String> = env::args().collect();

    // Handle multiple commands (e.g., "idf-rs build flash monitor")
    if let Ok(parsed_commands) = parse_multiple_commands(&args) {
        return execute_multiple_commands(parsed_commands).await;
    }

    // Handle the special case of "flash monitor" by checking raw args
    let has_flash_monitor = args
        .windows(2)
        .any(|window| window[0] == "flash" && window[1] == "monitor");

    let cli = Cli::parse();

    // Handle global flags first
    if cli.idf_version {
        println!("ESP-IDF Rust CLI v{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if cli.list_targets {
        utils::list_targets();
        return Ok(());
    }

    // Execute the command
    match &cli.command {
        Some(Commands::Build { args }) => commands::build::execute(&cli, args).await,
        Some(Commands::App) => commands::build::execute_app(&cli).await,
        Some(Commands::Bootloader) => commands::build::execute_bootloader(&cli).await,
        Some(Commands::Clean) => commands::build::execute_clean(&cli).await,
        Some(Commands::Fullclean) => commands::build::execute_fullclean(&cli).await,
        Some(Commands::Flash {
            extra_args,
            force,
            trace,
            args,
        }) => {
            commands::flash::execute(&cli, args, extra_args.as_deref(), *force, *trace).await?;

            // If "flash monitor" was detected, start monitor after successful flash
            if has_flash_monitor {
                println!("Starting monitor after successful flash...");
                commands::monitor::execute(&cli, &[]).await
            } else {
                Ok(())
            }
        }
        Some(Commands::AppFlash {
            extra_args,
            force,
            trace,
        }) => commands::flash::execute_app(&cli, extra_args.as_deref(), *force, *trace).await,
        Some(Commands::BootloaderFlash) => commands::flash::execute_bootloader(&cli).await,
        Some(Commands::Monitor { args }) => commands::monitor::execute(&cli, args).await,
        Some(Commands::Menuconfig) => commands::config::execute_menuconfig(&cli).await,
        Some(Commands::SetTarget { target }) => {
            commands::config::execute_set_target(&cli, target).await
        }
        Some(Commands::EraseFlash) => commands::flash::execute_erase(&cli).await,
        Some(Commands::Size) => commands::size::execute(&cli).await,
        Some(Commands::SizeComponents) => commands::size::execute_components(&cli).await,
        Some(Commands::SizeFiles) => commands::size::execute_files(&cli).await,
        Some(Commands::Reconfigure) => commands::build::execute_reconfigure(&cli).await,
        Some(Commands::CreateProject { name, path }) => {
            let path_ref = path.as_deref();
            commands::project::create_project(&cli, name, path_ref).await
        }
        Some(Commands::BuildSystemTargets) => commands::build::list_build_targets(&cli).await,
        Some(Commands::InstallAlias { force }) => execute_install_alias(*force).await,
        Some(Commands::UninstallAlias) => execute_uninstall_alias().await,
        None => {
            // Default behavior - show help
            println!("No command specified. Use --help for available commands.");
            Ok(())
        }
    }
}
