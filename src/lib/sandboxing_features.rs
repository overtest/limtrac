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
use libc::{c_char, c_int, c_ulonglong, rlim64_t, rlimit64, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO};
use syscallz::Syscall;
use crate::{ExecProgGuard, ExecProgInfo, ExecProgIO, ExecProgLimits, SYS_EXEC_FAILED};
use crate::constants::{SYS_EXEC_OK, TIME_MULTIPLIER};

pub fn unshare_resources(exec_prog_guard : &ExecProgGuard)
{
    /*
     * Unshare system resources so this process and its child
     * processes won't be able to do some things related to
     * other processes and actions running in the system.
     *
     * Note that some of enforced `unshare` system call
     * policies require CAP_SYS_ADMIN capability of a caller.
     */
    if exec_prog_guard.unshare_common
    {
        unsafe {
            let result = libc::unshare(
                libc::CLONE_NEWNS | libc::CLONE_NEWIPC
                    | libc::CLONE_NEWUTS | libc::CLONE_NEWPID
                    | libc::CLONE_NEWCGROUP | libc::CLONE_SYSVSEM);
            // Panic if system call execution failed
            if result == SYS_EXEC_FAILED
            { crate::helper_functions::panic_on_syscall!("unshare"); }
        }
    }

    // Unshare network namespace (requires CAP_SYS_ADMIN)
    if exec_prog_guard.unshare_network && unsafe { libc::unshare(libc::CLONE_NEWNET) } == SYS_EXEC_FAILED
    { crate::helper_functions::panic_on_syscall!("unshare"); }
}

pub fn set_work_dir(exec_prog_info : &ExecProgInfo)
{
    // Change working directory of a child process
    if unsafe { libc::chdir(exec_prog_info.working_path) } == SYS_EXEC_FAILED
    { crate::helper_functions::panic_on_syscall!("chdir"); }
}

pub fn kill_on_parent_exit()
{
    if unsafe { libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) } == SYS_EXEC_FAILED
    { crate::helper_functions::panic_on_syscall!("prctl"); }
}

/*
 * This function covers I/O streams redirection to files stored on a disk or other streams.
 * We use system APIs to ensure that all things will work no matter of the situation.
 */

