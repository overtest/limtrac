use std::env;
use std::ffi::CString;

pub fn clean_environment_variables(starts_with : &str)
{
    let env_vars = env::vars();
    for (item_name, _) in env_vars
    {
        if !item_name.starts_with(starts_with) { continue; }
        env::remove_var(item_name);
    }
}

pub unsafe fn str_explode_to_native_list(in_str : &str) -> *const *const nix::libc::c_char
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