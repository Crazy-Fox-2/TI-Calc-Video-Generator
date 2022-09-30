#![allow(dead_code)]
#![allow(non_snake_case)]


mod helper;
mod video;
mod args;
mod load;
mod compress;

use crate::video::video::*;
use crate::args::*;



fn process() -> Result<(), String> {
    // Get command-line arguments
    let args = getargs()?;
    // Load video
    let mut vid = Video::new(&args)?;
    // Convert video
    vid.create_app()?;
    vid.close()?;
    Ok(())
}


fn main() {
    
    match process() {
        Ok(()) => {},
        Err(err) => println!("Error: {}", err)
    };
    
    
}