pub fn redirect_io_streams(exec_prog_io : &ExecProgIO)
{
    if !exec_prog_io.io_redirected { return; }

    let io_path_stdin  : &CStr = unsafe { CStr::from_ptr(exec_prog_io.io_path_stdin) };
    let io_path_stdout : &CStr = unsafe { CStr::from_ptr(exec_prog_io.io_path_stdout) };
    let io_path_stderr : &CStr = unsafe { CStr::from_ptr(exec_prog_io.io_path_stderr) };

    // Standard input stream redirection
    if !io_path_stdin.to_bytes().is_empty()
    {
        let file_fd = try_get_fd(exec_prog_io.io_path_stdin, libc::O_RDONLY);
        try_dup_fd(file_fd, STDIN_FILENO);
    }
    else { dup_to_dev_null(STDIN_FILENO); }

    // Standard output stream redirection
    if !io_path_stdout.to_bytes().is_empty()
    {
        let file_fd = try_get_fd(exec_prog_io.io_path_stdout, libc::O_WRONLY);
        try_dup_fd(file_fd, STDOUT_FILENO);

        // Duplication of STDERR into a new STDOUT FD
        if exec_prog_io.io_dup_err_out
        { try_dup_fd(file_fd, STDERR_FILENO); }
    }
    else { dup_to_dev_null(STDOUT_FILENO); }

    // Standard error stream redirection (if not redirected to STDOUT)
    if !exec_prog_io.io_dup_err_out
    {
        if !io_path_stderr.to_bytes().is_empty()
        {
            let file_fd = try_get_fd(exec_prog_io.io_path_stderr, libc::O_WRONLY);
            try_dup_fd(file_fd, STDERR_FILENO);
        }
        else { dup_to_dev_null(STDERR_FILENO); }
    }

    fn dup_to_dev_null(dest_fd: c_int)
    {
        let dev_null = CString::new("/dev/null").unwrap();
        let file_fd = try_get_fd(dev_null.as_ptr(), libc::O_RDWR);
        try_dup_fd(file_fd, dest_fd);
    }

    /* @A lightweight `dup2` system call wrapper */
    fn try_dup_fd(src_fd: c_int, dst_fd: c_int)
    {
        if unsafe { libc::dup2(src_fd, dst_fd) } == SYS_EXEC_FAILED
        { crate::helper_functions::panic_on_syscall!("dup2"); }
    }
    /* @/A lightweight `dup2` system call wrapper */

    fn try_get_fd(file_path: *const c_char, file_flag : c_int) -> c_int
    {
        // Note that O_PATH specifies that we don't need to open a file,
        // but only get a descriptor pointing at it to use with `dup2`.
        // Flag O_CREAT indicates that `open` system call must create a
        // file on the specified path, in case it was not found.
        let file_fd = unsafe { libc::open(file_path, file_flag | libc::O_CREAT) };

        // Check whether file opened successfully
        if file_fd == SYS_EXEC_FAILED
        { crate::helper_functions::panic_on_syscall!("open"); }

        // Return a file descriptior pointing to file
        file_fd
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
        let mut limit_in_seconds : c_ulonglong;

        if (exec_prog_limits.limit_proc_time % TIME_MULTIPLIER as c_ulonglong) == 0
        { limit_in_seconds = exec_prog_limits.limit_proc_time / TIME_MULTIPLIER as c_ulonglong; }
        else { limit_in_seconds = exec_prog_limits.limit_proc_time / TIME_MULTIPLIER as c_ulonglong + 1; }

        /*
         * In general, we don't want to use hard limit because of some unexpected behaviours
         * it brings in our librarie's logics. Instead of it, we use soft limiter that fetches
         * processor time usage information from `/proc/[pid]/stat` file and kills the process
         * if it exceeds the limit set by the library user.
         *
         * But for security reasons, we need to set the hard limit so that the child process
         * will be killed even if our soft imiter will stuck or something else.
         */
        limit_in_seconds += 1;

        set_rlimit(libc::RLIMIT_CPU, limit_in_seconds as rlim64_t);
    }
    /* @/Set total processor time consumption limit */

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
                  limit_value : libc::c_ulong)
    {
        let rlim_val : rlim64_t = limit_value as rlim64_t;
        let rlim_dat : rlimit64 = rlimit64 { rlim_cur: rlim_val, rlim_max: rlim_val };

        if unsafe { libc::setrlimit64(resource, &rlim_dat) } == SYS_EXEC_FAILED
        { crate::helper_functions::panic_on_syscall!("setrlimit"); }
    }
    /* @/Function that utilizes `setrlimit` system call to set resource limit */
}

pub fn init_set_user_id(exec_prog_info : &ExecProgInfo)
{
    let username = unsafe { CStr::from_ptr(exec_prog_info.exec_as_user) };

    if username.to_bytes().is_empty() { return; }

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
        Err(err) => { panic!("Cannot initialize SECCOMP context: {}", err) }
    };

    /* @Prevent process from using common unwanted system calls */
    if exec_prog_guard.scmp_deny_common
    {
        // Generate a list of common unwanted system calls
        let syscalls_list = [
            // Deny creating child processes, changing process ownership, etc.
            Syscall::reboot, Syscall::setuid, Syscall::setgid, Syscall::prctl,
            Syscall::unshare, Syscall::setrlimit, //Syscall::prlimit64, // Syscall::getrlimit,

            // Operations on per-process timer are denied
            Syscall::timer_create, Syscall::timer_gettime, Syscall::timer_settime, Syscall::timer_delete,
            Syscall::timer_getoverrun, Syscall::timerfd_create, Syscall::timerfd_gettime, Syscall::timerfd_settime,

            // Deny making unwanted changes to filesystem
            Syscall::chdir, Syscall::fchdir, Syscall::chmod, Syscall::fchmod, Syscall::fchmodat,
            Syscall::chown, Syscall::fchown, Syscall::lchown, Syscall::fchownat
            //Syscall::link, Syscall::unlink
        ];

        // Enumerate through common system calls list
        for sys_call in syscalls_list {
            // Add a 'KillProcess' action filter to every single system call in the list
            if let Err(err) = ctx.set_action_for_syscall(syscallz::Action::KillProcess, sys_call)
            { panic!("Cannot add new SECCOMP filter for system call '{}': {}", sys_call.into_i32(), err) }
        }
    }
    /* @/Prevent process from using common unwanted system calls */

    // Try to enforce the SECCOMP policy we built for the current process
    if let Err(err) = ctx.load()
    { panic!("SECCOMP policy enforcement failed: {}", err) }
}