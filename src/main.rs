#![allow(dead_code)]
#![allow(non_snake_case)]

mod frame;


use crate::frame::img::*;






fn main() {
    

    // Test image compression
    let mut data: Vec<u8> = Vec::new();
    data.push(0);
    data.push(1);
    data.push(2);
    data.push(3);
    data.push(1);
    data.push(2);
    data.push(4);
    data.push(2);
    data.push(4);
    data.push(2);
    data.push(1);
    data.push(7);
    data.push(0);
    
    let mut frame = Img::new(data);
    frame.compress();
    


}






