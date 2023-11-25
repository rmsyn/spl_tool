// SPDX-License-Identifier: GPL-2.0+

use core::{cmp, mem};

use super::{Error, Result};

/// Default value of the offset of SPL header: `64+256+256 = 0x240`
pub const DEF_SOFS: u32 = 0x240;
/// Default SPL version ID.
pub const DEF_VERS: u32 = 0x01010101;
/// Default value of `SBL_BAK_OFFSET`.
pub const DEF_BACKUP: u32 = 0x200000;
/// Default value for the offset from `HDR` to `SPL_IMAGE`.
pub const DEF_RESL: u32 = 0x400;
/// Default filename of the U-Boot SPL binary.
pub const DEF_SPL_FILE: &str = "u-boot-spl.bin";
/// Maximum path length: defined in `linux/limits.h`.
pub const PATH_MAX: usize = 4096;
/// Value indicating a failed CRC32 calculation/check.
pub const CRC_FAILED: u32 = 0x5a5a5a5a;
/// Default size for an U-Boot SPL header.
///
/// Currently set to `1 KiB` (including reserved padding).
pub const SPL_HEADER_LEN: usize = 1024;
/// Maximum supported length of an U-Boot SPL image.
pub const MAX_SPL_LEN: usize = 180048;

const PATH_ZERO_BYTES: [u8; PATH_MAX] = [0u8; PATH_MAX];
const RES_PAD2_LEN: usize = 636;
const RES_PAD3_LEN: usize = 364;

/// Represents the U-Boot header for the SPL binary.
///
/// All `u32` end up little endian in output header.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UbootSplHeader {
    sofs: u32,
    bofs: u32,
    // reserved offset padding buffer
    zro2: [u8; RES_PAD2_LEN],
    vers: u32,
    fsiz: u32,
    resl: u32,
    crcs: u32,
    // reserved offset padding buffer
    zro3: [u8; RES_PAD3_LEN],
}

impl UbootSplHeader {
    /// Create a new [UbootSplHeader].
    pub const fn new() -> Self {
        Self {
            sofs: DEF_SOFS,
            bofs: DEF_BACKUP,
            zro2: [0; 636],
            vers: DEF_VERS,
            fsiz: 0,
            resl: DEF_RESL,
            crcs: 0,
            zro3: [0; 364],
        }
    }

    /// Gets the offset of SPL header: 64+256+256 = 0x240
    pub const fn sofs(&self) -> u32 {
        self.sofs
    }

    /// Gets the `SBL_BAK_OFFSET`:
    ///
    /// Offset of backup SBL from Flash info start from `input_sbl_normal.cfg`
    pub const fn bofs(&self) -> u32 {
        self.bofs
    }

    /// Sets the `SBL_BAK_OFFSET`:
    ///
    /// Offset of backup SBL from Flash info start from `input_sbl_normal.cfg`
    pub fn set_bofs(&mut self, val: u32) {
        self.bofs = val;
    }

    /// Builder function that sets the `SBL_BAK_OFFSET`:
    ///
    /// Offset of backup SBL from Flash info start from `input_sbl_normal.cfg`
    pub fn with_bofs(mut self, val: u32) -> Self {
        self.set_bofs(val);
        self
    }

    /// Gets the version: `0x01010101` by default
    pub const fn vers(&self) -> u32 {
        self.vers
    }

    /// Sets the version.
    pub fn set_vers(&mut self, val: u32) {
        self.vers = val;
    }

    /// Builder function that sets the version.
    pub fn with_vers(mut self, val: u32) -> Self {
        self.set_vers(val);
        self
    }

    /// Gets the `u-boot-spl.bin` size in bytes.
    pub const fn fsiz(&self) -> u32 {
        self.fsiz
    }

    /// Sets the `u-boot-spl.bin` size in bytes.
    pub fn set_fsiz(&mut self, val: u32) {
        self.fsiz = val;
    }

    /// Builder function that sets the `u-boot-spl.bin` size in bytes.
    pub fn with_fsiz(mut self, val: u32) -> Self {
        self.set_fsiz(val);
        self
    }

    /// Gets the offset from `HDR` to `SPL_IMAGE`.
    ///
    /// Defaults to `0x400 (00 04 00 00)` currently.
    pub const fn resl(&self) -> u32 {
        self.resl
    }

    /// Sets the offset from `HDR` to `SPL_IMAGE`.
    pub fn set_resl(&mut self, val: u32) {
        self.resl = val;
    }

