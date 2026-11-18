use rand::TryRngCore;

pub fn fill_secure(dst: &mut [u8]) -> Result<(), &'static str> {
    let mut rng = rand::rngs::OsRng;
    rng.try_fill_bytes(dst).map_err(|_| "rng")
}
