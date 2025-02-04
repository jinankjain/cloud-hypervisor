use serde::{Deserialize, Serialize};

pub mod emulator;

///
/// Export generically-named wrappers of mshv_bindings for Unix-based platforms
///
pub use mshv_bindings::StandardRegisters as MshvStandardRegisters;

#[derive(Clone, Serialize, Deserialize)]
pub struct VcpuMshvState {
    pub regs: MshvStandardRegisters,
}
