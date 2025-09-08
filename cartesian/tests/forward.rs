use cartesian::Cartesian;
use cartesian::IntoIterCartesian as _;

#[test]
fn custom_default() {
    #[derive(Cartesian)]
    #[cartesian(derive(Default))]
    struct CustomDefault {
        a: u32,
    }

    impl Default for CustomDefault {
        fn default() -> Self {
            Self { a: 10 }
        }
    }

    let iter = cartesian::IntoIter::<CustomDefault>::default();

    // FIXME: default should probably produce the default base type once
    assert_eq!(iter.into_iter_cartesian().count(), 0);
}
