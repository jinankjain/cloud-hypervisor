// Copyright © 2019 Intel Corporation
//
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause
//
// Copyright © 2020, Microsoft Corporation
//
// Copyright 2018-2019 CrowdStrike, Inc.
//
//

//! A generic abstraction around hypervisor functionality
//!
//! This crate offers a trait abstraction for underlying hypervisors
//!
//! # Platform support
//!
//! - x86_64
//! - arm64
//!

#[macro_use]
extern crate anyhow;
#[allow(unused_imports)]
#[macro_use]
extern crate log;

/// Architecture specific definitions
#[macro_use]
pub mod arch;

#[cfg(feature = "kvm")]
/// KVM implementation module
pub mod kvm;

/// Microsoft Hypervisor implementation module
#[cfg(all(feature = "mshv", target_arch = "x86_64"))]
pub mod mshv;

/// Hypervisor related module
mod hypervisor;

/// Vm related module
mod vm;

/// CPU related module
mod cpu;

/// Device related module
mod device;

pub use crate::hypervisor::{Hypervisor, HypervisorError};
#[cfg(target_arch = "x86_64")]
pub use cpu::CpuVendor;
pub use cpu::{HypervisorCpuError, Vcpu, VmExit};
pub use device::HypervisorDeviceError;
#[cfg(all(feature = "kvm", target_arch = "aarch64"))]
pub use kvm::{aarch64, GicState};
use std::sync::Arc;
pub use vm::{
    DataMatch, HypervisorVmError, InterruptSourceConfig, LegacyIrqSourceConfig, MsiIrqSourceConfig,
    Vm, VmOps,
};

#[derive(Debug, Copy, Clone)]
pub enum HypervisorType {
    #[cfg(feature = "kvm")]
    Kvm,
    #[cfg(feature = "mshv")]
    Mshv,
}

pub fn new() -> std::result::Result<Arc<dyn Hypervisor>, HypervisorError> {
    #[cfg(feature = "kvm")]
    if kvm::KvmHypervisor::is_available()? {
        return kvm::KvmHypervisor::new();
    }

    #[cfg(feature = "mshv")]
    if mshv::MshvHypervisor::is_available()? {
        return mshv::MshvHypervisor::new();
    }

    Err(HypervisorError::HypervisorCreate(anyhow!(
        "no supported hypervisor"
    )))
}

// Returns a `Vec<T>` with a size in bytes at least as large as `size_in_bytes`.
fn vec_with_size_in_bytes<T: Default>(size_in_bytes: usize) -> Vec<T> {
    let rounded_size = (size_in_bytes + size_of::<T>() - 1) / size_of::<T>();
    let mut v = Vec::with_capacity(rounded_size);
    v.resize_with(rounded_size, T::default);
    v
}

// The kvm API has many structs that resemble the following `Foo` structure:
//
// ```
// #[repr(C)]
// struct Foo {
//    some_data: u32
//    entries: __IncompleteArrayField<__u32>,
// }
// ```
//
// In order to allocate such a structure, `size_of::<Foo>()` would be too small because it would not
// include any space for `entries`. To make the allocation large enough while still being aligned
// for `Foo`, a `Vec<Foo>` is created. Only the first element of `Vec<Foo>` would actually be used
// as a `Foo`. The remaining memory in the `Vec<Foo>` is for `entries`, which must be contiguous
// with `Foo`. This function is used to make the `Vec<Foo>` with enough space for `count` entries.
use std::mem::size_of;
pub fn vec_with_array_field<T: Default, F>(count: usize) -> Vec<T> {
    let element_space = count * size_of::<F>();
    let vec_size_bytes = size_of::<T>() + element_space;
    vec_with_size_in_bytes(vec_size_bytes)
}

///
/// User memory region structure
///
#[derive(Debug, Default, Eq, PartialEq)]
pub struct UserMemoryRegion {
    pub slot: u32,
    pub guest_phys_addr: u64,
    pub memory_size: u64,
    pub userspace_addr: u64,
    pub flags: u32,
}

///
/// Flags for user memory region
///
pub const USER_MEMORY_REGION_READ: u32 = 1;
pub const USER_MEMORY_REGION_WRITE: u32 = 1 << 1;
pub const USER_MEMORY_REGION_EXECUTE: u32 = 1 << 2;
pub const USER_MEMORY_REGION_LOG_DIRTY: u32 = 1 << 3;
pub const USER_MEMORY_REGION_ADJUSTABLE: u32 = 1 << 4;

#[derive(Debug)]
pub enum MpState {
    #[cfg(feature = "kvm")]
    Kvm(kvm_bindings::kvm_mp_state),
    #[cfg(all(feature = "mshv", target_arch = "x86_64"))]
    Mshv, /* MSHV does not support MpState yet */
}

