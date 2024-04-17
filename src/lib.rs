use std::fmt::{Debug, Display};
pub use anyhow::Error as AnyError;

pub type Result<T> = std::result::Result<T, AnyError>;

pub struct Error<T> {
    code: T,
    msg: String,
}

impl<T: Debug + Clone + Copy + Eq + PartialEq + Sync + Send + 'static> Error<T> {
    pub fn new(code: T, msg: String) -> AnyError {
        AnyError::new(Error {
            code,
            msg,
        })
    }

    pub fn code(&self) -> T {
        self.code
    }

    pub fn msg(&self) -> &str {
        &self.msg
    }
}

impl<T: Debug + Clone + Copy + Eq + PartialEq> std::error::Error for Error<T> {}

impl<T: Debug + Clone + Copy + Eq + PartialEq> Debug for Error<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {:?}, msg: {}", self.code, self.msg)
    }
}

impl<T: Debug + Clone + Copy + Eq + PartialEq> Display for Error<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {:?}, msg: {}", self.code, self.msg)
    }
}

impl<T: Default> From<String> for Error<T> {
    fn from(value: String) -> Self {
        Self {
            code: Default::default(),
            msg: value,
        }
    }
}

#[macro_export]
macro_rules! err {
    ( $err: expr, $($arg:tt)*) => {
        {
            #[cfg(feature = "log")]
            log::error!("{}", format!($($arg)*));
            sfo_result::Error::new($err, format!("{}", format!($($arg)*)))
        }
    };
}

#[macro_export]
macro_rules! into_err {
    ($err: expr) => {
        |e| {
            #[cfg(feature = "log")]
            log::error!("err:{:?}", e);
            sfo_result::Error::new($err, format!("err {}", e))
        }
    };
    ($err: expr, $($arg:tt)*) => {
        |e| {
            #[cfg(feature = "log")]
            log::error!("{} err:{:?}", format!($($arg)*), e);
            sfo_result::Error::new($err, format!("{} err {}", format!($($arg)*), e))
        }
    };
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        use crate as sfo_result;
        let error = sfo_result::Error::new(1, "test".to_string());
        println!("{:?}", error);

        let error = err!(1, "test");
        println!("{:?}", error);
        // assert_eq!(format!("{:?}", error), "Error: 1, msg: test");
        // assert_eq!(format!("{}", error), "Error: 1, msg: test");
    }
}
