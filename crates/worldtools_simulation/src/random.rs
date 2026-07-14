#[derive(Debug, Clone, Copy)]
pub(crate) struct StableRng(u64);

impl StableRng {
    pub(crate) const fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub(crate) fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.0;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    pub(crate) fn unit_f32(&mut self) -> f32 {
        let bits = u16::try_from(self.next_u64() >> 48).expect("16-bit random value fits u16");
        f32::from(bits) / 65_536.0
    }

    pub(crate) fn signed_f32(&mut self) -> f32 {
        self.unit_f32().mul_add(2.0, -1.0)
    }
}

pub(crate) fn hash_unit(seed: u64, index: usize, domain: u64) -> f32 {
    let index = u64::try_from(index).expect("atlas index fits u64");
    let mut rng = StableRng::new(seed ^ index.wrapping_mul(0xd6e8_feb8_6659_fd93) ^ domain);
    rng.unit_f32()
}
