use crate::compress;
use crate::load::{loadimg, audiostream};
use crate::helper::funcs::{vec_copy, find_fit, print_if, print_ln_if};
use crate::helper::macros::passerr;

use std::fs;
use std::process::Command;
use std::env;
use std::path;
use std::fs::File;
use std::io::Write;
use std::io::SeekFrom;
use std::io::Seek;
use std::cmp;


const PAGE_SIZE: usize = 16384;
const DICT_END: usize = 4096;



pub struct Video {
    path: String,
    name: String,
    fps: f64,
    num_frames: usize,
    
}
impl Video {
    
    pub fn new(vid_path: &str, name: &str, folder_path: &str, verb: bool) -> Result<Video, String> {
        // Attempt to extract video
        match Self::extract_video(vid_path, folder_path, name, verb) {
            Ok(vid) => Ok(vid),
            Err(s) => Err(format!("Error during video extraction: {}", s))
        }
    }

    pub fn convert(&mut self, frame_start: usize, max_frames: usize, verb: bool) -> Result<(), String> {
        let mut total_img_size = 0;
        let mut total_aud_size = 0;
        // Create new app
        // Load the base app
        let mut page = passerr!(file_from_wd_or_exe("appbase.bin"));
        let mut data_start = page.len();
        assert!(data_start < DICT_END - 256, "Base app does not leave enough room for the dictionary");
        page.resize(PAGE_SIZE, 255);
        let mut framenum = 0;
        let mut pagenum = 0;
        // Setup audio iterator
        let mut apath = self.path.clone();
        apath.push_str("audio.wav");
        let mut auditer = audiostream::AudIter::new(&apath)?;
        // Skip audio frames
        for _i in 0..frame_start {
            auditer.next();
        }
        print_ln_if("".to_string(), verb);
        // Setup output file
        let mut outpath = self.path.clone();
        outpath.push_str("out.bin");
        let mut out = passerr!(File::options().write(true).create(true).open(&outpath), "Error opening output file: {}");
        out.seek(SeekFrom::Start(0)).unwrap();
        // Setup list of compressed frames
        let mut est_size = data_start;
        let mut frame_sizes: Vec<usize> = Vec::new();
        let mut frame_imgs: Vec<Vec<u8>> = Vec::new();
        let mut frame_auds: Vec<Vec<u8>> = Vec::new();
        loop {
            // Get the frame number from the origional video
            let orig_frame = (((framenum + frame_start) as f64 * (self.fps / 20.0)) as usize) + 1;
            if orig_frame > self.num_frames {
                break;
            }
            if max_frames != 0 && framenum > max_frames {
                break;
            }
            // Encode & Compress image & audio
            let mut fpath = self.path.clone();
            fpath.push_str("frame");    fpath.push_str(&orig_frame.to_string());  fpath.push_str(".png");
            let img = loadimg::load_interleaved(&fpath)?;
            let img_data = compress::lzss_alt::compress(img);
            let aud = auditer.next().unwrap();
            let aud_data = compress::diff::compress(aud);
            // Add frame
            let frame_size = img_data.len() + aud_data.len();
            total_img_size += img_data.len();
            total_aud_size += aud_data.len();
            est_size += frame_size + 4;
            frame_sizes.push(img_data.len());   // Push img first
            frame_sizes.push(aud_data.len());
            frame_imgs.push(img_data);
            frame_auds.push(aud_data);
            if est_size >= PAGE_SIZE {
                write_page(&mut out, &mut page, &mut frame_sizes, &mut frame_imgs, &mut frame_auds, &mut pagenum, &mut data_start)?;
                est_size = 0;
            }
            print_progress(framenum, cmp::min(self.num_frames, max_frames), pagenum, verb);
            framenum += 1;
        }
        // Write data for last page & write it to file
        let mut data_end = write_page(&mut out, &mut page, &mut frame_sizes, &mut frame_imgs, &mut frame_auds, &mut pagenum, &mut data_start)?;
        if frame_sizes.len() > 0 {
            passerr!(out.write(&page));
            data_end = write_page(&mut out, &mut page, &mut frame_sizes, &mut frame_imgs, &mut frame_auds, &mut pagenum, &mut data_start)?;
        }
        page[DICT_END-2] = 255;
        passerr!(out.write(&page));
        print_progress(framenum-1, cmp::min(self.num_frames, max_frames), pagenum, verb);
        // Write app name & number of pages
        out.seek(SeekFrom::Start(0x0C)).unwrap();
        for c in self.name.chars() {
            if c.is_alphanumeric() {
                passerr!(out.write(&[c as u8]));
            } else {
                passerr!(out.write(&[0x20]));
            }
        }
        out.seek(SeekFrom::Start(0x16)).unwrap();
        passerr!(out.write(&[pagenum+1 as u8]));
        
        passerr!(out.set_len(((pagenum as usize * PAGE_SIZE) + data_end) as u64));
        
        println!("Average image size: {}", (total_img_size / framenum));
        println!("Average audio size: {}", (total_aud_size / framenum));
        
        fn write_page(out: &mut File, page: &mut Vec<u8>, sizes: &mut Vec<usize>, imgs: &mut Vec<Vec<u8>>, auds: &mut Vec<Vec<u8>>, pagenum: &mut u8, data_start: &mut usize) -> Result<usize, String> {
            // Page number & begin/end flag
            page[DICT_END-1] = *pagenum;
            page[DICT_END-2] = match *pagenum {
                0 => 1,
                _ => 0
            };
            let mut data_pos;
            // Try to fit in as many frames as we can: first try all frames, then all frames but
            // the last, etc.
            // Frames that did not fit will be put in the new vectors
            let mut sizes_next: Vec<usize> = Vec::new();
            let mut imgs_next: Vec<Vec<u8>> = Vec::new();
            let mut auds_next: Vec<Vec<u8>> = Vec::new();
            loop {
                let dict_size = imgs.len() * 4 + 4;
                let dict_start = DICT_END - dict_size;
                // Get the most efficient fit into the section before the dictionary
                let sec1_len = dict_start - *data_start;
                let (_, marked, _) = find_fit(&sizes, sec1_len);
                // Get how long the section after the dictionary will be
                let mut sec2_len = 0;
                for i in 0..sizes.len() {
                    if marked[i] == false {
                        sec2_len += sizes[i];
                    }
                }
                if sec2_len >= PAGE_SIZE - DICT_END {
                    // Will not fit, remove last frame and try again
                    sizes_next.insert(0, sizes.pop().unwrap());
                    sizes_next.insert(0, sizes.pop().unwrap());
                    imgs_next.insert(0, imgs.pop().unwrap());
                    auds_next.insert(0, auds.pop().unwrap());
                } else {
                    // Will fit, write data & dictionary to page
                    // Write dict start position
                    page[DICT_END-3] = (dict_start / 256) as u8;
                    page[DICT_END-4] = (dict_start % 256) as u8;
                    // Write frames before dictionary
                    data_pos = *data_start;
                    for i in 0..sizes.len() {
                        if marked[i] == true {
                            // Get position in dictionary
                            let dict_pos = dict_start + (2 * i);
                            // Get data
                            let data = match i % 2 {
                                0 => &imgs[i / 2],
                                _ => &auds[i / 2]
                            };
                            // Copy data into page
                            vec_copy(page, data_pos, data, 0, data.len());
                            // Write data position
                            page[dict_pos] = (data_pos % 256) as u8;
                            page[dict_pos+1] = (data_pos / 256) as u8;
                            data_pos += data.len();
                        }
                    }
                    // Write frames after dictionary
                    data_pos = DICT_END;
                    for i in 0..sizes.len() {
                        if marked[i] == false {
                            // Get position in dictionary
                            let dict_pos = dict_start + (2 * i);
                            // Get data
                            let data = match i % 2 {
                                0 => &imgs[i / 2],
                                _ => &auds[i / 2]
                            };
                            // Copy data into page
                            vec_copy(page, data_pos, data, 0, data.len());
                            // Write data position
                            page[dict_pos] = (data_pos % 256) as u8;
                            page[dict_pos+1] = (data_pos / 256) as u8;
                            data_pos += data.len();
                        }
                    }
                    break;
                }
                    
            }

            // Write page to output
            passerr!(out.write(page));
            
            page[0] = 0xBA;
            *data_start = 1;
            *pagenum += 1;
            *sizes = sizes_next;
            *imgs = imgs_next;
            *auds = auds_next;

            Ok(data_pos)
            
        }
        
        Ok(())
        
        
    }

    
    
