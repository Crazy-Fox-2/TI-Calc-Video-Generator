use std::path::{self, PathBuf};
use std::env;
use crate::helper::macros::passerr;


pub fn vec_copy<T: Copy>(to: &mut Vec<T>, tp: usize, from: &Vec<T>, fp: usize, len: usize) {
    for i in 0..len {
        to[i+tp] = from[i+fp];
    }
}


pub fn print_ln_if(print: String, verbose: bool) {
    if verbose {
        println!("{}", print);
    }
}
pub fn print_if(print: String, verbose: bool) {
    if verbose {
        print!("{}", print);
    }
}

pub fn redist_range(val: f64, la: f64, ra: f64, lb: f64, rb: f64) -> f64 {
    let range_a = ra - la;
    let pos = (val - la) / range_a;
    let range_b = rb - lb;
    let newval = (pos * range_b) + lb;
    newval
}


pub fn find_file(name: &str, search_in: &[PathBuf]) -> Result<PathBuf, String> {
    // Search for file in the given directories
    for search in search_in {
        let mut path = search.clone();
        path.push(name);
        if path.exists() {
            return Ok(path.to_path_buf());
        }
    }
    Err(format!("File not found: {}", name))
}


fn find_file_parent(name: &str, search_in: &[PathBuf], parent: &PathBuf) -> Result<PathBuf, String> {
    // Search for a file, starting in a given parent directory and searching in each given
    // sub-directory
    let mut search_in_parent: Vec<PathBuf> = Vec::with_capacity(search_in.len());
    for search in search_in {
        let mut path = parent.clone();
        path.push(search.clone());
        search_in_parent.push(path.to_path_buf());
    }
    find_file(name, &search_in_parent)
}

pub fn find_file_exe(name: &str, search_in: &[PathBuf]) -> Result<PathBuf, String> {
    // Search for given file in the executable directory, searching then in the given
    // sub-directories
    let mut exe_path = passerr!(env::current_exe(), "Error finding file in executable path: {}");
    exe_path.pop();
    find_file_parent(name, search_in, &exe_path)
}

pub fn find_file_wd(name: &str, search_in: &[PathBuf]) -> Result<path::PathBuf, String> {
    // Search for given file starting in the working directory
    let wd_path = passerr!(env::current_dir(), "Error finding file in the current directory: {}");
    find_file_parent(name, search_in, &wd_path)
}
    



// Given a set of sizes, find the combination of sizes which will sum up the closest to the target
// without going over
// Returns an array of indexes to the sizes that were added, an array of booleans saying which ones
// were added, and the total sum they achieve
// 
// Origionally this function looked for the best fit, but that took forever to calculate, and after
// looking around I don't believe there are any clever tricks to speed this up enough to be
// acceptable.
// Now it'll just iterate through all the sizes, adden them up, and subtracting some when going
// over to hopefully find a combination which is "pretty good".
pub fn find_fit(sizes: &Vec<usize>, target: usize) -> (Vec<usize>, Vec<bool>, usize) {
    
    let len = sizes.len();
    let mut marked: Vec<usize> = Vec::new();
    let mut total = 0;
    let mut max_marked = marked.clone();
    let mut max_total = 0;
    
    for i in 0..len {
        // Add current to total and check if we've gone above
        total += sizes[i];
        if total > target {
            if sizes[i] > target {
                total -= sizes[i];
            } else {
                // Remove previous marked until we are back in range
                while total > target {
                    let ind = marked.pop().unwrap();    // Should only panic if one of the sizes is greater than target (which is handled above)
                    total -= sizes[ind];
                }
            }
        }
        // Set value as marked and check if this is the best so far
        marked.push(i);
        if total > max_total {
            max_total = total;
            max_marked = marked.clone();
        }
    }
    

    // Convert to boolean vector
    let mut bool_vec: Vec<bool> = Vec::with_capacity(len);
    // Indexes in marked vector are always sequential
    let marked_iter = &mut marked.into_iter();
    let mut next_marked = marked_iter.next();
    for i in 0..len {
        match next_marked {
            None => bool_vec.push(false),
            Some(ind) => {
                if i == ind {
                    bool_vec.push(true);
                    next_marked = marked_iter.next();
                } else {
                    bool_vec.push(false);
                }
            }
        }
    }

    
    (max_marked, bool_vec, max_total)
    
}

