
#[derive(Debug)]
struct LZSS {
    offset: usize
}
#[derive(Debug)]
struct Stream {
    stream: Vec<u8>
}
#[derive(Debug)]
struct Stream_Alt {
    stream: Vec<u8>,
    flip: bool,
    cbyte: u8,
    cstart: bool,
}

#[derive(Debug)]
enum Instr_Type {
    Lzss(LZSS),
    Stream(Stream),
    Stream_Alt(Stream_Alt),
}
use crate::compress::lzss_st::Instr_Type::*;

#[derive(Debug)]
struct Instr {
    kind: Instr_Type,
    len: usize,
    pos: usize,
    bytecode: Option<Vec<u8>>,
    cost: u32,
}
impl Instr {

    fn new(kind: Instr_Type, len: usize, pos: usize) -> Instr {
        Instr {kind: kind, len: len, pos: pos, bytecode: None, cost: 0}
    }
    /*
    fn gen_bytecode(&mut self) -> usize {
        // Generates bytecode for this instruction, returning the size of the bytecode
        let mut bytecode:Vec<u8> = Vec::new();
        let mut len = self.len as i32;
        let mut stream_ind = 0;
        while len > 0 {
            let mut put_len = len;
            if len > 128 {
                put_len = 128;
            }
            match self.lzss {
                true => {
                    // LZSS Instruction
                    bytecode.push(((put_len - 1) * 2) as u8);
                    if self.offset >= 128 {
                        bytecode.push(1 + (self.offset / 256 * 2) as u8);
                        bytecode.push((self.offset % 256) as u8);
                    } else {
                        bytecode.push(self.offset as u8);
                    }
                }, false => {
                    // Stream Instruction
                    bytecode.push(1 + ((put_len - 1) * 2) as u8);
                    let stream = self.stream.as_ref().unwrap();
                    for _i in 0..put_len {
                        bytecode.push(stream[stream_ind]);
                        stream_ind += 1;    // We're not using i as the index variable because it wouldn't work if the instruction gets split
                    }
                },
            }
            len -= 128;
        }
        // Save bytecode and return length
	let len = bytecode.len();
        self.bytecode = Some(bytecode);
        len
    }
    */

    fn get_cost(&mut self) -> u32 {
        // Get number of cycles this instruction will take to run
        0
    }
    
    /*
    fn combine_to_stream(&self, other: &Instr, data: &Vec<u8>) -> Instr {
        // Combines this and another instruction into a single stream instruction
        let mut stream: Vec<u8> = Vec::new();
        stream.extend_from_slice(&data[self.pos..self.pos+self.len]);
        stream.extend_from_slice(&data[other.pos..other.pos+other.len]);
        let instr = Instr::new(false, self.len + other.len, 0, Some(stream), self.pos);
        instr
    }
    */

}




