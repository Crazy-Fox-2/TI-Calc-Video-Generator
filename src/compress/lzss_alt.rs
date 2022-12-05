use std::cmp;
use crate::compress::graph_solve;
use crate::compress::graph_solve::{GraphFuncs};
use crate::compress::instr::{Instr, InstrGen};
use crate::compress::cycle_limit::{CycleInstr};
//use num::integer::Integer;

pub fn compress(data: &[u8]) -> Vec<Box<dyn CycleInstr>> {
    
    graph_solve::compress(data, 
            vec![Box::new(STREAM_INSTRGEN),   Box::new(LZSS_INSTRGEN),   Box::new(ALTFLIP_INSTRGEN),   Box::new(ALTWHITE_INSTRGEN),   Box::new(ALTBLACK_INSTRGEN)],
            vec![Box::new(STREAM_GRAPHFUNCS), Box::new(LZSS_GRAPHFUNCS), Box::new(ALTFLIP_GRAPHFUNCS), Box::new(ALTWHITE_GRAPHFUNCS), Box::new(ALTBLACK_GRAPHFUNCS)])
    
}



/*          LZSS Instruction        */

const LZSS_MAX_LEN: usize = 32;

const LZSS_INSTRGEN: LZSSInstrGen = LZSSInstrGen{};
struct LZSSInstrGen {
}
impl InstrGen<dyn CycleInstr> for LZSSInstrGen {
    fn gen_instr(&self, data: &[u8], pos: usize, offset: usize, len: usize) -> Box<dyn CycleInstr> {
        let decomp: Vec<u8> = data[pos..pos+len].to_vec();
        let numsplit = num::Integer::div_ceil(&len, &LZSS_MAX_LEN);
        Box::new(LZSSInstr {decomp: decomp, offset: offset, len: len, numsplit: numsplit})
    }
}

struct LZSSInstr {
    decomp: Vec<u8>,
    offset: usize,
    len: usize,
    numsplit: usize,
}
impl Instr for LZSSInstr {
    fn gen_bytecode(&self, last: bool) -> Vec<u8> {
        // Generate lzss bytecode
        let mut bytecode: Vec<u8> = Vec::new();
        // Figure out how many instructions we'll need to use and the lengths of each one
        let max_put_len = num::Integer::div_ceil(&self.len, &self.numsplit);
        let mut len = self.len;
        for _i in 0..self.numsplit {
            let put_len = cmp::min(len, max_put_len);
            bytecode.push(((put_len - 1) as u8) << 3);
            // Offset stored in 1 or 2 bytes depending on size
            if self.offset >= 128 {
                bytecode.push(((self.offset & 0x7F00) >> 7) as u8 + 1);
                bytecode.push(self.offset as u8);
            } else {
                bytecode.push((self.offset as u8) << 1);
            }
            len -= put_len;
        }
        if last {
            bytecode.push(0x00);
        }
        bytecode
    }
    fn get_comp_size(&self) -> usize {
        // Calculate size of instruction
        self.numsplit * match self.offset >= 128 {
            false => 2,
            true => 3,
        }
    }
    fn get_decomp_size(&self) -> usize {
        self.len
    }
    fn get_decomp(&self) -> Vec<u8> {
        self.decomp.clone()
    }
}
impl CycleInstr for LZSSInstr {
    fn get_cycles(&self) -> usize {
        (55 + 40 + match self.offset >= 128 {
            false => 20,
            true => 36,
        } + 58 + 20) * self.numsplit + (21 * self.len - 5 * self.numsplit)
    }
    fn is_minimum(&self) -> bool {
        false
    }
    fn to_minimum(&self) -> Box<dyn CycleInstr> {
        // Generate stream instruction from our data
        STREAM_INSTRGEN.gen_instr(&self.decomp, 0, 0, self.len)
    }
    fn combine_with_right(&mut self, _other: &dyn CycleInstr) {}
    fn combine_with_left(&mut self, _other: &dyn CycleInstr) {}
}

