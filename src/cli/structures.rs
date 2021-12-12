use std::{env, str, fmt};
use crate::constants;

pub struct ProcExecRequest
{
    pub full_path : String,
    pub arguments : String,
    //pub work_dir : String,
    pub runas_user : String,

    pub rlimit_enabled : bool,
    pub rlimit_core : i32,
    pub rlimit_nproc : i32,
    pub rlimit_nofile : i32,

    pub scmp_enabled : bool,
    pub scmp_fs_guard : bool
}

impl ProcExecRequest {
    pub fn new() -> Self
    {
        fn load_env_var<T: str::FromStr>(var_name : &str, default_value : T) -> T where <T as str::FromStr>::Err: fmt::Debug
        {
            return match env::var(var_name)
            {
                Ok(lim) => lim.parse().unwrap(),
                _ => default_value
            };
        }

        ProcExecRequest
        {
            full_path : env::var("LIMTRAC_FULLPATH").expect("LIMTRAC_FULLPATH not set!"),
            arguments : env::var("LIMTRAC_ARGUMENTS").expect("LIMTRAC_ARGUMENTS not set!"),
            runas_user : env::var("LIMTRAC_RUNAS").expect("LIMTRAC_RUNAS not set!"),

            rlimit_enabled : load_env_var("LIMTRAC_RLIM_ENABLED", constants::DEFAULT_RLIM_ENABLED),
            rlimit_core : load_env_var("LIMTRAC_RLIM_CORE", constants::DEFAULT_RLIM_CORE),
            rlimit_nproc : load_env_var("LIMTRAC_RLIM_NPROC", constants::DEFAULT_RLIM_NPROC),
            rlimit_nofile : load_env_var("LIMTRAC_RLIM_NOFILE", constants::DEFAULT_RLIM_NOFILE),
            scmp_enabled: load_env_var("LIMTRAC_SCMP_ENABLED", constants::DEFAULT_SCMP_ENABLED),
            scmp_fs_guard: load_env_var("LIMTRAC_SCMP_FS_GUARD", constants::DEFAULT_SCMP_FS_GUARD)
        }
    }
}