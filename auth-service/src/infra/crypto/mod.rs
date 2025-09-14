use zeroize::Zeroizing;

pub mod jwt_issuer_rs256;
pub mod password_hasher_argon2;
pub mod refresh_factory;

#[derive(Clone)]
pub struct PepperSet {
    keys: Vec<Zeroizing<Vec<u8>>>,
}

impl PepperSet {
    pub fn new_current_only(current: Vec<u8>) -> Self {
        Self {
            keys: vec![Zeroizing::new(current)],
        }
    }

    pub fn new_with_rotation(current: Vec<u8>, old: Vec<Vec<u8>>) -> Self {
        let mut v = Vec::with_capacity(1 + old.len());
        v.push(Zeroizing::new(current));
        for k in old {
            v.push(Zeroizing::new(k));
        }
        Self { keys: v }
    }

    #[inline]
    fn iter(&self) -> impl Iterator<Item = &[u8]> {
        self.keys.iter().map(|z| z.as_slice())
    }

    #[inline]
    fn current(&self) -> &[u8] {
        self.keys[0].as_slice()
    }
}