use std::{error::Error, num::ParseIntError, string::FromUtf8Error};

macro_rules! ActualError {
    ($e:ty) => {
        impl From<$e> for MyOwnError {
            fn from(e: $e) -> Self {
                MyOwnError::ActualError(e.into())
            }
        }
    };
}

#[derive(Debug)]
pub enum MyOwnError {
    EarlyExit,
    ActualError(Box<dyn Error>),
    ActualErrorWithDescription(Box<dyn Error>, String),
}

pub trait DescribableError<T, E: Error> {
    fn error_description<TDescription: Into<String>>(
        self,
        description: TDescription,
    ) -> Result<T, MyOwnError>;

    fn with_error_description<S: Into<String>, F: FnOnce() -> S>(
        self,
        with_description: F,
    ) -> Result<T, MyOwnError>;
}

impl<'a, T, E: Error + 'static> DescribableError<T, E> for Result<T, E> {
    fn error_description<TDescription: Into<String>>(
        self,
        description: TDescription,
    ) -> Result<T, MyOwnError> {
        self.map_err(|e| MyOwnError::ActualErrorWithDescription(e.into(), description.into()))
    }

    fn with_error_description<S: Into<String>, F: FnOnce() -> S>(
        self,
        with_description: F,
    ) -> Result<T, MyOwnError> {
        self.map_err(|e| {
            MyOwnError::ActualErrorWithDescription(e.into(), with_description().into())
        })
    }
}

ActualError!(FromUtf8Error);
ActualError!(ParseIntError);
ActualError!(&str);
ActualError!(String);

impl From<std::io::Error> for MyOwnError {
    fn from(e: std::io::Error) -> Self {
        if e.kind() == std::io::ErrorKind::BrokenPipe {
            MyOwnError::EarlyExit
        } else {
            MyOwnError::ActualError(Box::new(e))
        }
    }
}
