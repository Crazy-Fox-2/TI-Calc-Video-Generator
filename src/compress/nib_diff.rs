use crate::compress::instr::{Instr};
use crate::compress::cycle_limit::{CycleInstr};


// 
// 
// Used for the audio compression




pub fn compress(data: &[u8], start: u8) -> (Vec<Box<dyn CycleInstr>>, u8) {

    // Compress audio samples
    let mut prev_samp = start / 2;
    let mut prev_nib = false;
    let mut instrs: Vec<Box<dyn CycleInstr>> = Vec::new();
    let mut diffs: Vec<u8> = Vec::new();
    let mut first = true;
    
    for samp in data.iter() {
        let samp = *samp / 2;
        let diff: i16 = (samp as i16) - (prev_samp as i16);
        if diff > -8 && diff < 8 && !first {
            if !prev_nib {
                instrs.push(Box::new(ByteInstr{diffs: diffs}));
                diffs = Vec::new();
                prev_nib = true;
            }
            diffs.push(diff as u8);
        } else {
            if prev_nib {
                instrs.push(Box::new(NibbleInstr{diffs: diffs}));
                diffs = Vec::new();
                prev_nib = false;
            }
            diffs.push(diff as u8);
        }
        prev_samp = samp;
        first = false;
    }
    if prev_nib {
        instrs.push(Box::new(NibbleInstr{diffs: diffs}));
    } else {
        instrs.push(Box::new(ByteInstr{diffs: diffs}));
    }
    
    (instrs, prev_samp*2)
    
}




/*          Byte Difference         */

#[derive(Clone)]
struct ByteInstr {
    diffs: Vec<u8>,
}
impl Instr for ByteInstr {
    fn gen_bytecode(&self, _last: bool) -> Vec<u8> {
        let mut bytecode: Vec<u8> = Vec::with_capacity(self.diffs.len());
        for diff in &self.diffs {
            bytecode.push(diff << 1);
        }
        let len = bytecode.len();
        bytecode[len-1] += 1;
        bytecode
    }
    fn get_comp_size(&self) -> usize {
        self.diffs.len()
    }
    fn get_decomp_size(&self) -> usize {
        self.diffs.len()
    }
    fn get_decomp(&self) -> Vec<u8> {
        self.diffs.clone()
    }
}
impl CycleInstr for ByteInstr {
    fn get_cycles(&self) -> usize {
        35 * self.diffs.len() + 5
    }
    fn is_minimum(&self) -> bool {
        true
    }
    fn to_minimum(&self) -> Box<dyn CycleInstr> {
        Box::new(Self::clone(self))
    }
    fn combine_with_left(&mut self, other: &dyn CycleInstr) {
        let mut new = other.get_decomp().to_vec();
        new.append(&mut self.diffs);
        self.diffs = new;
    }
    fn combine_with_right(&mut self, other: &dyn CycleInstr) {
        self.diffs.append(&mut other.get_decomp());
    }
}



/*          Nibble Difference         */

#[derive(Clone)]
struct NibbleInstr {
    diffs: Vec<u8>,
}
impl Instr for NibbleInstr {
    fn gen_bytecode(&self, last: bool) -> Vec<u8> {
        let mut bytecode: Vec<u8> = Vec::with_capacity(self.diffs.len()/2);
        let mut byte = 0;   let mut parity = 0;
        for diff in &self.diffs {
            if parity == 0 {
                byte = diff & 0x0F;
                parity = 1;
            } else {
                byte += (diff << 4) & 0xF0;
                bytecode.push(byte);
                parity = 0;
            }
        }
        if parity == 0 {
            if !last {
                byte = 0x88;
                bytecode.push(byte);
            }
        } else {
            byte += 0x80;
            bytecode.push(byte);
        }
        //let len = bytecode.len();
        //bytecode[len-1] += 1;
        bytecode
    }
    fn get_comp_size(&self) -> usize {
        self.diffs.len() / 2
    }
    fn get_decomp_size(&self) -> usize {
        self.diffs.len()
    }
    fn get_decomp(&self) -> Vec<u8> {
        self.diffs.clone()
    }
}
impl CycleInstr for NibbleInstr {
    fn get_cycles(&self) -> usize {
        let mut cycles = 0;
        let iter = &mut self.diffs.iter();
        loop {
            match iter.next() {
                Some(&b) => cycles += 38 + match b < 128 /* b >= 0 */ {
                    true => 12,
                    false => 14,
                } + 26,
                None => {cycles += 43; break},
            }
            match iter.next() {
                Some(_b) => cycles += 76,
                None => {cycles += 55; break},
            }
        }
        cycles
    }
    fn is_minimum(&self) -> bool {
        false
    }
    fn to_minimum(&self) -> Box<dyn CycleInstr> {
        Box::new(ByteInstr {diffs: self.diffs.clone()})
    }
    fn combine_with_left(&mut self, _other: &dyn CycleInstr) {}
    fn combine_with_right(&mut self, _other: &dyn CycleInstr) {}
}

