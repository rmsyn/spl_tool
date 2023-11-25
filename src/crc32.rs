// SPDX-License-Identifier: GPL-2.0+

/// Reverses the bits in the CRC-32 calculation.
///
/// From the `spl_tool` C implementation: <https://github.com/starfive-tech/Tools/blob/master/spl_tool/crc32.c>
pub const fn crc32_reverse(mut x: u32) -> u32 {
    x = ((x & 0x55555555) << 1) | ((x >> 1) & 0x55555555);
    x = ((x & 0x33333333) << 2) | ((x >> 2) & 0x33333333);
    x = ((x & 0x0F0F0F0F) << 4) | ((x >> 4) & 0x0F0F0F0F);
    x = (x << 24) | ((x & 0xFF00) << 8) | ((x >> 8) & 0xFF00) | (x >> 24);
    x
}

/// Calculate the CRC-32 value over the provided data buffer.
///
/// Parameters:
///
/// - `iv`: initialization vector for the CRC-32 polynomial.
/// - `sv`: state vector for the CRC-32 polynomial.
/// - `data`: byte buffer to calculate the checksum.
///
/// From the `spl_tool` C implementation: <https://github.com/starfive-tech/Tools/blob/master/spl_tool/crc32.c>
pub fn crc32(iv: u32, sv: u32, data: &[u8]) -> u32 {
    let mut crc = iv;

    for &byte in data.iter() {
        let mut sum = crc32_reverse(byte as u32);
        for x in 0..8 {
            sum <<= (x != 0) as u32;
            crc = if ((crc ^ sum) & 0x80000000) != 0 {
                (crc << 1) ^ sv
            } else {
                crc << 1
            };
        }
    }

    crc
}

/// Performs the final round of the CRC-32 calculation.
///
/// From the `spl_tool` C implementation: <https://github.com/starfive-tech/Tools/blob/master/spl_tool/crc32.c>
pub const fn crc32_final(iv: u32) -> u32 {
    crc32_reverse(iv ^ !0u32)
}
