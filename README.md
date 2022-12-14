# `cargo-nw`

Application deployment package builder for [Node-Webkit](https://nwjs.io)

[![Crates.io](https://img.shields.io/crates/l/manual-serializer.svg?maxAge=2592000)](https://crates.io/crates/manual-serializer)
[![Crates.io](https://img.shields.io/crates/v/manual-serializer.svg?maxAge=2592000)](https://crates.io/crates/manual-serializer)

### Overview

`cargo-nw` allows creation custom application deployment packages for NW applications by bundling application files with NW binaries.
This tool was created to build Node Webkit Rust WASM applications, but it can be used with any new or existing project.

### OS support
* MacOS: DMG, archive
* Windows: InnoSetup, archive
* Linux: archive, (SNAP is under development)

### Features
* No external dependencies for basic functionality
* Automatic download of Node Webkit distribution binaries
* Automatic handling of application icons
* Automatic handling of MacOS and Windows resource manifests
* Automatic handling of DMG resources
* Custom strings for Windows resource manifests
* Creation of firewall rules during Windows installs
* Optional inclusion of FFMPEG libraries

### Dependencies
* [Rust](https://www.rust-lang.org/tools/install)
* [InnoSetup for Windows](https://jrsoftware.org/isdl.php) for creation of interactive Windows installers
* [Wasmpack](https://rustwasm.github.io/wasm-pack/installer/) if building Rust WASM applications

### Installation
```bash
cargo install cargo-nw
```

### Usage

* `cargo nw init` creates a new Rust WASM project;
    * if the folder is empty, `cargo-nw` will create a new Rust WASM (wasm-pack) project
    * if an existing `package.json` file already exists in the current folder, only `nw.toml` manifest will be created.
* `cargo nw init --manifest` creates only the `nw.toml` manifest file
* `cargo nw build` executes the default deployment build (DMG on MacOS, InnoSetup on Windows)
* `cargo nw clean` removes temporary files
* `cargo nw clean --deps` removes downloaded Node Webkit binaries

### Issues
Linux
```bash
sudo apt install libssl-dev
```

### Manifest

The `nw.toml` package manifest file contains TOML specification for the project. It is typically located in the project root.

```toml
[application]
name = "my-app"
title = "My App"
version = "0.1.0"
organization = "My Organization"

[package]
exclude = ["resources/setup"]
execute = [
    { build = { cmd = "my-package-script $OUTPUT" } },
]

[nwjs]
version = "0.70.1"
ffmpeg = false

[windows]
uuid = "95ba1908-ff97-4281-8dca-7461bc1ee058"
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

