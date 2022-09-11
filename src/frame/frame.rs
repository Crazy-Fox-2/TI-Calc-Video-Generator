
use crate::frame::aud::*;
use crate::frame::img::*;


pub struct Frame {

    img: Img,
    aud: Aud,

}
impl Frame {


    pub fn compress(&mut self) -> (Vec<u8>, Vec<u8>) {
        // Compress both video and audio
        self.img.compress();
        self.img.reduce_cost(101600);
        self.aud.compress();
        // Put these compressed states into two vecotrs and return them
        (self.img.output(), self.aud.output())
    }



}




