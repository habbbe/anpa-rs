#[macro_export]
macro_rules! create_parser {
    ($state:ident, $f:expr) => {
        move |$state: &mut AnpaState<_, _>| $f
    }
}

#[macro_export]
macro_rules! lift {
    ($f:expr, $($e:expr),*) => {
        create_parser!(s, Some($f($($e(s)?),*)))
    };
}

#[macro_export]
macro_rules! pure {
    ($e:expr) => {
        create_parser!(_s, Some($e))
    };
}

#[macro_export]
macro_rules! variadic {
    ($f:ident, $e:expr) => {
        $e
    };
    ($f:ident, $e:expr, $e2:expr) => {
        $f($e, $e2)
    };
    ($f:ident, $e:expr, $($e2:expr),*) => {
        $f($e, variadic!($f, $($e2),*))
    };
}

#[macro_export]
macro_rules! or {
    ($($e:expr),*) => {
        variadic!(or, $($e),*)
    };
}

#[macro_export]
macro_rules! or_diff {
    ($($e:expr),*) => {
        variadic!(or_diff, $($e),*)
    };
}

#[macro_export]
macro_rules! left {
    ($($e:expr),*) => {
        variadic!(left, $($e),*)
    };
}

#[macro_export]
macro_rules! right {
    ($($e:expr),*) => {
        variadic!(right, $($e),*)
    };
}
