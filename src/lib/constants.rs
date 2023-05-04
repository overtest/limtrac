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

use libc::c_int;

/// cbindgen:ignore
pub const SYS_EXEC_FAILED : c_int = -1;
/// cbindgen:ignore
pub const SYS_EXEC_OK: c_int = 0;
/// cbindgen:ignore
pub const TIME_MULTIPLIER : c_int = 1000;

/*
 * Child process kill reasons, used to fill the
 * `kill_reason` field of `ProcExecResult` struct.
 */

pub const KILL_REASON_UNSET: c_int = SYS_EXEC_FAILED;
pub const KILL_REASON_NONE: c_int = 0;
pub const KILL_REASON_SECURITY : c_int = 1;
pub const KILL_REASON_REALTIME : c_int = 2;
pub const KILL_REASON_PROCTIME : c_int = 3;
pub const KILL_REASON_PROCWSET : c_int = 4;