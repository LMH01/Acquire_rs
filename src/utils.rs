use std::ops::RangeInclusive;

/// Transforms the range into a vector
pub fn gemerate_number_vector(min: u32, max: u32) -> Vec<u32> {
    let mut vec = Vec::new();
    for i in min..=max {
        vec.push(i);
    }
    vec
}
