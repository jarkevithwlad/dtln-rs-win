// 32 ms @ 16khz per DTLN docs: https://github.com/breizhn/DTLN
pub const DTLN_BLOCK_LEN: usize = 512;

// 8 ms @ 16khz per DTLN docs.
pub const DTLN_BLOCK_SHIFT: usize = 128;

pub const DTLN_FFT_OUT_SIZE: usize = DTLN_BLOCK_LEN / 2 + 1;
