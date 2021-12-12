mod constants;
mod structures;
mod helpers;

use std::ffi::CString;
use std::io::{Error, ErrorKind};
use nix::errno::errno;
use syscallz::Syscall;
use crate::structures::ProcExecRequest;

fn main() {
    // Load execution request details from environment variables
    let exec_request = ProcExecRequest::new();
    // Clean environment variables used by limtrac
    helpers::clean_environment_variables("LIMTRAC_");
    // Execute process in secure environment
    execute_process(exec_request);
}

fn execute_process(exec_request: ProcExecRequest)
{
    // Set up resource limiting features
    prepare_environment(&exec_request).unwrap();
    unsafe {
        // Prepare program execution parameters
        let program_fullpath = CString::new(exec_request.full_path.as_str()).unwrap();
        // Argv always must be specified and must contain at least one element (name of the executable)
        let program_argv = helpers::str_explode_to_native_list(exec_request.arguments.as_str());
        // Execute requested program
        if nix::libc::execv(program_fullpath.as_ptr(), program_argv) == -1
        {
            panic!("Program execution failed with ERRNO {}! Refer to <errno.h> to troubleshoot this issue.", errno());
            // https://android.googlesource.com/kernel/lk/+/dima/for-travis/include/errno.h
        }
    }
}

fn prepare_environment(exec_request: &ProcExecRequest) -> Result<(), std::io::Error>
{
    //init_cgroups();
    unsafe { // These functions use unsafe system calls
        init_prlimit(exec_request).unwrap();
        init_setuid(exec_request).unwrap();
    }
    init_scmp(exec_request).unwrap();

    //fn init_cgroups()
    //{
    //    let nanoid_gen = constants::DEFAULT_CGROUP_NAME;
    //    let cgroup_id = nanoid_gen.as_str();
    //    let hierarchy = cgroups_rs::hierarchies::auto();
    //    let cg_builder = CgroupBuilder::new(&cgroup_id)
    //        .devices()
    //            // https://www.kernel.org/doc/html/v4.11/admin-guide/devices.html
    //            .done();
    //}
    
    unsafe fn init_prlimit(exec_request: &ProcExecRequest) -> Result<(), std::io::Error>
    {
        if !((*exec_request).rlimit_enabled) { return Ok(()); }

        /*
         * RLIMIT_CORE
         */
        let rlimit_core = nix::libc::rlimit64
        {
            rlim_cur: (*exec_request).rlimit_core as nix::libc::rlim64_t,
            rlim_max: (*exec_request).rlimit_core as nix::libc::rlim64_t
        };
        if nix::libc::setrlimit64(nix::libc::RLIMIT_CORE, &rlimit_core) != 0
        { return Err(Error::new(ErrorKind::Other, "System call SETRLIMIT64 failed (RLIMIT_CORE)!")); }

        /*
         * RLIMIT_NPROC
         */
        let rlimit_nproc = nix::libc::rlimit64{
            rlim_cur: (*exec_request).rlimit_nproc as nix::libc::rlim64_t,
            rlim_max: (*exec_request).rlimit_nproc as nix::libc::rlim64_t
        };
        if nix::libc::setrlimit64(nix::libc::RLIMIT_NPROC, &rlimit_nproc) != 0
        { return Err(Error::new(ErrorKind::Other, "System call SETRLIMIT64 failed (RLIMIT_NPROC)!")); }

        /*
         * RLIMIT_NOFILE
         */
        let rlimit_nofile = nix::libc::rlimit64{
            rlim_cur: (*exec_request).rlimit_nofile as nix::libc::rlim64_t,
            rlim_max: (*exec_request).rlimit_nofile as nix::libc::rlim64_t
        };
        if nix::libc::setrlimit64(nix::libc::RLIMIT_NOFILE, &rlimit_nofile) != 0
        { return Err(Error::new(ErrorKind::Other, "System call SETRLIMIT64 failed (RLIMIT_NOFILE)!")); }
        
        Ok(())
    }
    
    fn init_scmp(exec_request: &ProcExecRequest) -> Result<(), std::io::Error>
    {
        if !((*exec_request).scmp_enabled) { return Ok(()); }

        unsafe {
            // Unshare internet connection with the current program
            nix::libc::unshare(nix::libc::CLONE_NEWNET);
        }

        /* We use SYSCALLZ crate as the lbseccomp binding: https://docs.rs/syscallz/ */
        
        // We need to add all unwanted syscalls to this list
        let disallowed_syscalls = [
            Syscall::fork, Syscall::vfork, Syscall::clone, Syscall::clone3, // process fork & clone
            Syscall::reboot, Syscall::setuid, Syscall::setgid, Syscall::setrlimit,
            //Syscall::getrlimit, Syscall::prlimit64, // resource limiting
            Syscall::chmod, Syscall::fchmod, Syscall::fchmodat, // chmod
            Syscall::chown, Syscall::fchown, Syscall::lchown, Syscall::fchownat, // chown
            //Syscall::link, Syscall::unlink
        ];

        // Initialize a new SECCOMP context
        let mut ctx = syscallz::Context::init_with_action(syscallz::Action::Allow).unwrap();

        // Add restriction rules to the created context
        for sys_call in disallowed_syscalls {
            ctx.set_action_for_syscall(syscallz::Action::KillProcess, sys_call).unwrap();
        }

        // Enforce the SECCOMP policy we built
        ctx.load().unwrap();

        Ok(())
    }
    
    unsafe fn init_setuid(exec_request: &ProcExecRequest) -> Result<(), std::io::Error>
    {
        if (*exec_request).runas_user.is_empty() { return Ok(()) }

        let raw_username = CString::new((*exec_request).runas_user.as_str()).unwrap();
        // Get PASSWD information about the user behind the username
        let pwnam = nix::libc::getpwnam(raw_username.as_ptr());

        // Check whether specified user not found
        if pwnam.is_null()
        {
            return Err(Error::new(
                ErrorKind::Other,
                format!("System call GETPWNAM failed! User '{}' was not found!", (*exec_request).runas_user)
            ));
        }
        
        // Try to execute SETUID system call on the current process
        if nix::libc::setuid((*pwnam).pw_uid) != 0
        { return Err(Error::new(ErrorKind::Other, "System call SETUID failed!")); }
        
        //nix::libc::setgid((*pwnam).pw_gid);
        
        Ok(())
    }
    
    Ok(())
}