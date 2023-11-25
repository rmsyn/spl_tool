// SPDX-License-Identifier: GPL-2.0+

//! SPL header tool for the StarFive VisionFive2 board.
//!
//! Based on the C implementation: <https://github.com/starfive-tech/Tools/tree/master/spl_tool>

#![no_std]

mod crc32;
mod error;
mod spl_header;

pub use crc32::*;
pub use error::*;
pub use spl_header::*;
