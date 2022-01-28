/*
 * LIMTRAC, a part of Overtest free software project.
 * Copyright (C) 2021-2022, Yurii Kadirov <contact@sirkadirov.com>
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

use std::ffi::CStr;
use std::fs::File;
use std::mem::MaybeUninit;
use libc::{c_char, c_int, rlimit64, timer_t};
use nix::errno::errno;
use nix::NixPath;
use syscallz::Syscall;
use crate::{ExecProgGuard, ExecProgInfo, ExecProgIO, ExecProgLimits, SYS_EXEC_FAILED};
use crate::constants::{SYS_EXEC_OK, TIME_MULTIPLIER};

pub fn kill_on_parent_exit()
{
    if unsafe { libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) } == SYS_EXEC_FAILED
    { panic!("System call 'PRCTL' failed with 'ERRNO = {}'!", errno()); }
}

/*
 * This function covers I/O streams redirection to files stored on a disk or other streams.
 * We use system APIs to ensure that all things will work no matter of the situation.
 */

pub fn redirect_io_streams(exec_prog_io : &ExecProgIO)
{
    if !exec_prog_io.io_redirected { return; }

    const FD_STDIN : c_int = 0;
    const FD_STDOUT : c_int = 1;
    const FD_STDERR : c_int = 2;

    let io_path_stdin = unsafe { CStr::from_ptr(exec_prog_io.io_path_stdin) };
    let io_path_stdout = unsafe { CStr::from_ptr(exec_prog_io.io_path_stdout) };
    let io_path_stderr = unsafe { CStr::from_ptr(exec_prog_io.io_path_stderr) };
    // Standard input stream redirection
    if !io_path_stdin.is_empty()
    {
        let file_fd = try_get_fd(exec_prog_io.io_path_stdin, libc::O_RDONLY, false);
        try_dup_fd(file_fd, FD_STDIN);
    }
    // Standard output stream redirection
    if !io_path_stdout.is_empty()
    {
        let file_fd = try_get_fd(exec_prog_io.io_path_stdout, libc::O_WRONLY, true);
        try_dup_fd(file_fd, FD_STDOUT);
        // Duplication of STDERR into a new STDOUT FD
        if exec_prog_io.io_dup_err_out
        { try_dup_fd(file_fd, FD_STDERR); }
    }
    // Standard error stream redirection (if not redirected to STDOUT)
    if !io_path_stderr.is_empty() && !exec_prog_io.io_dup_err_out
    {
        let file_fd = try_get_fd(exec_prog_io.io_path_stderr, libc::O_WRONLY, true);
        try_dup_fd(file_fd, FD_STDERR);
    }

    /* @A lightweight `dup2` system call wrapper */
    fn try_dup_fd(src_fd: c_int, dst_fd: c_int)
    {
        if unsafe { libc::dup2(src_fd, dst_fd) } == SYS_EXEC_FAILED
        { crate::helper_functions::panic_on_syscall!("dup2"); }
    }
    /* @/A lightweight `dup2` system call wrapper */

    fn try_get_fd(file_path: *const c_char, file_flag : c_int, try_create : bool) -> c_int
    {
        // Check whether the specified file exists
        let file_exists = unsafe { libc::access(file_path, libc::F_OK) } == SYS_EXEC_OK;

        // Try to get a Rust string containing a path to file from a C string
        let file_path_str = match unsafe { CStr::from_ptr(file_path) }.to_str() {
            Ok(s) => { s }
            Err(_) => { panic!("Function [try_dup_file] failed: file path C string is corrupted!"); }
        };

        if try_create
        {
            // Try to create (or clear / truncate) a file with the specified path.
            // If file exists, we need to truncate it (clear its contents) because
            // stream redirection corrupts file contents if it is not empty.
            let _file = match File::create(file_path_str) {
                Ok(obj) => { obj }
                Err(e) => { panic!("Function [try_dup_file] failed: can't create a file - {}", e.to_string()); }
            };
        }
        else if !file_exists { panic!("Function [try_dup_file] failed: specified file not found!"); }

        // Verify that we can access the file for reading and writing
        if unsafe { libc::access(file_path, libc::R_OK | libc::W_OK) } != SYS_EXEC_OK
        { panic!("Function [try_dup_file] failed: specified file is not accessible!"); }

        // Note that O_PATH specifies that we don't need to open a file,
        // but only get a descriptor pointing at it to use with `dup2`.
        // Flag O_CREAT indicates that `open` system call must create a
        // file on the specified path, in case it was not found.
        let file_fd = unsafe { libc::open(file_path, file_flag | libc::O_CREAT) };

        // Check whether file opened successfully
        if file_fd == SYS_EXEC_FAILED
        { crate::helper_functions::panic_on_syscall!("open"); }

        // Return a file descriptior pointing to file
        return file_fd;
    }
}

