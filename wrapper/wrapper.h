// Copyright (C) 2023 Petr Pavlu <petr.pavlu@dagobah.cz>
// SPDX-License-Identifier: GPL-3.0-or-later

// GNU binutils
#include <dis-asm.h>

// LLVM
#include <llvm-c/Disassembler.h>
#include <llvm-c/Initialization.h>
#include <llvm-c/Target.h>

void LLVM_InitializeAllTargetInfos(void);
void LLVM_InitializeAllTargetMCs(void);
void LLVM_InitializeAllDisassemblers(void);
