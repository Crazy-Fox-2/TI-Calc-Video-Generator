use std::cmp;
use crate::compress::graph_solve;
//use num::integer::Integer;

/*
pub fn compress(nt: usize, d: &[u8], ins: &[&dyn Fn(&[u8], usize) -> Vec<(usize, usize)>], stcs: &[&dyn Fn(&[u8], usize, usize) -> isize], encs: &[&dyn Fn(&[u8], usize, usize) -> isize], ctcs: &[&dyn Fn(&[u8], usize, usize, usize) -> isize], bcs: &[&dyn Fn(&[u8], usize, usize, usize)->Vec<u8>]) -> Vec<u8> {
        let gs = GraphSolve { numt: nt, data: d, get_instrs: ins, get_step_costs: stcs, get_entry_costs: encs, get_cont_costs: ctcs, gen_bytecodes: bcs};
        gs.gen_bytecode(gs.make_graph())
    }
*/
/*

fn c_test(data: &[u8]) {
    println!("{:?}", graph_solve::GraphSolve::compress(5, &data, &[&stream_get_instrs, &lzss_get_instrs, &alt_flip_get_instrs, &alt_white_get_instrs, &alt_black_get_instrs],
                                    &[&stream_get_step_cost, &lzss_get_step_cost, &alt_flip_get_step_cost, &alt_white_get_step_cost, &alt_black_get_step_cost],
                                    &[&stream_get_entry_cost, &lzss_get_entry_cost, &alt_flip_get_entry_cost, &alt_white_get_entry_cost, &alt_black_get_entry_cost],
                                    &[&stream_get_cont_cost, &lzss_get_cont_cost, &alt_flip_get_cont_cost, &alt_white_get_cont_cost, &alt_black_get_cont_cost],
                                    &[&stream_gen_bytecode, &lzss_gen_bytecode, &alt_flip_gen_bytecode, &alt_white_gen_bytecode, &alt_black_gen_bytecode] ) );
}
*/

pub fn compress(data: &[u8]) -> Vec<u8> {
    
    /*
    // Test compression
    let d0 = vec![0, 1, 2, 3, 4, 5];
    let d1 = vec![0, 5, 0, 5];
    let d2 = vec![0, 5, 0, 5, 0, 5, 0, 5, 0, 5];
    c_test(&d0);
    c_test(&d1);
    c_test(&d2);
    */
    
    // Compress
    let mut comp = graph_solve::GraphSolve::compress(5, &data, &[&stream_get_instrs, &lzss_get_instrs, &alt_flip_get_instrs, &alt_white_get_instrs, &alt_black_get_instrs],
                                    &[&stream_get_step_cost, &lzss_get_step_cost, &alt_flip_get_step_cost, &alt_white_get_step_cost, &alt_black_get_step_cost],
                                    &[&stream_get_entry_cost, &lzss_get_entry_cost, &alt_flip_get_entry_cost, &alt_white_get_entry_cost, &alt_black_get_entry_cost],
                                    &[&stream_get_cont_cost, &lzss_get_cont_cost, &alt_flip_get_cont_cost, &alt_white_get_cont_cost, &alt_black_get_cont_cost],
                                    &[&stream_gen_bytecode, &lzss_gen_bytecode, &alt_flip_gen_bytecode, &alt_white_gen_bytecode, &alt_black_gen_bytecode] );
    comp.push(0x00);
    comp

    
}



