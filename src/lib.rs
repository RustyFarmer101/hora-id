//! Time sorted unique ID generator
//! IDs are time-sorted and 8 bytes long, which is half the length of a UUID and ULID
//!
//! ## Composition
//! TUID has 3 parts
//! - 4 byte timestamp high
//! - 1 byte timestamp low
//! - 3 bytes of randomness

#[cfg(feature = "chrono")]
use chrono::{DateTime, NaiveDateTime, Utc};
use rand::prelude::*;
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

// Unix Epoch on Jan 01 2023 12:00:00 am
const EPOCH: u64 = 1672531200000;

fn current_epoch() -> Result<u64, String> {
    let mut now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    if now < EPOCH {
        return Err("Your device time is incorrect.".to_owned());
    }
    now = now - EPOCH;
    Ok(now)
}

pub struct TuidGenerator {
    machine_id: u8,
    queue: HashSet<[u8; 8]>,
    last_clean: u32,
    last_gen: u32,
}

impl TuidGenerator {
    pub fn new(id: u8) -> Result<Self, String> {
        let epoch = current_epoch()?;
        let epoch = (epoch / 1000) as u32;
        Ok(Self {
            machine_id: id,
            queue: HashSet::new(),
            last_clean: epoch,
            last_gen: epoch,
        })
    }

    pub fn generate(&mut self) -> Tuid {
        loop {
            let epoch = current_epoch().unwrap();
            // clean the queue
            let epoch_high = (epoch / 1000) as u32;
            if epoch_high > self.last_gen && !self.queue.is_empty() {
                self.last_clean = epoch_high;
                self.queue.clear();
            }
            // generate_id
            let id = Tuid::with_epoch(Some(self.machine_id), epoch);
            // detect duplicate ID
            if self.queue.contains(&id.inner) {
                continue;
            }
            self.queue.insert(id.inner.clone());
            self.last_gen = epoch_high;
            break id;
        }
    }
}

#[derive(Debug)]
pub struct Tuid {
    inner: [u8; 8],
}

impl Tuid {
    /// Generate a new TUID
    pub fn new(machine_id: Option<u8>) -> Result<Self, String> {
        let epoch = current_epoch()?;
        let id = Self::with_epoch(machine_id, epoch);
        Ok(id)
    }

    fn with_epoch(machine_id: Option<u8>, epoch: u64) -> Self {
        let high = (epoch / 1000) as u32;
        let low = (epoch % 1000) as u16;

        // create a default bytes array
        let mut tuid = [0u8; 8];

        // set time high
        let bytes = high.to_be_bytes();
        tuid[0] = bytes[0];
        tuid[1] = bytes[1];
        tuid[2] = bytes[2];
        tuid[3] = bytes[3];

        // set time low
        tuid[4] = rescale_low(low);

        // add randomness
        let mut rng = rand::thread_rng();
        tuid[5] = machine_id.unwrap_or_else(|| rng.gen::<u8>());
        tuid[6] = rng.gen::<u8>();
        tuid[7] = rng.gen::<u8>();

        Self { inner: tuid }
    }

    pub fn to_string(&self) -> String {
        format!(
            "{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            self.inner[0],
            self.inner[1],
            self.inner[2],
            self.inner[3],
            self.inner[4],
            self.inner[5],
            self.inner[6],
            self.inner[7]
        )
    }

    pub fn from_str(s: &str) -> Option<Self> {
        if s.len() != 16 {
            return None;
        }
        let num = u64::from_str_radix(s, 16).ok()?;
        let bytes: [u8; 8] = num.to_be_bytes();
        let id = Self { inner: bytes };
        Some(id)
    }

    // This conditionally includes a module which implements chrono support.
    #[cfg(feature = "chrono")]
    pub fn to_chrono(&self) -> DateTime<Utc> {
        let mut high = [0; 4];
        for i in 0..4 {
            high[i] = self.inner[i];
        }
        let high = u32::from_be_bytes(high);
        let low = u8::from_be_bytes([self.inner[4]]);
        let low = upscale_low(low);

        let timestamp = (high as u64 * 1000) + low as u64 + EPOCH;
        let timestamp = NaiveDateTime::from_timestamp_millis(timestamp as i64).unwrap();
        let datetime = DateTime::<Utc>::from_utc(timestamp, Utc);
        datetime
    }
}

/// Convert u16 to u8 with rescaling process
fn rescale_low(value: u16) -> u8 {
    let new_val = (value as f32) * (256.0) / (1000.0);
    new_val as u8
}

fn upscale_low(value: u8) -> u16 {
    let new_val = (value as f32) * (1000.0) / 256.0;
    new_val as u16
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "chrono")]
    use chrono::Timelike;

    #[test]
    fn it_works() {
        let id = Tuid::new(None);
        assert!(id.is_ok());
    }

    #[test]
    fn strings() {
        let source_id = Tuid::new(None).unwrap();
        let s = source_id.to_string();
        let id = Tuid::from_str(&s);
        let derived_id = id.unwrap();
        assert_eq!(source_id.to_string(), derived_id.to_string());
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn chrono() {
        let id = Tuid::new(None).unwrap();
        let time = id.to_chrono();
        let now = Utc::now();
        assert_eq!(now.date_naive(), time.date_naive());
        assert_eq!(now.hour(), time.hour());
        assert_eq!(now.minute(), time.minute());
        assert_eq!(now.second(), time.second());
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

    #[test]
    fn rescale() {
        let value = upscale_low(rescale_low(500));
        assert_eq!(value, 500);
    }
}

#[cfg(test)]
mod gen_tests {
    use super::*;

    #[test]
    fn it_works() {
        let generator = TuidGenerator::new(1);
        assert!(generator.is_ok());
        let mut generator = generator.unwrap();
        let id = generator.generate();
    }
}
