use crate::{build_systems, utils, Cli};
use anyhow::Result;

pub async fn execute(cli: &Cli, args: &[String]) -> Result<()> {
    utils::setup_idf_environment()?;

    let project_dir = utils::get_project_dir(cli.project_dir.as_deref());
    let build_dir = utils::get_build_dir(cli.build_dir.as_deref(), &project_dir);

    println!("Building project in: {}", project_dir.display());
    println!("Build directory: {}", build_dir.display());

    // Get the appropriate generator (explicit, cached, or auto-detected)
    let generator = build_systems::get_build_generator(cli.generator.as_ref(), &build_dir)?;

    println!("Using generator: {}", generator);

    let mut cmake_args = vec![
        "-B",
        build_dir.to_str().unwrap(),
        "-S",
        project_dir.to_str().unwrap(),
        "-G",
        &generator,
    ];

    // Add cache entry if specified
    if let Some(cache_entry) = &cli.define_cache_entry {
        cmake_args.extend_from_slice(&["-D", cache_entry]);
    }

    // Configure step
    utils::run_command("cmake", &cmake_args, Some(&project_dir), cli.verbose).await?;

    // Build step
    let mut build_args = vec!["--build", build_dir.to_str().unwrap()];

    if cli.verbose {
        build_args.push("--verbose");
    }

    // Add additional arguments
    if !args.is_empty() {
        build_args.push("--");
        for arg in args {
            build_args.push(arg);
        }
    }

    utils::run_command("cmake", &build_args, Some(&project_dir), cli.verbose).await?;

    println!("Build completed successfully!");
    Ok(())
}

pub async fn execute_app(cli: &Cli) -> Result<()> {
    utils::setup_idf_environment()?;

    let project_dir = utils::get_project_dir(cli.project_dir.as_deref());
    let build_dir = utils::get_build_dir(cli.build_dir.as_deref(), &project_dir);

    println!("Building app only...");

    let build_args = vec!["--build", build_dir.to_str().unwrap(), "--target", "app"];

    utils::run_command("cmake", &build_args, Some(&project_dir), cli.verbose).await?;

    println!("App build completed successfully!");
    Ok(())
}

pub async fn execute_bootloader(cli: &Cli) -> Result<()> {
    utils::setup_idf_environment()?;

    let project_dir = utils::get_project_dir(cli.project_dir.as_deref());
    let build_dir = utils::get_build_dir(cli.build_dir.as_deref(), &project_dir);

    println!("Building bootloader only...");

    let build_args = vec![
        "--build",
        build_dir.to_str().unwrap(),
        "--target",
        "bootloader",
    ];

    utils::run_command("cmake", &build_args, Some(&project_dir), cli.verbose).await?;

    println!("Bootloader build completed successfully!");
    Ok(())
}

pub async fn execute_clean(cli: &Cli) -> Result<()> {
    let project_dir = utils::get_project_dir(cli.project_dir.as_deref());
    let build_dir = utils::get_build_dir(cli.build_dir.as_deref(), &project_dir);

    println!("Cleaning build directory: {}", build_dir.display());

    if build_dir.exists() {
        let build_args = vec!["--build", build_dir.to_str().unwrap(), "--target", "clean"];

        utils::run_command("cmake", &build_args, Some(&project_dir), cli.verbose).await?;
        println!("Clean completed successfully!");
    } else {
        println!("Build directory doesn't exist, nothing to clean.");
    }

    Ok(())
}

pub async fn execute_fullclean(cli: &Cli) -> Result<()> {
    let project_dir = utils::get_project_dir(cli.project_dir.as_deref());
    let build_dir = utils::get_build_dir(cli.build_dir.as_deref(), &project_dir);

    println!("Removing entire build directory: {}", build_dir.display());

    if build_dir.exists() {
        std::fs::remove_dir_all(&build_dir)?;
        println!("Build directory removed successfully!");
    } else {
        println!("Build directory doesn't exist, nothing to remove.");
    }

    Ok(())
}

pub async fn execute_reconfigure(cli: &Cli) -> Result<()> {
    utils::setup_idf_environment()?;

    let project_dir = utils::get_project_dir(cli.project_dir.as_deref());
    let build_dir = utils::get_build_dir(cli.build_dir.as_deref(), &project_dir);

    println!("Reconfiguring project...");

    // Remove CMake cache to force reconfigure
    let cmake_cache = build_dir.join("CMakeCache.txt");
    if cmake_cache.exists() {
        std::fs::remove_file(&cmake_cache)?
    }

    // Get the appropriate generator (explicit or auto-detected, since cache was removed)
    let generator = build_systems::get_build_generator(cli.generator.as_ref(), &build_dir)?;

    println!("Using generator: {}", generator);

    let cmake_args = vec![
        "-B",
        build_dir.to_str().unwrap(),
        "-S",
        project_dir.to_str().unwrap(),
        "-G",
        &generator,
    ];

    utils::run_command("cmake", &cmake_args, Some(&project_dir), cli.verbose).await?;

    println!("Reconfigure completed successfully!");
    Ok(())
}

pub async fn list_build_targets(cli: &Cli) -> Result<()> {
    utils::setup_idf_environment()?;

    let project_dir = utils::get_project_dir(cli.project_dir.as_deref());
    let build_dir = utils::get_build_dir(cli.build_dir.as_deref(), &project_dir);

    if !build_dir.exists() {
        println!("Build directory doesn't exist. Run 'build' command first.");
        return Ok(());
    }

    println!("Available build system targets:");

    // Use cmake to list targets
    let output = utils::run_command_with_output(
        "cmake",
        &["--build", build_dir.to_str().unwrap(), "--target", "help"],
        Some(&project_dir),
    )
    .await?;

    println!("{}", output);
    Ok(())
}
