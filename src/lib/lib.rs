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

use std::ffi::{CStr, CString};
use std::mem::MaybeUninit;
use std::time;
use std::time::SystemTime;
use libc::{c_int, c_ulonglong, pid_t};

mod constants;
mod sandboxing_features;
mod helper_functions;
mod request_structs;
mod result_structs;

use crate::constants::{KILL_REASON_NONE, KILL_REASON_PROCTIME, KILL_REASON_PROCWSET, KILL_REASON_REALTIME, KILL_REASON_SECURITY, SYS_EXEC_FAILED, SYS_EXEC_OK};
use crate::helper_functions::{get_obj_from_ptr};
use crate::request_structs::{ExecProgGuard, ExecProgInfo, ExecProgIO, ExecProgLimits};
use crate::result_structs::ProcExecResult;

#[no_mangle]
pub extern "C" fn limtrac_execute(
    exec_prog_info_ptr   : *const ExecProgInfo,
    exec_prog_io_ptr     : *const ExecProgIO,
    exec_prog_limits_ptr : *const ExecProgLimits,
    exec_prog_guard_ptr  : *const ExecProgGuard
) -> ProcExecResult
{
    // Try to dereference pointers to structs which contain limtrac execution details
    let exec_prog_info   : &ExecProgInfo   = get_obj_from_ptr(exec_prog_info_ptr,   "exec_prog_info_ptr"  );
    let exec_prog_io     : &ExecProgIO     = get_obj_from_ptr(exec_prog_io_ptr,     "exec_prog_io_ptr"    );
    let exec_prog_limits : &ExecProgLimits = get_obj_from_ptr(exec_prog_limits_ptr, "exec_prog_limits_ptr");
    let exec_prog_guard  : &ExecProgGuard  = get_obj_from_ptr(exec_prog_guard_ptr,  "exec_prog_guard_ptr" );

    // Verify data contained in `ExecProgInfo` struct
    if !exec_prog_info.verify()
    { panic!("ExecProgInfo struct contains invalid data!"); }

    // Verify data contained in `ExecProgIO` struct
    if !exec_prog_io.verify()
    { panic!("ExecProgIO struct contains invalid data!"); }

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
     * [Watchdog]
     */

    let mut execution_result  : ProcExecResult = ProcExecResult::new();
    let     loop_exec_timeout : time::Duration = std::time::Duration::from_millis(10);

    loop {
        // Use MaybeUninit to initialize variables used by `wait4` system call
        let mut waitpid_status : MaybeUninit<c_int>        = MaybeUninit::<c_int>::uninit();
        let mut waitpid_rusage : MaybeUninit<libc::rusage> = MaybeUninit::<libc::rusage>::uninit();

        let waitpid_result = unsafe { libc::wait4(child_pid,waitpid_status.as_mut_ptr(), libc::WNOHANG, waitpid_rusage.as_mut_ptr()) };

        // Panic on `wait4` system call execution error
        if waitpid_result == SYS_EXEC_FAILED
        { crate::helper_functions::panic_on_syscall!("wait4"); }

        // Get the child process execution period in milliseconds
        execution_result.res_usage.real_time = child_time_start.elapsed().unwrap().as_millis() as c_ulonglong;

        let waitpid_status : c_int        = unsafe { waitpid_status.assume_init() };
        let waitpid_rusage : libc::rusage = unsafe { waitpid_rusage.assume_init() };

        /* ===== @On child process [executing] ===== */
        if waitpid_result == 0 {

            // Fetch counters' values from the processes `stat` file
            match execution_result.res_usage.load_proc_stat(child_pid) { Ok(_) => {} Err(_) => { continue; } };

            // Wall clock time usage limiting
            if exec_prog_limits.limit_real_time > 0
            {
                if execution_result.res_usage.real_time > exec_prog_limits.limit_real_time
                { kill_with_reason(child_pid, &mut execution_result, KILL_REASON_REALTIME); continue; }
            }

            // Processor time usage imiting
            if exec_prog_limits.limit_proc_time > 0
            {
                if execution_result.res_usage.proc_time > exec_prog_limits.limit_proc_time
                { kill_with_reason(child_pid, &mut execution_result, KILL_REASON_PROCTIME); continue; }
            }

            // Peak working set usage limiting
            if exec_prog_limits.limit_proc_wset > 0
            {
                if execution_result.res_usage.proc_wset > exec_prog_limits.limit_proc_wset
                { kill_with_reason(child_pid, &mut execution_result, KILL_REASON_PROCWSET); continue; }
            }

            fn kill_with_reason(child_pid: pid_t, execution_result: &mut ProcExecResult, kill_reason: c_int)
            {
                match kill_pid(child_pid) { _ => { /* Ignore all results, for now. */ } }
                execution_result.is_killed   = true;
                execution_result.kill_reason = kill_reason;
            }

            /*
             * None of the checks passes, so it seems that the child process
             * not yet used all of the allowed amout of system resources, so
             * now we need to continue our loop after a certain timeout.
             */
            std::thread::sleep(loop_exec_timeout);
            continue;
        }
        /* ===== /@On child process [executing] ===== */

        /* ===== @On child process [state changed] ===== */

        // Gather process stats from `rusage` struct
        execution_result.res_usage.load_rusage(&waitpid_rusage);

        // Get the reason of child process termination
        if libc::WIFEXITED(waitpid_status)
        {
            execution_result.exit_code = libc::WEXITSTATUS(waitpid_status);
            if !execution_result.is_killed {
                execution_result.exit_sign = SYS_EXEC_OK;
                execution_result.kill_reason = KILL_REASON_NONE;
            }
        }
        else if libc::WIFSIGNALED(waitpid_status)
        {
            execution_result.exit_code = SYS_EXEC_FAILED;
            execution_result.exit_sign = libc::WTERMSIG(waitpid_status);

            if !execution_result.is_killed
            {
                // Handle `SIGSYS` like when child process tries to use forbidden system features.
                // For example, `seccomp` kernel feature uses `SIGSYS` to kill processes that try
                // to use system calls, forbidden by the current enforced policy.
                if execution_result.exit_sign == libc::SIGSYS
                { execution_result.kill_reason = KILL_REASON_SECURITY; }

                // WALL CLOCK TIME LIMIT
                else if exec_prog_limits.limit_real_time > 0 && execution_result.res_usage.real_time > exec_prog_limits.limit_real_time
                { execution_result.kill_reason = KILL_REASON_REALTIME; }

                // PROCESSOR TIME LIMIT
                else if exec_prog_limits.limit_proc_time > 0 && execution_result.res_usage.proc_time > exec_prog_limits.limit_proc_time
                { execution_result.kill_reason = KILL_REASON_PROCTIME; }

                // RESIDENT SET SIZE LIMIT
                else if exec_prog_limits.limit_proc_wset > 0 && execution_result.res_usage.proc_wset > exec_prog_limits.limit_proc_wset
                { execution_result.kill_reason = KILL_REASON_PROCWSET; }

                execution_result.is_killed = true;
            }
        }

        // Exit from the loop, because the child process not exists anymore
        break;

        /* ===== /@On child process [state changed] ===== */
    }

    return execution_result;

    /* ===== /[PARENT] PROCESS CODE FRAGMENT ===== */
}

