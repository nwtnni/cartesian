use cartesian::Cartesian;
use cartesian::IntoIterCartesian;

#[derive(Cartesian, Debug, PartialEq, Eq)]
// NOTE: need to separate derive macros so `derive(Clone)` is
// visible to `derive(Cartesian)`, and can propagate to the
// generated type.
// See: https://internals.rust-lang.org/t/question-does-rust-as-a-language-require-derive-macros-be-kept-in-order/15947/18
#[rustfmt::skip]
#[derive(Clone)]
struct Inner {
    a: u64,
}

#[derive(Cartesian, Debug, PartialEq, Eq)]
struct Outer {
    #[cartesian(flatten)]
    inner_1: Inner,
    b: u32,
    c: u32,
    #[cartesian(flatten)]
    inner_2: Inner,
}

#[test]
fn flatten() {
    let outer = cartesian::IntoIter::<Outer> {
        inner_1: cartesian::IntoIter::<Inner> { a: vec![1, 2] },
        b: vec![10],
        c: vec![11],
        inner_2: cartesian::IntoIter::<Inner> { a: vec![3, 4] },
    };

    assert_eq! {
        outer.into_iter_cartesian().collect::<Vec<_>>(),
        vec![
            Outer {
                inner_1: Inner { a: 1 },
                b: 10,
                c: 11,
                inner_2: Inner { a: 3 },
            },
            Outer {
                inner_1: Inner { a: 1 },
                b: 10,
                c: 11,
                inner_2: Inner { a: 4 },
            },
            Outer {
                inner_1: Inner { a: 2 },
                b: 10,
                c: 11,
                inner_2: Inner { a: 3 },
            },
            Outer {
                inner_1: Inner { a: 2 },
                b: 10,
                c: 11,
                inner_2: Inner { a: 4 },
            },
        ],
    }
}
