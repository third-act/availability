// This is only used for serialize.
//#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn _is_zero(num: &u32) -> bool {
    *num == 0
}

//#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn is_false(value: &bool) -> bool {
    return !value;
}

// This is only used for serialize.
//#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn is_empty<T>(value: &Vec<T>) -> bool {
    value.is_empty()
}
