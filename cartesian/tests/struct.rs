use cartesian::Cartesian;
use cartesian::IntoIterCartesian as _;

#[derive(Cartesian, Debug, PartialEq, Eq)]
struct Named {
    a: u32,
    b: u64,
}

#[test]
fn named() {
    assert_eq!(
        cartesian::IntoIter::<Named> {
            a: vec![3, 4],
            b: vec![1, 2]
        }
        .into_iter_cartesian()
        .collect::<Vec<_>>(),
        vec![
            Named { a: 3, b: 1 },
            Named { a: 3, b: 2 },
            Named { a: 4, b: 1 },
            Named { a: 4, b: 2 }
        ],
    )
}

#[derive(Cartesian, Debug, PartialEq, Eq)]
struct Tuple(u32, u64);

#[test]
fn tuple_struct() {
    assert_eq!(
        // Type alias does not define constructor function for tuple :(
        // https://users.rust-lang.org/t/error-expected-function-tuple-struct-or-tuple-variant-found-type-alias-rgbspectrum/77571
        cartesian::IntoIter::<Tuple> {
            0: vec![3, 4],
            1: vec![1, 2]
        }
        .into_iter_cartesian()
        .collect::<Vec<_>>(),
        vec![Tuple(3, 1), Tuple(3, 2), Tuple(4, 1), Tuple(4, 2)],
    )
}
