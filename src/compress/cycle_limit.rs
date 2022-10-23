use crate::compress::instr::{Instr};


pub trait CycleInstr: Instr {
    fn get_cycles(&self) -> usize;
    fn is_minimum(&self) -> bool;
    fn to_minimum(&self) -> Box<dyn CycleInstr>;
    fn combine_with_left(&mut self, other: &dyn CycleInstr);
    fn combine_with_right(&mut self, other: &dyn CycleInstr);
}


pub fn get_total_cycles(instr_streams: &Vec<Vec<Box<dyn CycleInstr>>>) -> usize {
    let mut total = 0;
    for stream in instr_streams.iter() {
        for inst in stream.iter() {
            total += inst.get_cycles();
        }
    }
    total
}

pub fn reduce_cycles_to(instr_streams: &mut Vec<Vec<Box<dyn CycleInstr>>>, reduce_to: usize) -> usize {
    // Reduces the total cycle cost of the provinded compressed data streams
    // Will iteratively convert the instructions with the worst ratio of cycle cost to encoded
    // bytes into a more efficiant encoding. Works with multiple streams
    // Get cycle cost & decompressed size of each instruction
    let mut info: Vec<(usize, usize, usize, usize, f64)> = Vec::with_capacity(2048);     // stream ind, ind, cycles, decomplen, ratio
    let mut total = 0;
    for (sind, stream) in instr_streams.iter().enumerate() {
        for (iind, inst) in stream.iter().enumerate() {
            let c = inst.get_cycles();
            let s = inst.get_decomp_size();
            info.push((sind, iind, c, s, (c as f64) / (s as f64)));
            total += c;
        }
    }
    // Sort list by ratio
    info.sort_by(|a, b| a.4.partial_cmp(&b.4).unwrap());
    // Loop while cycle count is above what we want
    while total > reduce_to {
        // Look at worst ratio
        let (w_sind, mut w_iind, w_c, _w_s, _w_ratio) = info.pop().unwrap();
        let worst = &instr_streams[w_sind][w_iind];
        // Convert worst ratio to a most efficient encoding
        if worst.is_minimum() {
            continue;
        }
        total -= w_c;
        let mut newmin = worst.to_minimum();
        // Combine with left instruction if that has the same instruction type
        if w_iind >= 1 {
            let left_ind = w_iind - 1;
            let left = &instr_streams[w_sind][left_ind];
            if left.is_minimum() {
                total -= left.get_cycles();
                newmin.combine_with_left(&**left);
                // Remove left instruction from list
                instr_streams[w_sind].remove(left_ind);
                let mut remove_ind: Option<usize> = None;
                for (info_ind, (l_sind, l_iind, _l_c, _, _)) in info.iter_mut().enumerate() {
                    if *l_sind == w_sind {
                        if *l_iind == left_ind {
                            remove_ind = Some(info_ind);
                        } else if *l_iind > left_ind {
                            *l_iind -= 1;
                        }
                    }
                }
                w_iind -= 1;
                match remove_ind {
                    Some(ind) => {info.remove(ind);},
                    None => {},
                }
            }
        }
        // Combine with right instruction if that has the same instruction type
        if w_iind < instr_streams[w_sind].len()-1 {
            let right_ind = w_iind + 1;
            let right = &instr_streams[w_sind][right_ind];
            if right.is_minimum() {
                total -= right.get_cycles();
                newmin.combine_with_right(&**right);
                // Remove right instruction from list
                instr_streams[w_sind].remove(right_ind);
                let mut remove_ind: Option<usize> = None;
                for (info_ind, (r_sind, r_iind, _r_c, _, _)) in &mut info.iter_mut().enumerate() {
                    if *r_sind == w_sind {
                        if *r_iind == right_ind {
                            remove_ind = Some(info_ind);
                        } else if *r_iind > right_ind {
                            *r_iind -= 1;
                        }
                    }
                }
                match remove_ind {
                    Some(ind) => {info.remove(ind);},
                    None => {},
                }
            }
        }
        // Put in new efficient instruction
        total += newmin.get_cycles();
        instr_streams[w_sind][w_iind] = newmin;
    }
    total
}

