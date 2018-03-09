use failure::{Error, SyncFailure};

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

pub fn exit_with_error<T>(error: T)
    where T: Into<Error> {
    eprintln!("Error: {}", error.into());
    super::std::process::exit(1);
}