    /// Builder function that sets the offset from `HDR` to `SPL_IMAGE`.
    pub fn with_resl(mut self, val: u32) -> Self {
        self.set_resl(val);
        self
    }

    /// Gets the CRC32 of `u-boot-spl.bin`.
    pub const fn crcs(&self) -> u32 {
        self.crcs
    }

    /// Sets the CRC32 of `u-boot-spl.bin`.
    pub fn set_crcs(&mut self, val: u32) {
        self.crcs = val;
    }

    /// Builder function that Sets the CRC32 of `u-boot-spl.bin`.
    pub fn with_crcs(mut self, val: u32) -> Self {
        self.set_crcs(val);
        self
    }
}

impl From<&UbootSplHeader> for [u8; SPL_HEADER_LEN] {
    fn from(val: &UbootSplHeader) -> Self {
        const WORD_LEN: usize = mem::size_of::<u32>();

        let mut res = [0u8; SPL_HEADER_LEN];
        let mut idx = 0usize;

        // serialize SOFS field to buffer
        res[idx..idx.saturating_add(WORD_LEN)].copy_from_slice(val.sofs.to_le_bytes().as_ref());
        idx = idx.saturating_add(WORD_LEN);

        // serialize BOFS field to buffer
        res[idx..idx.saturating_add(WORD_LEN)].copy_from_slice(val.bofs.to_le_bytes().as_ref());
        idx = idx.saturating_add(WORD_LEN);

        // skip `zro2` reserved padding
        idx = idx.saturating_add(RES_PAD2_LEN);

        // serialize VERS field to buffer
        res[idx..idx.saturating_add(WORD_LEN)].copy_from_slice(val.vers.to_le_bytes().as_ref());
        idx = idx.saturating_add(WORD_LEN);

        // serialize FSIZ field to buffer
        res[idx..idx.saturating_add(WORD_LEN)].copy_from_slice(val.fsiz.to_le_bytes().as_ref());
        idx = idx.saturating_add(WORD_LEN);

        // serialize RESL field to buffer
        res[idx..idx.saturating_add(WORD_LEN)].copy_from_slice(val.resl.to_le_bytes().as_ref());
        idx = idx.saturating_add(WORD_LEN);

        // serialize CRCS field to buffer
        res[idx..idx.saturating_add(WORD_LEN)].copy_from_slice(val.crcs.to_le_bytes().as_ref());

        res
    }
}

impl From<UbootSplHeader> for [u8; SPL_HEADER_LEN] {
    fn from(val: UbootSplHeader) -> Self {
        (&val).into()
    }
}

impl TryFrom<&[u8]> for UbootSplHeader {
    type Error = Error;

    fn try_from(val: &[u8]) -> Result<Self> {
        const WORD_LEN: usize = mem::size_of::<u32>();

        if val.len() < SPL_HEADER_LEN {
            Err(Error::InvalidHeaderLen((val.len(), SPL_HEADER_LEN)))
        } else {
            let mut idx = 0usize;

            // deserialize SOFS field from buffer
            let sofs = u32::from_le_bytes(val[idx..idx.saturating_add(WORD_LEN)].try_into()?);
            idx = idx.saturating_add(WORD_LEN);

            // deserialize BOFS field from buffer
            let bofs = u32::from_le_bytes(val[idx..idx.saturating_add(WORD_LEN)].try_into()?);
            idx = idx.saturating_add(WORD_LEN);

            // deserialize `zro2` reserved padding
            // TODO: should we reject non-zero padding here?
            // If CRC32 validates, the header should be valid.
            // Maybe too early to reject here.
            let zro2: [u8; RES_PAD2_LEN] = val[idx..idx.saturating_add(RES_PAD2_LEN)].try_into()?;
            idx = idx.saturating_add(RES_PAD2_LEN);

            // deserialize VERS field from buffer
            let vers = u32::from_le_bytes(val[idx..idx.saturating_add(WORD_LEN)].try_into()?);
            idx = idx.saturating_add(WORD_LEN);

            // deserialize FSIZ field from buffer
            let fsiz = u32::from_le_bytes(val[idx..idx.saturating_add(WORD_LEN)].try_into()?);
            idx = idx.saturating_add(WORD_LEN);

            // deserialize RESL field from buffer
            let resl = u32::from_le_bytes(val[idx..idx.saturating_add(WORD_LEN)].try_into()?);
            idx = idx.saturating_add(WORD_LEN);

            // deserialize CRCS field from buffer
            let crcs = u32::from_le_bytes(val[idx..idx.saturating_add(WORD_LEN)].try_into()?);
            idx = idx.saturating_add(WORD_LEN);

            // deserialize `zro3` reserved padding
            // TODO: should we reject non-zero padding here?
            // If CRC32 validates, the header should be valid.
            // Maybe too early to reject here.
            let zro3: [u8; RES_PAD3_LEN] = val[idx..idx.saturating_add(RES_PAD2_LEN)].try_into()?;

            Ok(Self {
                sofs,
                bofs,
                zro2,
                vers,
                fsiz,
                resl,
                crcs,
                zro3,
            })
        }
    }
}

