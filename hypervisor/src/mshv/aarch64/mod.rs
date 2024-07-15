// Copyright © 2024, Microsoft Corporation

use serde::{Deserialize, Serialize};

pub mod emulator;
pub mod gic;

#[derive(Clone, Serialize, Deserialize)]
pub struct VcpuMshvState {}
