use std::cmp;
use crate::compress::graph_solve;
//use num::integer::Integer;

/*
pub fn compress(nt: usize, d: &[u8], ins: &[&dyn Fn(&[u8], usize) -> Vec<(usize, usize)>], stcs: &[&dyn Fn(&[u8], usize, usize) -> isize], encs: &[&dyn Fn(&[u8], usize, usize) -> isize], ctcs: &[&dyn Fn(&[u8], usize, usize, usize) -> isize], bcs: &[&dyn Fn(&[u8], usize, usize, usize)->Vec<u8>]) -> Vec<u8> {
        let gs = GraphSolve { numt: nt, data: d, get_instrs: ins, get_step_costs: stcs, get_entry_costs: encs, get_cont_costs: ctcs, gen_bytecodes: bcs};
        gs.gen_bytecode(gs.make_graph())
    }
*/


pub fn compress(data: Vec<u8>) -> Vec<u8> {
    
    // Compress
    graph_solve::GraphSolve::compress(5, &data, &[&stream_get_instrs, &lzss_get_instrs, &const0_get_instrs, &const1_get_instrs],
                                                &[&stream_get_step_cost, &lzss_get_step_cost, &const0_get_step_cost, &const1_get_step_cost],
                                                &[&stream_get_entry_cost, &lzss_get_entry_cost, &const0_get_entry_cost, &const1_get_entry_cost],
                                                &[&stream_get_cont_cost, &lzss_get_cont_cost, &const0_get_cont_cost, &const1_get_cont_cost],
                                                &[&stream_gen_bytecode, &lzss_gen_bytecode, &const0_gen_bytecode, &const1_gen_bytecode] )

    
}



// LZSS Functions
const LZSS_MAX_LEN: usize = 64;
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
                if len >= 2 {
                    let offset = pos - from_start;  // offset will be the unique identifier
                    list.push((len, offset));
                }
                break;
            }
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
        let put_len = cmp::min(len, max_put_len) - 1;
        bytecode.push(((put_len as u8) << 2) + 0x00);
        // Offset stored in 1 or 2 bytes depending on size
        if off >= 128 {
            bytecode.push(((off & 0x7F00) >> 7) as u8 + 1);
            bytecode.push(off as u8);
        } else {
            bytecode.push((off as u8) << 1);
        }
        len -= LZSS_MAX_LEN;
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
        let put_len = cmp::min(len, max_put_len) - 1;
        bytecode.push(((put_len as u8) << 2) + 0x01);
        for byte in &data[pos..pos+put_len] {
            bytecode.push(*byte);
            pos += 1;
        }
        len -= STREAM_MAX_LEN;
    }
    bytecode
}


// Const 00 Functions
const CONST_00_MAX_LEN: usize = 64;
fn const0_get_instrs(data: &[u8], pos: usize) -> Vec<(usize, usize)> {
    let mut instrs: Vec<(usize, usize)> = Vec::new();
    // Get length of data which is just 00
    let mut len = 0;
    for i in pos..data.len() {
        if data[i] != 0x00 {
            break;
        }
        len += 1;
    }
    if len > 0 {
        instrs.push((len, 0));
    }
    instrs
}
fn const0_get_step_cost(_data: &[u8], _pos: usize, _id: usize) -> isize {
    0
}
fn const0_get_entry_cost(_data: &[u8], _pos: usize, _id: usize) -> isize {
    1
}
fn const0_get_cont_cost(data: &[u8], pos: usize, id: usize, rel_pos: usize) -> isize {
    // Same as entry cost if length is a multiple of the max length
    match rel_pos % CONST_00_MAX_LEN {
        0 => stream_get_entry_cost(data, pos, id),
        _ => 0
    }
}
fn const0_gen_bytecode(_data: &[u8], _pos: usize, _id: usize, mut len: usize) -> Vec<u8> {
    // Generate bytecode
    let mut bytecode: Vec<u8> = Vec::new();
    // Figure out how many instructions we'll need to use and the lengths of each one
    let num = num::Integer::div_ceil(&len, &CONST_00_MAX_LEN);
    let max_put_len = num::Integer::div_ceil(&len, &num);
    for _i in 0..num {
        let put_len = cmp::min(len, max_put_len) - 1;
        bytecode.push(((put_len as u8) << 2) + 0x02);
        len -= CONST_00_MAX_LEN;
    }
    bytecode
}


// Const FF Functions
const CONST_FF_MAX_LEN: usize = 64;
fn const1_get_instrs(data: &[u8], pos: usize) -> Vec<(usize, usize)> {
    let mut instrs: Vec<(usize, usize)> = Vec::new();
    // Get length of data which is just FF
    let mut len = 0;
    for i in pos..data.len() {
        if data[i] != 0xFF {
            break;
        }
        len += 1;
    }
    if len > 0 {
        instrs.push((len, 0xFF));
    }
    instrs
}
fn const1_get_step_cost(_data: &[u8], _pos: usize, _id: usize) -> isize {
    0
}
fn const1_get_entry_cost(_data: &[u8], _pos: usize, _id: usize) -> isize {
    1
}
fn const1_get_cont_cost(data: &[u8], pos: usize, id: usize, rel_pos: usize) -> isize {
    // Same as entry cost if length is a multiple of the max length
    match rel_pos % CONST_FF_MAX_LEN {
        0 => stream_get_entry_cost(data, pos, id),
        _ => 0
    }
}
fn const1_gen_bytecode(_data: &[u8], _pos: usize, _id: usize, mut len: usize) -> Vec<u8> {
    // Generate bytecode
    let mut bytecode: Vec<u8> = Vec::new();
    // Figure out how many instructions we'll need to use and the lengths of each one
    let num = num::Integer::div_ceil(&len, &CONST_FF_MAX_LEN);
    let max_put_len = num::Integer::div_ceil(&len, &num);
    for _i in 0..num {
        let put_len = cmp::min(len, max_put_len) - 1;
        bytecode.push(((put_len as u8) << 2) + 0x03);
        len -= CONST_FF_MAX_LEN;
    }
    bytecode
}
