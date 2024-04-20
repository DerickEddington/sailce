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


/// Test that [`Path`] is automatically implemented for all desired types.
#[test]
fn types()
{
    use sailce_data_model::Path;

    /// Will fail to compile if given a type that doesn't `impl` [`Path`].
    fn f(path: &(impl Path + ?Sized)) -> Vec<&[u8]>
    {
        // Might as well return its components.
        cfg_if::cfg_if! { if #[cfg(feature = "alloc")] {
            use sailce_data_model::path::Extra as _;
            Vec::from_path(path)
        } else {
            path.components().map(|comp| comp.inner).collect()
        } }
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

            #[cfg(feature = "alloc")]
            {
                use std::collections::{
                    LinkedList,
                    VecDeque,
                };

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
            }
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

    assert_eq!(f(&["a", "b"]), [b"a", b"b"]);
    assert_eq!(f(&["a", "b"][..]), [b"a", b"b"]);
    assert_eq!(f(&&["a", "b"]), [b"a", b"b"]);
    assert_eq!(f(&&["a", "b"][..]), [b"a", b"b"]);

    #[cfg(feature = "alloc")]
    {
        assert_eq!(f(&vec!["a".to_owned(), "b".to_owned()]), [b"a", b"b"]);
    }
}


#[cfg(feature = "alloc")]
#[test]
fn from_path()
{
    use {
        sailce_data_model::path::Extra as _,
        std::sync::Arc,
    };

    let p1: Vec<Vec<u8>> = Vec::from_path(&[&b"a"[..], &b"bb"[..]]);
    assert_eq!(p1, &[&[97][..], &[98, 98][..]]);

    let p2: Arc<[Arc<[u8]>]> = Arc::from_path(&["a", "b"][..]);
    assert!(p2.eq_components(&["a", "b"]));

    // TODO: try_from_path

    // TODO: w/ path::Strish helper wrapper type that can TryFrom<&[u8]>
}


// TODO: path::Extra::eq and path::Extra::cmp


mod empty
{
    use sailce_data_model::EmptyPath;

    /// Test that [`EmptyPath`] is automatically implemented for all desired types.
    #[test]
    fn types()
    {
        assert_eq!(<&[[u8; 1]] as EmptyPath>::empty(), {
            let empty: &[[u8; 1]] = &[];
            empty
        });
        assert_eq!(<[&str; 0] as EmptyPath>::empty(), {
            let empty: [&str; 0] = [];
            empty
        });

        #[cfg(feature = "alloc")]
        {
            use std::{
                borrow::Cow,
                collections::{
                    LinkedList,
                    VecDeque,
                },
                rc::Rc,
                sync::Arc,
            };

            assert_eq!(<Box<[&str]> as EmptyPath>::empty(), {
                let empty: Box<[&str]> = [].into();
                empty
            });
            assert_eq!(<Rc<[[u8; 2]]> as EmptyPath>::empty(), {
                let empty: Rc<[[u8; 2]]> = [].into();
                empty
            });
            assert_eq!(<Arc<[String]> as EmptyPath>::empty(), {
                let empty: Arc<[String]> = [].into();
                empty
            });
            assert_eq!(<Vec<&[u8]> as EmptyPath>::empty(), {
                let empty: Vec<&[u8]> = [].into();
                empty
            });
            assert_eq!(<VecDeque<Vec<u8>> as EmptyPath>::empty(), {
                let empty: VecDeque<Vec<u8>> = [].into();
                empty
            });
            assert_eq!(<LinkedList<String> as EmptyPath>::empty(), {
                let empty: LinkedList<String> = [].into();
                empty
            });
            assert_eq!(<Cow<'_, [Cow<'_, [u8]>]> as EmptyPath>::empty(), {
                let empty: Cow<'_, [Cow<'_, [u8]>]> = Cow::Borrowed(&[][..]);
                empty
            });
            assert_eq!(<Cow<'_, [&str]> as EmptyPath>::empty(), {
                let empty: Cow<'_, [&str]> = Cow::Owned(vec![]);
                empty
            });
        }
    }
}
