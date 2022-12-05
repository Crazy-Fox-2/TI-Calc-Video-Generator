// General structure for compressing a data stream
// Can be used with compression schemes which consist of a series of instructions (each an integer
// number of bytes long) which all compress the data differently
// This solver is provided various dynamic structs for determining properties each instruction will
// have given a sertain section of data, and it will automaticall run the supplied functions and use
// those results to determine the most efficiant way of arranging those instructions to produce a
// compressed result which is as small as possible


use crate::compress::instr::{/*Instr,*/ InstrGen};
use crate::compress::cycle_limit::CycleInstr;


pub trait GraphFuncs {
    fn get_instr_info(&self, data: &[u8], pos: usize) -> Vec<(usize, usize)>;   // Returns Vec<len, id>
    fn get_step_cost(&self, data: &[u8], pos: usize, uid: usize) -> isize;
    fn get_entry_cost(&self, data: &[u8], pos: usize, uid: usize) -> isize;
    fn get_cont_cost(&self, data: &[u8], pos: usize, uid: usize, rel_pos: usize) -> isize;
}

pub struct GraphSolve<'a> {
    numt: usize,
    data: &'a [u8],
    gtypes: Vec<Box<dyn GraphFuncs>>,
    itypes: Vec<Box<dyn InstrGen<dyn CycleInstr>>>
}



#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
struct Node {
    cost: isize,
    from: isize,
    rel_pos: usize,
    prev_ind: isize,
    instr_type: usize,
    id: usize,
    len: usize
}
impl Node {
    fn new() -> Node {
        Node {cost: -1, from: -2, rel_pos: 0, prev_ind: -1, instr_type: 0, id: 0, len: 0}
    }
}
    


pub fn compress(data: &[u8], instr_types: Vec<Box<dyn InstrGen<dyn CycleInstr>>>, graph_types: Vec<Box<dyn GraphFuncs>>) -> Vec<Box<dyn CycleInstr>> {
    let gs: GraphSolve = GraphSolve { numt: instr_types.len(), data: data, itypes: instr_types, gtypes: graph_types };
    let graph = gs.make_graph();
    gs.gen_instrs(graph)
}


impl GraphSolve<'_> {
    
    fn make_graph(&self) -> Vec<Vec<Node>> {
        // Most of the magic is done in here
        // This function figures which instructions should be used at what times utilizing a
        // graph-like structure.
        // Each row of 'nodes' represents a single byte in the uncompressed data stream, with each
        // node then representing one of the instructions vying for control over that byte.
        // A path is found along this graph which results in the lowest possible cost (the least
        // amount of bytes in the compressed stream). Each node only connects to nodes in the next
        // and the previous row.
        // Each row of nodes can contain a different number of nodes, depending on which
        // instructions are available to use when. Likewise a row can contain multiple nodes of the
        // same instruction as some instrucitons take additional arguments for how they compress
        // the data, for example there could be multiple LZSS instructions possible which take
        // their data from different positions in the stream.
        // Determining the path starts from the first row (first byte) and moves down the list. Each
        // node in the row looks at the cost it would take to move to each node in the next row. If
        // the destination row's current cost is higher than what it would be from the current node
        // the path to that node is updated.
        // By the end of this process we look at the last row's node with the lowest total cost. We
        // can follow the path backwards along the from pointers and can then construct the best
        // sequence of instructions
        
        let len = self.data.len();
        
        let mut graph: Vec<Vec<Node>> = Vec::with_capacity(len+1);
        graph.push(Vec::new());
        
        for pos in 1..(len+1) {
            
            let mut row: Vec<Node> = Vec::new();

            let mut prev_instr_count = vec![0; self.numt];
            // Continue instructions from last row
            for (ind, node) in graph[pos-1].iter().enumerate() {
                let ind = ind as isize;
                if node.len > 1 {
                    let mut node = *node;
                    node.len -= 1;
                    node.prev_ind = ind;
                    node.rel_pos += 1;
                    node.cost = -1;
                    prev_instr_count[node.instr_type] += 1;
                    if prev_instr_count[node.instr_type] < 50 {     // *Technically* reduces effectiveness, but it's worth it
                        row.push(node);
                    }
                }
            }
            // Get new instructions
            for itype in 0..self.numt {
                let insts: Vec<(usize, usize)> = self.gtypes[itype].get_instr_info(self.data, pos-1);
                'iloop: for (len, id) in insts.iter() {
                    // Add if given id does not match one we're already doing
                    for node in graph[pos-1].iter() {
                        if node.id == *id && node.instr_type == itype {
                            continue 'iloop;
                        }
                    }
                    let mut node = Node::new();
                    node.instr_type = itype;  node.id = *id;   node.len = *len;
                    row.push(node);
                }
            }
            // Get step cost & entry cost for each instruction
            let mut step_costs: Vec<isize> = Vec::with_capacity(row.len());
            let mut entr_costs: Vec<isize> = Vec::with_capacity(row.len());
            let mut cont_costs: Vec<isize> = Vec::with_capacity(row.len());
            for node in row.iter() {
                let gen = &self.gtypes[node.instr_type];
                step_costs.push(gen.get_step_cost(self.data, pos-1, node.id));
                entr_costs.push(gen.get_entry_cost(self.data, pos-1, node.id));
                cont_costs.push(gen.get_cont_cost(self.data, pos-1, node.id, node.rel_pos));
            }
            // Find best path to each valid instruction
            if pos == 1 {
                for (ind, node) in row.iter_mut().enumerate() {
                    node.cost = entr_costs[ind] + step_costs[ind];
                }
            } else {
                for (from_ind, from_node) in graph[pos-1].iter_mut().enumerate() {
                    for (to_ind, to_node) in row.iter_mut().enumerate() {
                        // Get total cost after jumping to this command
                        let mut to_cost = from_node.cost + step_costs[to_ind];
                        to_cost += match from_ind as isize == to_node.prev_ind {
                            true => cont_costs[to_ind],
                            false => entr_costs[to_ind]
                        };
                        if to_node.cost == -1 || to_node.cost > to_cost {
                            to_node.cost = to_cost as isize;
                            to_node.from = from_ind as isize;
                            // Set relative position
                            if from_ind as isize == to_node.prev_ind {
                                to_node.rel_pos = from_node.rel_pos + 1;
                            } else {
                                to_node.rel_pos = 0;
                            }
                        }
                    }
                }
            }
            
            graph.push(row);
            
        }
        
        graph
        
    }
    
    fn gen_instrs(&self, graph: Vec<Vec<Node>>) -> Vec<Box<dyn CycleInstr>> {
        
        // Follow path backwards and construct the list of instruction bytecodes
        let mut list: Vec<Box<dyn CycleInstr>> = Vec::new();
        // Get finishing instruction with the lowest total cost
        let mut min_cost = isize::max_value();
        let mut cur_node = &graph[graph.len()-1][0];
        for node in graph[graph.len()-1].iter() {
            if node.cost < min_cost {
                min_cost = node.cost;
                cur_node = node;
            }
        }
        // Follow path
        let mut len = 1;
        for pos in (0..graph.len()-1).rev() {
            if cur_node.from != cur_node.prev_ind {
                // This is the first node on this instruction to be encoded
                let gen = &self.itypes[cur_node.instr_type];
                list.push(gen.gen_instr(self.data, pos, cur_node.id, len));
                len = 0;
            }
            if cur_node.from >= 0 {
                // Move to previous node unless this is the end of the graph
                cur_node = &graph[pos][cur_node.from as usize];
            }
            len += 1;
        }

        // Reverse list
        let list = list.into_iter().rev().collect();
        
        list
    }
    
    
}


