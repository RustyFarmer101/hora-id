//! Time sorted unique ID generator
//! IDs are time-sorted and 8 bytes long, which is half the length of a UUID and ULID
//!
//! ## Composition
//! TUID has 3 parts
//! - 4 byte timestamp high
//! - 1 byte timestamp low
//! - 3 bytes of randomness
use std::time::{SystemTime, UNIX_EPOCH};
use rand::prelude::*;

// Unix Epoch on Jan 01 2023 12:00:00 am
const EPOCH: u64 = 1672531200000;

#[derive(Debug)]
pub struct Tuid {
    inner: [u8;8]
}

impl Tuid {
    /// Generate a new TUID
    pub fn new() -> Self {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64 - EPOCH;
        let high = (now / 1000) as u32;
        let low = (now % 1000) as u16;

        // create a default bytes array
        let mut tuid = [0u8; 8];

        // set time high
        let bytes = high.to_be_bytes();
        tuid[0] = bytes[0];
        tuid[1] = bytes[1];
        tuid[2] = bytes[2];
        tuid[3] = bytes[3];

        // set time low
        let new_low = rescale_low(low);
        tuid[4] = new_low;

        // add randomness
        let mut rng = rand::thread_rng();
        tuid[5] = rng.gen::<u8>();
        tuid[6] = rng.gen::<u8>();
        tuid[7] = rng.gen::<u8>();

        Self{inner: tuid}
    }
}

/// Convert u16 to u8 with rescaling process
fn rescale_low(value: u16) -> u8 {
    let new_val = (value as f32) * (256.0) / (1000.0);
    new_val as u8
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let id = Tuid::new();
        println!("{:?}", id);
    }

    #[test]
    fn rescaling() {
        assert_eq!(rescale_low(0), 0);
        assert_eq!(rescale_low(1), 0);
        assert_eq!(rescale_low(5), 1);
        assert_eq!(rescale_low(498), 127);
        assert_eq!(rescale_low(500), 128);
        assert_eq!(rescale_low(995), 254);
        assert_eq!(rescale_low(997), 255);
        assert_eq!(rescale_low(999), 255);
    }
}
