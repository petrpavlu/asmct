// Copyright (C) 2023 Petr Pavlu <petr.pavlu@dagobah.cz>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::fmt::Write;

/// GNU disassembler.
pub struct GNU {
    dinfo: disassemble_info,
    disasm: disassembler_ftype,
}

/// Variable argument type.
enum VarArgType {
    Char,
    Int,
    LongLong,
    PointerToChar,
}

/// Variable argument value.
enum VarArgValue {
    Char(std::os::raw::c_char),
    Int(std::os::raw::c_int),
    LongLong(std::os::raw::c_longlong),
    PointerToChar(*const std::os::raw::c_char),
}

/// Minimal ad-hoc fprintf() implementation.
fn fprintf_out(
    stream: *mut std::os::raw::c_void,
    format: *const std::os::raw::c_char,
    mut next_arg: impl FnMut(VarArgType) -> VarArgValue,
) -> std::os::raw::c_int {
    let stream = unsafe { &mut *(stream as *mut String) };
    let mut printed: std::os::raw::c_int = 0;

    let format = unsafe { std::ffi::CStr::from_ptr(format) };
    let format = String::from_utf8_lossy(format.to_bytes());
    let format: Vec<char> = format.chars().collect();

    let mut i = 0;
    while i < format.len() {
        if format[i] != '%' {
            write!(stream, "{}", format[i]).expect("Write to stream");

            printed += 1;
            i += 1;
            continue;
        }

        i += 1;
        assert!(i < format.len());
        if format[i] == 'c' {
            let char_ = match next_arg(VarArgType::Char) {
                VarArgValue::Char(char_) => char_,
                _ => unreachable!(),
            };
            let char_ = unsafe { char::from_u32_unchecked(char_ as u32) };
            write!(stream, "{}", char_).expect("Write to stream");

            printed += 1;
            i += 1;
        } else if format[i] == 'd' || format[i] == 'x' {
            let int = match next_arg(VarArgType::Int) {
                VarArgValue::Int(int) => int,
                _ => unreachable!(),
            };
            let string: String;
            if format[i] == 'd' {
                string = format!("{}", int);
            } else {
                string = format!("{:x}", int);
            }
            write!(stream, "{}", string).expect("Write to stream");

            printed += string.chars().count() as std::os::raw::c_int;
            i += 1;
        } else if format[i] == 's' {
            let string = match next_arg(VarArgType::PointerToChar) {
                VarArgValue::PointerToChar(string) => string,
                _ => unreachable!(),
            };
            let string = unsafe { std::ffi::CStr::from_ptr(string) };
            let string = String::from_utf8_lossy(string.to_bytes());
            write!(stream, "{}", string).expect("Write to stream");

            printed += string.chars().count() as std::os::raw::c_int;
            i += 1;
        } else if format[i..].starts_with(&['l', 'l', 'x']) {
            let int = match next_arg(VarArgType::LongLong) {
                VarArgValue::LongLong(int) => int,
                _ => unreachable!(),
            };
            let string = format!("{:x}", int);
            write!(stream, "{}", string).expect("Write to stream");

            printed += string.chars().count() as std::os::raw::c_int;
            i += 3;
        } else {
            panic!("unhandled conversion specifier '%{}...'", format[i]);
        }
    }

    printed
}

/// fprintf() callback for the disassembler.
unsafe extern "C" fn fprintf_callback(
    stream: *mut std::os::raw::c_void,
    format: *const std::os::raw::c_char,
    mut args: ...
) -> std::os::raw::c_int {
    fprintf_out(stream, format, |type_: VarArgType| -> VarArgValue {
        match type_ {
            VarArgType::Char => VarArgValue::Char(args.arg::<std::os::raw::c_char>()),
            VarArgType::Int => VarArgValue::Int(args.arg::<std::os::raw::c_int>()),
            VarArgType::LongLong => VarArgValue::LongLong(args.arg::<std::os::raw::c_longlong>()),
            VarArgType::PointerToChar => {
                VarArgValue::PointerToChar(args.arg::<*const std::os::raw::c_char>())
            }
        }
    })
}

/// "Styled" fprintf() callback for the disassembler.
unsafe extern "C" fn fprintf_styled_callback(
    stream: *mut std::os::raw::c_void,
    _style: disassembler_style,
    format: *const std::os::raw::c_char,
    mut args: ...
) -> std::os::raw::c_int {
    fprintf_out(stream, format, |type_: VarArgType| -> VarArgValue {
        match type_ {
            VarArgType::Char => VarArgValue::Char(args.arg::<std::os::raw::c_char>()),
            VarArgType::Int => VarArgValue::Int(args.arg::<std::os::raw::c_int>()),
            VarArgType::LongLong => VarArgValue::LongLong(args.arg::<std::os::raw::c_longlong>()),
            VarArgType::PointerToChar => {
                VarArgValue::PointerToChar(args.arg::<*const std::os::raw::c_char>())
            }
        }
    })
}

impl GNU {
    /// Creates a new instance of the GNU disassembler.
    pub fn new() -> Result<Self, crate::Error> {
        let mut gnu = GNU {
            dinfo: unsafe { std::mem::zeroed() },
            disasm: None,
        };

        unsafe {
            init_disassemble_info(
                &mut gnu.dinfo,
                std::ptr::null_mut(),
                Some(fprintf_callback),
                Some(fprintf_styled_callback),
            )
        };

        gnu.dinfo.arch = bfd_architecture_bfd_arch_riscv;
        gnu.dinfo.mach = bfd_mach_riscv64 as u64;
        //gnu.dinfo.disassembler_options = "".as_bytes().as_ptr() as *const i8;
        gnu.dinfo.read_memory_func = Some(buffer_read_memory);
        gnu.dinfo.buffer_vma = 0;

        unsafe { disassemble_init_for_target(&mut gnu.dinfo) };
        gnu.disasm = unsafe {
            disassembler(
                bfd_architecture_bfd_arch_riscv,
                false,
                bfd_mach_riscv64 as u64,
                std::ptr::null_mut(),
            )
        };
        if gnu.disasm.is_none() {
            unsafe { disassemble_free_target(&mut gnu.dinfo) }
            return Err(crate::Error::new("Failed to initialize GNU disassembler"));
        }

        Ok(gnu)
    }

    /// Disassembles one instruction.
    pub fn disassemble(&mut self, bytes: &[u8]) -> String {
        let mut res = String::new();
        self.dinfo.stream = &mut res as *mut String as *mut std::os::raw::c_void;

        let mut bytes_vec = bytes.to_owned();
        self.dinfo.buffer = bytes_vec.as_mut_ptr();
        self.dinfo.buffer_length = bytes_vec.len() as u64;

        _ = unsafe { self.disasm.unwrap()(0, &mut self.dinfo) };

        res
    }
}

impl Drop for GNU {
    /// Frees resources associated with the GNU disassembler.
    fn drop(&mut self) {
        unsafe { disassemble_free_target(&mut self.dinfo) };
    }
}
