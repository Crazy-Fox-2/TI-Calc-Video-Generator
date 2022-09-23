// General structure for compressing a data stream
// Can be used with compression schemes which consist of a series of instructions (each an integer
// number of bytes long) which all compress the data differently
// This structure is provided various functions for determining properties each instruction will
// have given a sertain section of data, and it will automaticall run those functions and use those
// results to determine the most efficiant way of arranging those instructions to produce a
// compressed result which is as small as possible




pub struct GraphSolve {
    numt: usize,
    entry_costs: &[u8],
    get_progress_costs: &[&dyn Fn(&[u8], usize, usize)->isize],     // Data, Absolute position, Relative Position
    gen_bytecodes: &[&dyn Fn(&[u8], usize, usize)->Vec<u8>],       // Data, Absolute position, Instruction length
    is_valids: &[&dyn Fn(&[u8], usize)->bool],              // Data, Absolute position
    
}
impl GraphSolve {
    
    pub fn compress(num_types: usize, entry_costs: &[u8], get_progress_costs: &[&dyn Fn(&[u8], usize, usize)], gen_bytecodes: &[&dyn Fn(&[u8], usize)], is_valids: &[&dyn Fn(&[u8], usize)]) -> Vec<u8> {
        let mut gs = GraphSolve { numt: num_types, entry_costs: entry_costs, get_progress_costs: get_progress_costs, gen_bytecodes: gen_bytecodes, is_valids: is_valids };
        let list = gs.make_graph();
        let bytecode = gs.gen_bytecode(list);
        bytecode
    }
    
    fn make_graph(&self) -> Vec<(usize, usize, usize)> {
        // Most of the magic is done in here
        // This function figures which instructions should be used at what times utilizing a
        // graph-like structure.
        // Each row of 'nodes' represents a single byte in the uncompressed data stream, with each
        // node then representing one of the instructions vying for control over that byte.
        // A path is found along this graph which results in the lowest possible cost (the least
        // amount of bytes in the compressed stream). Each node only connects to nodes in the next
        // and the previous row.
        // Each node keeps track of the cost to get to it, which node it comes from on the previous
        // row, and how long the path has been on this instruction type. These are stored in 3 2d
        // vectors.
        // Determining the path starts from the first row (first byte) moves down the list. Each
        // node in the row looks at the cost it would take to move to each node in the next row. If
        // the destination row's current cost is higher than what it would be from the current node
        // the path to that node is updated.
        // By the end of this process we look at the last row's node with the lowest total cost. We
        // can follow the path backwards along the from pointers and can then construct the best
        // sequence of instructions
        
        let len = self.data.len();
        
        let mut costs: Vec<Vec<isize>> = Vec::with_capacity(len);
        let mut froms: Vec<Vec<isize>> = Vec::with_capacity(len);
        let mut relps: Vec<Vec<usize>> = Vec::with_capacity(len);
        
        for pos in 0..len {
            
            let mut cost_row: Vec<isize> = Vec::with_capacity(self.numt);
            let mut from_row: Vec<isize> = Vec::with_capacity(self.numt);
            let mut relp_row: Vec<isize> = vec![0; self.numt];
            // Check which instructions are valid for this byte
            for iv in self.is_valids {
                cost_row.push( if iv(self.data, pos) {0} else {-2} );
                from_row.push(-1);
            }
            // Get base cost for each instruction
            let mut prog_cost_row: Vec<isize> = Vec::new();
            for (ind, gpc) in self.get_progress_costs.iter().enumerate() {
                rel_pos = match pos {
                    0 => 0,
                    _ => relps[pos-1][ind]
                };
                prog_cost_row.push(gpc(data, pos, rel_pos));
            }
            // Find best path to each valid instruction
            if pos == 0 {
                for ind in 0..self.numt {
                    cost_row[ind] = entry_costs[ind] + prog_cost_row[to_ind];
                }
            } else {
                for from_ind in 0..self.numt {
                    for to_ind in 0..self.numt {
                        if cost_row[to_ind] == -2 {
                            continue;
                        }
                        // Get total cost after jumping to this command
                        let mut to_cost = costs[pos-1][from_ind] + prog_cost_row[to_ind];
                        if from_ind != to_ind { to_cost += self.entry_costs[to_ind] };
                        if cost_row[to_ind] == -1 || cost_row[to_ind] > to_cost {
                            cost_row[to_ind] = to_cost;
                            from_row[to_ind] = from_ind;
                            // Increment relative position
                            if to_ind == from_ind {
                                relp_row[to_ind] = relps[pos-1][to_ind] + 1;
                            }
                        }
                    }
                }
            }
            
            costs.push(cost_row);
            froms.push(from_row);
            relps.push(relp_row);
        
        }
        
        // Follow path backwards and construct the list of instructions
        let mut list: Vec<(usize, usize, usize)> = Vec::new();
        let mut cur_inst = 0;
        let mut inst_len = 1;
        // Get finishing instruction with the lowest total cost
        let mut min_cost = usize::max_value();
        for (ind, cost) in &cost_row[len-1].enumerate() {
            if cost < min_cost {
                min_cost = cost;
                cur_inst = ind;
            }
        }
        // Follow path
        for pos in (0..len).rev() {
            let new_inst = froms[pos][cur_inst];
            if new_inst != cur_instr {
                list.push((cur_inst, pos, inst_len));
                cur_inst = new_inst;
                inst_len = 0;
            }
            inst_len += 1;
        }
        list.push((cur_inst, pos, inst_len));
        
        list
        
    }
    
    fn gen_bytecode(list: Vec<(usize, usize)>) -> Vec<u8> {
        // Converts the list of instructions generated by the above function into the final
        // compressed data stream
        
        let mut bytecode: Vec<u8> = Vec::new();
        for (inst, pos, len) in list {
            let mut instr_bytecode = self.gen_bytecodes[inst](self.data, pos, len);
            bytecode.append(instr_bytecode);
        }
        
        bytecode
    }
    
    
}