impl<const N: usize> TryFrom<&[u8; N]> for UbootSplHeader {
    type Error = Error;

    fn try_from(val: &[u8; N]) -> Result<Self> {
        val.as_ref().try_into()
    }
}

impl<const N: usize> TryFrom<[u8; N]> for UbootSplHeader {
    type Error = Error;

    fn try_from(val: [u8; N]) -> Result<Self> {
        val.as_ref().try_into()
    }
}

impl Default for UbootSplHeader {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents configuration arguments for SPL header generation.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeaderConf {
    name: [u8; PATH_MAX],
    vers: u32,
    bofs: u32,
    create_header: bool,
    fix_image_header: bool,
}

impl HeaderConf {
    /// Creates a new [HeaderConf].
    pub const fn new() -> Self {
        Self {
            name: [0u8; PATH_MAX],
            vers: DEF_VERS,
            bofs: DEF_BACKUP,
            create_header: false,
            fix_image_header: false,
        }
    }

    /// Gets the header name as a string.
    pub fn name(&self) -> &str {
        core::str::from_utf8(self.name[..self.name_len()].as_ref()).unwrap_or("")
    }

    fn name_len(&self) -> usize {
        self.name
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(self.name.len())
    }

    /// Sets the header name from a string.
    pub fn set_name(&mut self, val: &str) {
        let val_bytes = val.as_bytes();
        let len = cmp::min(PATH_MAX - 1, val_bytes.len());
        self.name[..len].copy_from_slice(val_bytes[..len].as_ref());
        self.name[len..].copy_from_slice(PATH_ZERO_BYTES[len..].as_ref());
    }

    /// Sets the header name from a string.
    pub fn with_name(mut self, val: &str) -> Self {
        self.set_name(val);
        self
    }

    /// Gets the version.
    pub const fn vers(&self) -> u32 {
        self.vers
    }

    /// Sets the version.
    pub fn set_vers(&mut self, val: u32) {
        self.vers = val;
    }

    /// Builder function that sets the version.
    pub fn with_vers(mut self, val: u32) -> Self {
        self.set_vers(val);
        self
    }

    /// Gets the `SBL_BAK_OFFSET`:
    ///
    /// Offset of backup SBL from Flash info start from `input_sbl_normal.cfg`
    pub const fn bofs(&self) -> u32 {
        self.bofs
    }

    /// Sets the `SBL_BAK_OFFSET`:
    ///
    /// Offset of backup SBL from Flash info start from `input_sbl_normal.cfg`
    pub fn set_bofs(&mut self, val: u32) {
        self.bofs = val;
    }

    /// Builder function that sets the `SBL_BAK_OFFSET`:
    ///
    /// Offset of backup SBL from Flash info start from `input_sbl_normal.cfg`
    pub fn with_bofs(mut self, val: u32) -> Self {
        self.set_bofs(val);
        self
    }

    /// Gets whether to create the SPL header.
    pub const fn create_header(&self) -> bool {
        self.create_header
    }

    /// Sets whether to create the SPL header.
    pub fn set_create_header(&mut self, val: bool) {
        self.create_header = val;
    }

    /// Builder function that sets whether to create the SPL header.
    pub fn with_create_header(mut self, val: bool) -> Self {
        self.set_create_header(val);
        self
    }

    /// Gets whether to create the fix image header.
    pub const fn fix_image_header(&self) -> bool {
        self.fix_image_header
    }

    /// Sets whether to create the fix image header.
    pub fn set_fix_image_header(&mut self, val: bool) {
        self.fix_image_header = val;
    }

    /// Builder function that sets whether to create the fix image header.
    pub fn with_fix_image_header(mut self, val: bool) -> Self {
        self.set_fix_image_header(val);
        self
    }
}

impl Default for HeaderConf {
    fn default() -> Self {
        Self::new()
    }
}
