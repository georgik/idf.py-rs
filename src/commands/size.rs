use crate::{utils, Cli};
use anyhow::Result;

pub async fn execute(cli: &Cli) -> Result<()> {
    utils::setup_idf_environment()?;
    
    let project_dir = utils::get_project_dir(cli.project_dir.as_deref());
    let build_dir = utils::get_build_dir(cli.build_dir.as_deref(), &project_dir);
    
    if !build_dir.exists() {
        return Err(anyhow::anyhow!("Build directory doesn't exist. Run 'build' command first."));
    }
    
    println!("Getting project size information...");
    
    let python = utils::get_python_executable()?;
    let idf_path = utils::get_idf_path()?;
    let size_tool_path = idf_path.join("tools/idf_size.py");
    
    let mut size_args = vec![
        size_tool_path.to_str().unwrap(),
    ];
    
    // Find the ELF file - typically project_name.elf in build directory
    let elf_files: Vec<_> = std::fs::read_dir(&build_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            if let Some(extension) = entry.path().extension() {
                extension == "elf"
            } else {
                false
            }
        })
        .collect();
    
    if elf_files.is_empty() {
        return Err(anyhow::anyhow!("No ELF files found in build directory. Build the project first."));
    }
    
    let elf_path_str;
    // Use the first ELF file found
    if let Some(elf_file) = elf_files.first() {
        elf_path_str = elf_file.path().to_string_lossy().to_string();
        size_args.push(&elf_path_str);
    }
    
    utils::run_command(&python, &size_args, Some(&project_dir), cli.verbose).await?;
    
    Ok(())
}

pub async fn execute_components(cli: &Cli) -> Result<()> {
    utils::setup_idf_environment()?;
    
    let project_dir = utils::get_project_dir(cli.project_dir.as_deref());
    let build_dir = utils::get_build_dir(cli.build_dir.as_deref(), &project_dir);
    
    if !build_dir.exists() {
        return Err(anyhow::anyhow!("Build directory doesn't exist. Run 'build' command first."));
    }
    
    println!("Getting per-component size information...");
    
    let python = utils::get_python_executable()?;
    let idf_path = utils::get_idf_path()?;
    let size_tool_path = idf_path.join("tools/idf_size.py");
    
    let mut size_args = vec![
        size_tool_path.to_str().unwrap(),
        "--archives", // Show per-component (archive) sizes
    ];
    
    // Find the ELF file
    let elf_files: Vec<_> = std::fs::read_dir(&build_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            if let Some(extension) = entry.path().extension() {
                extension == "elf"
            } else {
                false
            }
        })
        .collect();
    
    if elf_files.is_empty() {
        return Err(anyhow::anyhow!("No ELF files found in build directory. Build the project first."));
    }
    
    let elf_path_str;
    if let Some(elf_file) = elf_files.first() {
        elf_path_str = elf_file.path().to_string_lossy().to_string();
        size_args.push(&elf_path_str);
    }
    
    utils::run_command(&python, &size_args, Some(&project_dir), cli.verbose).await?;
    
    Ok(())
}

pub async fn execute_files(cli: &Cli) -> Result<()> {
    utils::setup_idf_environment()?;
    
    let project_dir = utils::get_project_dir(cli.project_dir.as_deref());
    let build_dir = utils::get_build_dir(cli.build_dir.as_deref(), &project_dir);
    
    if !build_dir.exists() {
        return Err(anyhow::anyhow!("Build directory doesn't exist. Run 'build' command first."));
    }
    
    println!("Getting per-source-file size information...");
    
    let python = utils::get_python_executable()?;
    let idf_path = utils::get_idf_path()?;
    let size_tool_path = idf_path.join("tools/idf_size.py");
    
    let mut size_args = vec![
        size_tool_path.to_str().unwrap(),
        "--files", // Show per-file sizes
    ];
    
    // Find the ELF file
    let elf_files: Vec<_> = std::fs::read_dir(&build_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            if let Some(extension) = entry.path().extension() {
                extension == "elf"
            } else {
                false
            }
        })
        .collect();
    
    if elf_files.is_empty() {
        return Err(anyhow::anyhow!("No ELF files found in build directory. Build the project first."));
    }
    
    let elf_path_str;
    if let Some(elf_file) = elf_files.first() {
        elf_path_str = elf_file.path().to_string_lossy().to_string();
        size_args.push(&elf_path_str);
    }
    
    utils::run_command(&python, &size_args, Some(&project_dir), cli.verbose).await?;
    
    Ok(())
}