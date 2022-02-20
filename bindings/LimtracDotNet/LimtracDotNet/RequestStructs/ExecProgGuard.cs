using System.Diagnostics.CodeAnalysis;
using System.Runtime.InteropServices;

namespace Sirkadirov.Libraries.Limtrac.RequestStructs;

[StructLayout(LayoutKind.Sequential)]
[SuppressMessage("ReSharper", "MemberCanBePrivate.Global")]
[SuppressMessage("ReSharper", "FieldCanBeMadeReadOnly.Global")]
public struct ExecProgGuard
{
    [MarshalAs(UnmanagedType.I1)] public bool scmp_enabled;
    [MarshalAs(UnmanagedType.I1)] public bool scmp_deny_common;
    [MarshalAs(UnmanagedType.I1)] public bool unshare_common;
    [MarshalAs(UnmanagedType.I1)] public bool unshare_network;
}