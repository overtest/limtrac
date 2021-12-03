# LimTrac

**LimTrac** is a simple program written in Rust, designed for GNU/Linux-based operation systems, which enforces process security policies (using such GNU/Linux capabilities as `seccomp`, `prlimit`, `cgroups`, etc) to the current running process and executes a requested program with specified arguments as the specified user.

### Features

`Limtrac` is a part of [Overtest](https://github.com/overtest) free software project, and is being used by Overtest Verification Agent (also known as `overtest-agent`) for untrusted programs execution, so it contains only features, needed by `overtest-agent` in GNU/Linux environment. For now, the quickest way to determine available features list is to browse the source code of the program.

**WARNING:** As it said before, `limtrac` is only a third-party dependence for Overtest, so its features depend on the main project needs. Feel free to use `limtrac` source code base to create your own apps - it is the safest way to use its features.

### Building

You can only build `limtrac` in GNU/Linux environment (WSL also supported). To do it, use standard Cargo build command (for debug purposes): `cargo build`. If you need a release build, use this command: `cargo build --release`.

### Executing

Overtest executes `limtrac` using [CliWrap](https://github.com/Tyrrrz/CliWrap) (a library for .NET applications) with I/O streams piping and setting these environment variables (setting them are required):

```powershell
# A string containing a full path to the executable.
LIMTRAC_FULLPATH = "/usr/bin/example_program"

# A string containing arguments list. Always must contain
# a minimum of one argument. First passed argument must
# be a name of the program.
LIMTRAC_ARGUMENTS = "example_program first_argument second_argument"

# A string which represents a "run as another user" feature.
# If set to empty string, "run as" feature will be disabled.
LIMTRAC_RUNAS = "testuser"
```

Also, when executing `limtrac`, you must set a working directory yourself. `Limtrac` not depends on any other files, so working directory can be safely set to specified in `LIMTRAC_FULLPATH` environment variable program's working directory.

### License

```
LIMTRAC, a part of Overtest free software project.
Copyright (C) 2021, Yurii Kadirov <contact@sirkadirov.com>

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as
published by the Free Software Foundation, either version 3 of the
License, or (at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.
```