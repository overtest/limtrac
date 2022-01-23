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

/*pub fn panic_on_syscall(syscall_name: &str)
{
    panic!("System call '{}' failed with 'ERRNO = {}'!", syscall_name.to_uppercase().as_str(), errno());
}*/

macro_rules! panic_on_syscall {
    ($($syscall_name:tt)*) => {
        panic!("System call '{}' failed with 'ERRNO = {}'!", $($syscall_name)*, nix::errno::errno());
    };
}
pub(crate) use panic_on_syscall;

pub fn get_obj_from_ptr<T>(ptr: *const T, msg: &str) -> &T
{
    match unsafe { ptr.as_ref() } {
        None => { panic!("Couldn't dereference a pointer ({})!", msg); }
        Some(obj) => obj
    }
}