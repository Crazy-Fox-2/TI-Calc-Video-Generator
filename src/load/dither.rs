use image::GrayImage;


// Warning,
// A good chunk of the code below was written at 8PM while I was very tiard, please do not judge my
// coding ability by what you find in this file. Thank you.





fn nearest_shade(shade: u8, allowed: &Vec<u8>) -> u8 {
    // Find the shade closest to this one
    let mut closest_off: i16 = 9999;
    let mut closest = 0;
    for i in 0..allowed.len() {
        let s = allowed[i];
        let off = (shade as i16 - s as i16).abs();
        if off < closest_off {
            closest_off = off;
            closest = s;
        }
    }
    closest
}

macro_rules! bound {
    ( $val:expr, $left:expr, $right:expr ) => {
        {
            let val = $val;
            let left = $left;
            let right = $right;
            let (left, right) = match left < right {
                true => (left, right),
                false => (right, left),
            };
            if val < left {left}
            else if val > right {right}
            else {val}
        }
    };
}


pub fn dither(source: &GrayImage, kind: char, shades: Vec<u8>) -> GrayImage {
    
    let (width, height) = source.dimensions();
    let mut img = source.clone();

    match kind {
        'f'|'s' => {
            
            // Floyd-Steinberg Dithering
            // https://en.wikipedia.org/wiki/Floyd-Steinberg_dithering (accessed on 12 Sept)
            
            fn add_error(img: &mut GrayImage, error: i16, scale: f32, x: u32, y: u32, w: u32, h: u32) {
                if x < w && y < h {
                    let pxl = img.get_pixel_mut(x, y);
                    let oldval = pxl[0];
                    let newval = (oldval as i16) + (error as f32 * scale).round() as i16;
                    let newval = if newval < 0 { 0 }
                                else if newval > 255 { 255 }
                                else { newval } as u8;
                    pxl[0] = newval;
                }
            }
            
            for y in 0..height {
                for x in 0..width {
                    let pxl = img.get_pixel_mut(x, y);
                    let oldval = pxl[0];
                    let newval = nearest_shade(oldval, &shades);
                    pxl[0] = newval;
                    let error = oldval as i16 - newval as i16;
                    add_error(&mut img, error, 0.4375, x+1, y, width, height);
                    add_error(&mut img, error, 0.1875, x-1, y+1, width, height);
                    add_error(&mut img, error, 0.3125, x, y+1, width, height);
                    add_error(&mut img, error, 0.0625, x+1, y+1, width, height);
                    //img.put_pixel(x, y, *pxl);
                }
            }
            
            img
        },
        _ => {

            // Ordered Dithering
            // https://en.wikipedia.org/wiki/Ordered_dithering (accessed on 12 Sept)
            
            fn apply_mask(x: u32, y: u32) -> f64 {
                // Mask if taken from the article above, but with the math already applied
                static MASK: &'static [[f64; 8]; 8] = &[[-0.5, 0.0, -0.375, 0.125, -0.46875, 0.03125, -0.34375, 0.15625],
                                                        [0.25, -0.25, 0.375, -0.125, 0.28125, -0.21875, 0.40625, -0.09375],
                                                        [-0.3125, 0.1875, -0.4375, 0.0625, -0.28125, 0.21875, -0.40625, 0.09375],
                                                        [0.4375, -0.0625, 0.3125, -0.1875, 0.46875, -0.03125, 0.34375, -0.15625],
                                                        [-0.453125, 0.046875, -0.328125, 0.171875, -0.484375, 0.015625, -0.359375, 0.140625],
                                                        [0.296875, -0.203125, 0.421875, -0.078125, 0.265625, -0.234375, 0.390625, -0.109375],
                                                        [-0.265625, 0.234375, -0.390625, 0.109375, -0.296875, 0.046875, -0.421875, 0.078125],
                                                        [0.484375, -0.015625, 0.359375, -0.140625, 0.453125, -0.046875, 0.328125, -0.171875]];
                MASK[y as usize % 8][x as usize % 8]
            }

            fn redist_range(val: f64, la: f64, ra: f64, lb: f64, rb: f64) -> f64 {
                let range_a = ra - la;
                let pos = (val - la) / range_a;
                let range_b = rb - lb;
                let newval = (pos * range_b) + lb;
                newval
            }
            
            // Prior to going through the algorithm normally we do one pass changing the shades
            // since as best I can tell this algorithm only works if the shades are evenly spaced
            // Each pixel between the first two shades is re-distributed to comprise the brightest
            // third of the colorspace, each pixel between the middle two is the middle third, etc.

            // Compute the dither for each pixel
            for y in 0..height {
                for x in 0..width {
                    let pxl = img.get_pixel_mut(x, y);
                    let mut val = pxl[0];
                    val = bound!(val, shades[0], shades[shades.len()-1]);
                    // Find which range it's in
                    let mut ind = shades.len()-1;
                    for i in 1..shades.len() {
                        if val >= shades[i] {
                            ind = i;
                            break;
                        }
                    }
                    // Redistribute
                    let fval = redist_range(val.try_into().unwrap(), shades[ind-1].try_into().unwrap(), shades[ind].try_into().unwrap(), (ind-1) as f64, ind as f64);
                    // Pass through mask
                    let newval = bound!(fval + apply_mask(x, y), 0.0, 3.0);
                    // Change back into allowed shades
                    val = shades[newval.round() as usize];
                    pxl[0] = val;
                }
            }

            img
            
        }
    }
    
}





