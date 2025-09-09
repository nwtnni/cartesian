use cartesian::Cartesian;
use cartesian::IntoIterCartesian as _;

#[test]
fn custom_default() {
    #[derive(Cartesian, PartialEq, Eq, Debug, Clone)]
    #[cartesian(default)]
    #[cartesian(derive(PartialEq, Eq, Debug, Clone))]
    struct CustomDefault {
        a: u32,
    }

    impl Default for CustomDefault {
        fn default() -> Self {
            Self { a: 10 }
        }
    }

    let iter = cartesian::IntoIter::<CustomDefault>::default();

    assert_eq!(iter.clone().into_iter_cartesian().count(), 1);
    assert_eq!(
        iter.into_iter_cartesian().next(),
        Some(CustomDefault::default())
    );
}
