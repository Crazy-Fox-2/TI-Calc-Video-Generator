
// This compression routine assumes valid numbers range from 0-127
// Right now just encodes the differences and nothing else
// 
// Used for the audio compression




pub fn compress(data: &[u8]) -> Vec<u8> {

    // Compress audio samples
    let mut prev_samp: u8 = 64;
    let mut offsets: Vec<i8> = Vec::new();
    let mut prev_rel = false;
    let mut comp: Vec<u8> = Vec::new();
    
    for samp in data.iter() {
        let samp = *samp / 2;
        let diff: i16 = (samp as i16) - (prev_samp as i16);
        comp.push(diff as u8);
        prev_samp = samp;
    }
    
    
    comp

}


