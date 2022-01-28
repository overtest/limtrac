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

use std::ffi::{CStr};
use std::mem::MaybeUninit;
use std::time::SystemTime;
use libc::{c_int, c_ulonglong, pid_t};

mod constants;
mod sandboxing_features;
mod helper_functions;
mod request_structs;
mod result_structs;

use crate::constants::SYS_EXEC_FAILED;
use crate::helper_functions::{get_obj_from_ptr};
use crate::request_structs::{ExecProgGuard, ExecProgInfo, ExecProgIO, ExecProgLimits};
use crate::result_structs::{ProcExecResult, ProcResUsage};

#[no_mangle]
pub extern "C" fn limtrac_execute(
    exec_prog_info_ptr   : *const ExecProgInfo,
    exec_prog_io_ptr     : *const ExecProgIO,
    exec_prog_limits_ptr : *const ExecProgLimits,
    exec_prog_guard_ptr  : *const ExecProgGuard
) -> ProcExecResult
{
    // Try to dereference pointers to structs which contain limtrac execution details
    let exec_prog_info   : &ExecProgInfo   = get_obj_from_ptr(exec_prog_info_ptr, "exec_prog_info_ptr");
    let exec_prog_io     : &ExecProgIO     = get_obj_from_ptr(exec_prog_io_ptr, "exec_prog_io_ptr");
    let exec_prog_limits : &ExecProgLimits = get_obj_from_ptr(exec_prog_limits_ptr, "exec_prog_limits_ptr");
    let exec_prog_guard  : &ExecProgGuard  = get_obj_from_ptr(exec_prog_guard_ptr, "exec_prog_guard_ptr");

    // Verify data contained in `ExecProgInfo` struct
    if !exec_prog_info.verify()
    { panic!("ExecProgInfo struct contains invalid data!"); }

    // Verify data contained in `ExecProgIO` struct
    if !exec_prog_io.verify()
    { panic!("ExecProgIO struct contains invalid data!"); }

    /*
     * Try to create a new child process based on the current one, so we
     * can control everything about it in the parent (current) process.
     */

    // Try to fork (try to create a child process)
    let child_pid = unsafe { libc::fork() };
    let child_time_start = SystemTime::now();

    // If `child_pid` variable equals to '-1', `fork` system call failed!
    if child_pid == SYS_EXEC_FAILED
    { crate::helper_functions::panic_on_syscall!("fork"); }

    // Use `ptrace` syscall to ensure that the child process exits
    // on parent process crash: https://linux.die.net/man/2/ptrace

    /* ===== [CHILD] PROCESS CODE FRAGMENT ===== */
    if child_pid == 0
    {
        // We are in a child process right now, so we can execute whatever we want
        exec_child_cmd(exec_prog_info, exec_prog_io, exec_prog_limits, exec_prog_guard);
    }
    /* ===== /[CHILD] PROCESS CODE FRAGMENT ===== */

    /* ===== [PARENT] PROCESS CODE FRAGMENT ===== */

    /*
     * [Watchdog thread]
     */

    // Clone variables to move them to a thread scope
    //let thread_child_pid : pid_t = child_pid.clone() as pid_t;
    //let thread_exec_prog_limits : &ExecProgLimits = exec_prog_limits.clone();
    // Start a watchdog thread and pass cloned variables to it
    //std::thread::spawn(move || exec_watchdog(thread_child_pid, thread_exec_prog_limits)).join().unwrap();

    let mut execution_result : ProcExecResult = ProcExecResult::new();
    let mut execution_rusage : ProcResUsage = ProcResUsage::new();

    // Use MaybeUninit to initialize variables used by `wait4` system call
    let mut waitpid_status = MaybeUninit::<c_int>::uninit();
    let mut waitpid_rusage = MaybeUninit::<libc::rusage>::uninit();

    // Execute `wait4` system call to wait for child exit
    let waitpid_result = unsafe { libc::wait4(child_pid, waitpid_status.as_mut_ptr(), 0, waitpid_rusage.as_mut_ptr()) };

    if waitpid_result == SYS_EXEC_FAILED
    { crate::helper_functions::panic_on_syscall!("wait4"); }

    // Get the child process execution period in milliseconds
    execution_rusage.real_time = child_time_start.elapsed().unwrap().as_millis() as c_ulonglong;

    let waitpid_status = unsafe { waitpid_status.assume_init() };
    let waitpid_rusage = unsafe { waitpid_rusage.assume_init() };

    // Gather process stats from `rusage` struct
    execution_rusage.load_rusage(&waitpid_rusage);

    // Get the reason of child process termination
    if libc::WIFEXITED(waitpid_status)
    { execution_result.exit_code = libc::WEXITSTATUS(waitpid_status); }
    else if libc::WIFSIGNALED(waitpid_status)
    {
        execution_result.exit_code = -1;
        execution_result.exit_sign = libc::WSTOPSIG(waitpid_status);
    }

    /*
     * Prepare execution results and return them to the caller.
     */
    execution_result.res_usage = &mut execution_rusage;
    return execution_result;

    /* ===== /[PARENT] PROCESS CODE FRAGMENT ===== */
}