const LZSS_GRAPHFUNCS: LZSSGraphFuncs = LZSSGraphFuncs{};
struct LZSSGraphFuncs {
}
impl GraphFuncs for LZSSGraphFuncs {
    fn get_instr_info(&self, data: &[u8], pos: usize) -> Vec<(usize, usize)> {   // Returns Vec<len, id>
        // Search through data stream for all occurances that match our current position (over a
        // certain length)
        let mut list: Vec<(usize, usize)> = Vec::new();
        for from_start in 0..pos {
            let mut len = 0;
            for to in pos..data.len() {
                let from = (to - pos) + from_start;
                if data[to] == data[from] {
                    len += 1;
                } else {
                    break;
                }
            }
            if len >= 2 {
                let offset = pos - from_start;  // offset will be the unique identifier
                list.push((len, offset));
                // Make sure not to overload the graph with LZSS instructions
                if list.len() >= 8 {
                    break;
                }
            }
        }
        list
    }
    fn get_step_cost(&self, _data: &[u8], _pos: usize, _uid: usize) -> isize {
        0
    }
    fn get_entry_cost(&self, _data: &[u8], _pos: usize, off: usize) -> isize {
        match off >= 128 {
            true => 3,
            false => 2,
        }
    }
    fn get_cont_cost(&self, data: &[u8], pos: usize, uid: usize, rel_pos: usize) -> isize {
        match rel_pos % LZSS_MAX_LEN {
            0 => self.get_entry_cost(data, pos, uid),
            _ => 0
        }
    }
}








/*          Stream Instruction      */

const STREAM_MAX_LEN: usize = 64;

const STREAM_INSTRGEN: StreamInstrGen = StreamInstrGen{};
struct StreamInstrGen {
}
impl InstrGen<dyn CycleInstr> for StreamInstrGen {
    fn gen_instr(&self, data: &[u8], pos: usize, _: usize, len: usize) -> Box<dyn CycleInstr> {
        // Copy data straight with no compression
        let stream: Vec<u8> = data[pos..pos+len].to_vec();
        let numsplit = num::Integer::div_ceil(&len, &STREAM_MAX_LEN);
        Box::new(StreamInstr {stream: stream, len: len, numsplit: numsplit})
    }
}

#[derive(Clone)]
struct StreamInstr {
    stream: Vec<u8>,
    len: usize,
    numsplit: usize,
}
impl Instr for StreamInstr {
    fn gen_bytecode(&self, last: bool) -> Vec<u8> {
        // Generate stream bytecode
        let mut bytecode: Vec<u8> = Vec::new();
        // Figure out how many instructions we'll need to use and the lengths of each one
        let max_put_len = num::Integer::div_ceil(&self.len, &self.numsplit);
        let mut spos = 0;
        let mut len = self.len;
        for _i in 0..self.numsplit {
            let put_len = cmp::min(len, max_put_len);
            bytecode.push((((put_len - 1) as u8) << 2) + 0x02);
            // Write stream
            for _j in 0..put_len {
                bytecode.push(self.stream[spos]);
                spos += 1;
            }
            len -= put_len;
        }
        if last {
            bytecode.push(0x00);
        }
        bytecode
    }
    fn get_comp_size(&self) -> usize {
        // Calculate size of instruction
        self.numsplit + self.len
    }
    fn get_decomp_size(&self) -> usize {
        self.len
    }
    fn get_decomp(&self) -> Vec<u8> {
        self.stream.clone()
    }
}
impl CycleInstr for StreamInstr {
    fn get_cycles(&self) -> usize {
        69 * self.numsplit + (21 * self.len - 5 * self.numsplit)
    }
    fn is_minimum(&self) -> bool {
        true
    }
    fn to_minimum(&self) -> Box<dyn CycleInstr> {
        Box::new(Self::clone(self))
    }
    fn combine_with_left(&mut self, other: &dyn CycleInstr) {
        let mut new_stream = other.get_decomp().to_vec();
        new_stream.append(&mut self.stream);
        self.len = new_stream.len();
        self.numsplit = num::Integer::div_ceil(&self.len, &STREAM_MAX_LEN);
        self.stream = new_stream;
    }
    fn combine_with_right(&mut self, other: &dyn CycleInstr) {
        self.stream.append(&mut other.get_decomp().to_vec());
        self.len = self.stream.len();
        self.numsplit = num::Integer::div_ceil(&self.len, &STREAM_MAX_LEN);
    }
}

