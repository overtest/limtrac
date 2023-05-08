using System.Diagnostics.CodeAnalysis;
using System.Runtime.InteropServices;

namespace Sirkadirov.Libraries.Limtrac.RequestStructs;

// ReSharper disable InconsistentNaming
[StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
[SuppressMessage("ReSharper", "MemberCanBePrivate.Global")]
[SuppressMessage("ReSharper", "FieldCanBeMadeReadOnly.Global")]
public struct ExecProgIO
{
    [MarshalAs(UnmanagedType.I1)]        public bool   io_redirected;
    [MarshalAs(UnmanagedType.LPUTF8Str)] public string io_path_stdin  = "";
    [MarshalAs(UnmanagedType.LPUTF8Str)] public string io_path_stdout = "";
    [MarshalAs(UnmanagedType.LPUTF8Str)] public string io_path_stderr = "";
    [MarshalAs(UnmanagedType.I1)]        public bool   io_dup_err_out;

    public ExecProgIO()
    {
        io_redirected = false;
        io_dup_err_out = false;
    }
}
// ReSharper restore InconsistentNaming