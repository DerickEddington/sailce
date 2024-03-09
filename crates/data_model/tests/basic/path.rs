#[test]
fn is_prefix_of()
{
    use sailce_data_model::Path as _;

    fn is<const PLEN: usize, const OLEN: usize>(
        prefix: [&str; PLEN],
        of: [&str; OLEN],
    ) -> bool
    {
        prefix.is_prefix_of(&of)
    }

    assert!(is([], []));
    assert!(is([], [""]));
    assert!(is([], ["a"]));
    assert!(is([], ["", ""]));
    assert!(is([], ["a", "b"]));
    assert!(!is([""], []));
    assert!(!is(["a"], []));
    assert!(is([""], [""]));
    assert!(is([""], ["", ""]));
    assert!(!is([""], ["a"]));
    assert!(is(["a"], ["a"]));
    assert!(!is([""], ["a", "b"]));
    assert!(is(["a"], ["a", "b"]));
    assert!(!is(["a"], ["ab"]));
    assert!(!is(["a"], ["", "ab"]));
    assert!(!is(["a"], ["ab", ""]));
    assert!(is(["aaaa", "bbb", "cc"], ["aaaa", "bbb", "cc"]));
    assert!(is(["aaaa", "bbb", "cc"], ["aaaa", "bbb", "cc", "d"]));
    assert!(!is(["aaaa", "bbb", "cc", "d"], ["aaaa", "bbb", "cc"]));
}


/// Test that [`Path`] is automatically `impl`emented for all desired types.
#[test]
fn types()
{
    extern crate alloc;
    use {
        alloc::collections::{
            LinkedList,
            VecDeque,
        },
        sailce_data_model::Path,
    };

    /// Will fail to compile if given a type that doesn't `impl` [`Path`].
    fn f(path: &(impl Path + ?Sized)) -> Vec<&[u8]>
    {
        // Might as well return its components.
        path.components().map(|comp| comp.bytes).collect()
    }

    macro_rules! check {
        ([$elem_type:ty; $len:literal] = $value_expr:expr) => {{
            let array: [$elem_type; $len] = $value_expr;

            // Might as well verify its components.
            let expected: Vec<Vec<u8>> = Vec::from_iter(array.iter().map(|comp| {
                let comp: &[u8] = comp.as_ref();
                comp.to_vec()
            }));

            assert_eq!(f(&array), expected);

            let slice: &[$elem_type] = &array;
            assert_eq!(f(slice), expected);

            let boxed_array: Box<[$elem_type; $len]> = Box::new(array);
            assert_eq!(f(&*boxed_array), expected);

            let boxed_slice: Box<[$elem_type]> = boxed_array;
            assert_eq!(f(&*boxed_slice), expected);

            let vec: Vec<$elem_type> = boxed_slice.into_vec();
            assert_eq!(f(&vec), expected);

            let list: LinkedList<$elem_type> = LinkedList::from_iter(vec);
            assert_eq!(f(&list), expected);

            let deq: VecDeque<$elem_type> = VecDeque::from_iter(list);
            assert_eq!(f(&deq), expected);

            // TODO: Add more types as appropriate.
        }};
    }

    check!([[u8; 0]; 0] = []);
    check!([[u8; 1]; 1] = [*b"a"]);
    check!([[u8; 2]; 3] = [*b"aa", *b"bb", *b"cc"]);
    check!([&[u8]; 0] = []);
    check!([&[u8]; 1] = [b"a"]);
    check!([&[u8]; 3] = [b"a", b"bb", b"ccc"]);
    check!([Vec<u8>; 0] = []);
    check!([Vec<u8>; 2] = [&b"a"[..], &b"bb"[..]].map(Vec::from));
    check!([&str; 0] = []);
    check!([&str; 1] = ["a"]);
    check!([&str; 3] = ["a", "bb", "ccc"]);
    check!([String; 0] = []);
    check!([String; 2] = ["a", "bb"].map(String::from));

    assert_eq!(f(&vec!["a".to_owned(), "b".to_owned()]), [b"a", b"b"]);
}
