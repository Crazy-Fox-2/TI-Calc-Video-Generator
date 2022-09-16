use crate::video::img::*;
use std::fs;
use std::process::Command;
use crate::macros::*;
use std::env;



pub struct Video {
    path: String,
    name: String,
    fps: f64,
    num_frames: usize,
    
}
impl Video {
    
    pub fn new(vid_path: &str, name: &str, folder_path: &str) -> Result<Video, String> {
        // Attempt to extract video
        match Self::extract_video(vid_path, folder_path, name) {
            Ok(vid) => Ok(vid),
            Err(s) => Err(format!("Error during video extraction: {}", s))
        }
    }

    pub fn convert(&mut self) -> Result<(), String> {
        // Create new app
        // Load the base app
        let mut path = match env::current_exe() {
            Ok(mut exe_path) => {
                // Get path without the executable
                exe_path.pop();
                exe_path
            },
            Err(s) => return Err("Error getting path of executable".to_string())
        };
        path.push("appbase.bin");
        
        let mut page = fs::read(path).expect("Error reading base app file");
        let mut data_ptr = page.len();
        page.resize(4096, 0);
        let mut dict_ptr = 4096;
        let mut framenum = 1;
        loop {
            // Encode & Compress image & audio
            let mut fpath = self.path.clone();
            fpath.push_str("frame");    fpath.push_str(&framenum.to_string());  fpath.push_str(".png");
            let mut img = Img::load(&fpath)?;
            img.compress();
        }
        
    }
    
    fn extract_video(vid_path: &str, folder_path: &str, name: &str) -> Result<Video, String> {
        // Check if video folder exists
        if fs::metadata(folder_path).is_ok() {
            // Remove folder
            passerr!(fs::remove_dir_all(folder_path));

        }
        // Create new folder
        passerr!(fs::create_dir(folder_path));
        
        // Extract video frames to this new folder using ffmpeg
        // In theory it would be better to use gstreamer or ffmpeg rust libraries, but those make
        // my head spin and this is easier. Sorry.
        let mut fname_arg = folder_path.to_string();    fname_arg.push_str("frame%1d.png");
        match Command::new("ffmpeg").args(["-r", "1", "-i", vid_path, "-r", "1", &fname_arg, "-y"]).output() {
            Ok(_) => {},
            Err(e) => { 
                return Err(format!("Error extracting video frames: {}", e));
            }
        };
        // Get number of files in the folder, this is the number of frames
        let paths = passerr!(fs::read_dir(folder_path));
        let num_frames = paths.count();
        if num_frames == 0 {
            return Err("No video frames extracted, is ffmpeg istalled?".to_string());
        }
        // Extract audio
        let mut aname_arg = folder_path.to_string();    aname_arg.push_str("audio.wav");
        match Command::new("ffmpeg").args(["-i", vid_path, "-f", "wav", "-ab", "10920", "-ac", "1", "-vn", &aname_arg]).output() {
            Ok(_) => {},
            Err(e) => {
                return Err(format!("Error extracting audio: {}", e));
            }
        }
        // Get framerate
        let fps: f64 = match Command::new("ffprobe").args(["-v", "0", "-of", "csv=p=0", "-select_streams", "V:0", "-show_entries", "stream=avg_frame_rate", vid_path]).output() {
            Ok(out) => {
                // Figure out framerate from output
                let s = match std::str::from_utf8(&out.stdout) {
                    Ok(v) => v,
                    Err(e) => return Err(format!("Invalid UTF=8 sequence when extracting framerate: {}", e))
                };
                let mut split = s.split(&['/', '\n']);
                let num: i32 = split.next().unwrap().parse().unwrap();
                let den: i32 = split.next().unwrap().parse().unwrap();
                let fps: f64 = num as f64 / den as f64;
                fps
            },
            Err(e) => {
                return Err(format!("Error extracting framerate: {}", e));
            }
        };
        
        // Setup struct
        Ok( Video { path: folder_path.to_string(), name: name.to_string(), fps: fps, num_frames: num_frames } )
    }


}
