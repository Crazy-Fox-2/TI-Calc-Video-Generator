use std::io::{Read, BufReader, Bytes};
use std::fs::File;
use crate::helper::macros::{passerr, bound};
use crate::helper::funcs::redist_range;


// Simple iterator over all the samples in a wave file



pub struct AudIter {
    bytes: Bytes<BufReader<File>>,
    range_low: i16,
    range_high: i16,
}
impl Iterator for AudIter {
    type Item = Vec<u8>;
    
    fn next(&mut self) -> Option<Self::Item> {
        // Read 512 samples
        let mut samps: Vec<u8> = Vec::with_capacity(512);
        for _i in 0..512 {
            // Read sample from bytestream
            self.bytes.next();      // Ignore least-significant byte
            let mut signed_samp = match self.bytes.next() {
                Some(s) => s.unwrap() as i8 as i16,
                None => return Some(vec![((self.range_low + self.range_high) / 2) as u8; 512]),
            };
            //println!("{}", signed_samp);
            signed_samp *= 2;
            signed_samp = bound!(signed_samp, -128, 128);
            let samp = redist_range(signed_samp as f64, -128.0, 128.0, self.range_low as f64, self.range_high as f64) as u8;
            samps.push(samp);
        }
        Some(samps)
    }
}

impl AudIter {
    pub fn new(fname: &str, range_low: i16, range_high: i16) -> Result<AudIter, String> {
        // Load wave file provided
        let f = passerr!(File::open(fname));
        let mut bytes = BufReader::new(f).bytes();
        // Look for start of data
        loop {
            if bytes.next().unwrap().unwrap() == 0x64 {
                if bytes.next().unwrap().unwrap() == 0x61 {
                    if bytes.next().unwrap().unwrap() == 0x74 {
                        if bytes.next().unwrap().unwrap() == 0x61 {
                            break;
                        }
                    }
                }
            }
        }
        // Skip next 4 bytes
        for _ in 0..4 {
            bytes.next();
        }
        Ok(AudIter {bytes: bytes, range_low: range_low, range_high: range_high})
    }
}





