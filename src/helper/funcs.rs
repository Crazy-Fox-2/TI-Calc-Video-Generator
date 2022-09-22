

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
// The origional code is commented out below the new code.
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
    
    
    
    /*
    
    let len = sizes.len();
    let mut marked: Vec<bool> = vec![false; len];
    let mut ptr = 0;
    let mut total = 0;
    let mut max_total = 0;
    let mut max_marked = marked.clone();
    
    'main: loop {
        
        loop {
            // Check if adding this value will keep us under the target
            if total + sizes[ptr] <= target {
                // Add value at pointer & mark as added
                total += sizes[ptr];
                marked[ptr] = true;
                // Move to next value
                // If we can't then continue to evaluate current total
                if ptr+1 >= len {
                    break;
                }
                ptr += 1;
            } else {
                // Continue to evaluate current total
                break;
            }
        }
        
        // Is current total greater than max total?
        if total > max_total {
            max_total = total;
            max_marked = marked.clone();
        }
        
        loop {
            // Move back to previous marked value
            // If there are no more, then we're done
            while marked[ptr] == false {
                if ptr == 0 {
                    break 'main;
                }
                ptr -= 1;
            }
            // Unmark this one & subtract from total
            marked[ptr] = false;
            total -= sizes[ptr];
            // Begin evaluating next value if we have one, otherwise loop
            if ptr+1 < len {
                ptr += 1;
                break;
            }
        }

        println!("{:?}", marked);
        
    }

    panic!();
    
    (max_marked, max_total)

    */
    
}

