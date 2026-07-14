use bevy::prelude::Message;

/// Requests a complete, deterministic world-history rebuild for `seed`.
///
/// The application owns generation and updates the active document only after
/// the replacement snapshot is ready.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegenerateWorld {
    pub seed: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regeneration_request_preserves_the_full_seed() {
        let request = RegenerateWorld { seed: u64::MAX };
        assert_eq!(request.seed, u64::MAX);
    }
}
