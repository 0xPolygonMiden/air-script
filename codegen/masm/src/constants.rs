// WINTERFELL CONSTANTS ---------------------------------------------------------------------------
pub const MAIN_TRACE: air_ir::TraceSegmentId = 0;
pub const AUX_TRACE: air_ir::TraceSegmentId = 1;

// MIDEN CONSTANTS -------------------------------------------------------------------------------
// https://github.com/0xPolygonMiden/miden-vm/blob/next/stdlib/asm/crypto/stark/constants.masm#L33
pub const COMPOSITION_COEF_ADDRESS: u32 = 4294900200;

// https://github.com/0xPolygonMiden/miden-vm/blob/next/stdlib/asm/crypto/stark/constants.masm#L14
pub const PUBLIC_INPUTS_ADDRESS: u32 = 4294800000;

// https://github.com/0xPolygonMiden/miden-vm/blob/next/stdlib/asm/crypto/stark/constants.masm#L19
pub const OOD_FRAME_ADDRESS: u32 = 4294900000;

// https://github.com/0xPolygonMiden/miden-vm/blob/next/stdlib/asm/crypto/stark/ood_frames.masm#L11-L16
pub const MAIN_TRACE_COLUMN_COUNT: u32 = 72;
pub const OOD_AUX_FRAME_ADDRESS: u32 = OOD_FRAME_ADDRESS + MAIN_TRACE_COLUMN_COUNT;

// https://github.com/0xPolygonMiden/miden-vm/blob/next/stdlib/asm/crypto/stark/constants.masm#LL30C25-L30C35
pub const AUX_RAND_ELEM_PTR: u32 = 4294900150;

// https://github.com/0xPolygonMiden/miden-vm/blob/next/stdlib/asm/crypto/stark/constants.masm#L81
pub const TRACE_LEN_ADDRESS: u32 = 4294903306;
pub const LOG2_TRACE_LEN_ADDRESS: u32 = 4294903307;

// https://github.com/0xPolygonMiden/miden-vm/blob/next/stdlib/asm/crypto/stark/constants.masm#L79
pub const Z_ADDRESS: u32 = 4294903304;

pub const TRACE_DOMAIN_GENERATOR_ADDRESS: u32 = 4294799999;

// CODEGEN CONSTANTS ------------------------------------------------------------------------------
pub const PERIODIC_VALUES_ADDRESS: u32 = 500000000;
pub const Z_EXP_ADDRESS: u32 = 500000100;
