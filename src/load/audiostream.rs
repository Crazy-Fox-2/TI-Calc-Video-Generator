use std::io::{Read, BufReader, Bytes};
use std::fs::File;
use crate::helper::macros::passerr;


// Simple iterator over all the samples in a wave file



pub struct AudIter {
    bytes: Bytes<BufReader<File>>
}
impl Iterator for AudIter {
    type Item = Vec<u8>;
    
    fn next(&mut self) -> Option<Self::Item> {
        // Read 546 samples
        let mut samps: Vec<u8> = Vec::with_capacity(546);
        for _i in 0..546 {
            // Read sample from bytestream
            self.bytes.next();      // Ignore least-significant byte
            let samp = match self.bytes.next() {
                Some(s) => (s.unwrap() as i8 as i16) + 128,
                None => return None,
            } / 2;
            samps.push(samp as u8);
        }
        Some(samps)
    }
}

impl AudIter {
    pub fn new(fname: &str) -> Result<AudIter, String> {
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
        Ok(AudIter {bytes: bytes})
    }
}





