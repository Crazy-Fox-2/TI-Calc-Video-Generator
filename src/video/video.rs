use crate::load::{loadimg, audiostream};
use crate::helper::macros::{passerr, strcat};
use crate::args::VArgs;
use std::process::Command;
use crate::video::app::App;
use std::sync::mpsc::Receiver;
use crate::video::extract::{load_vid_data, save_vid_data};
use crate::helper::funcs::{print_ln_if, find_file_exe};
use std::io::{self, Write};
use std::path::Path;



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
    pub calc_fps: f64,
    pub start: usize,
    pub durr: usize,
    pub temp: bool,
}
impl<'a> Video<'a> {
    
    pub fn new(args: &'a VArgs) -> Result<Video<'a>, String> {
        // Setup video struct
        let mut vid = Video { args: args, num_frames: NumFrames::Num(0), folder: args.vid_folder.clone(), file: args.vid_file.clone(), name: args.name.clone(), out: args.out.clone(), fps: 0.0, calc_fps: args.calc_fps, durr: args.dur, start: args.start, temp: false };
        load_vid_data(&mut vid, args)?;
        Ok(vid)
    }

    pub fn create_app(&mut self) -> Result<(), String> {
        print_ln_if("".to_string(), !self.args.mute);
        let mut app = App::new(self.args, &self)?;
        let mut cur_frame = 0;
        let mut auditer = audiostream::AudIter::new(&strcat!(self.folder, "audio.wav"), 8, 120)?;
        
        // Skip audio before start of encoded video
        // TODO: Make negative offset work with start at 0
        for _i in 0..(self.start as isize + self.args.audoff as isize) {
            let _ = auditer.next().unwrap();
        }
        loop {
            // Get frame number to encode
            let src_frame = (((cur_frame + self.start) as f64 / self.calc_fps) * self.fps) as usize + 1;
            // Check on ffmpeg thread
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
            // Check if done with encoding frames
            if self.durr != 0 && cur_frame >= self.durr {
                break;
            }
            
            // Load image & audio data
            let fpath = strcat!(self.folder, "frame", src_frame.to_string(), ".png");
            let img = loadimg::load_interleaved(&fpath, self.args.dither, self.args.dbg_out)?;
            let aud = auditer.next().unwrap();
            // Add to app
            app.add_frame(&img, &aud)?;
            // Print progress
            app.print_progress(self.durr, 0);
            cur_frame += 1;
        }
        // Finish app
        let (num_pages, avg_img, avg_aud, avg_cycle) = app.finish()?;
        app.print_progress(self.durr, num_pages);
        print_ln_if("\nFinished converting app".to_string(), !self.args.mute);
        print_ln_if(format!("Avg. Img Frame Size: {}", avg_img), !self.args.mute);
        print_ln_if(format!("Avg. Aud Frame Size: {}", avg_aud), !self.args.mute);
        print_ln_if(format!("Avg.  Frame  Cycles: {}", avg_cycle), !self.args.mute);
        // Get keyfile location
        let key_path = match &self.args.key_source {
            None => {
                passerr!(find_file_exe("0104.key", &[ Path::new("keys").to_path_buf(),
                                                      Path::new("rabbitsign").to_path_buf(),
                                                      Path::new("./").to_path_buf() ]))
            }, Some(path_str) => Path::new(&path_str).to_path_buf(),
        };
        // Check if local or global rabbitsign executable (def not correct terminology)
        let bin_path = strcat!(self.folder, "out.bin");
        let exe_path = match std::path::Path::new("rabbitsign/").exists() {
            true => "rabbitsign/rabbitsign",
            false => "rabbitsign",
        };
        // Run rabbitsign
        let output = passerr!(Command::new(exe_path).args(["-g", "-v", "-P", "-p", "-k", key_path.to_str().unwrap(), &bin_path, "-o", &strcat!(self.args.out, ".8xk")]).output(), "{}: Could not find rabbitsign program, double-check installation instructions");
        if !output.status.success() {
            io::stdout().write_all(&output.stderr).unwrap();
            io::stdout().write_all(&output.stdout).unwrap();
            Err("Failed to sign app, see above".to_string())
        } else {
            print_ln_if("\nFinished signing app".to_string(), !self.args.mute);
            Ok(())
        }
    }
    
    fn try_recv(&mut self) -> bool{
        match &self.num_frames {
            NumFrames::Rec(rec) => {
                // Waiting on thread
                match rec.try_recv() {
                    Ok(num) => {
                        // Thread has finished and given us the total number of frames
                        self.num_frames = NumFrames::Num(num);
                        // Set durration
                        self.set_durr(num);
                        true
                    },
                    Err(_) => false
                }
            },
            NumFrames::Num(num) => {
                self.set_durr(*num);    // Why??????
                true
            },
        }
    }
    fn set_durr(&mut self, max: usize) {
        let max_durr = (((max-1) as f64 / self.fps) * self.calc_fps) as usize - self.start;
        if self.durr == 0 || self.durr > max_durr {
            self.durr = max_durr;
        }
    }
    
    pub fn close(mut self) -> Result<(), String> {
        if !self.temp {
            while !self.try_recv() {}
            save_vid_data(&self, self.args)?;
        }
        Ok(())
    }
    

}





