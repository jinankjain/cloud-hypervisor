// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause
//
// Copyright © 2024, Microsoft Corporation

use crate::arch::aarch64::gic::{Result, Vgic, VgicConfig};
use crate::mshv::MshvVm;
use crate::CpuState;
use crate::GicState;
use crate::Vm;
use std::any::Any;

pub struct MshvGicV3Its {
    /// GIC distributor address
    dist_addr: u64,

    /// GIC distributor size
    dist_size: u64,

    /// GIC distributors address
    redists_addr: u64,

    /// GIC distributors size
    redists_size: u64,

    /// GIC MSI address
    msi_addr: u64,

    /// GIC MSI size
    msi_size: u64,

    /// Number of CPUs handled by the device
    vcpu_count: u64,
}

impl MshvGicV3Its {
    /// Device trees specific constants
    pub const ARCH_GIC_V3_MAINT_IRQ: u32 = 9;

    pub fn new(vm: &dyn Vm, config: VgicConfig) -> Result<MshvGicV3Its> {
        let vm = vm
            .as_any()
            .downcast_ref::<MshvVm>()
            .expect("Wrong VM type?");

        let gic_device = MshvGicV3Its {
            dist_addr: config.dist_addr,
            dist_size: config.dist_size,
            redists_addr: config.redists_addr,
            redists_size: config.redists_size,
            msi_addr: config.msi_addr,
            msi_size: config.msi_size,
            vcpu_count: config.vcpu_count,
        };

        Ok(gic_device)
    }
}

impl Vgic for MshvGicV3Its {
    fn fdt_compatibility(&self) -> &str {
        "arm,gic-v3"
    }

    fn msi_compatible(&self) -> bool {
        true
    }

    fn msi_compatibility(&self) -> &str {
        "arm,gic-v2m-frame"
    }

    fn fdt_maint_irq(&self) -> u32 {
        MshvGicV3Its::ARCH_GIC_V3_MAINT_IRQ
    }

    fn vcpu_count(&self) -> u64 {
        self.vcpu_count
    }

    fn device_properties(&self) -> [u64; 4] {
        [
            self.dist_addr,
            self.dist_size,
            self.redists_addr,
            self.redists_size,
        ]
    }

    fn msi_properties(&self) -> [u64; 2] {
        [self.msi_addr, self.msi_size]
    }

    fn set_gicr_typers(&mut self, _vcpu_states: &[CpuState]) {
        unimplemented!();
    }

    fn as_any_concrete_mut(&mut self) -> &mut dyn Any {
        self
    }

    /// Save the state of GICv3ITS.
    fn state(&self) -> Result<GicState> {
        unimplemented!();
    }

    /// Restore the state of GICv3ITS.
    fn set_state(&mut self, _state: &GicState) -> Result<()> {
        unimplemented!();
    }

    /// Saves GIC internal data tables into RAM, including:
    /// - RDIST pending tables
    /// - ITS tables into guest RAM.
    fn save_data_tables(&self) -> Result<()> {
        // Flash RDIST pending tables
        unimplemented!();
    }
}
