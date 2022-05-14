using System.Diagnostics.CodeAnalysis;
using System.Runtime.InteropServices;

namespace Sirkadirov.Libraries.Limtrac.RequestStructs;

[StructLayout(LayoutKind.Sequential)]
[SuppressMessage("ReSharper", "MemberCanBePrivate.Global")]
[SuppressMessage("ReSharper", "FieldCanBeMadeReadOnly.Global")]
public struct ExecProgLimits
{
    [MarshalAs(UnmanagedType.U8)] public ulong limit_real_time;
    [MarshalAs(UnmanagedType.U8)] public ulong limit_proc_time;
    [MarshalAs(UnmanagedType.U8)] public ulong limit_proc_wset;
    
    [MarshalAs(UnmanagedType.I1)] public bool  rlimit_enabled;
    [MarshalAs(UnmanagedType.U8)] public ulong rlimit_core;
    [MarshalAs(UnmanagedType.U8)] public ulong rlimit_npoc;
    [MarshalAs(UnmanagedType.U8)] public ulong rlimit_nofile;
}