    fn extract_video(vid_path: &str, folder_path: &str, name: &str, verb: bool) -> Result<Video, String> {
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
        print_ln_if("Extracting video frames, this may take some time.".to_string(), verb);
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
        print_ln_if("Extracting audio stream".to_string(), verb);
        let mut aname_arg = folder_path.to_string();    aname_arg.push_str("audio.wav");
        match Command::new("ffmpeg").args(["-i", vid_path, "-f", "wav", "-ar", "10920", "-ac", "1", "-vn", &aname_arg]).output() {
            Ok(_) => {},
            Err(e) => {
                return Err(format!("Error extracting audio: {}", e));
            }
        }
        // Get framerate
        print_if("Extracting frame rate: ".to_string(), verb);
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
        print_ln_if(fps.to_string(), verb);
        
        // Setup struct
        Ok( Video { path: folder_path.to_string(), name: name.to_string(), fps: fps, num_frames: num_frames } )
    }


}



fn print_progress(frame: usize, num_frames: usize, page: u8, verbose: bool) {
    if verbose {
        let base = match page <= 30 {
            true => 'X',
            false => ' ',
        };
        let se = match page <= 94 {
            true => 'X',
            false => ' ',
        };
        println!("\x1B[1AFrames:{:04}/{:04}   Pages:{:02}/??    Will fit on:  ({}) 84+  ({}) 83/84+SE", frame, num_frames, page, base, se);
    }
}



fn file_from_wd_or_exe(name: &str) -> Result<Vec<u8>, String> {
    // Searches for the given file in the path of the executable then in the working directory
    // First search in folder with executable
    match env::current_exe() {
        Ok(mut exe_path) => {
            exe_path.pop();
            match file_from_dir(exe_path, name) {
                Ok(data) => return Ok(data),
                Err(_) => {},
            }
        },
        Err(_) => {},
    };
    // Second search in working directory
    match env::current_dir() {
        Ok(path) => {
            Ok(passerr!(file_from_dir(path, name), "Error reading base app file: {}, is it in the executable folder or in the working directory?"))
        },
        Err(_err) => {
            Err("Error finding both executable path and working directory".to_string())
        }
    }
}
pub fn file_from_dir(mut path: path::PathBuf, name: &str) -> Result<Vec<u8>, String> {
    path.push(name);
    Ok(passerr!(fs::read(path)))
}
