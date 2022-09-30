use crate::load::{loadimg, audiostream};
use crate::helper::macros::{passerr, strcat};
use crate::args::VArgs;
use std::process::Command;
use crate::video::app::App;
use std::sync::mpsc::Receiver;
use crate::video::extract::{load_vid_data, save_vid_data};
use crate::helper::funcs::print_ln_if;



pub enum NumFrames {
    Rec(Receiver<usize>),
    Num(usize)
}
pub struct Video<'a> {
    pub args: &'a VArgs,
    pub num_frames: NumFrames,
    pub folder: String,
    pub file: String,
    pub name: String,
    pub out: String,
    pub fps: f64,
    pub start: usize,
    pub durr: usize,
    pub temp: bool,
}
impl<'a> Video<'a> {
    
    pub fn new(args: &'a VArgs) -> Result<Video<'a>, String> {
        // Setup video struct
        let mut vid = Video { args: args, num_frames: NumFrames::Num(0), folder: args.vid_folder.clone(), file: args.vid_file.clone(), name: args.name.clone(), out: args.out.clone(), fps: args.fps, durr: args.dur, start: args.start, temp: false };
        load_vid_data(&mut vid, args)?;
        Ok(vid)
    }

    pub fn create_app(&mut self) -> Result<(), String> {
        print_ln_if("".to_string(), !self.args.mute);
        let mut app = App::new(self.args)?;
        let mut cur_frame = 0;
        let mut auditer = audiostream::AudIter::new(&strcat!(self.folder, "audio.wav"))?;
        
        // Skip audio before start of encoded video
        for _i in 0..self.start {
            auditer.next();
        }
        loop {
            // Get frame number to encode
            let src_frame = (((cur_frame + self.start) as f64 / 20.0) * self.fps) as usize + 1;
            // Check on ffmpeg thread
            match &self.num_frames {
                NumFrames::Rec(_rec) => {
                    // Wait for next frame to exist (currently outputing from ffmpeg) or thread has
                    // finished
                    loop {
                        // Check if thread finished
                        if self.try_recv() {
                            break;
                        }
                        
                        // Check if frame after current frame exists
                        let frame_name = strcat!(self.folder, "frame", (src_frame+1).to_string(), ".png");
                        if std::path::Path::new(&frame_name).exists() {
                            break;
                        }
                        // Cannot continue yet, sleep for a little bit
                        std::thread::sleep(std::time::Duration::from_millis(400));
                    }
                },
                NumFrames::Num(_num) => {},
            };
            // Check if done with encoding frames
            if self.durr != 0 && cur_frame >= self.durr {
                break;
            }
            
            // Load image & audio data
            let fpath = strcat!(self.folder, "frame", src_frame.to_string(), ".png");
            let img = loadimg::load_interleaved(&fpath)?;
            let aud = auditer.next().unwrap();
            // Add to app
            app.add_frame(&img, &aud)?;
            // Print progress
            app.print_progress(self.durr, 0);
            cur_frame += 1;
        }
        // Finish app
        let num_pages = app.finish()?;
        app.print_progress(self.durr, num_pages);
        // Run rabbitsign
        let bin_path = strcat!(self.out, ".bin");
        passerr!(Command::new("rabbitsign").args(["-g", "-v", "-P", "-p", &bin_path]).output());
        Ok(())
    }
    
    fn try_recv(&mut self) -> bool{
        match &self.num_frames {
            NumFrames::Rec(rec) => {
                match rec.try_recv() {
                    Ok(num) => {
                        // Thread has finished and given us the total number of frames
                        self.num_frames = NumFrames::Num(num);
                        // Set durration
                        let max_durr = ((num as f64 / self.fps) * 20.0) as usize - self.start;
                        if self.durr == 0 || self.durr > max_durr {
                            self.durr = max_durr;
                        }
                        true
                    },
                    Err(_) => false
                }
            },
            NumFrames::Num(_num) => {true},
        }
    }
    
    pub fn close(mut self) -> Result<(), String> {
        while !self.try_recv() {}
        save_vid_data(&self, self.args)?;
        Ok(())
    }
    

}





