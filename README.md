# idf-rs

A fast Rust implementation of ESP-IDF's `idf.py` build tool.

## Why idf-rs?

The original `idf.py` is written in Python and can be slow to start up, especially for simple operations like showing help or listing targets. `idf-rs` provides a **98% faster startup time** while maintaining compatibility with the original `idf.py` interface.

### Performance Comparison

| Operation | idf.py (Python) | idf-rs (Rust) | Speedup |
|-----------|-----------------|---------------|---------|
| `--help`  | 0.590s         | 0.006s        | **98x faster** |

## Features

✅ **Implemented Commands:**
- `build` / `all` - Build the project
- `app` - Build only the app
- `bootloader` - Build only bootloader
- `clean` - Delete build output files
- `fullclean` - Delete entire build directory
- `flash` - Flash the project
- `app-flash` - Flash app only
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
# Instead of: idf.py build
idf-rs build

# Instead of: idf.py -p /dev/ttyUSB0 flash monitor
idf-rs -p /dev/ttyUSB0 flash monitor

# Create a new project
idf-rs create-project my-project
cd my-project
idf-rs set-target esp32s3
idf-rs build
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
- `src/commands/` - Individual command implementations
  - `build.rs` - Build-related commands
  - `flash.rs` - Flash-related commands
  - `monitor.rs` - Serial monitor
  - `config.rs` - Configuration commands (menuconfig, set-target)
  - `size.rs` - Size analysis commands
  - `project.rs` - Project creation

## Implementation Details

- **Command Execution**: Uses the existing ESP-IDF Python tools (esptool.py, idf_monitor.py, etc.) but with much faster startup
- **Configuration**: Parses and manipulates `sdkconfig` files directly
- **Environment**: Respects all ESP-IDF environment variables
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