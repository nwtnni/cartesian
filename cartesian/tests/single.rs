use cartesian::Cartesian;
use cartesian::IntoIterCartesian;

#[derive(Cartesian, Debug, PartialEq, Eq)]
struct Single {
    #[cartesian(single)]
    first: String,

    #[cartesian(single)]
    a: u32,
    b: u32,
    #[cartesian(single)]
    c: Vec<usize>,
}

#[test]
fn flatten() {
    let single = cartesian::IntoIter::<Single> {
        first: "test".to_owned(),
        a: 1,
        b: vec![2, 3],
        c: vec![4, 5],
    };

    assert_eq! {
        single.into_iter_cartesian().collect::<Vec<_>>(),
        vec![
            Single {
                first: "test".to_owned(),
                a: 1,
                b: 2,
                c: vec![4, 5],
            },
            Single {
                first: "test".to_owned(),
                a: 1,
                b: 3,
                c: vec![4, 5],
            },
        ],
    }
}
