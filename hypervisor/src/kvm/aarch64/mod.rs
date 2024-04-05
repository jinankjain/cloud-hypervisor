// Copyright © 2019 Intel Corporation
//
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause
//
// Copyright © 2020, Microsoft Corporation
//
// Copyright 2018-2019 CrowdStrike, Inc.
//
//

pub mod gic;

use crate::arch::aarch64::{Register, StandardRegisters, VcpuInit};
use crate::kvm::{KvmError, KvmResult};
use kvm_bindings::{
    kvm_mp_state, kvm_one_reg, kvm_regs, KVM_REG_ARM_COPROC_MASK, KVM_REG_ARM_CORE,
    KVM_REG_SIZE_MASK, KVM_REG_SIZE_U32, KVM_REG_SIZE_U64,
};
pub use kvm_bindings::{kvm_vcpu_init, user_fpsimd_state, user_pt_regs, RegList};
use serde::{Deserialize, Serialize};
pub use {kvm_ioctls::Cap, kvm_ioctls::Kvm};

// This macro gets the offset of a structure (i.e `str`) member (i.e `field`) without having
// an instance of that structure.
#[macro_export]
macro_rules! offset_of {
    ($str:ty, $field:ident) => {{
        let tmp: std::mem::MaybeUninit<$str> = std::mem::MaybeUninit::uninit();
        let base = tmp.as_ptr();

        // Avoid warnings when nesting `unsafe` blocks.
        #[allow(unused_unsafe)]
        // SAFETY: The pointer is valid and aligned, just not initialised. Using `addr_of` ensures
        // that we don't actually read from `base` (which would be UB) nor create an intermediate
        // reference.
        let member = unsafe { core::ptr::addr_of!((*base).$field) } as *const u8;

        // Avoid warnings when nesting `unsafe` blocks.
        #[allow(unused_unsafe)]
        // SAFETY: The two pointers are within the same allocated object `tmp`. All requirements
        // from offset_from are upheld.
        unsafe {
            member.offset_from(base as *const u8) as usize
        }
    }};
}

// Following are macros that help with getting the ID of a aarch64 core register.
// The core register are represented by the user_pt_regs structure. Look for it in
// arch/arm64/include/uapi/asm/ptrace.h.

// Get the ID of a core register
#[macro_export]
macro_rules! arm64_core_reg_id {
    ($size: tt, $offset: tt) => {
        // The core registers of an arm64 machine are represented
        // in kernel by the `kvm_regs` structure. This structure is a
        // mix of 32, 64 and 128 bit fields:
        // struct kvm_regs {
        //     struct user_pt_regs      regs;
        //
        //     __u64                    sp_el1;
        //     __u64                    elr_el1;
        //
        //     __u64                    spsr[KVM_NR_SPSR];
        //
        //     struct user_fpsimd_state fp_regs;
        // };
        // struct user_pt_regs {
        //     __u64 regs[31];
        //     __u64 sp;
        //     __u64 pc;
        //     __u64 pstate;
        // };
        // The id of a core register can be obtained like this:
        // offset = id & ~(KVM_REG_ARCH_MASK | KVM_REG_SIZE_MASK | KVM_REG_ARM_CORE). Thus,
        // id = KVM_REG_ARM64 | KVM_REG_SIZE_U64/KVM_REG_SIZE_U32/KVM_REG_SIZE_U128 | KVM_REG_ARM_CORE | offset
        KVM_REG_ARM64 as u64
            | u64::from(KVM_REG_ARM_CORE)
            | $size
            | (($offset / mem::size_of::<u32>()) as u64)
    };
}

/// Specifies whether a particular register is a system register or not.
/// The kernel splits the registers on aarch64 in core registers and system registers.
/// So, below we get the system registers by checking that they are not core registers.
///
/// # Arguments
///
/// * `regid` - The index of the register we are checking.
pub fn is_system_register(regid: u64) -> bool {
    if (regid & KVM_REG_ARM_COPROC_MASK as u64) == KVM_REG_ARM_CORE as u64 {
        return false;
    }

    let size = regid & KVM_REG_SIZE_MASK;

    assert!(
        !(size != KVM_REG_SIZE_U32 && size != KVM_REG_SIZE_U64),
        "Unexpected register size for system register {size}"
    );

    true
}

pub fn check_required_kvm_extensions(kvm: &Kvm) -> KvmResult<()> {
    macro_rules! check_extension {
        ($cap:expr) => {
            if !kvm.check_extension($cap) {
                return Err(KvmError::CapabilityMissing($cap));
            }
        };
    }

    // SetGuestDebug is required but some kernels have it implemented without the capability flag.
    check_extension!(Cap::ImmediateExit);
    check_extension!(Cap::Ioeventfd);
    check_extension!(Cap::Irqchip);
    check_extension!(Cap::Irqfd);
    check_extension!(Cap::IrqRouting);
    check_extension!(Cap::MpState);
    check_extension!(Cap::OneReg);
    check_extension!(Cap::UserMemory);
    Ok(())
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct VcpuKvmState {
    pub mp_state: kvm_mp_state,
    pub core_regs: kvm_regs,
    pub sys_regs: Vec<kvm_one_reg>,
}

impl From<StandardRegisters> for kvm_regs {
    fn from(regs: StandardRegisters) -> Self {
        Self {
            regs: user_pt_regs {
                regs: regs.gpr,
                sp: regs.sp,
                pc: regs.pc,
                pstate: regs.pstate,
            },
            sp_el1: regs.sp_el1,
            elr_el1: regs.elr_el1,
            spsr: regs.spsr,
            fp_regs: user_fpsimd_state {
                vregs: regs.vregs,
                fpsr: regs.fpsr as u32,
                fpcr: regs.fpcr as u32,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

impl From<kvm_regs> for StandardRegisters {
    fn from(regs: kvm_regs) -> Self {
        Self {
            gpr: regs.regs.regs,
            sp: regs.regs.sp,
            pc: regs.regs.pc,
            pstate: regs.regs.pstate,
            sp_el1: regs.sp_el1,
            elr_el1: regs.elr_el1,
            spsr: regs.spsr,
            vregs: regs.fp_regs.vregs,
            fpsr: regs.fp_regs.fpsr as u64,
            fpcr: regs.fp_regs.fpcr as u64,
        }
    }
}

impl From<Register> for kvm_one_reg {
    fn from(reg: Register) -> Self {
        Self {
            id: reg.id,
            addr: reg.addr,
        }
    }
}

impl From<kvm_one_reg> for Register {
    fn from(reg: kvm_one_reg) -> Self {
        Self {
            id: reg.id,
            addr: reg.addr,
        }
    }
}

impl From<VcpuInit> for kvm_vcpu_init {
    fn from(vcpu_init: VcpuInit) -> Self {
        Self {
            target: vcpu_init.target,
            features: vcpu_init.features,
        }
    }
}

impl From<kvm_vcpu_init> for VcpuInit {
    fn from(vcpu_init: kvm_vcpu_init) -> Self {
        Self {
            target: vcpu_init.target,
            features: vcpu_init.features,
        }
    }
}
