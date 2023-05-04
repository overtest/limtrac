/*
 * LIMTRAC, a part of Overtest free software project.
 * Copyright (C) 2021-2023, Yurii Kadirov <contact@sirkadirov.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Lesser General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use std::ffi::{CStr, CString};
use std::path::Path;
use libc::{c_char, c_ulong, c_ulonglong};

#[repr(C)]
pub struct ExecProgInfo
{
    pub program_path : *const c_char,
    pub program_args : *const c_char,
    pub working_path : *const c_char,
    pub exec_as_user : *const c_char
}

impl ExecProgInfo {
    fn check_ptrs(&self) -> bool
    {
        !self.program_path.is_null()
            && !self.program_args.is_null()
            && !self.working_path.is_null()
            && !self.exec_as_user.is_null()
    }

    fn check_paths(&self) -> bool
    {
        return Path::new(unsafe { CStr::from_ptr(self.program_path) }.to_str().unwrap()).is_file()
            && Path::new(unsafe { CStr::from_ptr(self.working_path) }.to_str().unwrap()).is_dir();
    }

    pub fn verify(&self) -> bool
    {
        self.check_ptrs() && self.check_paths()
    }

    pub fn get_cstring_argv_vec(&self) -> Vec<CString>
    {
        // Note that argv[0] must contain a name of the executable file
        let args_str_base = Path::new(unsafe { CStr::from_ptr(self.program_path) }.to_str().unwrap())
            .file_name().unwrap()
            .to_str().unwrap()
            .to_owned(); // allocate more resources to this string
        let args_str_append = unsafe { CStr::from_ptr(self.program_args) }.to_str().unwrap();

        // Form a string containing all program command line arguments (including argv[0])
        let args_str = args_str_base + " " + args_str_append;
        let args_str = args_str.trim_end(); // trim end in case of empty arguments list
        // Split an `args_str` into a vector containing substrings (words / arguments)
        let args_vec = args_str.split(' ').collect::<Vec<&str>>();

        let mut cstring_vec : Vec<CString> = vec![];

        for arg in args_vec.iter()
        {
            let arg = arg.to_string();
            cstring_vec.push(CString::new(arg).unwrap());
        }

        cstring_vec
    }
}

#[repr(C)]
pub struct ExecProgIO
{
    pub io_redirected : bool,
    pub io_path_stdin : *const c_char,
    pub io_path_stdout : *const c_char,
    pub io_path_stderr : *const c_char,
    pub io_dup_err_out : bool
}

impl ExecProgIO
{
    fn check_ptrs(&self) -> bool
    {
        !self.io_path_stdin.is_null()
            && !self.io_path_stdout.is_null()
            && !self.io_path_stderr.is_null()
    }

    pub fn verify(&self) -> bool
    {
        if !self.io_redirected { return true; }
        if !self.check_ptrs() { return false; }

        /*
         * Try to convert raw C strings into CStr, so
         * we can operate with them in a safe way.
         */
        let fpath_stdin = unsafe { CStr::from_ptr(self.io_path_stdin) };
        let fpath_stdout = unsafe { CStr::from_ptr(self.io_path_stdout) };
        let fpath_stderr = unsafe { CStr::from_ptr(self.io_path_stderr) };

        /*
         * All paths cannot be empty if we
         * see that IO redirection feature
         * is enabled by the LIMTRAC caller.
         */
        if fpath_stdin.to_bytes().is_empty()
            && fpath_stdout.to_bytes().is_empty()
            && fpath_stderr.to_bytes().is_empty()
        { return false; }

        /*
         * Stderr cannot be passed to file and to
         * stdout simulateously. After that, check
         * whether stdout is redirected to a file.
         */
        if self.io_dup_err_out
            && (!fpath_stderr.to_bytes().is_empty()
            || fpath_stdout.to_bytes().is_empty())
        { return false; }

        // If STDIN redirection is enabled, input file must be present
        if !fpath_stdin.to_bytes().is_empty() && !Path::new(fpath_stdin.to_str().unwrap()).is_file()
        { return false; }

        // All checks passed
        true
    }
}

#[repr(C)]
pub struct ExecProgLimits
{
    pub limit_real_time : c_ulonglong, // real execution time
    pub limit_proc_time : c_ulonglong, // processor time
    pub limit_proc_wset : c_ulonglong, // process working set

    pub rlimit_enabled : bool,    // Set other RLIMITs
    pub rlimit_core : c_ulong, // RLIM_CORE
    pub rlimit_npoc : c_ulong, // RLIM_NPROC
    pub rlimit_nofile : c_ulong  // RLIM_NOFILE
}

#[repr(C)]
pub struct ExecProgGuard
{
    pub scmp_enabled : bool,
    pub scmp_deny_common : bool,
    pub unshare_common : bool,
    pub unshare_network : bool
}