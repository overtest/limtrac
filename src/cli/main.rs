use std::env;
use std::ffi::CString;
use std::io::{Error, ErrorKind};
use nix::errno::errno;

fn main() {
    let exec_request = ProcExecRequestInt
    {
        full_path: env::var("LIMTRAC_FULLPATH").expect("LIMTRAC_FULLPATH not set!"),
        arguments: env::var("LIMTRAC_ARGUMENTS").expect("LIMTRAC_ARGUMENTS not set!"),
        runas_user: env::var("LIMTRAC_RUNAS").expect("LIMTRAC_RUNAS not set!")
    };

    unsafe { execute_process(exec_request); }
}

struct ProcExecRequestInt
{
    pub full_path: String,
    pub arguments: String,
    //pub work_dir: String,
    pub runas_user: String
}

unsafe fn execute_process(exec_request: ProcExecRequestInt)
{
    let argv = get_strings(exec_request.arguments.as_str());
    child_pre_execute(&exec_request).unwrap();
    
    let c_fullpath = CString::new(exec_request.full_path.as_str()).unwrap();
    if nix::libc::execv(c_fullpath.as_ptr(), argv) == -1
    {
        panic!("Program execution failed with ERRNO {}! Refer to <errno.h> to troubleshoot this issue.", errno());
        // https://android.googlesource.com/kernel/lk/+/dima/for-travis/include/errno.h
    }
}

unsafe fn get_strings(in_str : &str) -> *const *const nix::libc::c_char
{
    let mut v = vec![];
    
    if !(in_str.trim().is_empty())
    {
        let str_arr = in_str.split_whitespace();
        for substr in str_arr
        {
            v.push(CString::new(substr).unwrap());
        }
    }

    let mut out = v
        .into_iter()
        .map(|s| s.as_ptr())
        .collect::<Vec<_>>();

    out.push(std::ptr::null());
    out.shrink_to_fit();

    let ptr = out.as_ptr();
    std::mem::forget(out);

    return ptr;
}

unsafe fn child_pre_execute(exec_request: &ProcExecRequestInt) -> Result<(), std::io::Error>
{
    init_cgroups();
    //nix::libc::unshare(nix::libc::CLONE_NEWNET);
    init_prlimit().unwrap();
    init_setuid(exec_request).unwrap();
    init_scmp();
    
    fn init_cgroups()
    {
        //let nanoid_gen = nanoid!(10);
        //let cgroup_id = nanoid_gen.as_str();
        //let hierarchy = cgroups_rs::hierarchies::auto();
        //let cg_builder = CgroupBuilder::new(&cgroup_id)
        //    .devices()
        //        // https://www.kernel.org/doc/html/v4.11/admin-guide/devices.html
        //        .done();
    }
    
    unsafe fn init_prlimit() -> Result<(), std::io::Error>
    {
        let rlimit_core = nix::libc::rlimit64{ rlim_cur: 0, rlim_max: 0 };
        let rlimit_nproc = nix::libc::rlimit64{ rlim_cur: 0, rlim_max: 0 /* current process */ + 0 /* executed process */ };
        
        // RLIMIT_CORE
        if nix::libc::setrlimit64(nix::libc::RLIMIT_CORE, &rlimit_core) != 0
        { return Err(Error::new(ErrorKind::Other, "System call SETRLIMIT64 failed!")); }
        
        // RLIMIT_NPROC
        if nix::libc::setrlimit64(nix::libc::RLIMIT_NPROC, &rlimit_nproc) != 0
        { return Err(Error::new(ErrorKind::Other, "System call SETRLIMIT64 failed!")); }
        
        Ok(())
    }
    
    fn init_scmp()
    {
        /* We use SYSCALLZ crate as the lbseccomp binding: https://docs.rs/syscallz/ */
        
        // We need to add all unwanted syscalls to this list
        let disallowed_syscalls = [
            syscallz::Syscall::fork, syscallz::Syscall::vfork, syscallz::Syscall::clone, syscallz::Syscall::clone3, // process fork & clone
            syscallz::Syscall::reboot, syscallz::Syscall::setuid, syscallz::Syscall::setgid, syscallz::Syscall::setrlimit,
            //syscallz::Syscall::getrlimit, syscallz::Syscall::prlimit64, // resource limiting
            syscallz::Syscall::chmod, syscallz::Syscall::fchmod, syscallz::Syscall::fchmodat, // chmod
            syscallz::Syscall::chown, syscallz::Syscall::fchown, syscallz::Syscall::lchown, syscallz::Syscall::fchownat // chown
        ];
        // Initialize a new SECCOMP context
        let mut ctx = syscallz::Context::init_with_action(syscallz::Action::Allow).unwrap();
        // Add restriction rules to the created context
        for sys_call in disallowed_syscalls {
            ctx.set_action_for_syscall(syscallz::Action::KillProcess, sys_call).unwrap();
        }
        // Enforce the SECCOMP policy we built
        ctx.load().unwrap();
    }
    
    unsafe fn init_setuid(exec_request: &ProcExecRequestInt) -> Result<(), std::io::Error>
    {
        if (*exec_request).runas_user.is_empty() { return Ok(()) }

        let usrname = CString::new((*exec_request).runas_user.as_str()).unwrap();
        
        // Get PASSWD information about the user behind the username
        let pwnam = nix::libc::getpwnam(usrname.as_ptr());
        
        // Try to execute SETUID system call on the current process
        if nix::libc::setuid((*pwnam).pw_uid) != 0
        { return Err(Error::new(ErrorKind::Other, "System call SETUID failed!")); }
        
        //nix::libc::setgid((*pwnam).pw_gid);
        
        Ok(())
    }
    
    Ok(())
}