use crate::{utils, Cli};
use anyhow::Result;

pub async fn execute(cli: &Cli, args: &[String]) -> Result<()> {
    utils::setup_idf_environment()?;

    let project_dir = utils::get_project_dir(cli.project_dir.as_deref());

    println!("Starting monitor...");

    let python = utils::get_python_executable()?;
    let idf_path = utils::get_idf_path()?;
    let monitor_path = idf_path.join("tools/idf_monitor.py");

    let mut monitor_args = vec![monitor_path.to_str().unwrap()];

    // Add port if specified
    if let Some(port) = &cli.port {
        monitor_args.extend_from_slice(&["--port", port]);
    }

    // Add baud rate
    let baud_str = cli.baud.unwrap_or(115200).to_string();
    monitor_args.extend_from_slice(&["--baud", &baud_str]);

    // Add ELF file for symbol resolution
    let build_dir = utils::get_build_dir(cli.build_dir.as_deref(), &project_dir);
    let elf_file = build_dir.join("project.elf"); // This might need to be project-specific

    if elf_file.exists() {
        monitor_args.push(elf_file.to_str().unwrap());
    }

    // Add additional arguments
    for arg in args {
        monitor_args.push(arg);
    }

    utils::run_command(&python, &monitor_args, Some(&project_dir), cli.verbose).await?;

    Ok(())
}
