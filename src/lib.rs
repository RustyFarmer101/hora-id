//! Time sorted unique ID generator
//! IDs are time-sorted and 8 bytes long, which is half the length of a UUID and ULID
//!
//! ## Composition
//! TUID has 3 parts
//! - 4 byte timestamp high
//! - 1 byte timestamp low
//! - 1 byte for machine ID (0-255)
//! - 2 bytes for sequence number

#[cfg(feature = "chrono")]
use chrono::{DateTime, NaiveDateTime, Utc};
use std::time::{SystemTime, UNIX_EPOCH};

/// Unix Epoch on Jan 01 2023 12:00:00 am
const EPOCH: u64 = 1672531200000;

/// Get the current epoch with base epoch starting at [EPOCH]
///
/// ## Fail condition
/// If the system time is incorrect and before the [EPOCH] time
///
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

pub struct TuidParams {
    machine_id: u8,
    epoch: u64,
    sequence: u16,
}

/// TUID Generator with guarantee to generate unique IDs on a single machine
pub struct TuidGenerator {
    /// Unique Machine identifier with support for max 256 unique machines
    machine_id: u8,
    /// sequence number in the same epoch,
    sequence: u16,
    /// Last time an ID was generated
    last_gen: u64,
}

impl TuidGenerator {
    pub fn new(machine_id: u8) -> Result<Self, String> {
        let epoch = current_epoch()?;
        let epoch = rescale_epoch(epoch);
        Ok(Self {
            machine_id,
            sequence: 0,
            last_gen: epoch,
        })
    }

    /// Generate a new TUID
    pub fn next(&mut self) -> Tuid {
        loop {
            let epoch = current_epoch().unwrap();
            let scaled_epoch = rescale_epoch(epoch);
            if scaled_epoch > self.last_gen {
                self.sequence = 0;
            }

            // generate_id
            self.sequence += 1;
            let params = TuidParams {
                machine_id: self.machine_id,
                epoch,
                sequence: self.sequence + 1,
            };
            let id = Tuid::with_params(params);
            self.last_gen = scaled_epoch;
            break id;
        }
    }
}

/// A time-sorted 8-byte (64-bit) unique identifier
#[derive(Debug)]
pub struct Tuid {
    inner: [u8; 8],
}

impl Tuid {
    /// Quickly generate a new TUID
    ///
    /// ## Caution
    /// Calling this method doesn't guarantee a unique ID for every call.
    /// This method shall only be used when you need to generate a new id rapidly.
    ///
    pub fn new(machine_id: Option<u8>) -> Result<Self, String> {
        let epoch = current_epoch()?;
        let params = TuidParams {
            machine_id: machine_id.unwrap_or(0),
            epoch,
            sequence: 0,
        };
        let id = Self::with_params(params);
        Ok(id)
    }

    /// Generate a new TUID with custom epoch
    ///
    /// ## More info
    /// This method is mainly used by the [TuidGenerator] generator to get a new [Tuid].
    /// THe `Tuid::new` method also calls this method after getting the current epoch.
    ///
    fn with_params(params: TuidParams) -> Self {
        let high = (params.epoch / 1000) as u32;
        let low = (params.epoch % 1000) as u16;

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

        // add machine_id
        tuid[5] = params.machine_id;

        // add sequence
        let sequence_high = ((params.sequence >> 8) & 0xFF) as u8;
        let sequence_low = (params.sequence & 0xFF) as u8;

        tuid[6] = sequence_high;
        tuid[7] = sequence_low;

        Self { inner: tuid }
    }

    /// Convert a [Tuid] to a number
    pub fn to_u64(&self) -> u64 {
        u64::from_be_bytes(self.inner)
    }

    /// Convert a number to [Tuid]
    pub fn from_u64(num: u64) -> Option<Self> {
        let d: [u8; 8] = num.to_be_bytes();
        let id = Self { inner: d };
        Some(id)
    }

    /// Convert a [Tuid] to a [String]
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

    /// Create a [Tuid] from a string slice
    pub fn from_str(s: &str) -> Option<Self> {
        if s.len() != 16 {
            return None;
        }
        let num = u64::from_str_radix(s, 16).ok()?;
        let bytes: [u8; 8] = num.to_be_bytes();
        let id = Self { inner: bytes };
        Some(id)
    }

    /// Retrieve a chrono datetime from [Tuid]
    /// This conditionally includes a module which implements chrono support.
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

fn rescale_epoch(value: u64) -> u64 {
    let high = value / 1000;
    let low = (value % 1000) as u16;
    let low = (low as f32) * 0.256;
    let low = low as u64;
    high * 1000 + low
}

/// Convert u16 to u8 with rescaling process
fn rescale_low(value: u16) -> u8 {
    let new_val = (value as f32) * (256.0) / (1000.0);
    new_val as u8
}

/// Convert a u8 to u16 with rescaling process
#[allow(dead_code)]
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

    #[test]
    fn u64s() {
        let num = 57630818184577258;
        let id = Tuid::from_u64(num);
        assert!(id.is_some());
        let id = id.unwrap();
        assert_eq!(id.to_u64(), num);
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

    #[test]
    fn epoch_rescaling() {
        // test 1
        let value = 1672531200000;
        assert_eq!(rescale_epoch(value), value);
        // test 2
        assert_eq!(rescale_epoch(1672531200003), 1672531200000);
        // test 3
        assert_eq!(rescale_epoch(1672531200005), 1672531200001);
        assert_eq!(rescale_epoch(1672531200006), 1672531200001);
        // test 4
        assert_eq!(rescale_epoch(1672531200998), 1672531200255);
        assert_eq!(rescale_epoch(1672531200999), 1672531200255);
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
        generator.next();
    }
}
