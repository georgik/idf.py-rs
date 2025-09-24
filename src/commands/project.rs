use crate::{utils, Cli};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::fs;

pub async fn create_project(_cli: &Cli, name: &str, path: Option<&Path>) -> Result<()> {
    utils::setup_idf_environment()?;
    
    let project_path = if let Some(path) = path {
        path.join(name)
    } else {
        PathBuf::from(name)
    };
    
    if project_path.exists() {
        return Err(anyhow::anyhow!(
            "Directory {} already exists",
            project_path.display()
        ));
    }
    
    println!("Creating project '{}' at: {}", name, project_path.display());
    
    // Create project directory
    fs::create_dir_all(&project_path)?;
    
    // Create basic project structure
    create_basic_project_structure(&project_path, name)?;
    
    println!("Project '{}' created successfully!", name);
    println!("To get started:");
    println!("  cd {}", project_path.display());
    println!("  idf-rs set-target esp32");
    println!("  idf-rs build");
    
    Ok(())
}

fn create_basic_project_structure(project_path: &Path, name: &str) -> Result<()> {
    // Create main directory
    let main_dir = project_path.join("main");
    fs::create_dir_all(&main_dir)?;
    
    // Create CMakeLists.txt in root
    let cmake_content = format!(r#"# For more information about build system see
# https://docs.espressif.com/projects/esp-idf/en/latest/api-guides/build-system.html
# The following five lines of boilerplate have to be in your project's
# CMakeLists.txt.

cmake_minimum_required(VERSION 3.16)

include($ENV{{IDF_PATH}}/tools/cmake/project.cmake)
project({})
"#, name);
    
    fs::write(project_path.join("CMakeLists.txt"), cmake_content)?;
    
    // Create main/CMakeLists.txt
    let main_cmake_content = r#"idf_component_register(SRCS "main.c"
                    INCLUDE_DIRS ".")
"#;
    fs::write(main_dir.join("CMakeLists.txt"), main_cmake_content)?;
    
    // Create main/main.c
    let main_c_content = r#"#include <stdio.h>
#include <inttypes.h>
#include "sdkconfig.h"
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "esp_chip_info.h"
#include "esp_flash.h"

void app_main(void)
{
    printf("Hello world!\n");

    /* Print chip information */
    esp_chip_info_t chip_info;
    uint32_t flash_size;
    esp_chip_info(&chip_info);
    printf("This is %s chip with %d CPU core(s), %s%s%s%s, ",
           CONFIG_IDF_TARGET,
           chip_info.cores,
           (chip_info.features & CHIP_FEATURE_WIFI_BGN) ? "WiFi/" : "",
           (chip_info.features & CHIP_FEATURE_BT) ? "BT" : "",
           (chip_info.features & CHIP_FEATURE_BLE) ? "BLE" : "",
           (chip_info.features & CHIP_FEATURE_IEEE802154) ? ", 802.15.4 (Zigbee/Thread)" : "");

    unsigned major_rev = chip_info.revision / 100;
    unsigned minor_rev = chip_info.revision % 100;
    printf("silicon revision v%d.%d, ", major_rev, minor_rev);
    if(esp_flash_get_size(NULL, &flash_size) != ESP_OK) {
        printf("Get flash size failed");
        return;
    }

    printf("%" PRIu32 "MB %s flash\n", flash_size / (uint32_t)(1024 * 1024),
           (chip_info.features & CHIP_FEATURE_EMB_FLASH) ? "embedded" : "external");

    printf("Minimum free heap size: %" PRIu32 " bytes\n", esp_get_minimum_free_heap_size());

    for (int i = 10; i >= 0; i--) {
        printf("Restarting in %d seconds...\n", i);
        vTaskDelay(1000 / portTICK_PERIOD_MS);
    }
    printf("Restarting now.\n");
    fflush(stdout);
    esp_restart();
}
"#;
    fs::write(main_dir.join("main.c"), main_c_content)?;
    
    // Create README.md
    let readme_content = format!(r#"# {}

This is the {} ESP-IDF project.

## Build and Flash

Build the project:
```
idf-rs build
```

Flash the project:
```
idf-rs flash
```

Monitor the output:
```
idf-rs monitor
```
"#, name, name);
    fs::write(project_path.join("README.md"), readme_content)?;
    
    // Create .gitignore
    let gitignore_content = r#"build/
managed_components/
dependencies.lock
*.tmp
"#;
    fs::write(project_path.join(".gitignore"), gitignore_content)?;
    
    Ok(())
}