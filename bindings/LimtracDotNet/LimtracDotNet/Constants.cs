using System.Diagnostics.CodeAnalysis;

namespace Sirkadirov.Libraries.Limtrac;

[SuppressMessage("ReSharper", "MemberCanBePrivate.Global")]
[SuppressMessage("ReSharper", "UnusedType.Global")]
[SuppressMessage("ReSharper", "UnusedMember.Global")]
public static class Constants
{
    public const int ExitCodeOnFailure  = -1;
    public const int KillReasonUnset    = ExitCodeOnFailure;
    public const int KillReasonNone     = 0;
    public const int KillReasonSecurity = 1;
    public const int KillReasonRealTime = 2;
    public const int KillReasonProcTime = 3;
    public const int KillReasonProcWSet = 4;
}