pub fn compress(data: Vec<u8>) -> Vec<u8> {
    
    // Generate compressed stream
    let mut comp:Vec<Instr> = Vec::new();
    let dlen = data.len();

    // Step 1: generate all possible instructions we could have at each position
    let mut possible_instrs: Vec<Instr> = Vec::new();
    let (mut lzss_cd, mut sai_cd, mut sa0_cd, mut sa1_cd) = (0isize, 0isize, 0isize, 0isize);
    let mut ind = 0;
    while ind < dlen {
        
        // Search for largest lzss size we can get
        if lzss_cd <= 0 {
            let mut largest_off = 0;
            let mut largest_size = 0;
            for off in 1..(ind+1) {
                // Check how long this run is
                let mut cur_size = 0;
                for from_ind in (ind-off)..dlen {
                    let to_ind = from_ind + off;
                    if !(to_ind >= dlen) && data[from_ind] == data[to_ind] {
                        cur_size += 1;
                        if cur_size > largest_size {
                            largest_size = cur_size;
                            largest_off = off;
                        }
                    } else {
                        break;
                    }
                }
            }
            if largest_size > 1 {
                // Record LZSS instruction
                possible_instrs.push(Instr::new(Lzss(LZSS{offset: largest_off}), largest_size, ind));
                lzss_cd = largest_size as isize;
            }
        }
        lzss_cd -= 1;

        // Try and do alternating streams
        if sa0_cd <= 0 && ind < dlen - 1 {
            let instr = make_alt_instr(&data, ind, 0x00, false);
            match instr {
                Some(instr) => {
                    sa0_cd = instr.len as isize;
                    possible_instrs.push(instr);
                },
                None => {}
            }
        }
        sa0_cd -= 1;
        if sa1_cd <= 0 && ind < dlen - 1 {
            let instr = make_alt_instr(&data, ind, 0xFF, false);
            match instr {
                Some(instr) => {
                    sa1_cd = instr.len as isize;
                    possible_instrs.push(instr);
                },
                None => {}
            }
        }
        sa1_cd -= 1;
        if sai_cd <= 0 && ind < dlen - 1 {
            let instr = make_alt_instr(&data, ind, 7, true);
            match instr {
                Some(instr) => {
                    sai_cd = instr.len as isize;
                    possible_instrs.push(instr);
                },
                None => {}
            }
        }
        sai_cd -= 1;
        fn make_alt_instr(data: &Vec<u8>, pos: usize, cbyte: u8, flip: bool) -> Option<Instr> {
            // Attemt to construct the alternating instruction
            // Try starting with both constant and actual byte
            let (inst_startconst, c_len) = make_alt_instr_sub(data, pos, cbyte, flip, true);
            let (inst_startbyte, b_len) = make_alt_instr_sub(data, pos, cbyte, flip, false);
            if b_len > c_len || flip {
                return inst_startbyte;
            } else {
                return inst_startconst;
            }
            fn make_alt_instr_sub(data: &Vec<u8>, pos: usize, cbyte: u8, flip: bool, start: bool) -> (Option<Instr>, usize) {
                let mut isconst = start;
                let mut len = 0;
                let mut stream: Vec<u8> = Vec::new();
                let mut prev_byte = 0;
                for i in pos..data.len() {
                    let byte = data[i];
                    if isconst {
                        if flip {
                            if byte != !prev_byte {
                                break;
                            }
                        } else {
                            if byte != cbyte {
                                break;
                            }
                        }
                    } else {
                        stream.push(byte);
                    }
                    len += 1;
                    prev_byte = byte;
                    isconst = !isconst;
                }
                if len > 1 {
                    (Some(Instr::new(Stream_Alt(Stream_Alt{stream: stream, flip: flip, cbyte: cbyte, cstart: start}), len, pos)), len)
                } else {
                    (None, len)
                }
            }
        }
        ind += 1;
    }
    
    // print out vectory
    println!("{:?}", possible_instrs);
    for instr in possible_instrs {
        match instr.kind {
            Instr_Type::Lzss(_) =>   print!("  LZSS: "),
            Instr_Type::Stream(_) => print!("Stream: "),
            Instr_Type::Stream_Alt(a) => {
                print!(" Alt-");
                if a.flip {
                    print!("f: ");
                } else if a.cbyte == 0 {
                    print!("0: ");
                } else {
                    print!("1: ");
                }
            }
        }
        print!("  pos={:03}", instr.pos);
        println!("  len={:03}", instr.len);
    }
    panic!();
    let out: Vec<u8> = Vec::new();
    out
    
    /*
    // Step 2: Convert bytes not included in a valid LZSS instruction into a Stream instruction
    let mut stream: Vec<u8> = Vec::new();
    let mut stream_start = 0;
    let mut ind = 0;
    while ind < comp.len() {
	let instr = &mut comp[ind];
	// Is this an LZSS command?
	if !instr.lzss {
	    if stream.len() == 0 {
		stream_start = instr.pos;
	    }
	    // Not valid, add this to stream
	    stream.push(data[instr.pos]);
	    // Remove this node from the vector
	    comp.remove(ind);
	} else {
	    // Is valid!
	    // If a stream is being created, add it
	    if stream.len() > 0 {
		let stream_instr = Instr::new(false, stream.len(), 0, Some(stream), stream_start);
		comp.insert(ind, stream_instr);     ind += 1;
		stream = Vec::new();
	    }
	    ind += 1;
	}
    }

    // Step 3: combine LZSS instructions into Stream instructions if that would take up less
    // space
    ind = 0;
    while ind < comp.len() - 1 {
	// Look at this and next instruction
	let instA = &mut comp[ind];     let instA_len = instA.gen_bytecode();   // Dancing around the borrow-checker :)
	let instB = &mut comp[ind+1];   let instB_len = instB.gen_bytecode();
	let instA = &comp[ind];     let instB = &comp[ind+1];
	// Combine them into a single stream instruction and check if that takes up less or
	// more space
	let sep_len = instA_len + instB_len;
	let mut instC = instA.combine_to_stream(instB, &data);
	let comb_len = instC.gen_bytecode();
	if comb_len <= sep_len {
	    // Yes! Combining these will result in a smaller result
	    // Replace the current instruction with the combined instruction
	    // and remove the next instruction
	    comp[ind] = instC;
	    comp.remove(ind+1);
	} else {
	    // No, keep these seperate and move down the list
	    ind += 1;
	}   
    }
    
    // Step 4: Reduce cost (unimplimented)
    
    // Step 5: Convert our instructions into one long byte stream
    let mut bstream: Vec<u8> = Vec::new();
    for instr in comp.iter() {
	for byte in instr.bytecode.as_ref().unwrap().iter() {
	    bstream.push(*byte);
	}
    }
    bstream
    */
}
