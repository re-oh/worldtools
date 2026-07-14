use core::{mem, slice};
use wide::f32x4;

#[unsafe(no_mangle)]
pub extern "C" fn bombo_alloc_f32(length: u32) -> *mut f32 {
    let mut values = Vec::<f32>::with_capacity(length as usize);
    let pointer = values.as_mut_ptr();
    mem::forget(values);
    pointer
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bombo_free_f32(pointer: *mut f32, capacity: u32) {
    if pointer.is_null() || capacity == 0 {
        return;
    }
    unsafe { drop(Vec::from_raw_parts(pointer, 0, capacity as usize)) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bombo_affine_clamp_f32(
    pointer: *mut f32,
    length: u32,
    scale: f32,
    bias: f32,
    minimum: f32,
    maximum: f32,
) -> u32 {
    if pointer.is_null() || !valid_bounds(scale, bias, minimum, maximum) {
        return 1;
    }
    let values = unsafe { slice::from_raw_parts_mut(pointer, length as usize) };
    let mut chunks = values.chunks_exact_mut(4);
    let scale4 = f32x4::splat(scale);
    let bias4 = f32x4::splat(bias);
    let minimum4 = f32x4::splat(minimum);
    let maximum4 = f32x4::splat(maximum);
    for chunk in &mut chunks {
        let input = f32x4::new([chunk[0], chunk[1], chunk[2], chunk[3]]);
        let output: [f32; 4] = (input * scale4 + bias4).max(minimum4).min(maximum4).into();
        chunk.copy_from_slice(&output);
    }
    for value in chunks.into_remainder() {
        *value = (*value * scale + bias).clamp(minimum, maximum);
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn bombo_max_f32(target: *mut f32, source: *const f32, length: u32) -> u32 {
    if target.is_null() || source.is_null() {
        return 1;
    }
    let target_values = unsafe { slice::from_raw_parts_mut(target, length as usize) };
    let source_values = unsafe { slice::from_raw_parts(source, length as usize) };
    let vector_length = target_values.len() / 4 * 4;
    for offset in (0..vector_length).step_by(4) {
        let a = f32x4::new([target_values[offset], target_values[offset + 1], target_values[offset + 2], target_values[offset + 3]]);
        let b = f32x4::new([source_values[offset], source_values[offset + 1], source_values[offset + 2], source_values[offset + 3]]);
        let output: [f32; 4] = a.max(b).into();
        target_values[offset..offset + 4].copy_from_slice(&output);
    }
    for offset in vector_length..target_values.len() {
        target_values[offset] = target_values[offset].max(source_values[offset]);
    }
    0
}

fn valid_bounds(scale: f32, bias: f32, minimum: f32, maximum: f32) -> bool {
    scale.is_finite() && bias.is_finite() && minimum.is_finite() && maximum.is_finite() && minimum <= maximum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn affine_kernel_handles_vector_and_scalar_tail() {
        let mut values = [-2.0, -1.0, 0.0, 1.0, 2.0, 3.0, 4.0];
        let status = unsafe { bombo_affine_clamp_f32(values.as_mut_ptr(), values.len() as u32, 0.5, 0.25, 0.0, 1.5) };
        assert_eq!(status, 0);
        assert_eq!(values, [0.0, 0.0, 0.25, 0.75, 1.25, 1.5, 1.5]);
    }

    #[test]
    fn maximum_kernel_handles_vector_and_scalar_tail() {
        let mut target = [0.0, 4.0, 2.0, 8.0, -1.0];
        let source = [1.0, 3.0, 6.0, 2.0, 5.0];
        let status = unsafe { bombo_max_f32(target.as_mut_ptr(), source.as_ptr(), target.len() as u32) };
        assert_eq!(status, 0);
        assert_eq!(target, [1.0, 4.0, 6.0, 8.0, 5.0]);
    }
}
