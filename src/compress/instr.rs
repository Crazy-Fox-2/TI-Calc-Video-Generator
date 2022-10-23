use crate::compress::cycle_limit::CycleInstr;


pub trait InstrGen<T: ?Sized> {
    fn gen_instr(&self, data: &[u8], pos: usize, uid: usize, len: usize) -> Box<T>;     // uid is optional
}

pub trait Instr {
    fn gen_bytecode(&self, last: bool) -> Vec<u8>;
    fn get_comp_size(&self) -> usize;
    fn get_decomp_size(&self) -> usize;
    fn get_decomp(&self) -> Vec<u8>;
}


pub fn gen_bytecode(instrs: &Vec<Box<dyn CycleInstr>>) -> Vec<u8> {
    let mut bytecode = Vec::new();
    let last = instrs.len()-1;
    for (i, inst) in instrs.iter().enumerate() {
        bytecode.append(&mut inst.gen_bytecode(i == last));
    }
    bytecode
}