fn exec_child_cmd(
    exec_prog_info   : &ExecProgInfo,
    exec_prog_io     : &ExecProgIO,
    exec_prog_limits : &ExecProgLimits,
    exec_prog_guard  : &ExecProgGuard)
{

    let exec_path = unsafe { CStr::from_ptr(exec_prog_info.program_path) };
    let exec_argv = exec_prog_info.get_cstring_argv_vec();//= exec_prog_info.get_program_args_list();

    // Change working directory of a child process
    if unsafe { libc::chdir(exec_prog_info.working_path) } == SYS_EXEC_FAILED
    { crate::helper_functions::panic_on_syscall!("chdir"); }

    // Execute various resource limiting and sandboxing functions
    crate::sandboxing_features::kill_on_parent_exit();
    crate::sandboxing_features::set_resource_limits(exec_prog_limits);
    crate::sandboxing_features::init_set_user_id(exec_prog_info);
    crate::sandboxing_features::unshare_resources(exec_prog_guard);
    crate::sandboxing_features::redirect_io_streams(exec_prog_io);
    // TODO: Convert nanosec to sec and nanosec
    //crate::sandboxing_features::init_set_kill_timer(exec_prog_limits.limit_real_time);
    crate::sandboxing_features::init_secure_computing(exec_prog_guard);

    // Try to execute program - on success, `execv` never returns
    match nix::unistd::execv(exec_path, exec_argv.as_slice())
    {
        Err(err) => { panic!("System call 'execv' failed: {}", err.to_string()); }
        Ok(_) => { /* this never happens - on success, `exec` never returns by design */ }
    }
}

fn exec_watchdog(child_pid: pid_t, exec_prog_limits: &ExecProgLimits)
{
    let mut res_usage_cur = ProcResUsage::new();
    loop {
        // Use MaybeUninit to initialize variables used by `wait4` system call
        let mut child_rusage = MaybeUninit::<libc::rusage>::uninit();
        let mut child_status = MaybeUninit::<c_int>::uninit();

        // Try to fetch the current status of child process
        let child_waitpid = unsafe { libc::wait4(child_pid, child_status.as_mut_ptr(),
                                                libc::WNOHANG, child_rusage.as_mut_ptr()) };
        let child_rusage = unsafe { child_rusage.assume_init() };
        let _child_status = unsafe { child_status.assume_init() };

        // Panic if `waitpid` system call have failed
        if child_waitpid == SYS_EXEC_FAILED
        { crate::helper_functions::panic_on_syscall!("waitpid"); }
        // Break the loop if child process already exited
        else if child_waitpid != 0 { break; }

        // Load resources usage into easily redable struct
        res_usage_cur.load_rusage(&child_rusage);

        // Check current resources usage and kill the child if it exceeds the limits
        // TODO: Use `cgroup` to limit memory usage by the child process
        if res_usage_cur.proc_time > exec_prog_limits.limit_proc_time as c_ulonglong
            || res_usage_cur.proc_wset > exec_prog_limits.limit_proc_wset
        {
            // Try to kill a child process using `SIGKILL` signal
            match kill_pid(child_pid) { Err(_) => { break; } Ok(_) => { /* continue on success */ } }
        }
    }
}

fn kill_pid(child_pid: pid_t) -> Result<(), ()>
{
    let result = unsafe { libc::kill(child_pid, libc::SIGKILL) };
    if result == SYS_EXEC_FAILED { Err(()) }
    else { Ok(()) }
}