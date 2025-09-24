# idf-rs

A fast Rust implementation of ESP-IDF's `idf.py` build tool.

## Why idf-rs?

The original `idf.py` is written in Python and can be slow to start up, especially for simple operations like showing help or listing targets. `idf-rs` provides a **98% faster startup time** while maintaining compatibility with the original `idf.py` interface.

### Key Advantages

🚀 **Fast Builds**: Automatically detects and uses Ninja build system (like `idf.py`) for maximum build speed
⚡ **Instant Startup**: 98x faster command startup compared to Python version
🔧 **Drop-in Replacement**: Full compatibility with `idf.py` commands and options
🎯 **Smart Flashing**: Enhanced `app-flash` with extra arguments support

### Performance Comparison

| Operation | idf.py (Python) | idf-rs (Rust) | Speedup |
|-----------|-----------------|---------------|---------|
| `--help`  | 0.590s         | 0.006s        | **98x faster** |

## Features

### 🏗️ **Smart Build System Detection**

`idf-rs` automatically detects and uses the optimal build system, just like ESP-IDF's `idf.py`:

- **Ninja** (preferred): Fast parallel builds with live progress
- **Make** (fallback): Compatible fallback when Ninja unavailable
- **Cache-aware**: Remembers generator choice for consistent builds
- **Override support**: Use `-G` to force specific generator

```bash
# Automatically uses Ninja for fastest builds
idf-rs build

# Force specific generator
idf-rs -G "Unix Makefiles" build
```

### ✅ **Implemented Commands:**
- `build` / `all` - Build the project (auto-detects Ninja/Make)
- `app` - Build only the app
- `bootloader` - Build only bootloader
- `clean` - Delete build output files
- `fullclean` - Delete entire build directory
- `flash` - Flash the project with advanced options
- `app-flash` - Flash app only (⚡ faster development)
- `bootloader-flash` - Flash bootloader only
- `monitor` - Display serial output
- `menuconfig` - Run menuconfig tool
- `set-target` - Set chip target
- `erase-flash` - Erase entire flash
- `size` - Show size information
- `size-components` - Per-component sizes
- `size-files` - Per-file sizes
- `reconfigure` - Re-run CMake
- `create-project` - Create new project
- `build-system-targets` - List build targets

### ⚡ **Enhanced Flash Commands**

All flash commands support ESP-IDF compatible options:

```bash
# Basic app-flash (fastest for development)
idf-rs app-flash

# Flash with extra esptool arguments
idf-rs flash --extra-args="--compress --verify"
idf-rs app-flash --extra-args="--compress"

# Force mode (skip security checks)
idf-rs app-flash --force

# Trace mode (debug flashing issues)
idf-rs flash --trace

# With specific port and baud rate
idf-rs -p /dev/ttyUSB0 -b 921600 app-flash
```

**Flash Command Options:**
- `--extra-args` - Pass additional arguments to esptool
- `--force` - Force write, skip security and compatibility checks
- `--trace` - Enable detailed flasher tool interactions

✅ **Global Options:**
- `--version` - Show version
- `--list-targets` - List supported targets
- `-C, --project-dir` - Project directory
- `-B, --build-dir` - Build directory
- `-v, --verbose` - Verbose output
- `--preview` - Preview features
- `--ccache / --no-ccache` - ccache control
- `-G, --generator` - CMake generator
- `--no-hints` - Disable hints
- `-D, --define-cache-entry` - CMake cache entry
- `-p, --port` - Serial port
- `-b, --baud` - Baud rate

## Installation

### Prerequisites

- Rust toolchain (install from [rustup.rs](https://rustup.rs/))
- ESP-IDF environment set up (IDF_PATH must be set)

### Option 1: Install from crates.io (Recommended)

```bash
cargo install idf-rs
```

This will install the `idf-rs` binary to your Cargo bin directory (usually `~/.cargo/bin/`), which should already be in your PATH.

### Option 2: Build from Source

```bash
git clone <this-repo>
cd idf-rs
cargo build --release
```

The binary will be available at `target/release/idf-rs`.

### Option 3: Install from Git

```bash
cargo install --git https://github.com/georgik/idf-rs
```

### Usage

You can use `idf-rs` as a drop-in replacement for `idf.py`:

```bash
# Build with automatic Ninja detection (like idf.py)
idf-rs build

# Flash and monitor (with automatic "flash monitor" detection)
idf-rs -p /dev/ttyUSB0 flash monitor

# Fast app-only flashing for development
idf-rs app-flash

# Create and configure a new project
idf-rs create-project my-project
cd my-project
idf-rs set-target esp32s3
idf-rs build  # Uses Ninja automatically

# Advanced flashing with compression
idf-rs app-flash --extra-args="--compress --verify"
```

**Note**: After installing with `cargo install idf-rs`, the `idf-rs` command will be available globally in your terminal.

### Creating an Alias

Add this to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.):

```bash
alias idf.py='idf-rs'
```

This allows you to use the faster Rust implementation transparently.

## Architecture

The project is structured as follows:

- `src/main.rs` - CLI argument parsing and command dispatch
- `src/utils.rs` - Common utilities for running commands and environment setup
- `src/config.rs` - ESP-IDF configuration file handling (sdkconfig)
- `src/build_systems.rs` - **NEW**: Build system detection (Ninja/Make auto-selection)
- `src/commands/` - Individual command implementations
  - `build.rs` - Build-related commands with auto-generator detection
  - `flash.rs` - Enhanced flash commands with `--extra-args`, `--force`, `--trace`
  - `monitor.rs` - Serial monitor
  - `config.rs` - Configuration commands (menuconfig, set-target)
  - `size.rs` - Size analysis commands
  - `project.rs` - Project creation

## Implementation Details

- **Build System Detection**: Automatically detects Ninja vs Make (identical to `idf.py` logic)
- **Command Execution**: Uses the existing ESP-IDF Python tools (esptool.py, idf_monitor.py, etc.) but with much faster startup
- **Configuration**: Parses and manipulates `sdkconfig` files directly
- **Environment**: Respects all ESP-IDF environment variables (`ESPPORT`, `ESPBAUD`, etc.)
- **Flash Enhancement**: Supports all `idf.py` flash options (`--extra-args`, `--force`, `--trace`)
- **Cache Management**: Reads CMakeCache.txt to maintain generator consistency
- **Compatibility**: Designed to be a drop-in replacement for `idf.py`

## Supported Targets

- esp32
- esp32s2
- esp32s3  
- esp32c2
- esp32c3
- esp32c6
- esp32h2
- esp32p4

## Contributing

Contributions are welcome! Please ensure:

1. All existing functionality continues to work
2. New features maintain compatibility with `idf.py`
3. Performance improvements are preserved
4. Code follows Rust best practices

## License

This project is open source. Please refer to the LICENSE file for details.

## Future Enhancements

Potential areas for improvement:
- Native ESP-IDF CMake integration (avoid shelling out to cmake)
- Built-in esptool functionality (avoid Python dependency for flashing)
- Parallel build support
- Enhanced error reporting and hints
- Configuration validation