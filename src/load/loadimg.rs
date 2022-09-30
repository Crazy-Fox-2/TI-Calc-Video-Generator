use image::Pixel;
use image::imageops;
use image::GrayImage;
use image::DynamicImage;
use image::DynamicImage::{ImageRgba8, ImageLuma8};
use image::RgbaImage;
use crate::helper::macros::{passerr, ex_variant};
use crate::load::dither::dither;
use image::GenericImageView;


pub fn load_imgs(path: &str, keepall: bool, savefile: bool)
    -> Result<(GrayImage, Option<(RgbaImage, RgbaImage, GrayImage)>), String> {
    
    fn save_if(img: &DynamicImage, path: &str, save: bool) -> Result<(), String> {
        if save {
            passerr!(img.save(path), "Error saving image: {}");
        }
        Ok(())
    }
    
    // Read image from file
    let mut img = passerr!(image::open(path), "Error during image frame load: {}");
    // Resize & crop & make greyscale
    let (width, height) = img.dimensions();
    let ratio: f64 = width as f64 / height as f64;
    let crop = if ratio > 1.5 {
        // Crop out left & right sides
        let new_width: u32 = (height * 3) / 2;
        let x = (width - new_width) / 2;
        imageops::crop(&mut img, x, 0, new_width, height).to_image()
    } else {
        // Crop out the top & bottom
        let new_height: u32 = (width * 2) / 3;
        let y = (height - new_height) / 2;
        imageops::crop(&mut img, 0, y, width, new_height).to_image()
    };
    let resize = imageops::resize(&crop, 96, 64, imageops::FilterType::Lanczos3);
    let grey = imageops::colorops::grayscale(&resize);
    // Dither image
    let dither = dither(&grey, 'o', vec![232, 165, 68, 25]);
    // Save images
    let img = ImageRgba8(crop);   save_if(&img, "crop.png", savefile)?;     let crop = ex_variant!(ImageRgba8, img);    // Dancing around the borrow-checker
    let img = ImageRgba8(resize); save_if(&img, "resize.png", savefile)?;   let resize = ex_variant!(ImageRgba8, img);
    let img = ImageLuma8(grey);   save_if(&img, "grey.png", savefile)?;     let grey = ex_variant!(ImageLuma8, img);
    let img = ImageLuma8(dither); save_if(&img, "dither.png", savefile)?;   let dither = ex_variant!(ImageLuma8, img);
    
    if keepall {
        Ok((dither, Some((crop, resize, grey))))
    } else {
        Ok((dither, None))
    }
    
}



pub fn load_interleaved(path: &str) -> Result<Vec<u8>, String> {
    // Get dithered image
    let mut img = load_imgs(path, false, true)?.0;
    // Convert to byte stream
    let mut stream: Vec<u8> = vec![0; 12*64*2];
    let mut iter = img.pixels_mut();
    for y in 0..64 {
        for x in 0..96 {
            // Caclculate the position in the stream this pixel will get placed in
            let col = x / 8;
            let pos = (col * 64 + y) * 2;
            // Shift in most & least significant bits
            let pxl = match iter.next() {
                Some(pxl) => pxl,
                None => return Err("Number of pixels in image not as many as expected".to_string()),
            };
            let shade = 3 - (pxl.channels()[0] / 64);
            stream[pos] = (stream[pos] * 2) + (shade / 2);
            stream[pos+1] = (stream[pos+1] * 2) + (shade % 2);
        }
    }
    // Return image
    Ok(stream)
}

pub fn load_seperate(path: &str) -> Result<Vec<u8>, String> {
    // Get dithered image
    let mut img = load_imgs(path, false, true)?.0;
    // Convert to byte stream
    let mut stream: Vec<u8> = vec![0; 12*64*2];
    let mut iter = img.pixels_mut();
    for y in 0..64 {
        for x in 0..96 {
            // Caclculate the position in the stream this pixel will get placed in
            let col = x / 8;
            let pos = col * 64 + y;
            // Shift in most & least significant bits
            let pxl = match iter.next() {
                Some(pxl) => pxl,
                None => return Err("Number of pixels in image not as many as expected".to_string()),
            };
            let shade = 3 - (pxl.channels()[0] / 64);
            stream[pos] = (stream[pos] * 2) + (shade / 2);
            stream[pos+768] = (stream[pos+768] * 2) + (shade % 2);
        }
    }
    // Return image
    Ok(stream)
}

