use anyhow::Result;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn list_targets() {
    println!("Supported targets:");
    let targets = [
        "esp32", "esp32s2", "esp32s3", "esp32c2", "esp32c3", "esp32c6", "esp32h2", "esp32p4",
    ];

    for target in targets {
        println!("  {}", target);
    }
}

pub fn get_idf_path() -> Result<PathBuf> {
    env::var("IDF_PATH")
        .map(PathBuf::from)
        .map_err(|_| anyhow::anyhow!("IDF_PATH environment variable not set"))
}

pub fn get_project_dir(cli_project_dir: Option<&Path>) -> PathBuf {
    cli_project_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

pub fn get_build_dir(cli_build_dir: Option<&Path>, project_dir: &Path) -> PathBuf {
    cli_build_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| project_dir.join("build"))
}

pub async fn run_command(
    program: &str,
    args: &[&str],
    current_dir: Option<&Path>,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!("Running: {} {}", program, args.join(" "));
    }

    let mut cmd = Command::new(program);
    cmd.args(args);

    if let Some(dir) = current_dir {
        cmd.current_dir(dir);
    }

    let status = cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Command failed with exit code: {:?}",
            status.code()
        ))
    }
}

pub async fn run_command_with_output(
    program: &str,
    args: &[&str],
    current_dir: Option<&Path>,
) -> Result<String> {
    let mut cmd = Command::new(program);
    cmd.args(args);

    if let Some(dir) = current_dir {
        cmd.current_dir(dir);
    }

    let output = cmd.output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!("Command failed: {}", error))
    }
}

pub fn get_python_executable() -> Result<String> {
    // Try to find the ESP-IDF Python environment
    if let Ok(idf_python_env) = env::var("IDF_PYTHON_ENV_PATH") {
        let python_path = Path::new(&idf_python_env).join("bin/python");
        if python_path.exists() {
            return Ok(python_path.to_string_lossy().to_string());
        }
    }

    // Fallback to system python3
    Ok("python3".to_string())
}

pub fn setup_idf_environment() -> Result<()> {
    // Check if IDF_PATH is set
    if env::var("IDF_PATH").is_err() {
        return Err(anyhow::anyhow!(
            "IDF_PATH environment variable is not set. Please set up ESP-IDF environment first."
        ));
    }

    Ok(())
}
