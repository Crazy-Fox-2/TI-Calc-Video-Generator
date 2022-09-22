


struct Instr {

    lzss: bool,
    len: usize,
    offset: usize,
    stream: Option<Vec<u8>>,
    pos: usize,
    bytecode: Option<Vec<u8>>,
    cost: u32,
    
}
impl Instr {

    fn new(lzss: bool, len: usize, offset: usize, stream: Option<Vec<u8>>, pos: usize) -> Instr {
        Instr {lzss: lzss, len: len, offset: offset, stream: stream, pos: pos, bytecode: None, cost: 0}
    }
    
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

    fn get_cost(&mut self) -> u32 {
        // Get number of cycles this instruction will take to run
        self.cost = match self.lzss {
            true => {
                38 + 44 + match self.offset >= 128 {
                    true => 32, false => 20
                } + 34 + (21 * self.len as u32) - 5 + 22
            }
            false => {
                38 + 7 + (21 * self.len as u32) - 5 + 12
            }
        };
        self.cost
    }
    
    fn combine_to_stream(&self, other: &Instr, data: &Vec<u8>) -> Instr {
        // Combines this and another instruction into a single stream instruction
        let mut stream: Vec<u8> = Vec::new();
        stream.extend_from_slice(&data[self.pos..self.pos+self.len]);
        stream.extend_from_slice(&data[other.pos..other.pos+other.len]);
        let instr = Instr::new(false, self.len + other.len, 0, Some(stream), self.pos);
        instr
    }

}




pub fn compress(data: Vec<u8>) -> Vec<u8> {
    
    // Generate compressed stream
    let mut comp:Vec<Instr> = Vec::new();

    // Step 1: generate all lzss instructions (when we can)
    let mut ind = 0;
    while ind < data.len() {
        
        // Search for largest lzss size we can get
        let mut largest_off = 0;
        let mut largest_size = 0;
        for off in 1..(ind+1) {
            // Check how long this run is
            let mut cur_size = 0;
            for from_ind in (ind-off)..data.len() {
                let to_ind = from_ind + off;
                if !(to_ind >= data.len()) && data[from_ind] == data[to_ind] {
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

        // Record instruction and move pointer foreward
        let instr = if largest_size <= 1 {
            // Record stream instruction
            let instr = Instr::new(false, 1, 0, None, ind);
            ind += 1;
            instr
        } else {
            // Record LZSS instruction
            let instr = Instr::new(true, largest_size, largest_off, None, ind);
            ind += largest_size;
            instr
        };
        comp.push(instr);
    }

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
}