const STREAM_GRAPHFUNCS: StreamGraphFuncs = StreamGraphFuncs{};
struct StreamGraphFuncs {
}
impl GraphFuncs for StreamGraphFuncs {
    fn get_instr_info(&self, data: &[u8], _pos: usize) -> Vec<(usize, usize)> {   // Returns Vec<len, id>
        // Because the stream instruction is always available and arbitrarily long, this function
        // should only get called once at the start of the graph generation
        vec![(data.len(), 0)]
    }
    fn get_step_cost(&self, _data: &[u8], _pos: usize, _uid: usize) -> isize {
        1
    }
    fn get_entry_cost(&self, _data: &[u8], _pos: usize, _uid: usize) -> isize {
        1
    }
    fn get_cont_cost(&self, data: &[u8], pos: usize, uid: usize, rel_pos: usize) -> isize {
        match rel_pos % STREAM_MAX_LEN {
            0 => self.get_entry_cost(data, pos, uid),
            _ => 0
        }
    }
}








/*          Alt-Flip Instruction        */

const ALT_FLIP_MAX_LEN: usize = 32;

const ALTFLIP_INSTRGEN: AltFlipInstrGen = AltFlipInstrGen{};
struct AltFlipInstrGen {
}
impl InstrGen<dyn CycleInstr> for AltFlipInstrGen {
    fn gen_instr(&self, data: &[u8], pos: usize, parity: usize, len: usize) -> Box<dyn CycleInstr> {
        let decomp: Vec<u8> = data[pos..pos+len].to_vec();
        let numsplit = num::Integer::div_ceil(&len, &ALT_FLIP_MAX_LEN);
        Box::new(AltFlipInstr {decomp: decomp, parity: parity, pos: pos, len: len, numsplit: numsplit})
    }
}

struct AltFlipInstr {
    decomp: Vec<u8>,
    parity: usize,
    pos: usize,
    len: usize,
    numsplit: usize,
}
impl Instr for AltFlipInstr {
    fn gen_bytecode(&self, last: bool) -> Vec<u8> {
        // Generate alt-flip bytecode
        let mut bytecode: Vec<u8> = Vec::new();
        let max_put_len = ALT_FLIP_MAX_LEN;
        let mut pos = self.pos;
        let mut sub_pos = 0;
        let mut len = self.len;
        for _i in 0..self.numsplit {
            let put_len = cmp::min(len, max_put_len);
            bytecode.push((((put_len - 1) as u8) << 3) + 0x04);
            for &byte in &self.decomp[sub_pos..sub_pos+put_len] {
                if (pos % 2) == self.parity {
                    bytecode.push(byte);
                }
                pos += 1;
                sub_pos += 1;
            }
            len -= put_len;
        }
        if last {
            bytecode.push(0x00);
        }
        bytecode
    }
    fn get_comp_size(&self) -> usize {
        // Calculate size of instruction
        self.numsplit + match (self.pos % 2) == self.parity {
            true => self.len + 1,
            false => self.len,
        } / 2
    }
    fn get_decomp_size(&self) -> usize {
        self.len
    }
    fn get_decomp(&self) -> Vec<u8> {
        self.decomp.clone()
    }
}
impl CycleInstr for AltFlipInstr {
    fn get_cycles(&self) -> usize {
        let mut cycles = 0;
        let mut len = self.len;
        for _i in 0..self.numsplit {
            let put_len = cmp::min(len, ALT_FLIP_MAX_LEN);
            cycles += 70 + match put_len % 2 == 0 {
                true => 67 * (put_len / 2) - 5,
                false => 42 + 67 * (put_len / 2),
            };
            len -= put_len;
        }
        cycles
    }
    fn is_minimum(&self) -> bool {
        false
    }
    fn to_minimum(&self) -> Box<dyn CycleInstr> {
        // Generate stream instruction from our data
        STREAM_INSTRGEN.gen_instr(&self.decomp, 0, 0, self.len)
    }
    fn combine_with_right(&mut self, _other: &dyn CycleInstr) {}
    fn combine_with_left(&mut self, _other: &dyn CycleInstr) {}
}

