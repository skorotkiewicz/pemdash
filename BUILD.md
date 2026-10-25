# Building pemdash

## Build from source

Use a current stable Rust toolchain with support for edition 2024. On Linux/macOS, install Rust via [rustup](https://rustup.rs/). The commands below run from the repository root.

The app is written in Rust, but the `gtk4` crate links to the native GTK library. Building from source therefore also needs GTK development files, `pkg-config` to locate them, and a native C/C++ toolchain for linking (GCC/Clang on Linux/macOS, MSVC on Windows). These build tools are not required to run a prebuilt binary; the GTK runtime still is, either installed separately or bundled with the Windows ZIP.

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

### Windows x86_64 (native MSVC)

Install stable Rust through [rustup](https://rustup.rs/) and Visual Studio Build Tools with the **Desktop development with C++** workload. Use a **Developer PowerShell for Visual Studio** terminal with x64 tools. No MSYS2 installation or shell is required.

Download the native MSVC GTK 4 SDK from [Gvsbuild releases](https://github.com/wingtk/gvsbuild/releases). The release workflow pins version **2026.8.0**, asset `GTK4_Gvsbuild_2026.8.0_x64.zip`, and verifies its SHA-256:

```text
1f95a92d037f5292da05e6ab1037032ff21ddb7b20d4ac8e83e3674c864c07b0
```

Verify the downloaded ZIP with `Get-FileHash -Algorithm SHA256`, then extract it so that `C:\gtk` contains the SDK's `bin`, `lib`, and `share` directories. Configure the SDK in your current PowerShell session:

```powershell
$gtk = 'C:\gtk'
$env:PATH = "$gtk\bin;$env:PATH"
$env:PKG_CONFIG = "$gtk\bin\pkg-config.exe"
$env:PKG_CONFIG_PATH = "$gtk\lib\pkgconfig"

# Replace the upstream builder's prefix with this SDK's extraction directory.
$pcPrefix = $gtk.Replace('\', '/')
Get-ChildItem "$gtk\lib\pkgconfig\*.pc" | ForEach-Object {
    $content = Get-Content -LiteralPath $_.FullName -Raw
    $content = $content -replace '(?m)^prefix=.*$', "prefix=$pcPrefix"
    Set-Content -LiteralPath $_.FullName -Value $content -Encoding utf8NoBOM
}

rustup toolchain install stable-x86_64-pc-windows-msvc
cargo +stable-x86_64-pc-windows-msvc build --release --locked --target x86_64-pc-windows-msvc
.\target\x86_64-pc-windows-msvc\release\pemdash.exe
```

Use PowerShell 7 for the `utf8NoBOM` encoding option above. GTK libraries must be built for MSVC; do not mix MinGW GTK libraries with this Rust target. Locally built binaries require the SDK runtime on `PATH`. Release ZIPs additionally bundle GTK and the redistributable MSVC runtime DLLs, so users do not need the SDK installed.

### CI

The release workflow runs only when a `v*` tag is pushed or when started manually with `workflow_dispatch`. Ordinary branch pushes and pull requests do not trigger it.

It builds all five targets natively, retains the Linux format/Clippy quality job, and only publishes releases/AUR updates for tag builds. Windows uses native MSVC Rust and the pinned Gvsbuild GTK SDK; macOS uses Homebrew GTK. Automated unit and GUI tests are not run by this workflow. Windows packaging checks DLL imports and required runtime assets, but does not launch the app or verify interactive behavior.

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
| Windows x86_64 (MSVC) | `pemdash-v<version>-x86_64-pc-windows-msvc.zip` |

Linux and macOS archives require a separately installed GTK runtime. The Windows ZIP bundles GTK and MSVC runtime DLLs. Extract the archive for your architecture and run `./pemdash` on Linux/macOS, or double-click `pemdash.exe` on Windows. A graphical desktop session is required to use the app.

- **Linux:** requires system GTK 4 and glibc compatible with the Ubuntu build runner (`ubuntu-latest` for x86_64, `ubuntu-24.04-arm` for ARM64). These are not static binaries or a promise of compatibility with older distributions. On Debian/Ubuntu, install `libgtk-4-1`; on Arch, install `gtk4`. Clipboard copying additionally requires `wl-clipboard` on Wayland or `xclip` on X11, available on `PATH`.
- **macOS:** builds run natively on macOS 15 for both architectures and link to Homebrew GTK 4. Install `brew install gtk4`; the matching Homebrew installation must remain available at the build prefix (`/opt/homebrew` for Apple Silicon, `/usr/local` for Intel). The archive does not bundle dylibs, relocate Homebrew dependencies, or provide a signed/notarized `.app`. Compatibility with older macOS versions is not established. Clipboard copying uses the built-in `/usr/bin/pbcopy`.
- **Windows:** extract the **entire ZIP** into a folder, then double-click `pemdash.exe`. No MSYS2 installation or GTK setup is required. Keep the DLLs and bundled `share`, `lib`, and `etc` directories beside the EXE; moving only the EXE will break it. `GTK-SDK.txt` records the SDK source and checksum, and third-party license notices are under `share/doc`. Clipboard copying uses built-in Windows PowerShell (`powershell.exe` with `Set-Clipboard`), which must be reachable on `PATH`; no extra clipboard package is needed. This is an unsigned executable, not an installer.
