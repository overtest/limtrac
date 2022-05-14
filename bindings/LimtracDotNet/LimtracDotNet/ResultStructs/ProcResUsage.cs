using System.Diagnostics.CodeAnalysis;
using System.Runtime.InteropServices;

namespace Sirkadirov.Libraries.Limtrac.ResultStructs;

[StructLayout(LayoutKind.Sequential)]
[SuppressMessage("ReSharper", "MemberCanBePrivate.Global")]
[SuppressMessage("ReSharper", "FieldCanBeMadeReadOnly.Global")]
public struct ProcResUsage
{
    [MarshalAs(UnmanagedType.U8)] public ulong real_time;
    [MarshalAs(UnmanagedType.U8)] public ulong proc_time;
    [MarshalAs(UnmanagedType.U8)] public ulong proc_wset;
}