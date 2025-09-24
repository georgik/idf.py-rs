use crate::{utils, Cli};
use anyhow::Result;

pub async fn execute(cli: &Cli, _args: &[String]) -> Result<()> {
    utils::setup_idf_environment()?;

    let project_dir = utils::get_project_dir(cli.project_dir.as_deref());
    let build_dir = utils::get_build_dir(cli.build_dir.as_deref(), &project_dir);

    println!("Flashing project...");

    // First, ensure the project is built
    if !build_dir.exists() {
        println!("Build directory doesn't exist. Building project first...");
        crate::commands::build::execute(cli, &[]).await?;
    }

    // Use CMake flash target which handles all the complexity
    let flash_args = vec!["--build", build_dir.to_str().unwrap(), "--target", "flash"];

    // Set environment variables for port and baud if specified
    let mut env_vars = Vec::new();
    let baud_str;
    if let Some(port) = &cli.port {
        env_vars.push(("ESPPORT", port.as_str()));
    }
    if let Some(baud) = cli.baud {
        baud_str = baud.to_string();
        env_vars.push(("ESPBAUD", &baud_str));
    }

    // Set environment variables
    for (key, value) in &env_vars {
        std::env::set_var(key, value);
    }

    utils::run_command("cmake", &flash_args, Some(&project_dir), cli.verbose).await?;

    // Clean up environment variables
    for (key, _) in &env_vars {
        std::env::remove_var(key);
    }

    println!("Flash completed successfully!");
    Ok(())
}

pub async fn execute_app(cli: &Cli) -> Result<()> {
    utils::setup_idf_environment()?;

    let project_dir = utils::get_project_dir(cli.project_dir.as_deref());
    let build_dir = utils::get_build_dir(cli.build_dir.as_deref(), &project_dir);

    println!("Flashing app only...");

    // Build app if needed
    if !build_dir.join("app.bin").exists() {
        println!("App binary doesn't exist. Building app first...");
        crate::commands::build::execute_app(cli).await?;
    }

    // Flash app binary
    let python = utils::get_python_executable()?;
    let idf_path = utils::get_idf_path()?;
    let esptool_path = idf_path.join("components/esptool_py/esptool/esptool.py");

    let baud_str = cli.baud.unwrap_or(460800).to_string();
    let app_bin_path = build_dir.join("app.bin");
    let mut flash_args = vec![
        esptool_path.to_str().unwrap(),
        "--chip",
        "auto",
        "--baud",
        &baud_str,
    ];

    if let Some(port) = &cli.port {
        flash_args.extend_from_slice(&["--port", port]);
    }

    flash_args.extend_from_slice(&[
        "write_flash",
        "0x10000", // Default app offset
        app_bin_path.to_str().unwrap(),
    ]);

    utils::run_command(&python, &flash_args, Some(&project_dir), cli.verbose).await?;

    println!("App flash completed successfully!");
    Ok(())
}

pub async fn execute_bootloader(cli: &Cli) -> Result<()> {
    utils::setup_idf_environment()?;

    let project_dir = utils::get_project_dir(cli.project_dir.as_deref());
    let build_dir = utils::get_build_dir(cli.build_dir.as_deref(), &project_dir);

    println!("Flashing bootloader only...");

    // Build bootloader if needed
    if !build_dir.join("bootloader").join("bootloader.bin").exists() {
        println!("Bootloader binary doesn't exist. Building bootloader first...");
        crate::commands::build::execute_bootloader(cli).await?;
    }

    // Flash bootloader binary
    let python = utils::get_python_executable()?;
    let idf_path = utils::get_idf_path()?;
    let esptool_path = idf_path.join("components/esptool_py/esptool/esptool.py");

    let baud_str = cli.baud.unwrap_or(460800).to_string();
    let bootloader_bin_path = build_dir.join("bootloader").join("bootloader.bin");
    let mut flash_args = vec![
        esptool_path.to_str().unwrap(),
        "--chip",
        "auto",
        "--baud",
        &baud_str,
    ];

    if let Some(port) = &cli.port {
        flash_args.extend_from_slice(&["--port", port]);
    }

    flash_args.extend_from_slice(&[
        "write_flash",
        "0x1000", // Default bootloader offset
        bootloader_bin_path.to_str().unwrap(),
    ]);

    utils::run_command(&python, &flash_args, Some(&project_dir), cli.verbose).await?;

    println!("Bootloader flash completed successfully!");
    Ok(())
}

pub async fn execute_erase(cli: &Cli) -> Result<()> {
    utils::setup_idf_environment()?;

    let project_dir = utils::get_project_dir(cli.project_dir.as_deref());

    println!("Erasing flash...");

    let python = utils::get_python_executable()?;
    let idf_path = utils::get_idf_path()?;
    let esptool_path = idf_path.join("components/esptool_py/esptool/esptool.py");

    let baud_str = cli.baud.unwrap_or(460800).to_string();
    let mut erase_args = vec![
        esptool_path.to_str().unwrap(),
        "--chip",
        "auto",
        "--baud",
        &baud_str,
    ];

    if let Some(port) = &cli.port {
        erase_args.extend_from_slice(&["--port", port]);
    }

    erase_args.push("erase_flash");

    utils::run_command(&python, &erase_args, Some(&project_dir), cli.verbose).await?;

    println!("Flash erase completed successfully!");
    Ok(())
}
