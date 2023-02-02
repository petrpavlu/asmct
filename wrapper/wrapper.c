// Copyright (C) 2023 Petr Pavlu <petr.pavlu@dagobah.cz>
// SPDX-License-Identifier: GPL-3.0-or-later

#include "wrapper.h"

void LLVM_InitializeAllTargetInfos(void) { LLVMInitializeAllTargetInfos(); }
void LLVM_InitializeAllTargetMCs(void) { LLVMInitializeAllTargetMCs(); }
void LLVM_InitializeAllDisassemblers(void) { LLVMInitializeAllDisassemblers(); }
