use crate::writer::Writer;
use crate::CodegenError;
use processor::math::{Felt, FieldElement, StarkField};
use std::ops::{Bound, RangeBounds};

/// Precomputes the exemption points for a given power.
///
/// The current version of the generated code has hardcoded 2 exemption points, this means the
/// points can be precomputed during compilation time to save a few cycles during runtime.
fn points_for_power(power: u32) -> (u64, u64) {
    let g = Felt::get_root_of_unity(power);
    let trace_len = 2u64.pow(power);
    let one = g.exp(trace_len - 1).as_int();
    let two = g.exp(trace_len - 2).as_int();
    (one, two)
}

/// Generate code to push the exemptions point to the top of the stack.
///
/// This procedure handles two power using conditional drops, instead of control flow with if
/// statements, since the former is slightly faster for the small number of instructions used. The
/// emitted code assumes the trace_length is at the top of the stack, and afer executing it will
/// leave leave the stack as follows:
///
/// Stack: [g^{trace_len-2}, g^{trace_len-1}, ...]
fn exemption_points(writer: &mut Writer, small_power: u32) {
    let (lone, ltwo) = points_for_power(small_power);
    let (hone, htwo) = points_for_power(small_power + 1);

    writer.push(2u64.pow(small_power));
    writer.u32checked_and();
    writer.neq(0);
    writer.comment(format!(
        "Test if trace length is a power of 2^{}",
        small_power
    ));

    writer.push(hone);
    writer.push(lone);
    writer.dup(2);
    writer.cdrop();

    writer.push(htwo);
    writer.push(ltwo);
    writer.movup(3);
    writer.cdrop();
}

/// Helper function to emit efficient code to bisect the trace length value.
///
/// The callbacks `yes` and `no` are used to emit the code for each branch.
fn bisect_trace_len<L, R>(writer: &mut Writer, range: impl RangeBounds<u32>, yes: L, no: R)
where
    L: FnOnce(&mut Writer),
    R: FnOnce(&mut Writer),
{
    let mask = match (range.end_bound(), range.start_bound()) {
        (Bound::Included(&start), Bound::Included(&end)) => {
            let high_mask = 2u64.pow(start + 1) - 1;
            let low_mask = 2u64.pow(end) - 1;
            high_mask ^ low_mask
        }
        _ => panic!("Only inclusive ranges are supported"),
    };

    writer.dup(0);
    writer.push(mask);
    writer.u32checked_and();
    writer.neq(0);
    writer.comment(format!(
        "{:?}..{:?}",
        range.start_bound(),
        range.end_bound()
    ));

    writer.r#if();
    yes(writer);
    writer.r#else();
    no(writer);
    writer.r#end();
}

/// Emits bisect search for the exemptions points.
pub fn gen_get_exemptions_points(writer: &mut Writer) -> Result<(), CodegenError> {
    // Notes:
    // - Computing the exemption points on the fly would require 1 exponentiation to find the
    // root-of-unity from the two-adacity, followed by another exponetiation to compute the
    // two-to-last exemption point, and a multiplication to compute the last exemption point.
    // Each exponentiation is 41 cycles, giving around 83 cycles to compute the values.
    // - For the range from powers 3 to 32 there are 30 unique values, which requires 8 words
    // of data. Storing the data to memory requires pushing the 4 elements of a word to the
    // stack, the target address, the store, and cleaning the stack, resulting in 10 cycles per
    // word for a total of 80 cycles and some additional cycles to load the right value from
    // memory when needed.
    // - The code below instead uses a binary search to find the right value. And push only the
    // necessary data to memory, in 62/73 cycles.
    // - The smallest trace length is 2^3,
    // Ref: https://github.com/facebook/winterfell/blob/main/air/src/air/trace_info.rs#L34-L35
    // - The trace length is guaranteed to be a power-of-two and to fit in a u32.
    // Ref: https://github.com/0xPolygonMiden/miden-vm/blob/next/stdlib/asm/crypto/stark/random_coin.masm#L76

    bisect_trace_len(
        writer,
        3..=16,
        |writer: &mut Writer| {
            bisect_trace_len(
                writer,
                3..=10,
                |writer: &mut Writer| {
                    bisect_trace_len(
                        writer,
                        3..=6,
                        |writer: &mut Writer| {
                            bisect_trace_len(
                                writer,
                                3..=4,
                                |writer: &mut Writer| exemption_points(writer, 3),
                                |writer: &mut Writer| exemption_points(writer, 5),
                            )
                        },
                        |writer: &mut Writer| {
                            bisect_trace_len(
                                writer,
                                7..=8,
                                |writer: &mut Writer| exemption_points(writer, 7),
                                |writer: &mut Writer| exemption_points(writer, 9),
                            )
                        },
                    )
                },
                |writer: &mut Writer| {
                    bisect_trace_len(
                        writer,
                        11..=14,
                        |writer: &mut Writer| {
                            bisect_trace_len(
                                writer,
                                11..=12,
                                |writer: &mut Writer| exemption_points(writer, 11),
                                |writer: &mut Writer| exemption_points(writer, 13),
                            )
                        },
                        |writer: &mut Writer| exemption_points(writer, 15),
                    )
                },
            );
        },
        |writer: &mut Writer| {
            bisect_trace_len(
                writer,
                17..=24,
                |writer: &mut Writer| {
                    bisect_trace_len(
                        writer,
                        17..=20,
                        |writer: &mut Writer| {
                            bisect_trace_len(
                                writer,
                                17..=18,
                                |writer: &mut Writer| exemption_points(writer, 17),
                                |writer: &mut Writer| exemption_points(writer, 19),
                            )
                        },
                        |writer: &mut Writer| {
                            bisect_trace_len(
                                writer,
                                21..=22,
                                |writer: &mut Writer| exemption_points(writer, 21),
                                |writer: &mut Writer| exemption_points(writer, 23),
                            )
                        },
                    )
                },
                |writer: &mut Writer| {
                    bisect_trace_len(
                        writer,
                        25..=28,
                        |writer: &mut Writer| {
                            bisect_trace_len(
                                writer,
                                25..=26,
                                |writer: &mut Writer| exemption_points(writer, 25),
                                |writer: &mut Writer| exemption_points(writer, 27),
                            )
                        },
                        |writer: &mut Writer| {
                            bisect_trace_len(
                                writer,
                                29..=30,
                                |writer: &mut Writer| exemption_points(writer, 29),
                                |writer: &mut Writer| {
                                    writer.drop();
                                    let (one, two) = points_for_power(31);
                                    writer.push(one);
                                    writer.push(two);
                                },
                            )
                        },
                    )
                },
            );
        },
    );

    Ok(())
}
