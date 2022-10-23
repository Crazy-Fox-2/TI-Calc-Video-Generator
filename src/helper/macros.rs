

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

#[macro_export]
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
pub(crate) use bound;
