using System.Diagnostics.CodeAnalysis;
using System.Runtime.InteropServices;

namespace Sirkadirov.Libraries.Limtrac.RequestStructs;

[StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
[SuppressMessage("ReSharper", "MemberCanBePrivate.Global")]
[SuppressMessage("ReSharper", "FieldCanBeMadeReadOnly.Global")]
public struct ExecProgInfo
{
    [MarshalAs(UnmanagedType.LPUTF8Str)] public string program_path = "";
    [MarshalAs(UnmanagedType.LPUTF8Str)] public string program_args = "";
    [MarshalAs(UnmanagedType.LPUTF8Str)] public string working_path = "";
    [MarshalAs(UnmanagedType.LPUTF8Str)] public string exec_as_user = "";
}