const ALTFLIP_GRAPHFUNCS: AltFlipGraphFuncs = AltFlipGraphFuncs{};
struct AltFlipGraphFuncs {
}
impl GraphFuncs for AltFlipGraphFuncs {
    fn get_instr_info(&self, data: &[u8], pos: usize) -> Vec<(usize, usize)> {   // Returns Vec<len, id>
        let mut list: Vec<(usize, usize)> = Vec::new();
        match search_alt(data, pos, 'f', false) {
            Some(len) => list.push((len, pos % 2)),
            None => {}
        };
        list
    }
    fn get_step_cost(&self, _data: &[u8], pos: usize, uid: usize) -> isize {
        match pos % 2 == uid {
            true => 1,
            false => 0
        }
    }
    fn get_entry_cost(&self, _data: &[u8], pos: usize, uid: usize) -> isize {
        match pos % 2 == uid {
            true => 1,
            false => 999999,
        }
    }
    fn get_cont_cost(&self, data: &[u8], pos: usize, uid: usize, rel_pos: usize) -> isize {
        match rel_pos % ALT_FLIP_MAX_LEN {
            0 => self.get_entry_cost(data, pos, uid),
            _ => 0
        }
    }
}








/*          Alt-White Instruction        */

const ALT_WHITE_MAX_LEN: usize = 32;

const ALTWHITE_INSTRGEN: AltWhiteInstrGen = AltWhiteInstrGen{};
struct AltWhiteInstrGen {
}
impl InstrGen<dyn CycleInstr> for AltWhiteInstrGen {
    fn gen_instr(&self, data: &[u8], pos: usize, parity: usize, len: usize) -> Box<dyn CycleInstr> {
        let decomp: Vec<u8> = data[pos..pos+len].to_vec();
        let numsplit = num::Integer::div_ceil(&len, &ALT_WHITE_MAX_LEN);
        Box::new(AltWhiteInstr {decomp: decomp, parity: parity, pos: pos, len: len, numsplit: numsplit})
    }
}

struct AltWhiteInstr {
    decomp: Vec<u8>,
    parity: usize,
    pos: usize,
    len: usize,
    numsplit: usize,
}
impl Instr for AltWhiteInstr {
    fn gen_bytecode(&self, last: bool) -> Vec<u8> {
        // Generate alt-white bytecode
        let mut bytecode: Vec<u8> = Vec::new();
        let max_put_len = ALT_WHITE_MAX_LEN;
        let mut pos = self.pos;
        let mut sub_pos = 0;
        let mut len = self.len;
        for _i in 0..self.numsplit {
            let put_len = cmp::min(len, max_put_len);
            bytecode.push((((put_len - 1) as u8) << 3) + 0x01 + ((((self.parity+pos)%2) as u8) << 2));
            for &byte in &self.decomp[sub_pos..sub_pos+put_len] {
                if (pos % 2) == self.parity {
                    bytecode.push(byte);
                }
                pos += 1;
                sub_pos += 1;
            }
            len -= put_len;
        }
        if last {
            bytecode.push(0x00);
        }
        bytecode
    }
    fn get_comp_size(&self) -> usize {
        // Calculate size of instruction
        self.numsplit + match (self.pos % 2) == self.parity {
            true => self.len + 1,
            false => self.len,
        } / 2
    }
    fn get_decomp_size(&self) -> usize {
        self.len
    }
    fn get_decomp(&self) -> Vec<u8> {
        self.decomp.clone()
    }
}
impl CycleInstr for AltWhiteInstr {
    fn get_cycles(&self) -> usize {
        let mut cycles = 0;
        let mut len = self.len;
        for _i in 0..self.numsplit {
            let put_len = cmp::min(len, ALT_WHITE_MAX_LEN);
            cycles += 68 + 56 * (put_len / 2) + ((match self.pos + self.parity % 2 {
                0 => (match put_len % 2 {
                    1 => 42,
                    _ => -5,
                }) + 7,
                _ => (match put_len % 2 {
                    1 => 21,
                    _ => 12,
                }) + 12,
            } + 14) as usize);
            len -= put_len;
        }
        cycles
    }
    fn is_minimum(&self) -> bool {
        false
    }
    fn to_minimum(&self) -> Box<dyn CycleInstr> {
        // Generate stream instruction from our data
        STREAM_INSTRGEN.gen_instr(&self.decomp, 0, 0, self.len)
    }
    fn combine_with_right(&mut self, _other: &dyn CycleInstr) {}
    fn combine_with_left(&mut self, _other: &dyn CycleInstr) {}
}

