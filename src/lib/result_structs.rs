/*
 * LIMTRAC, a part of Overtest free software project.
 * Copyright (C) 2021-2022, Yurii Kadirov <contact@sirkadirov.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Lesser General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use libc::{c_char, c_int, c_long, c_ulonglong};
use crate::constants::TIME_MULTIPLIER;

#[repr(C)]
pub struct ProcExecResult
{
    pub exit_code : c_int,
    pub exit_sign : c_int,
    pub res_usage : *mut ProcResUsage,

    pub is_killed : bool,
    pub kill_reason : *const c_char
}

impl ProcExecResult {
    pub fn new() -> Self
    {
        Self {
            exit_code: -1,
            exit_sign: -1,
            res_usage: std::ptr::null_mut(),
            is_killed: false,
            kill_reason: std::ptr::null()
        }
    }
}

#[repr(C)]
pub struct ProcResUsage
{
    pub real_time : c_ulonglong,
    pub proc_time : c_ulonglong,
    pub proc_wset : c_long
}

impl ProcResUsage {
    pub fn new () -> Self
    {
        Self
        {
            real_time : 0,
            proc_time : 0,
            proc_wset : 0
        }
    }

    pub fn load_rusage(&mut self, res_usage_ptr: *mut libc::rusage)
    {
        // Try to dereference the pointer to `rusage` struct
        let res_usage = crate::helper_functions::get_obj_from_ptr(res_usage_ptr, "res_usage");

        // Processor time usage is a sum of user-space time and kernel time consumed by a process
        self.proc_time = timeval_to_ms(res_usage.ru_utime) + timeval_to_ms(res_usage.ru_stime);
        // On Windows, this called PeakWorkingSet, on Linux - MaxResidentSetSize
        self.proc_wset = res_usage.ru_maxrss;

        /* @Function that converts values present in `timeval` structure into milliseconds value */
        fn timeval_to_ms(val: libc::timeval) -> c_ulonglong
        {
            return (val.tv_sec as c_ulonglong  * TIME_MULTIPLIER as c_ulonglong)
                + (val.tv_usec as c_ulonglong / TIME_MULTIPLIER as c_ulonglong);
        }
        /* @/Function that converts values present in `timeval` structure into milliseconds value */
    }
}