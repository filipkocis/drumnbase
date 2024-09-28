use std::time::{SystemTime, UNIX_EPOCH};

pub struct Random {}

impl Random {
    pub fn gen() -> f64 {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let millis = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();

        let bytes = 0b11111111_11111111;

        let millis_shift = millis & 0b11111;
        let nanos_shift = nanos & 0b11111;

        let shift = ((millis_shift ^ nanos_shift)) | (1 << 0);

        let time_millis = (millis << shift) >> shift & bytes;
        let time_nanos = (nanos << shift) >> shift & bytes;

        let left = time_millis.min(time_nanos);
        let right = time_millis.max(time_nanos);
        let random = left as f64 / right as f64;
        let random = random * 1_000_000_000.0;
        let random = random - random.floor();

        random
    }

    pub fn gen_range(min: f64, max: f64) -> f64 {
        let random = Self::gen();
        random * (max - min) + min
    }

    pub fn gen_range_int(min: i64, max: i64) -> i64 {
        let random = Self::gen();
        (random * (max - min) as f64 + min as f64).floor() as i64
    }

    pub fn gen_range_uint(min: u64, max: u64) -> u64 {
        let random = Self::gen();
        (random * (max - min) as f64 + min as f64).floor() as u64
    }

    pub fn gen_bool() -> bool {
        let random = Self::gen();
        random > 0.5
    }

    pub fn gen_string(length: usize, charset: &str) -> String {
        let charset = charset.chars().collect::<Vec<char>>();
        let charset_len = charset.len();
        let mut result = String::new();

        for _ in 0..length {
            let random = Self::gen_range_uint(0, charset_len as u64);
            result.push(charset[random as usize]);
        }

        result
    }

    pub fn alphabet() -> &'static str {
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
    }

    pub fn numbers() -> &'static str {
        "0123456789"
    }

    pub fn alphanumeric() -> &'static str {
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
    }

    pub fn ascii() -> &'static str {
        "!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~"
    }

    pub fn lowercase() -> &'static str {
        "abcdefghijklmnopqrstuvwxyz"
    }

    pub fn uppercase() -> &'static str {
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
    }

    pub fn hex() -> &'static str {
        "0123456789abcdef"
    }

    pub fn all() -> &'static str {
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~"
    }
}
