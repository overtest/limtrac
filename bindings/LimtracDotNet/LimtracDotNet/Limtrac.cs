using System.Diagnostics.CodeAnalysis;
using System.Runtime.InteropServices;
using Sirkadirov.Libraries.Limtrac.RequestStructs;
using Sirkadirov.Libraries.Limtrac.ResultStructs;

namespace Sirkadirov.Libraries.Limtrac;

[SuppressMessage("ReSharper", "UnusedType.Global")]
[SuppressMessage("ReSharper", "UnusedMember.Global")]
public class Limtrac
{
    private ExecProgInfo   _execProgInfo;
    private ExecProgIO     _execProgIo;
    private ExecProgLimits _execProgLimits;
    private ExecProgGuard  _execProgGuard;
    
    private Limtrac() {  }
    public static Limtrac Prepare() { return new Limtrac(); }
    
    public Limtrac WithProgramInfo(ExecProgInfo execProgInfo)
    {
        _execProgInfo = execProgInfo;
        return this;
    }
    
    public Limtrac WithIoConfig(ExecProgIO execProgIo)
    {
        _execProgIo = execProgIo;
        return this;
    }

    public Limtrac WithLimits(ExecProgLimits execProgLimits)
    {
        _execProgLimits = execProgLimits;
        return this;
    }

    public Limtrac WithGuard(ExecProgGuard execProgGuard)
    {
        _execProgGuard = execProgGuard;
        return this;
    }

    private void ThrowIfNotReadyToExecute()
    {
        ArgumentNullException.ThrowIfNull(_execProgInfo,   nameof(ExecProgInfo));
        ArgumentNullException.ThrowIfNull(_execProgIo,     nameof(ExecProgIO));
        ArgumentNullException.ThrowIfNull(_execProgLimits, nameof(ExecProgLimits));
        ArgumentNullException.ThrowIfNull(_execProgGuard,  nameof(ExecProgGuard));
    }

    public ProcExecResult Execute()
    {
        if (!RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
            throw new PlatformNotSupportedException("Limtrac is available only on Linux!");
        ThrowIfNotReadyToExecute();
        
        return LimtracInterop.Execute(_execProgInfo!, _execProgIo!, _execProgLimits!, _execProgGuard!);
    }
}