use failure::{Error, SyncFailure};
use super::std::fmt;

pub type RustyNzbResult<T> = Result<T, Error>;

pub trait ResultExt<T, E> {
    fn sync(self) -> Result<T, SyncFailure<E>>
    where
        Self: Sized,
        E: ::std::error::Error + Send + 'static;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn sync(self) -> Result<T, SyncFailure<E>>
    where
        Self: Sized,
        E: ::std::error::Error + Send + 'static,
    {
        self.map_err(SyncFailure::new)
    }
}

pub fn exit_with_error(error: Error) {
    eprintln!("Error: {}", error);
    super::std::process::exit(1);
}
