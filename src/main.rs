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
    let verbose = !&args.value_of::<bool>("mute").unwrap();
    // Load video
    let mut vid = Video::new(&args.value_of::<String>("input").unwrap(), &args.value_of::<String>("out").unwrap(), "tempvids__/", verbose)?;
    // Convert video
    vid.convert(args.value_of::<usize>("start").unwrap(), args.value_of::<usize>("duration").unwrap(), verbose)
}


fn main() {
    
    match process() {
        Ok(()) => {},
        Err(err) => println!("Error: {}", err)
    };
    
    
}






