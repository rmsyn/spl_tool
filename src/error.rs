use core::fmt;

/// Convenience alias for a [`Result`](core::result::Result) type for the library.
pub type Result<T> = core::result::Result<T, Error>;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub enum Error {
    InvalidHeaderLen((usize, usize)),
    InvalidSplLen((usize, usize)),
    InvalidSlice(core::array::TryFromSliceError),
    InvalidHeaderFile,
    InvalidSplFile,
}

impl From<core::array::TryFromSliceError> for Error {
    fn from(err: core::array::TryFromSliceError) -> Self {
        Self::InvalidSlice(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHeaderLen((inv_len, exp_len)) => {
                write!(f, "invalid header len: {inv_len}, expected: {exp_len}")
            }
            Self::InvalidSplLen((inv_len, max_len)) => {
                write!(f, "invalid SPL len: {inv_len}, max: {max_len}")
            }
            Self::InvalidSlice(err) => write!(f, "invalid slice to array conversion: {err}"),
            Self::InvalidHeaderFile => {
                write!(f, "invalid SPL header file, ensure the path is valid")
            }
            Self::InvalidSplFile => write!(f, "invalid SPL file, ensure the path is valid"),
        }
    }
}