#[derive(Debug, Clone, Copy)]
pub enum IoEventAddress {
    Pio(u64),
    Mmio(u64),
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[allow(clippy::large_enum_variant)]
pub enum CpuState {
    #[cfg(feature = "kvm")]
    Kvm(kvm::VcpuKvmState),
    #[cfg(all(feature = "mshv", target_arch = "x86_64"))]
    Mshv(mshv::VcpuMshvState),
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[cfg(target_arch = "x86_64")]
pub enum ClockData {
    #[cfg(feature = "kvm")]
    Kvm(kvm_bindings::kvm_clock_data),
    #[cfg(feature = "mshv")]
    Mshv(mshv::MshvClockData),
}

#[cfg(target_arch = "x86_64")]
impl ClockData {
    pub fn reset_flags(&mut self) {
        match self {
            #[cfg(feature = "kvm")]
            ClockData::Kvm(s) => s.flags = 0,
            #[allow(unreachable_patterns)]
            _ => {}
        }
    }
}

#[derive(Copy, Clone)]
pub enum IrqRoutingEntry {
    #[cfg(feature = "kvm")]
    Kvm(kvm_bindings::kvm_irq_routing_entry),
    #[cfg(feature = "mshv")]
    Mshv(mshv_bindings::mshv_msi_routing_entry),
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum StandardRegisters {
    #[cfg(feature = "kvm")]
    Kvm(kvm_bindings::kvm_regs),
    #[cfg(all(feature = "mshv", target_arch = "x86_64"))]
    Mshv(mshv_bindings::StandardRegisters),
}

impl StandardRegisters {
    #[cfg(feature = "kvm")]
    fn kvm(self) -> kvm_bindings::kvm_regs {
        if let StandardRegisters::Kvm(s) = self {
            s
        } else {
            panic!("Unwrapping kvm_regs failed!")
        }
    }

    #[cfg(feature = "kvm")]
    pub fn get_default_kvm() -> Self {
        let kvm_regs = kvm_bindings::kvm_regs::default();
        StandardRegisters::Kvm(kvm_regs)
    }

    #[cfg(all(feature = "mshv", target_arch = "x86_64"))]
    pub fn get_default_mshv() -> Self {
        let mshv_regs = mshv_bindings::StandardRegisters::default();
        StandardRegisters::Mshv(mshv_regs)
    }

    #[cfg(all(feature = "mshv", target_arch = "x86_64"))]
    fn mshv(self) -> mshv_bindings::StandardRegisters {
        if let StandardRegisters::Mshv(s) = self {
            s
        } else {
            panic!("Unwrapping mshv standard register failed!")
        }
    }
}

macro_rules! set_x86_64_reg {
    ($op:ident, $reg_name:ident) => {
        #[cfg(target_arch = "x86_64")]
        impl StandardRegisters {
            pub fn $op(&mut self, val: u64) {
                match self {
                    #[cfg(feature = "kvm")]
                    StandardRegisters::Kvm(s) => s.$reg_name = val,
                    #[cfg(feature = "mshv")]
                    StandardRegisters::Mshv(s) => s.$reg_name = val,
                }
            }
        }
    };
}

macro_rules! get_x86_64_reg {
    ($op:ident, $reg_name:ident) => {
        #[cfg(target_arch = "x86_64")]
        impl StandardRegisters {
            pub fn $op(&self) -> u64 {
                match self {
                    #[cfg(feature = "kvm")]
                    StandardRegisters::Kvm(s) => s.$reg_name,
                    #[cfg(feature = "mshv")]
                    StandardRegisters::Mshv(s) => s.$reg_name,
                }
            }
        }
    };
}

get_x86_64_reg!(get_rax, rax);
get_x86_64_reg!(get_rbx, rbx);
get_x86_64_reg!(get_rcx, rcx);
get_x86_64_reg!(get_rdx, rdx);
get_x86_64_reg!(get_rsi, rsi);
get_x86_64_reg!(get_rdi, rdi);
get_x86_64_reg!(get_rsp, rsp);
get_x86_64_reg!(get_rbp, rbp);
get_x86_64_reg!(get_r8, r8);
get_x86_64_reg!(get_r9, r9);
get_x86_64_reg!(get_r10, r10);
get_x86_64_reg!(get_r11, r11);
get_x86_64_reg!(get_r12, r12);
get_x86_64_reg!(get_r13, r13);
get_x86_64_reg!(get_r14, r14);
get_x86_64_reg!(get_r15, r15);
get_x86_64_reg!(get_rip, rip);
get_x86_64_reg!(get_rflags, rflags);

set_x86_64_reg!(set_rax, rax);
set_x86_64_reg!(set_rbx, rbx);
set_x86_64_reg!(set_rcx, rcx);
set_x86_64_reg!(set_rdx, rdx);
set_x86_64_reg!(set_rsi, rsi);
set_x86_64_reg!(set_rdi, rdi);
set_x86_64_reg!(set_rsp, rsp);
set_x86_64_reg!(set_rbp, rbp);
set_x86_64_reg!(set_r8, r8);
set_x86_64_reg!(set_r9, r9);
set_x86_64_reg!(set_r10, r10);
set_x86_64_reg!(set_r11, r11);
set_x86_64_reg!(set_r12, r12);
set_x86_64_reg!(set_r13, r13);
set_x86_64_reg!(set_r14, r14);
set_x86_64_reg!(set_r15, r15);
set_x86_64_reg!(set_rip, rip);
set_x86_64_reg!(set_rflags, rflags);