const ALTWHITE_GRAPHFUNCS: AltWhiteGraphFuncs = AltWhiteGraphFuncs{};
struct AltWhiteGraphFuncs {
}
impl GraphFuncs for AltWhiteGraphFuncs {
    fn get_instr_info(&self, data: &[u8], pos: usize) -> Vec<(usize, usize)> {   // Returns Vec<len, id>
        let mut list: Vec<(usize, usize)> = Vec::new();
        match search_alt(data, pos, 'w', false) {
            Some(len) => list.push((len, pos % 2)),
            None => {}
        };
        match search_alt(data, pos, 'w', true) {
            Some(len) => list.push((len, (pos + 1) % 2)),
            None => {}
        };
        list
    }
    fn get_step_cost(&self, _data: &[u8], pos: usize, uid: usize) -> isize {
        match pos % 2 == uid {
            true => 1,
            false => 0
        }
    }
    fn get_entry_cost(&self, _data: &[u8], _pos: usize, _uid: usize) -> isize {
        1
    }
    fn get_cont_cost(&self, data: &[u8], pos: usize, uid: usize, rel_pos: usize) -> isize {
        match rel_pos % ALT_WHITE_MAX_LEN {
            0 => self.get_entry_cost(data, pos, uid),
            _ => 0
        }
    }
}








/*          Alt-Black Instruction        */

const ALT_BLACK_MAX_LEN: usize = 32;

const ALTBLACK_INSTRGEN: AltBlackInstrGen = AltBlackInstrGen{};
struct AltBlackInstrGen {
}
impl InstrGen<dyn CycleInstr> for AltBlackInstrGen {
    fn gen_instr(&self, data: &[u8], pos: usize, parity: usize, len: usize) -> Box<dyn CycleInstr> {
        let decomp: Vec<u8> = data[pos..pos+len].to_vec();
        let numsplit = num::Integer::div_ceil(&len, &ALT_BLACK_MAX_LEN);
        Box::new(AltBlackInstr {decomp: decomp, parity: parity, pos: pos, len: len, numsplit: numsplit})
    }
}

