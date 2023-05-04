#include "../../bindings/limtrac.h"
#include "./main.h"
#include <stdio.h>

int main()
{
    // Get process start information
    ExecProgInfo   execProgInfo   = get_exec_prog_info();
    ExecProgIO     execProgIo     = get_exec_prog_io();
    ExecProgLimits execProgLimits = get_exec_prog_limits();
    ExecProgGuard  execProgGuard  = get_exec_prog_guard();

    // Execute LIMTRAC runner
    ProcExecResult execResult = limtrac_execute(
            execProgInfo,
            execProgIo,
            execProgLimits,
            execProgGuard);

    // Print execution result
    printf("Exit code:\t%d\r\nExit signal:\t%d\r\nIs killed:\t%d\r\nKill reason:\t%d\r\n",
           execResult.exit_code, execResult.exit_sign, execResult.is_killed, execResult.kill_reason);
    printf("\r\n");
    // Print resources usage
    printf("Exec time:\t%llu\r\nProc time:\t%llu\r\nMax RSS:\t%llu\r\n",
           execResult.res_usage.real_time, execResult.res_usage.proc_time, execResult.res_usage.proc_wset);

    return 0;
}

ExecProgInfo get_exec_prog_info()
{
    ExecProgInfo execProgInfo;
    execProgInfo.program_path = "/usr/bin/python3";
    execProgInfo.program_args = "../test.py one two three four five six seven eight nine ten";
    execProgInfo.working_path = "./";
    execProgInfo.exec_as_user = "";
    return execProgInfo;
}

ExecProgIO get_exec_prog_io()
{
    ExecProgIO execProgIo;
    execProgIo.io_redirected  = true;
    execProgIo.io_path_stdin  = "../test.py";
    execProgIo.io_path_stdout = "./out.dat";
    execProgIo.io_path_stderr = "";
    execProgIo.io_dup_err_out = true;
    return execProgIo;
}

ExecProgLimits get_exec_prog_limits()
{
    ExecProgLimits execProgLimits;
    execProgLimits.limit_proc_time = 1000 * 1;
    execProgLimits.limit_real_time = 1000 * 5;
    execProgLimits.limit_proc_wset = 50 * 1000000; // 50 MB
    execProgLimits.rlimit_enabled  = false;
    return execProgLimits;
}

ExecProgGuard get_exec_prog_guard()
{
    ExecProgGuard execProgGuard;
    execProgGuard.scmp_enabled     = true;
    execProgGuard.scmp_deny_common = true;
    execProgGuard.unshare_common   = true;
    execProgGuard.unshare_network  = true;
    return execProgGuard;
}