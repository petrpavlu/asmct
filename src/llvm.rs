// Copyright (C) 2023 Petr Pavlu <petr.pavlu@dagobah.cz>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/// LLVM disassembler.
pub struct LLVM {
    disasm: LLVMDisasmContextRef,
}

impl LLVM {
    /// Creates a new instance of the LLVM disassembler.
    pub fn new() -> Result<Self, crate::Error> {
        let mut llvm = LLVM {
            disasm: std::ptr::null_mut(),
        };

        unsafe {
            LLVM_InitializeAllTargetInfos();
            LLVM_InitializeAllTargetMCs();
            LLVM_InitializeAllDisassemblers();
        }

        llvm.disasm = unsafe {
            LLVMCreateDisasmCPUFeatures(
                "riscv64\0".as_ptr() as *const i8,
                "\0".as_ptr() as *const i8,
                "+m,+a,+f,+d,+c\0".as_ptr() as *const i8,
                std::ptr::null_mut(),
                0,
                None,
                None,
            )
        };
        if llvm.disasm == std::ptr::null_mut() {
            return Err(crate::Error::new("Failed to initialize LLVM disassembler"));
        }

        Ok(llvm)
    }

    /// Disassembles one instruction.
    pub fn disassemble(&mut self, bytes: &[u8]) -> String {
        let mut bytes_vec = bytes.to_owned();
        let pc = 0;
        let mut output = [0u8; 256];
        let insn_len = unsafe {
            LLVMDisasmInstruction(
                self.disasm,
                bytes_vec.as_mut_ptr(),
                bytes_vec.len() as u64,
                pc,
                output.as_mut_ptr() as *mut i8,
                output.len() as u64,
            )
        };
        if insn_len == 0 {
            return "\tunimp".to_string();
        }

        let output_len = output.iter().position(|&x| x == 0).unwrap_or(0);
        String::from_utf8_lossy(&output[0..output_len]).into_owned()
    }
}

impl Drop for LLVM {
    /// Frees resources associated with the LLVM disassembler.
    fn drop(&mut self) {
        unsafe {
            LLVMDisasmDispose(self.disasm);
        }
    }
}
