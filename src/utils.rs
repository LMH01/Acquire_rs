use std::ops::RangeInclusive;

use miette::{miette, Result};

/// Transforms the range into a vector
pub fn generate_number_vector(min: u32, max: u32) -> Vec<u32> {
    let mut vec = Vec::new();
    for i in min..=max {
        vec.push(i);
    }
    vec
}

/// Removes the specified content from the vector.
/// Returns an error when the value could not be found.
pub fn remove_content_from_vec<T: PartialEq>(to_remove: T, vec: &mut Vec<T>) -> Result<()> {
    let mut index_to_remove = 0;
    let mut value_found = false;
    for (index, content) in vec.iter().enumerate() {
        if *content == to_remove {
            index_to_remove = index;
            value_found = true;
            break;
        }
    }
    if value_found {
        vec.remove(index_to_remove);
        Ok(())
    } else {
        Err(miette!(
            "Unable to remove value from vector: Value was not found."
        ))
    }
}

/// Returns the specified content from the vector as reference.
pub fn get_content_from_vec<T: PartialEq>(to_find: T, vec: &Vec<T>) -> Result<&T> {
    let mut index_to_remove = 0;
    let mut value_found = false;
    for (index, content) in vec.iter().enumerate() {
        if *content == to_find {
            index_to_remove = index;
            value_found = true;
            break;
        }
    }
    if value_found {
        Ok(vec.get(index_to_remove).unwrap())
    } else {
        Err(miette!(
            "Unable to get value from vector: Value was not found."
        ))
    }
}
