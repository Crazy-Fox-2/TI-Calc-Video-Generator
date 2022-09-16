


struct AudInstr {

    rel: bool,
    next_rel: bool,
    samp: u8,
    offset: Option<[i8; 2]>,
    pos: usize,
    byte: u8,
    
}
impl AudInstr {

    fn new(rel: bool, samp: u8, offset: Option<[i8; 2]>, pos: usize) -> AudInstr {
        AudInstr {rel: rel, next_rel: false, samp: samp, offset: offset, pos: pos, byte: 0}
    }

    fn gen_byte(&mut self) -> u8 {
        self.byte = match self.rel {
            true => {
                let offset = self.offset.unwrap();
                ((offset[0] * 2) + (offset[1] % 4)) as u8
            },
            false => {
                match self.next_rel {
                    true => 128 + self.samp,
                    false => self.samp
                }
            }
        };
        self.byte
    }

}


pub struct Aud {
    
    pub data: Vec<u8>,
    comp: Option<Vec<AudInstr>>,
    
}
impl Aud {
    
    pub fn new(data: Vec<u8>) -> Aud {
        Aud {data: data, comp: None}
    }

    pub fn compress(&mut self) {

        // Compress audio samples
        let mut prev_samp: u8 = 196;    // Guarenteed out of range
        let mut offsets: Vec<i8> = Vec::new();
        let mut prev_rel = false;
        let mut comp: Vec<AudInstr> = Vec::new();
        
        let mut pos = 0;
        let mut pos_start = 0;
        for samp in self.data.iter() {
            // Check difference in samples
            let diff: i8 = (*samp as i8) - (prev_samp as i8);
            if -8 < diff && diff < 8 {
                if !prev_rel {
                    pos_start = pos;
                    prev_rel = true;
                }
                // Flag previous instruction that the next will be relative
                let len = comp.len();
                comp[len-1].next_rel = true;
                // Add this difference to the vector
                offsets.push(diff);
                // And if it's 2 large add a new instruction
                if offsets.len() >= 2 {
                    comp.push(AudInstr::new(true, 0, Some(offsets.try_into().unwrap()), pos_start));
                    offsets = Vec::new();
                }
            } else {
                // If previous was relative add offset 8 to signal switching to absolute
                if prev_rel {
                    offsets.push(8);
                    while offsets.len() < 2 {
                        offsets.push(8);
                    }
                    comp.push(AudInstr::new(true, 0, Some(offsets.try_into().unwrap()), pos_start));
                    offsets = Vec::new();
                    prev_rel = false;
                }
                // Add sample
                comp.push(AudInstr::new(false, *samp, None, pos));
            }
            pos += 1;
            prev_samp = *samp;
        }
        // Put dangling offsets
        if offsets.len() > 0 {
            while offsets.len() < 2 {
                offsets.push(0);
            }
            comp.push(AudInstr::new(true, 0, Some(offsets.try_into().unwrap()), pos_start));
        }

    }

    pub fn output(&mut self) -> Vec<u8> {
        // Convert our instructions into one long byte stream
        let mut bstream: Vec<u8> = Vec::new();
        for comp in self.comp.iter_mut() {
            for instr in comp.iter_mut() {
                bstream.push(instr.gen_byte());
            }
        }
        bstream
    }


}
            







