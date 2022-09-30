

#[macro_export]
macro_rules! ex_variant {
    ($variant:path, $enum:expr) => {
        if let $variant(var) = $enum {
            var
        } else {
            panic!()
        }
    };
}
pub(crate) use ex_variant;


#[macro_export]
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


#[macro_export]
macro_rules! strcat {
    ($($s:expr),*) => {{
        let mut cat: String = String::new();
        $(
            cat.push_str(&$s);
        )*
        cat
    }};
}
pub(crate) use strcat;
