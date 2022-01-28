#include "../../bindings/limtrac.h"

#include <stdio.h>

int main()
{
    ExecProgInfo execProgInfo;
    execProgInfo.program_path = "/usr/bin/python3";
    execProgInfo.program_args = "../test.py";
    execProgInfo.working_path = "./";
    execProgInfo.exec_as_user = "";

    ExecProgIO execProgIo;
    execProgIo.io_redirected = true;
    execProgIo.io_path_stdin = "../test.py";
    execProgIo.io_path_stdout = "./out.dat";
    execProgIo.io_path_stderr = "";
    execProgIo.io_dup_err_out = true;

    ExecProgLimits execProgLimits;
    execProgLimits.limit_proc_time = 1000;
    execProgLimits.limit_real_time = 5000;
    execProgLimits.limit_proc_wset = 500 * 1000000;
    execProgLimits.rlimit_enabled = false;

    ExecProgGuard  execProgGuard;
    execProgGuard.scmp_enabled = false;
    execProgGuard.scmp_deny_common = true;
    execProgGuard.unshare_enabled = false;

    ProcExecResult execResult = limtrac_execute(
            &execProgInfo,
            &execProgIo,
            &execProgLimits,
            &execProgGuard);

    printf("Exit code:\t%d\r\nExit signal:\t%d\r\nIs killed:\t%d\r\n",
           execResult.exit_code, execResult.exit_sign, execResult.is_killed);
    return 0;
}