/*
 * LIMTRAC, a part of Overtest free software project.
 * Copyright (C) 2021-2023, Yurii Kadirov <contact@sirkadirov.com>
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

use libc::{c_int, c_ulonglong};
use crate::constants::{TIME_MULTIPLIER, KILL_REASON_UNSET};

#[repr(C)]
pub struct ProcExecResult
{
    pub exit_code: c_int,
    pub exit_sign: c_int,
    pub is_killed: bool,
    pub kill_reason : c_int,
    pub res_usage: ProcResUsage
}

impl ProcExecResult {
    pub(crate) fn new() -> Self
    {
        Self {
            exit_code: -1,
            exit_sign: -1,
            res_usage: ProcResUsage::new(),
            is_killed: false,
            kill_reason: KILL_REASON_UNSET
        }
    }
}

#[repr(C)]
pub struct ProcResUsage
{
    pub real_time : c_ulonglong,
    pub proc_time : c_ulonglong,
    pub proc_wset : c_ulonglong
}

impl ProcResUsage {
    pub(crate) fn new () -> Self
    {
        Self
        {
            real_time : 0,
            proc_time : 0,
            proc_wset : 0
        }
    }

    pub fn load_rusage(&mut self, res_usage: &libc::rusage)
    {
        // Processor time usage is a sum of user-space time and kernel time consumed by a process
        let proc_time = timeval_to_ms(res_usage.ru_utime) + timeval_to_ms(res_usage.ru_stime);
        if proc_time > self.proc_time { self.proc_time = proc_time; }

        // On Windows, this called PeakWorkingSet, on Linux - MaxResidentSetSize
        let proc_wset = res_usage.ru_maxrss as c_ulonglong * 1024;
        if proc_wset > self.proc_wset { self.proc_wset = proc_wset; }

        /* @Function that converts values present in `timeval` structure into milliseconds value */
        fn timeval_to_ms(val: libc::timeval) -> c_ulonglong
        {
            (val.tv_sec as c_ulonglong  * TIME_MULTIPLIER as c_ulonglong) +
                (val.tv_usec as c_ulonglong / TIME_MULTIPLIER as c_ulonglong)
        }
        /* @/Function that converts values present in `timeval` structure into milliseconds value */
    }

    pub fn load_proc_stat(&mut self, child_pid: libc::pid_t) -> Result<(), ()>
    {
        let child_proc = match procfs::process::Process::new(child_pid)
        { Ok(p) => p, Err(_) => return Err(())};

        let child_proc_stat = match child_proc.stat()
        { Ok(s) => s, Err(_) => return Err(())};

        let ticks_per_second = procfs::ticks_per_second();

        // Load data from process stat file
        self.proc_time = (child_proc_stat.utime + child_proc_stat.stime) as c_ulonglong +
            (child_proc_stat.cutime + child_proc_stat.cstime) as c_ulonglong;
        self.proc_time = ((self.proc_time as libc::c_double / ticks_per_second as libc::c_double)
            * 1000 as libc::c_double) as c_ulonglong;

        let child_proc_status : procfs::process::Status = match child_proc.status()
        { Ok(s) => s, Err(_) => return Err(())};

        //self.proc_wset = child_proc_status.vmhwm.unwrap() * 1024 as c_ulonglong;
        self.proc_wset = child_proc_status.vmpeak.unwrap_or_else(
            || child_proc_status.vmhwm.unwrap()) * 1024 as c_ulonglong;

        Ok(())
    }
}