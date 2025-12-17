type Val = p3_goldilocks::Goldilocks;

pub(crate) fn convert_var_len_pub_inputs_to_goldilocks(
    var_len_pub_inputs: Vec<Vec<Vec<u64>>>,
) -> Vec<Vec<Vec<Val>>> {
    let mut var_len_pub_inputs_goldilocks_vec: Vec<Vec<Vec<Val>>> = vec![];
    for arr in var_len_pub_inputs.iter() {
        let mut goldilocks_arr: Vec<Vec<Val>> = vec![];
        for slice in arr.iter() {
            let goldilocks_slice: Vec<Val> = slice
                .iter()
                .map(|&x| <Val as p3_field::PrimeCharacteristicRing>::from_u64(x))
                .collect();
            goldilocks_arr.push(goldilocks_slice);
        }
        var_len_pub_inputs_goldilocks_vec.push(goldilocks_arr);
    }
    var_len_pub_inputs_goldilocks_vec
}

pub(crate) fn convert_inner_vec_to_slice<'a>(
    var_len_pub_inputs: &'a Vec<Vec<Vec<Val>>>,
) -> Vec<Vec<&'a [Val]>> {
    var_len_pub_inputs
        .iter()
        .map(|outer| outer.iter().map(|inner| inner.as_slice()).collect())
        .collect()
}

pub(crate) fn convert_mid_vec_to_slice<'a>(
    var_len_pub_inputs: &'a Vec<Vec<&'a [Val]>>,
) -> Vec<&'a [&'a [Val]]> {
    var_len_pub_inputs.iter().map(|v| v.as_slice()).collect()
}
