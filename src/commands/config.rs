use crate::{utils, config, Cli};
use anyhow::Result;

pub async fn execute_menuconfig(cli: &Cli) -> Result<()> {
    utils::setup_idf_environment()?;
    
    let project_dir = utils::get_project_dir(cli.project_dir.as_deref());
    let build_dir = utils::get_build_dir(cli.build_dir.as_deref(), &project_dir);
    
    println!("Starting menuconfig...");
    
    // Ensure build directory exists and is configured
    if !build_dir.exists() {
        println!("Build directory doesn't exist. Configuring project first...");
        crate::commands::build::execute_reconfigure(cli).await?;
    }
    
    // Run menuconfig using cmake
    let menuconfig_args = vec![
        "--build", build_dir.to_str().unwrap(),
        "--target", "menuconfig"
    ];
    
    utils::run_command("cmake", &menuconfig_args, Some(&project_dir), cli.verbose).await?;
    
    println!("Menuconfig completed!");
    Ok(())
}

pub async fn execute_set_target(cli: &Cli, target: &str) -> Result<()> {
    let project_dir = utils::get_project_dir(cli.project_dir.as_deref());
    
    println!("Setting target to: {}", target);
    
    // Validate target
    let supported_targets = [
        "esp32", "esp32s2", "esp32s3", "esp32c2", "esp32c3", 
        "esp32c6", "esp32h2", "esp32p4"
    ];
    
    if !supported_targets.contains(&target) {
        return Err(anyhow::anyhow!(
            "Unsupported target: {}. Supported targets: {:?}", 
            target, supported_targets
        ));
    }
    
    // Load existing config
    let mut sdk_config = config::load_project_config(&project_dir)?;
    
    // Set target
    sdk_config.set_target(target);
    
    // Save config
    config::save_project_config(&project_dir, &sdk_config)?;
    
    println!("Target set to {} successfully!", target);
    println!("You may need to run 'reconfigure' or 'fullclean' if you are changing from a different target.");
    
    Ok(())
}