struct AltBlackInstr {
    decomp: Vec<u8>,
    parity: usize,
    pos: usize,
    len: usize,
    numsplit: usize,
}
impl Instr for AltBlackInstr {
    fn gen_bytecode(&self, last: bool) -> Vec<u8> {
        // Generate alt-white bytecode
        let mut bytecode: Vec<u8> = Vec::new();
        let max_put_len = ALT_BLACK_MAX_LEN;
        let mut pos = self.pos;
        let mut sub_pos = 0;
        let mut len = self.len;
        for _i in 0..self.numsplit {
            let put_len = cmp::min(len, max_put_len);
            bytecode.push((((put_len - 1) as u8) << 3) + 0x03 + ((((self.parity+pos)%2) as u8) << 2));
            for byte in &self.decomp[sub_pos..sub_pos+put_len] {
                if (pos % 2) == self.parity {
                    bytecode.push(*byte);
                }
                pos += 1;
                sub_pos += 1;
            }
            len -= put_len;
        }
        if last {
            bytecode.push(0x00);
        }
        bytecode
    }
    fn get_comp_size(&self) -> usize {
        // Calculate size of instruction
        self.numsplit + match (self.pos % 2) == self.parity {
            true => self.len + 1,
            false => self.len,
        } / 2
    }
    fn get_decomp_size(&self) -> usize {
        self.len
    }
    fn get_decomp(&self) -> Vec<u8> {
        self.decomp.clone()
    }
}
impl CycleInstr for AltBlackInstr {
    fn get_cycles(&self) -> usize {
        let mut cycles = 0;
        let mut len = self.len;
        for _i in 0..self.numsplit {
            let put_len = cmp::min(len, ALT_BLACK_MAX_LEN);
            cycles += 73 + 56 * (put_len / 2) + ((match self.pos + self.parity % 2 {
                0 => (match put_len % 2 {
                    1 => 42,
                    _ => -5,
                }) + 7,
                _ => (match put_len % 2 {
                    1 => 21,
                    _ => 12,
                }) + 12,
            } + 14) as usize);
            len -= put_len;
        }
        cycles
    }
    fn is_minimum(&self) -> bool {
        false
    }
    fn to_minimum(&self) -> Box<dyn CycleInstr> {
        // Generate stream instruction from our data
        STREAM_INSTRGEN.gen_instr(&self.decomp, 0, 0, self.len)
    }
    fn combine_with_right(&mut self, _other: &dyn CycleInstr) {}
    fn combine_with_left(&mut self, _other: &dyn CycleInstr) {}
}

const ALTBLACK_GRAPHFUNCS: AltBlackGraphFuncs = AltBlackGraphFuncs{};
struct AltBlackGraphFuncs {
}
impl GraphFuncs for AltBlackGraphFuncs {
    fn get_instr_info(&self, data: &[u8], pos: usize) -> Vec<(usize, usize)> {   // Returns Vec<len, id>
        let mut list: Vec<(usize, usize)> = Vec::new();
        match search_alt(data, pos, 'b', false) {
            Some(len) => list.push((len, pos % 2)),
            None => ()
        };
        match search_alt(data, pos, 'b', true) {
            Some(len) => list.push((len, (pos + 1) % 2)),
            None => {}
        };
        list
    }
    fn get_step_cost(&self, _data: &[u8], pos: usize, uid: usize) -> isize {
        match pos % 2 == uid {
            true => 1,
            false => 0
        }
    }
    fn get_entry_cost(&self, _data: &[u8], _pos: usize, _uid: usize) -> isize {
        1
    }
    fn get_cont_cost(&self, data: &[u8], pos: usize, uid: usize, rel_pos: usize) -> isize {
        match rel_pos % ALT_BLACK_MAX_LEN {
            0 => self.get_entry_cost(data, pos, uid),
            _ => 0
        }
    }
}





fn search_alt(data: &[u8], mut pos: usize, typ: char, start_const: bool) -> Option<usize> {
    // Returns the size the alternating instruction can achieve
    let mut len = 0;
    let mut is_const = start_const;
    let mut const_val: u8 = match typ {
        'w' => 0x00,
        'b' => 0xFF,
        _ => 0,
    };
    loop {
        if pos >= data.len() {
            break;
        }
        let byte = data[pos];
        if is_const && byte != const_val {
            break;
        } else {
            if typ == 'f' {
                const_val = !byte;
            }
        }
        pos += 1;
        len += 1;
        is_const = !is_const;
    }
    if len >= 3 {
        Some(len)
    } else {
        None
    }
}


