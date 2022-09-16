


macro_rules! passerr {
    ($result:expr, $message:literal) => {
       match $result {
            Ok(ok) => ok,
            Err(err) => { return(Err(format!($message, err.to_string()))); }
        }
    };
    ($result:expr) => {
        match $result {
            Ok(ok) => ok,
            Err(err) => { return(Err(err.to_string())); }
        }
    };
}

pub(crate) use passerr;



