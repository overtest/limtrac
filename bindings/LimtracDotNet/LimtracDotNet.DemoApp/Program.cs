using Sirkadirov.Libraries.Limtrac;
using Sirkadirov.Libraries.Limtrac.RequestStructs;

var executionResult = Limtrac.Prepare()
    .WithProgramInfo(new ExecProgInfo
    {
        program_path = "/usr/bin/ping",
        program_args = "1.1.1.1",
        working_path = Environment.CurrentDirectory,
        exec_as_user = "sirkadirov"
    })
    .WithIoConfig(new ExecProgIO
    {
        io_redirected  = true,
        io_path_stdin  = "",
        io_path_stdout = "./out.dat",
        io_path_stderr = "./err.dat",
        io_dup_err_out = false
    })
    .WithLimits(new ExecProgLimits
    {
        limit_real_time = 1000 * 5,     // 5 sec
        limit_proc_time = 1000 * 1,     // 1 sec
        limit_proc_wset = 50 * 1000000, // 50 MB
        
        rlimit_enabled = true,
        rlimit_core    = 0,
        rlimit_npoc    = 2,
        rlimit_nofile  = 100
    })
    .WithGuard(new ExecProgGuard
    {
        scmp_enabled     = true,
        scmp_deny_common = true,
        unshare_common   = true,
        unshare_network  = true
    }).Execute();

//var executionResult = new ProcExecResult() { res_usage = new ProcResUsage() };

Console.WriteLine($"Exit code:\t{executionResult.exit_code}");
Console.WriteLine($"Exit sign:\t{executionResult.exit_sign}");
Console.WriteLine($"Is killed:\t{executionResult.is_killed}");
Console.WriteLine($"Kill reason:\t{executionResult.kill_reason}");
Console.WriteLine();
Console.WriteLine($"Resources usage -> Processor time:\t{executionResult.res_usage.proc_time}");
Console.WriteLine($"Resources usage -> Process RSS (b):\t{executionResult.res_usage.proc_wset}");
Console.WriteLine($"Resources usage -> Real exec time:\t{executionResult.res_usage.real_time}");