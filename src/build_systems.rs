use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

/// Definition of a build system generator
#[derive(Debug, Clone)]
pub struct Generator {
    pub command: Vec<String>,
    pub version: Vec<String>,
    pub dry_run: Vec<String>,
    pub verbose_flag: String,
    pub force_progression: bool,
}

/// Get the ordered list of available generators, similar to ESP-IDF's GENERATORS
pub fn get_generators() -> BTreeMap<String, Generator> {
    let mut generators = BTreeMap::new();

    // Ninja comes first (preferred)
    generators.insert(
        "Ninja".to_string(),
        Generator {
            command: vec!["ninja".to_string()],
            version: vec!["ninja".to_string(), "--version".to_string()],
            dry_run: vec!["ninja".to_string(), "-n".to_string()],
            verbose_flag: "-v".to_string(),
            force_progression: true,
        },
    );

    // Unix Makefiles as fallback (similar to ESP-IDF's logic)
    #[cfg(not(target_os = "windows"))]
    {
        let make_cmd = if cfg!(target_os = "freebsd") {
            "gmake"
        } else {
            "make"
        };

        let cpu_count = num_cpus::get();
        generators.insert(
            "Unix Makefiles".to_string(),
            Generator {
                command: vec![
                    make_cmd.to_string(),
                    "-j".to_string(),
                    (cpu_count + 2).to_string(),
                ],
                version: vec![make_cmd.to_string(), "--version".to_string()],
                dry_run: vec![make_cmd.to_string(), "-n".to_string()],
                verbose_flag: "VERBOSE=1".to_string(),
                force_progression: false,
            },
        );
    }

    generators
}

/// Check if an executable exists by running its version command
pub fn executable_exists(args: &[String]) -> bool {
    if args.is_empty() {
        return false;
    }

    let mut cmd = Command::new(&args[0]);
    if args.len() > 1 {
        cmd.args(&args[1..]);
    }

    match cmd.output() {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// Detect the default cmake generator, if none was specified
/// Returns the first available generator, preferring Ninja over Make
pub fn detect_cmake_generator() -> Result<String> {
    let generators = get_generators();

    for (generator_name, generator) in generators.iter() {
        if executable_exists(&generator.version) {
            return Ok(generator_name.clone());
        }
    }

    anyhow::bail!(
        "To use idf-rs, either the 'ninja' or 'make' build tool must be available in the PATH"
    );
}

/// Parse CMakeCache.txt to extract the generator used
pub fn get_generator_from_cache(build_dir: &Path) -> Option<String> {
    let cache_path = build_dir.join("CMakeCache.txt");
    if !cache_path.exists() {
        return None;
    }

    match std::fs::read_to_string(&cache_path) {
        Ok(content) => {
            for line in content.lines() {
                if line.starts_with("CMAKE_GENERATOR:INTERNAL=") {
                    if let Some(generator) = line.split('=').nth(1) {
                        return Some(generator.to_string());
                    }
                }
            }
            None
        }
        Err(_) => None,
    }
}

/// Get the appropriate generator for the build
/// This follows ESP-IDF's logic:
/// 1. Use explicit generator if provided
/// 2. Use cached generator if build directory exists
/// 3. Auto-detect available generator
pub fn get_build_generator(
    explicit_generator: Option<&String>,
    build_dir: &Path,
) -> Result<String> {
    // If user explicitly specified a generator, use it
    if let Some(generator) = explicit_generator {
        return Ok(generator.clone());
    }

    // If build directory exists, try to get generator from cache
    if let Some(cached_generator) = get_generator_from_cache(build_dir) {
        return Ok(cached_generator);
    }

    // Otherwise, auto-detect
    detect_cmake_generator()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generators_order() {
        let generators = get_generators();
        let keys: Vec<_> = generators.keys().collect();

        // Ninja should be first (preferred)
        assert_eq!(keys[0], "Ninja");

        // Make sure we have at least 2 generators on non-Windows
        #[cfg(not(target_os = "windows"))]
        {
            assert!(keys.len() >= 2);
            assert!(keys.contains(&"Unix Makefiles"));
        }
    }

    #[test]
    fn test_executable_exists() {
        // This should exist on most systems
        assert!(executable_exists(&[
            "echo".to_string(),
            "--help".to_string()
        ]));

        // This definitely shouldn't exist
        assert!(!executable_exists(&[
            "nonexistent_command_12345".to_string()
        ]));
    }
}
