# ⚡ LIMTRAC

![GitHub](https://raster.shields.io/github/license/overtest/limtrac?style=for-the-badge) ![GitHub last commit](https://raster.shields.io/github/last-commit/overtest/limtrac?style=for-the-badge) ![GitHub all releases](https://raster.shields.io/github/downloads/overtest/limtrac/total?style=for-the-badge) ![GitHub Repo stars](https://raster.shields.io/github/stars/overtest/limtrac?style=for-the-badge) ![GitHub issues](https://raster.shields.io/github/issues/overtest/limtrac?style=for-the-badge)

**LimTrac** is a simple library written in `Rust`, designed for usage on `GNU/Linux` platform that executes potentially unsafe programs with enforcement of some security policies (using such Linux built-in capabilities as `seccomp`, `prlimit`, `cgroups`, etc). You can use it from your `C/C++` and `C#` apps (bindings available), and also from `Rust` (but using types, defined in `libc.rs` and `nix.rs` crates). Of course, you can create your own binding to use `limtrac` on other platforms.

### ✨ Features

`Limtrac` is a part of [Overtest](https://github.com/overtest) free software project, and is being used by Overtest Verification Agent for untrusted programs execution, so, for now, it contains only features, used by some parts of Overtest on `GNU/Linux` platform:

- Execute any program in a child process as another user
- Specify CLI arguments and a working dir for the program
- Redirect I/O streams to files, duplicate `stderr` to `stdout`
- Set up resource limits (using `setrlimit` capabilities)
- Automatically kill a child process on a specified timeout
- Block potentially malicious system calls (using `seccomp`)
- Isolate a child process from some local resources using `unshare`
- Get resources usage and execution results for the process

All pull requests, questions and ideas are welcomed 😃!

### ⚙ Usage in your product

As it said, you can use `limtrac` either in Rust, or using a binding for one of the supported languages and platforms, listed below. Also, you can manually create a binding for it on platforms that have support for interop with native libraries.

- **Rust applications:** using `nix` and `libc` crates
- **.NET applications:** `.dll targeting dotnet-6`
- **C/C++ applications:** `.h header file`

Don't forget that you need `seccomp` feature and package available and enabled in your development and target environments.

### 🏗 Building library and bindings

You can build `limtrac` only inside a `GNU/Linux` environment, or under Windows Subsystem for Linux (version 2 recommended). To build a project, you can use standard Cargo build commands. Header file with `C / C++` library bindings will be generated automatically (using `cbindgen` crate). Note that you need `seccomp` and `libseccomp-dev` packages installed on your system to build the library.

#### Library and `C/C++` header file:

```bash
cargo build           # for development builds
cargo build --release # for release builds
```

**Tip:** Use [JetBrains CLion](https://jetbrains.com/clion/) with official Rust plugin & `WSL 2` to build `limtrac` 😃!

### 🎁 Building sample applications

To build a demo app written in `C`, you need to have `GCC`, `make` and `cmake` in your system.

```bash
cd ./bindings/demoapp_c/
mkdir build && cd build
cmake ../ && make
```

Demo application written in `C#` is a part of `.NET` binding, so it can be built as a part of `LimtracDotNet` solution:

```bash
cargo build --release # requred to build .NET binding
cd ./bindings/LimtracDotNet/
dotnet build   # for development builds
dotnet publish # for release builds
```

### 📃 Licensing information

```
LIMTRAC, a part of Overtest free software project.
Copyright (C) 2021-2023, Yurii Kadirov <contact@sirkadirov.com>

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Lesser General Public License as
published by the Free Software Foundation, either version 3 of the
License, or (at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU Lesser General Public License for more details.

You should have received a copy of the GNU Lesser General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.
```
