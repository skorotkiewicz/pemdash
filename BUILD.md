# Building pemdash

## Build from source

Use a current stable Rust toolchain with support for edition 2024. On Linux/macOS, install Rust via [rustup](https://rustup.rs/). The commands below run from the repository root.

The app is written in Rust, but the `gtk4` crate links to the native GTK library. Building from source therefore also needs GTK development packages, `pkg-config` to locate them, and a C toolchain (GCC or Clang) for linking. These build tools are not required to run a prebuilt binary; the GTK runtime still is.

### Linux

Install the GTK 4 development packages and build tools:

```bash
# Arch Linux: base-devel includes GCC, the linker, and pkgconf.
sudo pacman -S --needed base-devel gtk4

# Debian / Ubuntu
sudo apt-get update
sudo apt-get install build-essential libgtk-4-dev pkg-config
```

Install the clipboard helper for your desktop session (one is enough):

```bash
sudo pacman -S wl-clipboard # Arch Linux, Wayland (including niri)
sudo pacman -S xclip        # Arch Linux, X11

# Debian / Ubuntu: install wl-clipboard for Wayland or xclip for X11.
```

Then build and run:

```bash
cargo build --release --locked
./target/release/pemdash
```

### macOS (Apple Silicon or Intel)

Install Xcode Command Line Tools (`xcode-select --install`) and native Homebrew for your architecture, then:

```bash
brew install gtk4 pkgconf
cargo build --release --locked
./target/release/pemdash
```

Use a native terminal/toolchain, not Rosetta for the ARM64 build. GTK must remain installed when running the resulting binary; `pbcopy` is already part of macOS.

### Windows x86_64 (MSYS2 UCRT64)

After updating MSYS2, run these commands in its **UCRT64** shell:

```bash
pacman -S --needed mingw-w64-ucrt-x86_64-rust \
  mingw-w64-ucrt-x86_64-gtk4 \
  mingw-w64-ucrt-x86_64-pkgconf \
  mingw-w64-ucrt-x86_64-gcc

# Must report host: x86_64-pc-windows-gnu, using /ucrt64/bin/rustc.
which rustc
rustc -vV
cargo build --release --locked --target x86_64-pc-windows-gnu
./target/x86_64-pc-windows-gnu/release/pemdash.exe
```

Use the MSYS2-provided Rust/Cargo and UCRT64 GTK toolchain together, rather than mixing in an MSVC Rust toolchain or MINGW64 libraries. The same MSYS2 runtime requirements apply to locally built binaries and release ZIPs.

### Tests and CI

```bash
cargo test --locked --all-features
```

Ordinary unit and documentation tests do not require a graphical display or access to the real clipboard; subprocess tests use fixture commands (including built-in `powershell.exe` on Windows). The explicitly ignored GUI/clipboard integration test is not run by default and requires a suitable desktop session and helpers.

The existing release workflow also runs on pull requests and pushes to `main`/`dev`. It tests and builds all five targets natively, retains the Linux format/Clippy quality job, and only publishes releases/AUR updates for tag builds (`v*` tags trigger automatically). For Windows, it uses MSYS2 Rust and GTK in UCRT64; for macOS, it uses Homebrew GTK. CI builds do not validate interactive UI behavior, clipboard integration, or standalone deployment.

## Use

Launch the installed/extracted binary or, after installing build dependencies:

```bash
cargo run --release --locked
```

## Release archives and runtime requirements

The release workflow builds these native targets; each archive contains the binary, README, and license:

| Platform | Archive name |
| --- | --- |
| Linux x86_64 | `pemdash-v<version>-x86_64-unknown-linux-gnu.tar.gz` |
| Linux ARM64 | `pemdash-v<version>-aarch64-unknown-linux-gnu.tar.gz` |
| macOS Apple Silicon | `pemdash-v<version>-aarch64-apple-darwin.tar.gz` |
| macOS Intel | `pemdash-v<version>-x86_64-apple-darwin.tar.gz` |
| Windows x86_64 (GNU / UCRT64) | `pemdash-v<version>-x86_64-pc-windows-gnu.zip` |

These are **not self-contained application bundles**. GTK and its runtime dependencies must be installed separately, as described below. Extract the archive for your architecture and run `./pemdash` on Linux/macOS, or `./pemdash.exe` from an MSYS2 UCRT64 shell on Windows. A graphical desktop session is required to use the app.

- **Linux:** requires system GTK 4 and glibc compatible with the Ubuntu build runner (`ubuntu-latest` for x86_64, `ubuntu-24.04-arm` for ARM64). These are not static binaries or a promise of compatibility with older distributions. On Debian/Ubuntu, install `libgtk-4-1`; on Arch, install `gtk4`. Clipboard copying additionally requires `wl-clipboard` on Wayland or `xclip` on X11, available on `PATH`.
- **macOS:** builds run natively on macOS 15 for both architectures and link to Homebrew GTK 4. Install `brew install gtk4`; the matching Homebrew installation must remain available at the build prefix (`/opt/homebrew` for Apple Silicon, `/usr/local` for Intel). The archive does not bundle dylibs, relocate Homebrew dependencies, or provide a signed/notarized `.app`. Compatibility with older macOS versions is not established. Clipboard copying uses the built-in `/usr/bin/pbcopy`.
- **Windows:** the ZIP contains an **unbundled executable, not a standalone Windows app**. Install [MSYS2](https://www.msys2.org/), update it with `pacman -Syu` (restart the shell and repeat if prompted), and open **MSYS2 UCRT64**, not MSYS or MINGW64. Run `pacman -S mingw-w64-ucrt-x86_64-gtk4`, then launch the extracted `pemdash.exe` from that shell. Keep the MSYS2 installation: `/ucrt64/bin` supplies GTK and its dependency DLLs through `PATH`, and the matching `/ucrt64/share` installation supplies GTK assets, schemas, and icon themes. Do not copy just the EXE to another machine and expect it to work. Clipboard copying uses built-in Windows PowerShell (`powershell.exe` with `Set-Clipboard`), which must also be reachable on `PATH`; no additional clipboard package is needed.
