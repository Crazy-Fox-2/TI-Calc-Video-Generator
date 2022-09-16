#![allow(dead_code)]
#![allow(non_snake_case)]

mod video;
mod args;
mod macros;

use crate::video::video::*;
use crate::args::*;



fn process() -> Result<(), String> {
    // Get command-line arguments
    let args = getargs()?;
    // Load video
    let mut vid = Video::new(&args.value_of::<String>("src").unwrap(), &args.value_of::<String>("out").unwrap(), "tempvids__/")?;
    vid.convert();
    Ok(())
}


fn main() {
    
    match process() {
        Ok(()) => {},
        Err(err) => println!("Error: {}", err)
    };
    
    
}