/*
 * This function covers the enforcement of system resources usage limits and
 * policies for the current (child) process, depending on execution request.
 */

pub fn set_resource_limits(exec_prog_limits : &ExecProgLimits)
{
    /* @Set total processor time consumption limit */
    if exec_prog_limits.limit_proc_time > 0
    {
        /*
         * Using RLIMIT_CPU in `setrlimit` system call gives us ability to set
         * resource limit on the total CPU time consumption of the process in
         * seconds, but we need to set it in milliseconds. To bypass this issue,
         * we set limit using that system call in seconds (rounding up the time),
         * then creating a watchdog thread on child execution started, which
         * will fetch current CPU time consumption value from time to time, and
         * can kill a child process if it uses more CPU time than we allow.
         *
         * P.S. Note that in BSD systems RLIMIT_CPU sets a limit in milliseconds.
         */
        let limit_in_seconds : c_int;

        if (exec_prog_limits.limit_proc_time % TIME_MULTIPLIER) == 0
        { limit_in_seconds = exec_prog_limits.limit_proc_time / TIME_MULTIPLIER; }
        else { limit_in_seconds = exec_prog_limits.limit_proc_time / TIME_MULTIPLIER + 1; }

        set_rlimit(libc::RLIMIT_CPU, limit_in_seconds);
    }
    /* @/Set total processor time consumption limit */
    // TODO: Set peak working set and other limits

    /* @Set resource limits using `SETRLIMIT` system call */
    if exec_prog_limits.rlimit_enabled
    {
        set_rlimit(libc::RLIMIT_CORE, exec_prog_limits.rlimit_core);
        set_rlimit(libc::RLIMIT_NPROC, exec_prog_limits.rlimit_npoc);
        set_rlimit(libc::RLIMIT_NOFILE, exec_prog_limits.rlimit_nofile);
    }
    /* @/Set resource limits using `SETRLIMIT` system call */

    /* @Function that utilizes `setrlimit` system call to set resource limit */
    fn set_rlimit(resource: libc::__rlimit_resource_t,
                  limit_value : c_int)
    {
        if limit_value < 0 { return; }

        let rlim_val = limit_value as libc::rlim64_t;
        let rlim_dat : rlimit64 = rlimit64 { rlim_cur: rlim_val, rlim_max: rlim_val };

        if unsafe { libc::setrlimit64(resource, &rlim_dat) } == SYS_EXEC_FAILED
        { crate::helper_functions::panic_on_syscall!("setrlimit"); }
    }
    /* @/Function that utilizes `setrlimit` system call to set resource limit */
}

pub fn init_set_user_id(exec_prog_info : &ExecProgInfo)
{
    let username = unsafe { CStr::from_ptr(exec_prog_info.exec_as_user) };

    if username.is_empty() { return; }

    // Get PASSWD information about the user behind the username
    let pwnam = unsafe { libc::getpwnam(username.as_ptr()) };

    let user_info = match unsafe { pwnam.as_ref() } {
        Some(obj) => obj,
        None => { panic!("System call 'GETPWNAM' failed: user with specified name was not found!") }
    };

    // Try to execute SETUID system call on the current process
    if unsafe { libc::setuid(user_info.pw_uid) } != 0
    { crate::helper_functions::panic_on_syscall!("setuid"); }
}

pub fn unshare_resources(exec_prog_guard: &ExecProgGuard)
{
    if !exec_prog_guard.unshare_enabled { return; }

    // TIP: For more info about `unshare` system call visit
    // https://man7.org/linux/man-pages/man2/unshare.2.html
    unsafe {
        let result = libc::unshare(libc::CLONE_NEWIPC
            | libc::CLONE_NEWNET
            | libc::CLONE_NEWNS
            | libc::CLONE_NEWUTS
            | libc::CLONE_SYSVSEM);

        if result == SYS_EXEC_FAILED
        { crate::helper_functions::panic_on_syscall!("unshare"); }
    }
}

/*
 * This function covers setting up a timer based on built-in features of Linux kernel,
 * which after a [period] of time sends a specified signal to the current process.
 * We use it to send SIGKILL signal to kill the child process (user program) after a
 * certain caller-defined period of time.
 */

