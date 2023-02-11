# `cargo-nw`

Application deployment package builder for [Node-Webkit](https://nwjs.io)

[<img alt="github" src="https://img.shields.io/badge/github-aspectron/cargo--nw-8da0cb?style=for-the-badge&labelColor=555555&color=8da0cb&logo=github" height="20">](https://github.com/aspectron/cargo-nw)
[<img alt="crates.io" src="https://img.shields.io/crates/v/cargo-nw.svg?maxAge=2592000&style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/cargo-nw)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-cargo--nw-56c2a5?maxAge=2592000&style=for-the-badge&logo=rust" height="20">](https://docs.rs/cargo-nw)
<img alt="license" src="https://img.shields.io/crates/l/workflow-nw.svg?maxAge=2592000&color=6ac&style=for-the-badge&logoColor=fff" height="20">


## Overview

`cargo-nw` allows creation custom application deployment packages for NW applications by bundling application files with NW binaries.
This tool was created to build Node Webkit Rust WASM applications, but it can be used with any new or existing project.

The deployment is controlled by `nw.toml` manifest file that allows full customization of information packages in distributables, creation of additional actions (such as program executions during different stages of the build process), copying groups of files using globs or regex filters.

For application icons, only a single large-size `.png` file needs to be supplied - `cargo-nw` will automatically resize and generate all appropriate icon variations and file formats. Custom icons can be supplied for each OS.

## OS support
* MacOS: DMG, archive
* Windows: InnoSetup, archive
* Linux: SNAP, archive

NOTE: To create a redistributable for each platform, you need to run `cargo-nw` on that specific platform.

## Features
* No external dependencies for basic functionality
* Automatic download of Node Webkit distribution binaries (optionally SDK)
* Automatic handling of application icons
* Automatic creation of `.desktop` file on Linux
* Automatic handling of DMG resources
* Automatic handling of MacOS and Windows resource manifests
* Custom strings for Windows resource manifests
* SNAP support for different types of confinement
* Creation of firewall rules during Windows installs
* Optional inclusion of FFMPEG libraries

## Dependencies
* [Rust](https://www.rust-lang.org/tools/install)
* [InnoSetup for Windows](https://jrsoftware.org/isdl.php) for creation of interactive Windows installers
* [Wasmpack](https://rustwasm.github.io/wasm-pack/installer/) if building Node Webkit WASM applications in Rust
* [SnapCraft](https://snapcraft.io/install/snapcraft/ubuntu) + [LXD](https://linuxcontainers.org/lxd/getting-started-cli/) 

## Installation
```bash
cargo install cargo-nw
```

## Documentation

For detailed documentation please see [Cargo NW Documentation](https://cargo-nw.aspectron.org)

## Manifest

The `nw.toml` package manifest file contains TOML specification for the project. It is typically located in the project root.

```toml
[application]
name = "my-app"
title = "My App"
version = "0.1.0"
organization = "My Organization"

[description]
short = "This is my app" # max 78 chars
long = """
Lorem ipsum dolor sit amet, consectetur adipiscing elit, 
sed do eiusmod tempor incididunt ut labore et dolore magna 
aliqua. Ut enim ad minim veniam, quis nostrud exercitation 
ullamco laboris nisi ut aliquip ex ea commodo consequat. 
Duis aute irure dolor in reprehenderit in voluptate velit 
esse cillum dolore eu fugiat nulla pariatur.
"""

[package]
exclude = ["resources/setup"]
execute = [
    { build = { cmd = "my-package-script $OUTPUT" } },
]

[nwjs]
version = "0.70.1"
ffmpeg = false

[windows]
uuid = "95ba1908-ff47-4281-4dca-7461bc1ee058"
group = "App Group" # Windows start menu group
run_on_startup = "everyone"
run_after_setup = true
resources = [
    { Custom = { name = "CustomString", value = "My Info" }},
]

[languages]
languages = ["english","french"]

[firewall]
application = "in:out"
in = [ "bin\\windows-x64\\my-utility.exe" ]
out = [ "bin\\windows-x64\\test.exe" ]
```

