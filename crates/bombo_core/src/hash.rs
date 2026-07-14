#[unsafe(no_mangle)]
pub extern "C" fn bombo_seed_hash(seed: u32, schema_version: u32) -> u32 {
    mix_hash(seed, schema_version, 0, 0)
}

#[unsafe(no_mangle)]
pub extern "C" fn bombo_random_at(seed: u32, stream: u32, index: u32, salt: u32) -> u32 {
    mix_hash(seed, stream, index, salt)
}

fn mix_hash(a: u32, b: u32, c: u32, d: u32) -> u32 {
    let mut value =
        a.wrapping_mul(0x9e37_79b1) ^ b.wrapping_mul(0x85eb_ca77) ^ c.wrapping_mul(0xc2b2_ae3d) ^ d;
    value ^= value >> 16;
    value = value.wrapping_mul(0x7feb_352d);
    value ^= value >> 15;
    value = value.wrapping_mul(0x846c_a68b);
    value ^ (value >> 16)
}
