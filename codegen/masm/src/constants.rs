pub const MAIN_TRACE: u8 = 0;
pub const AUX_TRACE: u8 = 1;

// https://github.com/0xPolygonMiden/miden-vm/blob/next/stdlib/asm/crypto/stark/verifier.masm#L21
pub const COMPOSITION_COEF_ADDRESS: u64 = 4294966016;

// https://github.com/0xPolygonMiden/miden-vm/blob/next/stdlib/asm/crypto/stark/ood_frames.masm#L2
pub const OOD_FRAME_ADDRESS: u64 = 4294965000;

// https://github.com/0xPolygonMiden/miden-vm/blob/next/stdlib/asm/crypto/stark/ood_frames.masm#L2
pub const OOD_AUX_FRAME_ADDRESS: u64 = OOD_FRAME_ADDRESS + 72;

// https://github.com/0xPolygonMiden/miden-vm/blob/next/stdlib/asm/crypto/stark/verifier.masm#L20
pub const AUX_RAND_ELEM_PTR: u64 = 4294965000;

// https://github.com/0xPolygonMiden/miden-vm/blob/next/stdlib/asm/crypto/stark/random_coin.masm#L12
pub const TRACE_LEN_ADDRESS: u64 = 3294967296;

// https://github.com/0xPolygonMiden/miden-vm/blob/next/stdlib/asm/crypto/stark/random_coin.masm#L5
pub const Z_ADDRESS: u64 = 4294967291;

// TODO: define these addresses
pub const PERIODIC_VALUES_ADDRESS: u64 = 500000000;
pub const Z_EXP_ADDRESS: u64 = 500000100;
