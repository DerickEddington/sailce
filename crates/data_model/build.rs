//! Set `cfg` options according to probing which features are enabled in the Rust compiler,
//! language, and library, without reference to versions of Rust.
//!
//! This detects when previously-unstable features become stabilized, based on feature presence
//! and not on Rust version.  This helps design conditionally-compiled code that can adjust
//! whenever a feature becomes stable in whichever unknown future version of Rust.


fn main()
{
    cfg_rust_features::emit!(["associated_type_defaults", "error_in_core"]).unwrap();
}