// LZSS Functions
const LZSS_MAX_LEN: usize = 32;
fn lzss_get_instrs(data: &[u8], pos: usize) -> Vec<(usize, usize)> {
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
        }
    }
    list
}
fn lzss_get_step_cost(_data: &[u8], _pos: usize, _off: usize) -> isize {
    0
}
fn lzss_get_entry_cost(_data: &[u8], _pos: usize, off: usize) -> isize {
    // Entry cost is 3 if offset is greater than or equal to 128, else 2
    match off >= 128 {
        true => 3,
        false => 2
    }
}
fn lzss_get_cont_cost(data: &[u8], pos: usize, off: usize, rel_pos: usize) -> isize{
    // Same as entry cost if length is 64 (or some multiple of that)
    match rel_pos % LZSS_MAX_LEN {
        0 => lzss_get_entry_cost(data, pos, off),
        _ => 0
    }
}
fn lzss_gen_bytecode(_data: &[u8], _pos: usize, off: usize, mut len: usize) -> Vec<u8> {
    // Generate lzss bytecode
    let mut bytecode: Vec<u8> = Vec::new();
    // Figure out how many instructions we'll need to use and the lengths of each one
    let num = num::Integer::div_ceil(&len, &LZSS_MAX_LEN);
    let max_put_len = num::Integer::div_ceil(&len, &num);
    for _i in 0..num {
        let put_len = cmp::min(len, max_put_len);
        bytecode.push(((put_len - 1) as u8) << 3);
        // Offset stored in 1 or 2 bytes depending on size
        if off >= 128 {
            bytecode.push(((off & 0x7F00) >> 7) as u8 + 1);
            bytecode.push(off as u8);
        } else {
            bytecode.push((off as u8) << 1);
        }
        len -= put_len;
    }
    bytecode
}


// Stream Functions
const STREAM_MAX_LEN: usize = 64;
fn stream_get_instrs(data: &[u8], _pos: usize) -> Vec<(usize, usize)> {
    // Because the stream instruction is always available and arbitrarily long, this function
    // should only get called once at the start of the graph generation
    vec![(data.len(), 0)]
}
fn stream_get_step_cost(_data: &[u8], _pos: usize, _id: usize) -> isize {
    1
}
fn stream_get_entry_cost(_data: &[u8], _pos: usize, _id: usize) -> isize {
    1
}
fn stream_get_cont_cost(data: &[u8], pos: usize, id: usize, rel_pos: usize) -> isize{
    // Same as entry cost if length is a multiple of the max length
    match rel_pos % STREAM_MAX_LEN {
        0 => stream_get_entry_cost(data, pos, id),
        _ => 0
    }
}
fn stream_gen_bytecode(data: &[u8], mut pos: usize, _id: usize, mut len: usize) -> Vec<u8> {
    // Generate tream bytecode
    let mut bytecode: Vec<u8> = Vec::new();
    // Figure out how many instructions we'll need to use and the lengths of each one
    let num = num::Integer::div_ceil(&len, &STREAM_MAX_LEN);
    let max_put_len = num::Integer::div_ceil(&len, &num);
    for _i in 0..num {
        let put_len = cmp::min(len, max_put_len);
        bytecode.push((((put_len - 1) as u8) << 2) + 0x02);
        for byte in &data[pos..pos+put_len] {
            bytecode.push(*byte);
            pos += 1;
        }
        len -= put_len;
    }
    bytecode
}


// Alt-Flip Functions
const ALT_FLIP_MAX_LEN: usize = 32;
fn alt_flip_get_instrs(data: &[u8], pos: usize) -> Vec<(usize, usize)> {
    let mut insts: Vec<(usize, usize)> = Vec::new();
    match search_alt(data, pos, 'f', false) {
        Some(len) => insts.push((len, pos % 2)),
        None => {}
    };
    insts
}
fn alt_flip_get_step_cost(_data: &[u8], pos: usize, id: usize) -> isize {
    match pos % 2 == id {
        true => 1,
        false => 0
    }
}
fn alt_flip_get_entry_cost(_data: &[u8], pos: usize, id: usize) -> isize {
    // Don't enter if not on data byte
    match pos % 2 == id {
        true => 1,
        false => 9999,
    }
}
fn alt_flip_get_cont_cost(data: &[u8], pos: usize, id: usize, rel_pos: usize) -> isize{
    // Same as entry cost if length is a multiple of the max length
    match rel_pos % ALT_FLIP_MAX_LEN {
        0 => stream_get_entry_cost(data, pos, id),
        _ => 0
    }
}
fn alt_flip_gen_bytecode(data: &[u8], mut pos: usize, id: usize, mut len: usize) -> Vec<u8> {
    // Generate tream bytecode
    let mut bytecode: Vec<u8> = Vec::new();
    // Figure out how many instructions we'll need to use and the lengths of each one
    let num = num::Integer::div_ceil(&len, &ALT_FLIP_MAX_LEN);
    let max_put_len = 32;
    for _i in 0..num {
        let put_len = cmp::min(len, max_put_len);
        bytecode.push((((put_len - 1) as u8) << 3) + 0x04);
        for byte in &data[pos..pos+put_len] {
            if (pos % 2) == id {
                bytecode.push(*byte);
            }
            pos += 1;
        }
        len -= put_len;
    }
    bytecode
}


