using System.Diagnostics.CodeAnalysis;
using System.Runtime.InteropServices;
using Sirkadirov.Libraries.Limtrac.RequestStructs;
using Sirkadirov.Libraries.Limtrac.ResultStructs;

namespace Sirkadirov.Libraries.Limtrac;

internal static class LimtracInterop
{
    [DllImport("liblimtrac.so",
        CallingConvention = CallingConvention.Cdecl,
        EntryPoint = "limtrac_execute")]
    [SuppressMessage("ReSharper", "InconsistentNaming")]
    internal static extern ProcExecResult Execute(
        ExecProgInfo   exec_prog_info,
        ExecProgIO     exec_prog_io,
        ExecProgLimits exec_prog_limits,
        ExecProgGuard  exec_prog_guard
    );
}