pub fn init_set_kill_timer(period: c_int)
{
    // We use `MaybeUninit` to allocate C-compatible object based on a struct
    let mut sigev = MaybeUninit::<libc::sigevent>::uninit();
    let mut timer_id = MaybeUninit::<timer_t>::uninit();

    let period_seconds : libc::time_t = period as libc::time_t / TIME_MULTIPLIER as libc::time_t;
    let period_nanosec : libc::c_long = (period as libc::c_long) % (TIME_MULTIPLIER as libc::c_long)
        * ((TIME_MULTIPLIER * TIME_MULTIPLIER) as libc::c_long);

    let itimer_spec = libc::itimerspec
    {
        it_interval: libc::timespec { tv_sec: 0, tv_nsec: 0 },
        it_value: libc::timespec { tv_sec: period_seconds, tv_nsec: period_nanosec }
    };

    unsafe {
        let mut sigev_ptr = sigev.as_mut_ptr();

        (*sigev_ptr).sigev_notify = libc::SIGEV_SIGNAL;
        (*sigev_ptr).sigev_signo = libc::SIGKILL;

        if libc::timer_create(libc::CLOCK_REALTIME, sigev_ptr, timer_id.as_mut_ptr() ) == SYS_EXEC_FAILED
        { crate::helper_functions::panic_on_syscall!("timer_create"); }

        let timer_id = timer_id.assume_init();
        let _sigev = sigev.assume_init();

        if libc::timer_settime(timer_id, 0, &itimer_spec, std::ptr::null_mut()) == SYS_EXEC_FAILED
        { crate::helper_functions::panic_on_syscall!("timer_settime"); }
    }
}

/*
 * This function covers initialization of several SECCOMP ("secure computing")
 * policies, so child process cannot use system calls, filtered by SECCOMP.
 *
 * Note that usage of this feature requires libseccomp-dev on development machine and
 * enabled support of libseccomp features on the targer computer. Refer to docs of your
 * GNU/Linux distribution on how to enable it.
 */

pub fn init_secure_computing(exec_prog_guard : &ExecProgGuard)
{
    if !exec_prog_guard.scmp_enabled { return; }

    // Initialize a new SECCOMP context with defaults to 'Allow' policy
    let mut ctx = match syscallz::Context::init_with_action(syscallz::Action::Allow) {
        Ok(ctx) => ctx,
        Err(err) => { panic!("Cannot initialize SECCOMP context: {}", err.to_string()) }
    };

    /* @Prevent process from using common unwanted system calls */
    if exec_prog_guard.scmp_deny_common
    {
        // Generate a list of common unwanted system calls
        let syscalls_list = [
            // Deny creating child processes, changing process ownership, etc.
            Syscall::fork, Syscall::vfork, Syscall::clone, Syscall::clone3,
            Syscall::reboot, Syscall::setuid, Syscall::setgid, Syscall::prctl,
            Syscall::setrlimit, Syscall::prlimit64, // Syscall::getrlimit,

            // Operations on per-process timer are denied
            Syscall::timer_create, Syscall::timer_gettime, Syscall::timer_settime, Syscall::timer_delete,
            Syscall::timer_getoverrun, Syscall::timerfd_create, Syscall::timerfd_gettime, Syscall::timerfd_settime,

            // Deny using namespaces and unwanted changes to filesystem
            Syscall::chdir, Syscall::fchdir, Syscall::unshare,
            Syscall::chmod, Syscall::fchmod, Syscall::fchmodat,
            Syscall::chown, Syscall::fchown, Syscall::lchown, Syscall::fchownat
            //Syscall::link, Syscall::unlink
        ];

        // Enumerate through common system calls list
        for sys_call in syscalls_list {
            // Add a 'KillProcess' action filter to every single system call in the list
            match ctx.set_action_for_syscall(syscallz::Action::KillProcess, sys_call) {
                Err(err) => { panic!("Cannot add new SECCOMP filter for system call '{}': {}",
                                     sys_call.into_i32(), err.to_string()) }
                Ok(_) => { /* A new SECCOMP filter successfully added to the current context! */ }
            }
        }
    }
    /* @/Prevent process from using common unwanted system calls */

    // Try to enforce the SECCOMP policy we built for the current process
    match ctx.load() {
        Err(err) => { panic!("SECCOMP policy enforcement failed: {}", err.to_string()) }
        Ok(_) => { /* SECCOMP context loading finished successfully! */ }
    }
}