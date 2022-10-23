
// This compression routine assumes valid numbers range from 0-127
// Compression data will look as follows:
// Starts in absolute mode, in which each value is just the read byte without the msb
// If the msb is set, then after this sample we switch to relative mode
// In relative mode each nybble represents an offset from the previous vale to be applied
// Most significant nybble is read first. A nybble of 0 mreans no change, 8 switch back to absolute
// 
// Used for the audio compression



fn putoffsets(comp: &mut Vec<u8>, offsets: &mut Vec<i8>) {
    let ub = (((offsets[0] as i16) * 16) as u8) & 0xF0;
    let lb = (offsets[1] as u8) & 0x0F;
    comp.push(ub | lb);
}


pub fn compress(data: &[u8]) -> Vec<u8> {

    // Compress audio samples
    let mut prev_samp: u8 = 196;    // Guarenteed out of range
    let mut offsets: Vec<i8> = Vec::new();
    let mut prev_rel = false;
    let mut comp: Vec<u8> = Vec::new();
    
    for samp in data.iter() {
        //if *samp > 127 {
        //    println!("{}", *samp);
        //}
        // Check difference in samples
        let diff: i16 = (*samp as i16) - (prev_samp as i16);
        if -8 < diff && diff < 8 {
            if !prev_rel {
                prev_rel = true;
                // Flag previous instruction that the next will be relative
                let len = comp.len();
                comp[len-1] |= 0x80;
            }
            // Add this difference to the vector
            offsets.push(diff as i8);
            // And if it's 2 large add a new instruction
            if offsets.len() >= 2 {
                putoffsets(&mut comp, &mut offsets);
                offsets = Vec::new();
            }
        } else {
            // If previous was relative add offset 8 to signal switching to absolute
            if prev_rel {
                while offsets.len() < 2 {
                    offsets.push(8);
                }
                putoffsets(&mut comp, &mut offsets);
                offsets = Vec::new();
                prev_rel = false;
            }
            // Add sample
            comp.push(*samp & 0x7F);
        }
        prev_samp = *samp;
    }
    // Put dangling offsets
    if offsets.len() > 0 {
        while offsets.len() < 2 {
            offsets.push(0);
        }
        putoffsets(&mut comp, &mut offsets);
    }
    
    comp

}


