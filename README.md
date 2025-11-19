# wcurl

A simple wrapper around curl to easily download files - Rust implementation.

## About

This is a Rust reimplementation of the original [wcurl](https://github.com/curl/wcurl) shell script. It provides a more user-friendly interface for downloading files with curl, handling common scenarios automatically.

## Features

- 🚀 **Simple downloads**: Just pass a URL and wcurl handles the rest
- 📦 **Multiple files**: Download several files in parallel (curl >= 7.66.0)
- 🔄 **Smart defaults**: Automatically uses `--location`, `--remote-time`, `--fail`, etc.
- 📝 **Filename extraction**: Automatically extracts and decodes filenames from URLs
- 🛡️ **Safe overwrites**: Uses `--no-clobber` when available (curl >= 7.83.0)
- ⚡ **Cross-platform**: Works on Windows, Linux, FreeBSD, macOS

## Installation

### Pre-built Binaries

Download the latest binary for your platform from the [releases page](../../releases):

**Windows:**
- x86_64 (Intel/AMD): `wcurl-VERSION-windows-amd64.exe`
- ARM64: `wcurl-VERSION-windows-arm64.exe`

**Linux:**
- x86_64 (Intel/AMD): `wcurl-VERSION-linux-amd64`
- ARM64 (aarch64): `wcurl-VERSION-linux-arm64`

**FreeBSD:**
- x86_64 (Intel/AMD): `wcurl-VERSION-freebsd-amd64`
- ARM64 (aarch64): `wcurl-VERSION-freebsd-arm64`

**OpenBSD:**
- x86_64 (Intel/AMD): `wcurl-VERSION-openbsd-amd64`
- ARM64 (aarch64): `wcurl-VERSION-openbsd-arm64`

**macOS:**
- x86_64 (Intel): `wcurl-VERSION-macos-amd64`
- ARM64 (Apple Silicon): `wcurl-VERSION-macos-arm64`

### Building from Source

#### Prerequisites

- Rust 1.70 or later
- curl (must be in PATH)

#### Build

```bash
# Clone the repository
git clone https://github.com/Ruslan-Isaev/wcurl-rs.git
cd wcurl-rs

# Build with Make
make

# Or build manually with cargo
cargo build --release

# The binary will be in target/release/wcurl (or wcurl.exe on Windows)
```

#### Cross-compilation

To build for all supported platforms:

```bash
# Install targets
make install-targets

# Build all
make all

# Binaries will be in dist/
```

## Usage

### Basic Examples

```bash
# Download a single file
wcurl https://example.com/file.zip

# Download multiple files in parallel
wcurl https://example.com/file1.zip https://example.com/file2.zip

# Specify output filename
wcurl -o myfile.zip https://example.com/file.zip

# Pass options to curl
wcurl --curl-options "--user-agent Mozilla/5.0" https://example.com/file.zip

# Multiple curl options
wcurl --curl-options "-H 'Accept: application/json'" --curl-options "--compressed" https://api.example.com/data

# Dry run (see what would be executed)
wcurl --dry-run https://example.com/file.zip

# URLs with spaces (automatically encoded)
wcurl "https://example.com/my file.zip"
```

### Command-line Options

```
Usage: wcurl <URL>...
       wcurl [--curl-options <CURL_OPTIONS>]... [--no-decode-filename] [-o|-O|--output <PATH>] [--dry-run] [--] <URL>...

Options:
  --curl-options <CURL_OPTIONS>  Specify extra options to be passed to curl
                                 May be specified multiple times
  
  -o, -O, --output <PATH>        Use the provided output path instead of 
                                 getting it from the URL
  
  --no-decode-filename           Don't percent-decode the output filename
  
  --dry-run                      Don't execute curl, just print the command
  
  -V, --version                  Print version information
  
  -h, --help                     Print help message
  
  <URL>                          URL to download (may be specified multiple times)
```

### What wcurl Does Automatically

For each URL, wcurl passes these options to curl:

- `--fail` - Fail silently on server errors
- `--globoff` - Disable URL globbing
- `--location` - Follow redirects
- `--proto-default https` - Use HTTPS by default
- `--remote-time` - Set file time to server's time
- `--retry 5` - Retry failed transfers up to 5 times
- `--no-clobber` - Don't overwrite existing files (curl >= 7.83.0)
- `--parallel` - Download multiple URLs in parallel (curl >= 7.66.0)

## Examples

### Download a file with custom headers

```bash
wcurl --curl-options "-H 'Authorization: Bearer token123'" https://api.example.com/file.zip
```

### Download and save with a different name

```bash
wcurl -o release.tar.gz https://github.com/user/repo/archive/refs/tags/v1.0.0.tar.gz
```

### Download multiple files to the same directory

```bash
wcurl \
  https://example.com/file1.zip \
  https://example.com/file2.zip \
  https://example.com/file3.zip
```

### Download with authentication

```bash
wcurl --curl-options "--user username:password" https://secure.example.com/file.zip
```

## Requirements

- curl >= 7.46.0 (released in 2015)
- For parallel downloads: curl >= 7.66.0
- For `--no-clobber` support: curl >= 7.83.0

## Differences from Original wcurl

This Rust implementation maintains full compatibility with the original shell script while offering:

- Better performance on Windows
- Easier cross-platform compilation
- Single binary distribution (no shell required)
- More robust error handling

## Building for Development

```bash
# Run tests
cargo test

# Run with debug info
cargo run -- https://example.com/test.html

# Build for your current platform
cargo build --release

# Build for specific platform
cargo build --release --target x86_64-unknown-linux-gnu
```

## Makefile Targets

```bash
make              # Build for current platform
make all          # Build for all supported platforms
make clean        # Clean build artifacts
make install      # Install to /usr/local/bin (or $PREFIX)
make uninstall    # Remove from installation directory
make test         # Run tests
make check        # Check code without building
```

## License

This project is licensed under the GNU General Public License v3.0 - see the LICENSE file for details.

The original wcurl shell script is licensed under the curl license.

## Credits

- Original wcurl by Samuel Henrique and Sergio Durigan Junior
- Rust reimplementation: [Your Name]

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Links

- Original wcurl: https://github.com/curl/wcurl
- curl project: https://curl.se/
- Report issues: https://github.com/Ruslan-Isaev/wcurl-rs/issues
