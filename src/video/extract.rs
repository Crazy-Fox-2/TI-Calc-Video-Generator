use crate::NumFrames;
use std::process::Command;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::fs;
use std::thread;
use crate::helper::funcs::*;
use crate::VArgs;
use crate::Video;
use crate::helper::macros::{passerr, strcat};


pub fn load_vid_data(video: &mut Video, args: &VArgs) -> Result<(), String> {
    if video.file.len() > 0 {
        // Check if temperary folder
        if video.folder.len() == 0 {
            video.temp = true;
            video.folder = "tempvid__/".to_string();
        }
        // Extract video into folder
        let (fps, rec) = extract_video(args, &video.folder, &video.file)?;
        video.fps = fps;    video.num_frames = NumFrames::Rec(rec);
    } else {
        // Check that video folder is defined
        if video.folder.len() == 0 {
            return Err("Error: Either video file or video folder must be supplied".to_string());
        }
        // Load video information from file
        let save_file = strcat!(video.folder, "save.txt");
        let contents = passerr!(fs::read_to_string(save_file), "Error loading video data file: {}");
        let mut lines = contents.split('\n');
        video.fps = lines.next().unwrap().parse().unwrap();
        video.calc_fps = lines.next().unwrap().parse().unwrap();
        video.num_frames = NumFrames::Num(lines.next().unwrap().parse().unwrap());
    }
    Ok(())
}
pub fn save_vid_data(video: &Video, _args: &VArgs) -> Result<(), String> {
    match video.num_frames {
        NumFrames::Rec(_) => Err("Error: Number of frames not known during video data save".to_string()),
        NumFrames::Num(n) => {
            if video.temp {
                // Do not save information, delete folder
                passerr!(fs::remove_dir_all(&video.folder));
            } else {
                // Save video information to file
                let data = strcat!(video.fps.to_string(), "\n", video.calc_fps.to_string(), "\n", n.to_string(), "\n");
                let save_file = strcat!(video.folder, "save.txt");
                fs::write(save_file, data).expect("Unable to write file");
            }
            Ok(())
        }
    }
}


pub fn extract_video(args: &VArgs, folder_path: &str, vid_path: &str) -> Result<(f64, Receiver<usize>), String> {   // Framerate, reciever gets number of frames once ffmpeg is finished
    // Check if video folder exists
    if fs::metadata(folder_path).is_ok() {
        // Remove folder
        passerr!(fs::remove_dir_all(folder_path));
    }
    // Create new folder
    passerr!(fs::create_dir(folder_path));
    // Check if video file exists
    if !fs::metadata(vid_path).is_ok() {
        return Err("Could not locate given video file, you sure it exists?".to_string());
    }
    // Extract audio
    print_ln_if("Extracting audio stream".to_string(), !args.mute);
    let sample_rate: usize = (args.calc_fps * 512.0) as usize;
    let mut aname_arg = folder_path.to_string();    aname_arg.push_str("audio.wav");
    match Command::new("ffmpeg").args(["-i", vid_path, "-f", "wav", "-ar", &sample_rate.to_string(), "-ac", "1", "-vn", &aname_arg]).output() {
        Ok(_) => {},
        Err(e) => {
            return Err(format!("{}: Failed to run ffmpeg, double-check installation instructions", e));
        }
    }
    // Get framerate
    print_if("Extracting frame rate: ".to_string(), !args.mute);
    let fps: f64 = match Command::new("ffprobe").args(["-v", "0", "-of", "csv=p=0", "-select_streams", "V:0", "-show_entries", "stream=avg_frame_rate", vid_path]).output() {
        Ok(out) => {
            // Figure out framerate from output
            let s = match std::str::from_utf8(&out.stdout) {
                Ok(v) => v,
                Err(e) => return Err(format!("Invalid UTF=8 sequence when extracting framerate: {}", e))
            };
            //println!("{}", s);
            let mut split = s.split(&['/', '\n', '\r']);
            //println!("{}", split.next().unwrap());
            //println!("{}", split.next().unwrap());
            let num: i32 = split.next().unwrap().parse().unwrap();
            let den: i32 = split.next().unwrap().parse().unwrap();
            let fps: f64 = num as f64 / den as f64;
            //panic!();
            fps
        },
        Err(e) => {
            return Err(format!("Error extracting framerate: {}", e));
        }
    };
    print_ln_if(fps.to_string(), !args.mute);
    
    
    // Extract video frames to this new folder using ffmpeg
    // Spin up new thread so we can continue doing stuff while ffmpeg runs
    // In theory it would be better to use gstreamer or ffmpeg rust libraries, but those make
    // my head spin and this is easier. Sorry.
    print_ln_if("Extracting video frames".to_string(), !args.mute);
    let (tx, rx): (Sender<usize>, Receiver<usize>) = mpsc::channel();
    let folder_path_clone = folder_path.to_string();
    let vid_path_clone = vid_path.to_string();
    thread::spawn(move || {
        // Code in here will be executed in a new thread
        let fname_arg = strcat!(folder_path_clone, "frame%1d.png");
        match Command::new("ffmpeg").args(["-r", "1", "-i", &vid_path_clone, "-r", "1", &fname_arg, "-y"]).output() {
            Ok(_) => {},
            Err(e) => { 
                println!("Error extracting video frames: {}", e);
            }
        };
        // Get number of files in the folder, this is the number of frames
        let paths = passerr!(fs::read_dir(folder_path_clone));
        let num_frames = paths.count();
        if num_frames == 0 {
            println!("No video frames extracted, is ffmpeg istalled?");
            tx.send(0).unwrap();
        } else {
            tx.send(num_frames).expect("Error sending data to channel, Press Ctrl+C to end");
        }
        Ok(())
    });
    Ok((fps, rx))
}
