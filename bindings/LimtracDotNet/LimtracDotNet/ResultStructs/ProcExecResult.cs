using System.Diagnostics.CodeAnalysis;
using System.Runtime.InteropServices;

namespace Sirkadirov.Libraries.Limtrac.ResultStructs;

[StructLayout(LayoutKind.Sequential)]
[SuppressMessage("ReSharper", "MemberCanBePrivate.Global")]
[SuppressMessage("ReSharper", "FieldCanBeMadeReadOnly.Global")]
public struct ProcExecResult
{
    [MarshalAs(UnmanagedType.I4)] public int  exit_code;
    [MarshalAs(UnmanagedType.I4)] public int  exit_sign;
    [MarshalAs(UnmanagedType.I1)] public bool is_killed;
    [MarshalAs(UnmanagedType.I4)] public int  kill_reason;
    
    [MarshalAs(UnmanagedType.Struct)] public ProcResUsage res_usage;
}