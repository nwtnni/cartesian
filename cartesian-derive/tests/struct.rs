use cartesian_derive::Cartesian;

#[derive(Cartesian, Debug, PartialEq, Eq)]
struct Named {
    a: u32,
    b: u64,
}

#[test]
fn named() {
    assert_eq!(
        NamedCartesian {
            a: vec![3, 4],
            b: vec![1, 2]
        }
        .cartesian()
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
        TupleCartesian(vec![3, 4], vec![1, 2],)
            .cartesian()
            .collect::<Vec<_>>(),
        vec![Tuple(3, 1), Tuple(3, 2), Tuple(4, 1), Tuple(4, 2)],
    )
}