// Alt-White Functions
const ALT_WHITE_MAX_LEN: usize = 32;
fn alt_white_get_instrs(data: &[u8], pos: usize) -> Vec<(usize, usize)> {
    let mut insts: Vec<(usize, usize)> = Vec::new();
    match search_alt(data, pos, 'w', false) {
        Some(len) => insts.push((len, pos % 2)),
        None => {}
    };
    match search_alt(data, pos, 'w', true) {
        Some(len) => insts.push((len, (pos + 1) % 2)),
        None => {}
    };
    insts
}
fn alt_white_get_step_cost(_data: &[u8], pos: usize, id: usize) -> isize {
    match pos % 2 == id {
        true => 1,
        false => 0
    }
}
fn alt_white_get_entry_cost(_data: &[u8], _pos: usize, _id: usize) -> isize {
    1
}
fn alt_white_get_cont_cost(data: &[u8], pos: usize, id: usize, rel_pos: usize) -> isize{
    // Same as entry cost if length is a multiple of the max length
    match rel_pos % ALT_WHITE_MAX_LEN {
        0 => stream_get_entry_cost(data, pos, id),
        _ => 0
    }
}
fn alt_white_gen_bytecode(data: &[u8], mut pos: usize, id: usize, mut len: usize) -> Vec<u8> {
    // Generate tream bytecode
    let mut bytecode: Vec<u8> = Vec::new();
    let num = num::Integer::div_ceil(&len, &ALT_WHITE_MAX_LEN);
    let max_put_len = num::Integer::div_ceil(&len, &num);
    for _i in 0..num {
        let put_len = cmp::min(max_put_len, len);
        bytecode.push((((put_len - 1) as u8) << 3) + 0x01 + ((((id+pos)%2) as u8) << 2));
        for byte in &data[pos..pos+put_len] {
            if pos % 2 == id {
                bytecode.push(*byte);
            }
            pos += 1;
        }
        len -= put_len;
    }
    bytecode
}


// Alt-Black Functions
const ALT_BLACK_MAX_LEN: usize = 32;
fn alt_black_get_instrs(data: &[u8], pos: usize) -> Vec<(usize, usize)> {
    let mut insts: Vec<(usize, usize)> = Vec::new();
    match search_alt(data, pos, 'b', false) {
        Some(len) => insts.push((len, pos % 2)),
        None => {}
    };
    match search_alt(data, pos, 'b', true) {
        Some(len) => insts.push((len, (pos + 1) % 2)),
        None => {}
    };
    insts
}
fn alt_black_get_step_cost(_data: &[u8], pos: usize, id: usize) -> isize {
    match pos % 2 == id {
        true => 1,
        false => 0
    }
}
fn alt_black_get_entry_cost(_data: &[u8], _pos: usize, _id: usize) -> isize {
    1
}
fn alt_black_get_cont_cost(data: &[u8], pos: usize, id: usize, rel_pos: usize) -> isize{
    // Same as entry cost if length is a multiple of the max length
    match rel_pos % ALT_BLACK_MAX_LEN {
        0 => stream_get_entry_cost(data, pos, id),
        _ => 0
    }
}
fn alt_black_gen_bytecode(data: &[u8], mut pos: usize, id: usize, mut len: usize) -> Vec<u8> {
    // Generate tream bytecode
    let mut bytecode: Vec<u8> = Vec::new();
    let num = num::Integer::div_ceil(&len, &ALT_BLACK_MAX_LEN);
    let max_put_len = num::Integer::div_ceil(&len, &num);
    for _i in 0..num {
        let put_len = cmp::min(len, max_put_len);
        bytecode.push((((put_len - 1) as u8) << 3) + 0x03 + ((((id+pos)%2) as u8) << 2));
        for byte in &data[pos..pos+put_len] {
            if pos % 2 == id {
                bytecode.push(*byte);
            }
            pos += 1;
        }
        len -= put_len;
    }
    bytecode
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