fn exec_child_cmd(
    exec_prog_info   : &ExecProgInfo,
    exec_prog_io     : &ExecProgIO,
    exec_prog_limits : &ExecProgLimits,
    exec_prog_guard  : &ExecProgGuard)
{
    let exec_path : &CStr        = unsafe { CStr::from_ptr(exec_prog_info.program_path) };
    let exec_argv : Vec<CString> = exec_prog_info.get_cstring_argv_vec();

    // Unshare network namespace (requires CAP_SYS_ADMIN)
    if exec_prog_guard.unshare_network
    {
        if unsafe { libc::unshare(libc::CLONE_NEWNET) } == SYS_EXEC_FAILED
        { crate::helper_functions::panic_on_syscall!("unshare"); }
    }

    // Change working directory of a child process
    if unsafe { libc::chdir(exec_prog_info.working_path) } == SYS_EXEC_FAILED
    { crate::helper_functions::panic_on_syscall!("chdir"); }

    // Execute various resource limiting and sandboxing functions
    crate::sandboxing_features::kill_on_parent_exit();
    crate::sandboxing_features::set_resource_limits(exec_prog_limits);
    crate::sandboxing_features::init_set_user_id(exec_prog_info);
    crate::sandboxing_features::redirect_io_streams(exec_prog_io);
    crate::sandboxing_features::init_secure_computing(exec_prog_guard);

    // Try to execute program - on success, `execv` never returns
    match nix::unistd::execv(exec_path, exec_argv.as_slice())
    {
        Err(err) => { panic!("System call 'execv' failed: {}", err.to_string()); }
        Ok(_) => { /* this never happens - on success, `exec` never returns by design */ }
    }
}

fn kill_pid(child_pid: pid_t) -> Result<(), ()>
{
    let result = unsafe { libc::kill(child_pid, libc::SIGKILL) };
    if result == SYS_EXEC_FAILED { Err(()) } else { Ok(()) }
}