
// This compression routine assumes valid numbers range from 0-127
// Compression data will look as follows:
// Starts in absolute mode, in which each value is just the read byte without the msb
// If the msb is set, then after this sample we switch to relative mode
// In relative mode each nybble represents an offset from the previous vale to be applied
// Most significant nybble is read first. A nybble of 0 mreans no change, 8 switch back to absolute
// 
// Used for the audio compression



fn putbit(comp: &mut Vec<u8>, curbyte: &mut u8, bitpos: &mut usize, bit: u8) {
    *curbyte = (*curbyte * 2) + (bit % 2);
    *bitpos += 1;
    if *bitpos >= 8 {
        comp.push(*curbyte);
        *curbyte = 0;
        *bitpos = 0;
    }
}
fn finishbyte(comp: &mut Vec<u8>, curbyte: &mut u8, bitpos: &mut usize) {
    if *bitpos > 0 {
        while *bitpos < 8 {
            *curbyte = *curbyte * 2;
            *bitpos += 1;
        }
        comp.push(*curbyte);
    }
}


pub fn compress(data: Vec<u8>) -> Vec<u8> {

    // Compress audio samples
    let mut prev_samp: u8 = 196;    // Guarenteed out of range
    let mut prev_diff: i16 = 0;
    let mut comp: Vec<u8> = Vec::new();
    let mut byte = 0;
    let mut bpos = 0;
    
    for samp in data.iter() {
        let samp = *samp;
        // Check difference in samples
        let diff: i16 = (samp as i16) - (prev_samp as i16);
        let accel = diff - prev_diff;
        let neg_accel = -diff - prev_diff;
        if accel == 0 {
            // No change in acceleration
            putbit(&mut comp, &mut byte, &mut bpos, 0);
            putbit(&mut comp, &mut byte, &mut bpos, 0);
        } else if (accel > -8) && (accel < 8) {
            // Normal acceleration
            let aput = accel as u8;
            putbit(&mut comp, &mut byte, &mut bpos, 1);
            putbit(&mut comp, &mut byte, &mut bpos, 0);
            putbit(&mut comp, &mut byte, &mut bpos, (aput / 8) % 2);
            putbit(&mut comp, &mut byte, &mut bpos, (aput / 4) % 2);
            putbit(&mut comp, &mut byte, &mut bpos, (aput / 2) % 2);
            putbit(&mut comp, &mut byte, &mut bpos, aput % 2);
        } else if (neg_accel > -8) && (neg_accel < 8) {
            // Aacceleration off of negative difference
            let aput = neg_accel as u8;
            putbit(&mut comp, &mut byte, &mut bpos, 1);
            putbit(&mut comp, &mut byte, &mut bpos, 1);
            putbit(&mut comp, &mut byte, &mut bpos, (aput / 8) % 2);
            putbit(&mut comp, &mut byte, &mut bpos, (aput / 4) % 2);
            putbit(&mut comp, &mut byte, &mut bpos, (aput / 2) % 2);
            putbit(&mut comp, &mut byte, &mut bpos, aput % 2);
        } else {
            // Put full sample
            let samp = samp / 2;
            putbit(&mut comp, &mut byte, &mut bpos, 0);
            putbit(&mut comp, &mut byte, &mut bpos, 1);
            putbit(&mut comp, &mut byte, &mut bpos, (samp / 32) % 2);
            putbit(&mut comp, &mut byte, &mut bpos, (samp / 16) % 2);
            putbit(&mut comp, &mut byte, &mut bpos, (samp / 8) % 2);
            putbit(&mut comp, &mut byte, &mut bpos, (samp / 4) % 2);
            putbit(&mut comp, &mut byte, &mut bpos, (samp / 2) % 2);
            putbit(&mut comp, &mut byte, &mut bpos, samp % 2);
        }
        
        prev_diff = diff;
        prev_samp = samp;
    }
    finishbyte(&mut comp, &mut byte, &mut bpos);
    
    comp

}


