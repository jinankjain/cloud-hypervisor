// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause
//
// Copyright © 2025, Microsoft Corporation
//
use std::fmt;

use serde::{Deserialize, Serialize};

///
/// Export generically-named wrappers of mshv_bindings for Unix-based platforms
///
pub use mshv_bindings::StandardRegisters as MshvStandardRegisters;

#[derive(Clone, Serialize, Deserialize)]
pub struct VcpuMshvState {
    pub regs: MshvStandardRegisters,
}

impl fmt::Display for VcpuMshvState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Standard registers: {:?}", self.regs)
    }
}
