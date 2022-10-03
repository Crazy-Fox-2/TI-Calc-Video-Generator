use crate::Video;
use crate::helper::funcs::*;
use crate::compress;
use std::io::{Seek, SeekFrom};
use crate::VArgs;
use std::fs::File;
use crate::helper::macros::{passerr, strcat};
use std::io::Write;

const PAGE_SIZE: usize = 16384;



pub struct App<'a> {
    total_img_size: usize,
    total_aud_size: usize,
    first_page: Vec<u8>,
    page: Vec<u8>,
    first_page_start: usize,
    frame_num: usize,
    page_num: usize,
    out: File,
    est_size: usize,
    target_size: usize,
    data_size: usize,
    frame_imgs: Vec<Vec<u8>>,
    frame_auds: Vec<Vec<u8>>,
    frame_sizes: Vec<usize>,
    args: &'a VArgs
}
impl<'a> App<'a> {
    
    pub fn new(args: &'a VArgs, vid: &Video) -> Result<App<'a>, String> {
        // Load first page
        let mut first_page = passerr!(file_from_wd_or_exe("appbase.bin"));
        let first_page_start = first_page.len();
        first_page.resize(PAGE_SIZE, 255);
        // Setup output file
        let outpath = strcat!(vid.folder, "out.bin");
        let mut out = passerr!(File::options().write(true).create(true).open(&outpath), "Error opening output file: {}");
        // Skip to second page, will write first page last
        passerr!(out.set_len(PAGE_SIZE as u64));
        passerr!(out.seek(SeekFrom::Start(PAGE_SIZE as u64)));
        Ok ( App {
            total_img_size: 0,
            total_aud_size: 0,
            first_page: first_page,
            page: vec![0xFF; PAGE_SIZE],
            first_page_start: first_page_start,
            frame_num: 0,
            page_num: 1,
            out: out,
            est_size: first_page_start,
            target_size: PAGE_SIZE*2,
            data_size: 0,
            frame_imgs: Vec::new(),
            frame_auds: Vec::new(),
            frame_sizes: Vec::new(),
            args: args,
        } )
    }
    
    pub fn add_frame(&mut self, img: &[u8], aud: &[u8]) -> Result<(), String> {
        // Compress image & audio
        let img_comp = compress::lzss_alt::compress(img);
        let aud_comp = compress::diff::compress(aud);
        // Output to debug file
        let mut file = passerr!(File::create(strcat!("dbg/img_", self.frame_num.to_string(), ".bin")));     passerr!(file.write_all(img));
        let mut file = passerr!(File::create(strcat!("dbg/imgc_", self.frame_num.to_string(), ".bin")));    passerr!(file.write_all(&img_comp));
        let mut file = passerr!(File::create(strcat!("dbg/aud_", self.frame_num.to_string(), ".bin")));     passerr!(file.write_all(aud));
        let mut file = passerr!(File::create(strcat!("dbg/audc_", self.frame_num.to_string(), ".bin")));    passerr!(file.write_all(&aud_comp));
        // Add frame to list
        let frame_size = img_comp.len() + aud_comp.len();
        self.total_img_size += img_comp.len();
        self.total_aud_size += aud_comp.len();
        self.est_size += frame_size + 4;
        self.data_size += frame_size;
        self.frame_sizes.push(img_comp.len());
        self.frame_sizes.push(aud_comp.len());
        self.frame_imgs.push(img_comp);
        self.frame_auds.push(aud_comp);
        // Write page to app if we've exceeded this page's capacity
        if self.est_size >= self.target_size {
            self.add_page(true)?;
        }
        self.frame_num += 1;
        Ok(())
    }
    
    pub fn finish(&mut self) -> Result<(usize, usize, usize), String> {
        // Finish writing app pages
        while self.frame_imgs.len() > 0 {
            self.add_page(false)?;
        }
        // Write app name & number of pages to first page
        let mut citer = self.args.name.chars();
        for i in 0..8 {
            let mut putchar: u8 = 0x20;
            match citer.next() {
                Some(c) => {
                    if c.is_alphanumeric() {
                        putchar = c as u8;
                    }
                }, None => {}
            }
            self.first_page[0x0C + i] = putchar;
        }
        self.first_page[0x16] = self.page_num as u8;
        // Write first page to file
        self.out.seek(SeekFrom::Start(0x00)).unwrap();
        passerr!(self.out.write(&self.first_page));
        Ok((self.page_num, self.total_img_size / self.frame_num, self.total_aud_size / self.frame_num))
    }
    
    fn add_page(&mut self, force_write_to_end: bool) -> Result<(), String> {
        let mut next_frame_imgs: Vec<Vec<u8>> = Vec::new();
        let mut next_frame_auds: Vec<Vec<u8>> = Vec::new();
        let mut pos;
        let mut next_est_size = 4;
        let mut next_data_size = 0;
        loop {
            // Try to write all the currently pending frames to the current page
            // If they can't all fit, set the last one asside for the next page to handle and loop
            // until they all fit
            // This will most likely fail at least once unless the compressed frames perfectly fill
            // up a page
            // Set aside space for the dictionary
            let dict_size = (self.frame_imgs.len() * 4) + 4;
            let data_size = PAGE_SIZE - dict_size;
            if self.page_num > 1 {
                if data_size < self.data_size {
                    // Not enough space, move last frame into new vectors
                    let last_img = self.frame_imgs.pop().unwrap();
                    let last_aud = self.frame_auds.pop().unwrap();
                    self.data_size -= last_img.len() + last_aud.len();
                    next_data_size += last_img.len() + last_aud.len();
                    next_est_size += last_img.len() + last_aud.len() + 4;
                    next_frame_imgs.insert(0, last_img);
                    next_frame_auds.insert(0, last_aud);
                    continue;
                }
                // Write dictionary header information
                self.page[0] = 0x50;
                self.page[1] = self.page_num as u8;
                self.page[2] = (dict_size - 4) as u8;
                self.page[3] = ((dict_size - 4) / 256 + 0x80) as u8;
                // Set page size, extend to end of page border if not last frame
                if force_write_to_end || next_frame_imgs.len() > 0 {
                    self.page.resize(PAGE_SIZE, 255);
                } else {
                    // Last page
                    self.page.resize(self.data_size + dict_size, 255);
                }
                // Write frames in-order (all image frames then all audio frames)
                pos = dict_size;
                for (i, img_comp) in self.frame_imgs.iter().enumerate() {
                    // Copy compressed data
                    vec_copy(&mut self.page, pos, img_comp, 0, img_comp.len());
                    // Write position in dictionary
                    self.page[(i*4)+4] = pos as u8;
                    self.page[(i*4)+5] = (pos / 256 + 0x80) as u8;
                    pos += img_comp.len();
                }
                for (i, aud_comp) in self.frame_auds.iter().enumerate() {
                    vec_copy(&mut self.page, pos, aud_comp, 0, aud_comp.len());
                    // Write position in dictionary
                    self.page[(i*4)+6] = pos as u8;
                    self.page[(i*4)+7] = (pos / 256 + 0x80) as u8;
                    pos += aud_comp.len();
                }
                break;
            } else {
                // The first two pages are special
                // The first contains the app header all the video-playback code at the start of
                // the page, where the dictionary usually goes
                // Because the first page is always loaded for code execution, the second page is
                // treated as the first video data page by the application and the dictionary there
                // points to data in both the second and the first page
                // Under this arrangement writing frames sequentially is not the most effieient
                // solution. Instead each frame's data is scattered between the two pages to
                // maximize the amount of frames that can be fit
                
                // Get the most efficient fit into the section before the dictionary
                let sec1_len = PAGE_SIZE - self.first_page_start;
                let (_, marked, _) = find_fit(&self.frame_sizes, sec1_len);
                // Get how long the section in the second page will be
                let mut sec2_len = 0;
                for i in 0..self.frame_sizes.len() {
                    if marked[i] == false {
                        sec2_len += self.frame_sizes[i];
                    }
                }
                if sec2_len >= PAGE_SIZE - dict_size {
                    // Not enough space, move last frame into new vectors
                    let last_img = self.frame_imgs.pop().unwrap();
                    let last_aud = self.frame_auds.pop().unwrap();
                    next_data_size += last_img.len() + last_aud.len();
                    next_est_size += last_img.len() + last_aud.len() + 4;
                    next_frame_imgs.insert(0, last_img);
                    next_frame_auds.insert(0, last_aud);
                    self.frame_sizes.pop(); self.frame_sizes.pop();
                    continue;
                }
                self.page.resize(PAGE_SIZE, 255);
                // Write dictionary header information
                self.page[0] = 0xA0;
                self.page[1] = self.page_num as u8;
                self.page[2] = (dict_size - 4) as u8;
                self.page[3] = ((dict_size - 4) / 256 + 0x80) as u8;
                // Write frames in first page
                pos = self.first_page_start;
                for i in 0..self.frame_sizes.len() {
                    if marked[i] == true {
                        // Get position in dictionary
                        let dict_pos = (2 * i) + 4;
                        // Get data
                        let data = match i % 2 {
                            0 => &self.frame_imgs[i / 2],
                            _ => &self.frame_auds[i / 2]
                        };
                        // Copy data into page
                        vec_copy(&mut self.first_page, pos, data, 0, data.len());
                        // Write data position
                        self.page[dict_pos] = (pos % 256) as u8;
                        self.page[dict_pos+1] = (pos / 256 + 0x40) as u8;
                        pos += data.len();
                    }
                }
                // Write frames in second page
                pos = dict_size;
                for i in 0..self.frame_sizes.len() {
                    if marked[i] == false {
                        // Get position in dictionary
                        let dict_pos = (2 * i) + 4;
                        // Get data
                        let data = match i % 2 {
                            0 => &self.frame_imgs[i / 2],
                            _ => &self.frame_auds[i / 2]
                        };
                        // Copy data into page
                        vec_copy(&mut self.page, pos, data, 0, data.len());
                        // Write data position
                        self.page[dict_pos] = (pos % 256) as u8;
                        self.page[dict_pos+1] = (pos / 256 + 0x80) as u8;
                        pos += data.len();
                    }
                }
                break;
            }
        }
        
        // Write first/last page flags
        if self.page_num <= 1 {
            self.page[0] += 1;  // First page
        }
        if !force_write_to_end && next_frame_imgs.len() == 0 {
            self.page[0] += 2;  // Last page
        }
        // Write page data
        passerr!(self.out.write(&self.page));
        // Setup next page
        self.frame_imgs = next_frame_imgs;
        self.frame_auds = next_frame_auds;
        self.page_num += 1;
        self.est_size = next_est_size;
        self.data_size = next_data_size;
        self.target_size = PAGE_SIZE;
        Ok(())
    }
    
    pub fn print_progress(&self, total_frames: usize, total_pages: usize) {
        if !self.args.mute {
            let base = match self.page_num <= 30 {
                true => 'X',
                false => ' ',
            };
            let se = match self.page_num <= 94 {
                true => 'X',
                false => ' ',
            };
            let total_frames = match total_frames {
                0 => "????".to_string(),
                _ => format!("{:04}", total_frames),
            };
            let total_pages = match total_pages {
                0 => "??".to_string(),
                _ => format!("{:02}", total_pages),
            };
            println!("\x1B[1AFrames:{:04}/{}   Pages:{:02}/{}    Will fit on:  ({}) 84+  ({}) 83/84+SE", self.frame_num, total_frames, self.page_num, total_pages, base, se);
        }

    }
    
}
