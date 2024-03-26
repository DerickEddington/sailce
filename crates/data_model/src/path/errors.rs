use core::fmt::{
    self,
    Display,
};


/// Error that occurs when a [`Path`](crate::Path) violated the limits on `Path`s required by a
/// [`Params`](crate::Params), in a context where the limits are enforced.
///
/// One or more of the `bool` fields will be `false` and this indicates which limit(s) were
/// exceeded.
#[derive(Copy, Clone, Eq, Ord, Hash, PartialEq, PartialOrd, Debug)]
#[allow(clippy::exhaustive_structs)]
pub struct PathLimitError
{
    /// Index of which [`Component`](crate::path::Component) caused this error.
    pub index:                       usize,
    /// Whether the [`Component`](crate::path::Component)'s size is less than or equal to
    /// [`Params::MAX_COMPONENT_LENGTH`](crate::Params::MAX_COMPONENT_LENGTH).
    pub within_max_component_length: bool,
    /// Whether the amount of [`Component`](crate::path::Component)s is less than or equal to
    /// [`Params::MAX_COMPONENT_COUNT`](crate::Params::MAX_COMPONENT_COUNT).
    pub within_max_component_count:  bool,
    /// Whether the total of the sizes of the [`Component`](crate::path::Component)s is less
    /// than or equal to [`Params::MAX_PATH_LENGTH`](crate::Params::MAX_PATH_LENGTH).
    pub within_max_path_length:      bool,
}

impl Display for PathLimitError
{
    #[inline]
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result
    {
        write!(f, "A `Path` exceeded limits at component {}:", self.index)?;
        if !self.within_max_component_length {
            write!(f, " component-length")?;
        }
        if !self.within_max_component_count {
            write!(f, " component-count")?;
        }
        if !self.within_max_path_length {
            write!(f, " path-length")?;
        }
        Ok(())
    }
}


#[cfg(any(feature = "std", feature = "anticipate", rust_lib_feature = "error_in_core"))]
mod standard_error
{
    use super::PathLimitError;

    cfg_if::cfg_if! { if #[cfg(any(feature = "anticipate", rust_lib_feature = "error_in_core"))]
    {
        use core::error::Error;
    }
    else if #[cfg(feature = "std")]
    {
        use std::error::Error;
    } }


    impl Error for PathLimitError {}
}
