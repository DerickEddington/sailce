#![cfg(test)] // Satisfy the `clippy::tests_outside_test_module` lint.
#![cfg_attr(test, allow(unused_crate_dependencies))]
#![allow(
    non_snake_case,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::std_instead_of_core,
    clippy::std_instead_of_alloc
)]

mod entry;

mod group
{
    mod range;
    mod area;
}

mod path;

mod payload;

mod store;

mod async_help;

/// Until [`Option::unwrap`] as `const` becomes stabilized (if ever).
const fn nz_usize(v: usize) -> std::num::NonZeroUsize
{
    if let Some(nz) = std::num::NonZeroUsize::new(v) { nz } else { panic!() }
}
