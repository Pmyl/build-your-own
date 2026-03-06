use std::{error::Error, fmt::Display, num::ParseIntError, string::FromUtf8Error};

macro_rules! ActualError {
    ($e:ty) => {
        impl From<$e> for MyOwnError {
            fn from(e: $e) -> Self {
                MyOwnError::ActualError(e.into())
            }
        }
    };
}

pub type MyOwnResult<TValue> = Result<TValue, MyOwnError>;

#[derive(Debug)]
pub enum MyOwnError {
    EarlyExit,
    ActualError(Box<dyn Error>),
    ActualErrorWithDescription(Box<dyn Error>, String),
    MyOwnErrorWithDescription(Box<MyOwnError>, String),
}

pub trait DescribableError<T, E> {
    fn error_description<TDescription: Into<String>>(
        self,
        description: TDescription,
    ) -> MyOwnResult<T>;

    fn with_error_description<S: Into<String>, F: FnOnce() -> S>(
        self,
        with_description: F,
    ) -> MyOwnResult<T>;
}

impl<'a, T, E: Error + 'static> DescribableError<T, E> for Result<T, E> {
    fn error_description<TDescription: Into<String>>(
        self,
        description: TDescription,
    ) -> MyOwnResult<T> {
        self.map_err(|e| MyOwnError::ActualErrorWithDescription(e.into(), description.into()))
    }

    fn with_error_description<S: Into<String>, F: FnOnce() -> S>(
        self,
        with_description: F,
    ) -> MyOwnResult<T> {
        self.map_err(|e| {
            MyOwnError::ActualErrorWithDescription(e.into(), with_description().into())
        })
    }
}

impl<'a, T> DescribableError<T, MyOwnError> for MyOwnResult<T> {
    fn error_description<TDescription: Into<String>>(
        self,
        description: TDescription,
    ) -> MyOwnResult<T> {
        self.map_err(|e| match e {
            MyOwnError::EarlyExit => e,
            err @ _ => MyOwnError::MyOwnErrorWithDescription(Box::new(err), description.into()),
        })
    }

    fn with_error_description<S: Into<String>, F: FnOnce() -> S>(
        self,
        with_description: F,
    ) -> MyOwnResult<T> {
        self.map_err(|e| match e {
            MyOwnError::EarlyExit => e,
            err @ _ => {
                MyOwnError::MyOwnErrorWithDescription(Box::new(err), with_description().into())
            }
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

impl Display for MyOwnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MyOwnError::EarlyExit => Ok(()),
            MyOwnError::ActualError(error) => write!(f, "{}", error),
            MyOwnError::ActualErrorWithDescription(error, description) => {
                write!(f, "{}: {}", error, description)
            }
            MyOwnError::MyOwnErrorWithDescription(my_own_error, description) => {
                write!(f, "{}: {}", my_own_error, description)
            }
        }
    }
}
