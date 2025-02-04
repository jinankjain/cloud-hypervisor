// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause
//
// Copyright © 2024, Microsoft Corporation
//

// use crate::arch::aarch64::regs::{EsrEl2, ExceptionClass, IssDataAbort};
use crate::mshv::MshvVcpu;

pub struct MshvEmulatorContext<'a> {
    pub vcpu: &'a MshvVcpu,
    pub map: (u64, u64), // Initial GVA to GPA mapping provided by the hypervisor
    pub syndrome: u64,
    pub instruction_bytes: [u8; 4],
    pub instruction_byte_count: u8,
    pub interruption_pending: bool,
    pub pc: u64,
}

pub struct Emulator<'a> {
    pub context: MshvEmulatorContext<'a>,
}

// impl<'a> Emulator<'a> {
//     pub fn new(context: MshvEmulatorContext<'a>) -> Self {
//         Emulator { context }
//     }

//     pub fn decode_with_syndrome(&mut self) -> bool {
//         let esr_el2 = EsrEl2::from(self.context.syndrome);
//         if !matches!(
//             ExceptionClass(esr_el2.ec()),
//             ExceptionClass::DATA_ABORT | ExceptionClass::DATA_ABORT_LOWER
//         ) {
//             return false;
//         }

//         let iss = IssDataAbort::from(esr_el2.iss());
//         if !iss.isv() {
//             return false;
//         }
//         let len = 1 << iss.sas();
//         let sign_extend = iss.sse();
//         let reg_index = iss.srt();

//         if iss.wnr() {
//             let data: [u8; 8] = match reg_index {
//                 0..=30 => self.context.vcpu.x(reg_index),
//                 31 => 0u64,
//                 _ => unreachable!(),
//             }
//             .to_ne_bytes();

//             if let Some(vm_ops) = &self.context.vcpu.vm_ops {
//                 vm_ops
//                     .mmio_write(self.context.map.1, &data[0..len])
//                     .unwrap();
//             }
//         } else {
//             let mut data = [0_u8; 8];
//             if let Some(vm_ops) = &self.context.vcpu.vm_ops {
//                 vm_ops
//                     .mmio_read(self.context.map.1, &mut data[0..len])
//                     .unwrap();
//             }

//             let mut data = u64::from_ne_bytes(data);
//             if sign_extend {
//                 let shift = 64 - len * 8;
//                 data = ((data as i64) << shift >> shift) as u64;
//                 if !iss.sf() {
//                     data &= 0xffffffff;
//                 }
//             }
//             self.context.vcpu.set_x(reg_index, data);
//         }

//         let pc = self.context.vcpu.pc();
//         self.context
//             .vcpu
//             .set_pc(if esr_el2.il() { pc + 4 } else { pc + 2 });
//         true
//     }

//     pub fn emulate(&mut self) -> bool {
//         if self.context.interruption_pending {
//             panic!("Let's handle this scenario differently");
//         }

//         if !self.decode_with_syndrome() {
//             panic!("Failed to decode using syndrome register")
//         }
//         false
//     }
// }
