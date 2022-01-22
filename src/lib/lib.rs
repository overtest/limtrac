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

use std::time::SystemTime;
use libc::{c_int, c_ulonglong};

mod constants;
mod data_models;
mod sandboxing_features;
mod helper_functions;

use crate::constants::SYS_EXEC_FAILED;
use crate::data_models::{ExecProgGuard, ExecProgInfo, ExecProgIO, ExecProgLimits, ProcExecResult, ProcResUsage};
use crate::helper_functions::{get_obj_from_ptr, panic_on_syscall};

#[no_mangle]
pub extern "C" fn limtrac_execute(
    exec_prog_info_ptr   : *const ExecProgInfo,
    exec_prog_io_ptr     : *const ExecProgIO,
    exec_prog_limits_ptr : *const ExecProgLimits,
    exec_prog_guard_ptr  : *const ExecProgGuard
) -> *mut ProcExecResult
{
    // Try to dereference pointers to structs which contain limtrac execution details
    let exec_prog_info: &ExecProgInfo = get_obj_from_ptr(exec_prog_info_ptr, "exec_prog_info_ptr");
    let exec_prog_io: &ExecProgIO = get_obj_from_ptr(exec_prog_io_ptr, "exec_prog_io_ptr");
    let exec_prog_limits: &ExecProgLimits = get_obj_from_ptr(exec_prog_limits_ptr, "exec_prog_limits_ptr");
    let exec_prog_guard: &ExecProgGuard = get_obj_from_ptr(exec_prog_guard_ptr, "exec_prog_guard_ptr");

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
    { panic_on_syscall("fork"); }

    // Use `ptrace` syscall to ensure that the child process exits
    // on parent process crash: https://linux.die.net/man/2/ptrace

    /* ===== [CHILD] PROCESS CODE FRAGMENT ===== */
    if child_pid == 0
    {
        // We are in a child process right now, so we can execute whatever we want
        exec_child_cmd(exec_prog_info, exec_prog_io, exec_prog_limits, exec_prog_guard);
        // Panicing command won't be executed, no matter of the situation
        panic!("LIMTRAC execution failed due to an unknown error!");
    }
    /* ===== /[CHILD] PROCESS CODE FRAGMENT ===== */

    /* ===== [PARENT] PROCESS CODE FRAGMENT ===== */

    let mut execution_result : ProcExecResult = ProcExecResult::new();
    let mut execution_rusage : ProcResUsage = ProcResUsage::new();

    let waitpid_status : *mut c_int = std::ptr::null_mut();
    let waitpid_rusage : *mut libc::rusage = std::ptr::null_mut();

    // Execute `WAIT4` system call to wait for child exit
    unsafe { libc::wait4(child_pid, waitpid_status, libc::WEXITED, waitpid_rusage) };

    // Get the child process execution period in milliseconds
    execution_rusage.real_time = child_time_start.elapsed().unwrap().as_millis() as c_ulonglong;
    // Format RUSAGE struct
    execution_rusage.load_rusage(waitpid_rusage);

    // Get the reason of child process termination
    if libc::WIFEXITED(unsafe { *waitpid_status })
    { execution_result.exit_code = libc::WEXITSTATUS(unsafe { *waitpid_status }); }
    else if libc::WIFSIGNALED(unsafe { *waitpid_status })
    {
        execution_result.exit_code = -1;
        execution_result.exit_sign = libc::WSTOPSIG(unsafe { *waitpid_status });
    }

    /*
     * Prepare execution results and return them to the caller.
     */

    execution_result.res_usage = Box::into_raw(Box::new(execution_rusage));
    return Box::into_raw(Box::new(execution_result));

    /* ===== /[PARENT] PROCESS CODE FRAGMENT ===== */
}

fn exec_child_cmd(
    exec_prog_info: &ExecProgInfo,
    exec_prog_io: &ExecProgIO,
    exec_prog_limits: &ExecProgLimits,
    exec_prog_guard: &ExecProgGuard)
{
    // Change working directory of a child process
    if unsafe { libc::chdir(exec_prog_info.working_dir) } == SYS_EXEC_FAILED
    { panic_on_syscall("chdir"); }

    // Execute various resource limiting and sandboxing functions
    crate::sandboxing_features::kill_on_parent_exit();
    crate::sandboxing_features::set_resource_limits(exec_prog_limits);
    crate::sandboxing_features::init_set_user_id(exec_prog_info);
    crate::sandboxing_features::unshare_resources(exec_prog_guard);
    crate::sandboxing_features::redirect_io_streams(exec_prog_io);
    crate::sandboxing_features::init_set_kill_timer(exec_prog_limits.limit_real_time);
    crate::sandboxing_features::init_secure_computing(exec_prog_guard);

    // Try to execute program - on success, `execv` never returns
    let exec_argv = exec_prog_info.get_program_args_list();
    let exec_result = unsafe { libc::execv(exec_prog_info.program_path, exec_argv) };

    // Handle an error if `exec` system call failed
    if exec_result == SYS_EXEC_FAILED
    { panic_on_syscall("execv"); }
}