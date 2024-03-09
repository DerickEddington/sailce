#![allow(unused_parens, clippy::reversed_empty_ranges, clippy::almost_complete_range)]

use sailce_data_model::group::Range;


mod end
{
    use sailce_data_model::group::range::End;

    #[test]
    fn ordering()
    {
        assert_eq!(End::Closed(123.4), End::Closed(123.4));
        assert_eq!(End::Closed("foo"), End::Closed("foo"));
        assert_ne!(End::Closed([true]), End::Open);
        assert_eq!(End::<i32>::Open, End::Open);
        assert!(End::Closed(0) < End::Closed(1));
        assert!(End::Closed(&[][..]) < End::Closed(&["bar"][..]));
        assert!(End::Closed(&["bar"][..]) < End::Closed(&["bar", "zab"][..]));
        assert!(End::Closed(i8::MIN) < End::Closed(-1));
        assert!(End::Closed(1) < End::Closed(u16::MAX));
        assert!(End::Closed(u128::MAX) < End::Open);
    }
}


#[test]
fn ordering()
{
    macro_rules! case {
        ($r1:tt == $r2:expr) => {
            assert_eq!(Range::from($r1), $r2.into());
        };
        ($r1:tt != $r2:expr) => {
            assert_ne!(Into::<Range<_>>::into($r1), Range::from($r2));
        };
        ($r1:tt < $r2:expr) => {
            assert!(Range::from($r1) < $r2.into());
        };
    }

    case!((0 .. 0) == (0 .. 0));
    case!(('a' ..) == ('a' ..));
    case!((1.2 .. 3.4) != (1.2 .. 5.6));
    case!((1.22 .. 3.4) != (1.2 .. 3.4));
    case!((0 ..) != (0 .. u128::MAX));
    case!((0 .. 0) < (0 .. 1));
    case!((0 .. 3) < (1 .. 2));
    case!(('a' .. 'z') < ('d' .. 'q'));
    case!(('d' .. 'q') < ('d' .. 'z'));
    case!((42 .. u128::MAX) < (42 ..));
    case!(("bar" .. "bar") < ("foo" .. "foo"));
}


#[test]
#[allow(clippy::needless_borrows_for_generic_args)]
fn includes()
{
    macro_rules! case {
        ($r:tt : $v:expr) => {
            assert!(Range::from($r).includes($v));
        };
        ($r:tt ! $v:expr) => {
            assert!(!Range::from($r).includes($v));
        };
    }

    case!((42 .. 99) : &77);
    case!((0 .. 10) ! -1);
    case!((i128::MIN .. 0) ! 0);
    case!((i128::MIN ..) : i128::MAX);
    case!(('b' .. 'u') ! &'v');
    case!(('b' ..) : 'v');
    case!(('b' ..) ! 'a');
}


#[test]
#[allow(clippy::reversed_empty_ranges)]
fn is_empty()
{
    fn is<T: Ord>(r: impl Into<Range<T>>) -> bool
    {
        r.into().is_empty()
    }

    assert!(is(0 .. 0));
    assert!(!is(0 ..));
    assert!(is(2 .. 1));
    assert!(!is('b' .. 'e'));
    assert!(is(u128::MAX .. u128::MAX));
    assert!(!is(u128::MAX ..));
    assert!(is("bar" .. "bar"));
    assert!(is("foo" .. "foo"));
    assert!(is("foo" .. "bar"));

    assert!(Range::<i32>::empty().is_empty());
    assert!(Range::<[u8; 2]>::empty().is_empty());
}


#[test]
#[allow(clippy::needless_borrows_for_generic_args)]
fn intersection()
{
    macro_rules! case {
        ($r1:tt & $r2:tt == $r3:expr) => {
            assert_eq!(Range::from($r1).intersection(Range::from($r2)), $r3.into());
        };
    }

    case!((0 .. 0) & (0 .. 0) == (0 .. 0));
    case!((0 .. 10) & (1 .. 9) == (1 .. 9));
    case!((0 .. 9) & (1 .. 10) == (1 .. 9));
    case!((-123 ..) & (-2 .. 88) == (-2 .. 88));
    case!((-123 ..) & (-2 ..) == (-2 ..));
    case!((-32 .. -12) & (-44 ..) == (-32 .. -12));
    case!(('f' .. 'j') & ('m' .. 'r') == ('m' .. 'j'));
    case!(('f' ..) & ('m' .. 'r') == ('m' .. 'r'));
    case!(('f' .. 'j') & ('m' ..) == ('m' .. 'j'));
}


mod three_dim;

mod least;
