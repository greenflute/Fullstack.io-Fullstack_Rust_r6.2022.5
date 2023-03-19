macro_rules! myvec {
    ($($x:expr),*) => ({
        let mut v = Vec::new();
        $(v.push($x);)*
        v
    });
    ($($x:expr,)*) => (myvec![$($x),*])
}

fn main() {
    let a = myvec![1, 2, 3, 4];

    let aa = vec![1, 2, 3, 4];
}
