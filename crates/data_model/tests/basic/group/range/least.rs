use sailce_data_model::group::{
    range::{
        End,
        Least,
    },
    Range,
    ThreeDimRange,
};


#[test]
fn range()
{
    assert!(Range::<usize>::least().is_empty());
    assert_eq!(Range::<i32>::least(), Range { start: i32::MIN, end: End::Closed(i32::MIN) });
}


#[test]
fn three_dim()
{
    assert!(ThreeDimRange::<char, &str>::least().is_empty());
    assert_eq!(ThreeDimRange::<u32, [u8; 0]>::least(), ThreeDimRange {
        subspaces: (0 .. 0).into(),
        paths:     ([] .. []).into(),
        times:     (0 .. 0).into(),
    });
}


#[cfg(feature = "alloc")]
#[test]
fn string()
{
    let least = Range::<String>::least();

    assert!(least.is_empty());
    assert_eq!(least, Range { start: String::new(), end: End::Closed(String::new()) });
    assert!(least < (String::new() .. String::from('\0')).into());
}


#[cfg(feature = "alloc")]
#[test]
fn cow()
{
    use std::borrow::Cow;

    let least = Range::<Cow<'_, str>>::least();
    let empty = Cow::from("");

    assert!(least.is_empty());
    assert_eq!(least, Range {
        start: Cow::from(empty.clone().into_owned()),
        end:   End::Closed(empty.clone()),
    });
    assert!(least < (empty .. Cow::from("\0".to_owned())).into());
}
