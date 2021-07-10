#[macro_export]
macro_rules! lift {
    ($f:expr, $($e:expr),*) => {
        |s: &mut State<_,_,_,_>| Some($f($($e(s)?),*))
    };
}

#[macro_export]
macro_rules! or {
    ($e:expr) => {
        $e
    };
    ($e:expr, $e2:expr) => {
        or($e, $e2)
    };
    ($e:expr, $($e2:expr),*) => {
        or($e, or!($($e2),*))
    };
}

#[macro_export]
macro_rules! create_parser {
    ($state:ident, $f:expr) => {
        move |$state: &mut State<_, _, I, _>| $f
    }
}
