use crate::core::{*};

pub fn bind<T, I: Iterator<Item=T>, S, R, R2, P>(parser: impl Parser<T, I, S, R> + Copy, f: impl Fn(R) -> P + Copy) -> impl Parser<T, I, S, R2> + Copy
    where P: Parser<T, I, S, R2> {
    create_parser!(s, f(parser(s)?)(s))
}

pub fn right<T, I: Iterator<Item=T>, S, R, R2>(p1: impl Parser<T, I, S, R> + Copy, p2: impl Parser<T, I, S, R2> + Copy) -> impl Parser<T, I, S, R2> + Copy {
    create_parser!(s, {p1(s)?; p2(s)})
}

pub fn left<T, I: Iterator<Item=T> + Clone, S, R, R2>(p1: impl Parser<T, I, S, R> + Copy, p2: impl Parser<T, I, S, R2> + Copy) -> impl Parser<T, I, S, R> + Copy {
    create_parser!(s, {
        if let a@Some(_) = p1(s) {
            p2(s)?;
            a
        } else {
            None
        }
    })
}

pub fn or_diff<T, I: Iterator<Item=T> + Clone, S, R, R2>(p1: impl Parser<T, I, S, R> + Copy, p2: impl Parser<T, I, S, R2> + Copy) -> impl Parser<T, I, S, ()> + Copy {
    create_parser!(s, {
        let pos = s.iterator.clone();
        return if let Some(_) = p1(s) {
            Some(())
        } else {
            s.iterator = pos;
            p2(s).map(|_| ())
        }
    })
}

pub fn or<T, I: Iterator<Item=T> + Clone, S, R>(p1: impl Parser<T, I, S, R> + Copy, p2: impl Parser<T, I, S, R> + Copy) -> impl Parser<T, I, S, R> + Copy {
    create_parser!(s, {
        let pos = s.iterator.clone();
        p1(s).or_else(|| { s.iterator = pos; p2(s)})
    })
}
//
pub fn try_parse<T, I: Iterator<Item=T> + Clone, S, R>(p1: impl Parser<T, I, S, R>) -> impl Parser<T, I, S, R> {
    create_parser!(s, {
        let pos = s.iterator.clone();
        match p1(s) {
            a@Some(_) => a,
            None => { s.iterator = pos; None }
        }
    })
}

// pub fn recursive<T, I: Iterator<Item=T>, S, R, P, F>(f: F) -> impl Parser<T, I, S, R>
//     where
//         P: Parser<T, I, S, R>,
//         F: Fn(&dyn Parser<T, I, S, R>) -> P + Copy {
// // F: Fn(&dyn Parser<T, I, R>) -> P + Copy {
// // F: Fn(&dyn Fn(&mut State<T, I>) -> Option<R>) -> P + Copy {
//         move |s: &mut State<T, I, S>| {
//         // fn rec<R, T, I: Iterator<Item=T>, P: Parser<T, I, R>, F: Fn(P) -> P>(f: F, s: &mut State<T, I>) -> Option<R> {
//         //     f(recursive(f))(s)
//         // }
//         let p = move |s: &mut _| {
//             recursive(f)(s)
//         };
//
//         f(&p)(s)
//     }
//
//     // fn rec<R, T, I: Iterator<Item=T>, P: Parser<T, I, R>, F: Fn(P) -> P>(f: F, s: &mut State<T, I>) -> Option<R> {
//     //     f(recursive(f))(s)
//     // }
//     // let p = |s: &mut _| {
//     //     recursive(f)(s)
//     // };
// }

pub fn not_empty<T, I: Iterator<Item=T>, S, R, I2: Iterator<Item=R> + Clone>(p: impl Parser<T, I, S, I2> + Copy) -> impl Parser<T, I, S, I2> + Copy {
    create_parser!(s, {
        let res = p(s)?;
        res.clone().next()?;
        Some(res)
    })
}

pub fn lift_to_state<T, I: Iterator<Item=T>, S, R, R2>(f: impl FnOnce(&mut S, R) -> R2 + Copy, p: impl Parser<T, I, S, R> + Copy) -> impl Parser<T, I, S, R2> + Copy {
    create_parser!(s, {
        let res = p(s)?;
        let new_res = f(&mut s.user_state, res);
        Some(new_res)
    })
}
