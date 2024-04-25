use std::any::{type_name};
use std::backtrace::{Backtrace, BacktraceStatus};
use std::fmt::{Debug, Display};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "serde")]
#[derive(Serialize, Deserialize)]
pub struct Error<T> {
    code: T,
    msg: String,
    #[serde(skip)]
    source: Option<Box<(dyn std::error::Error + 'static + Send + Sync)>>,
    #[serde(skip)]
    backtrace: Option<Backtrace>,
}

#[cfg(not(feature = "serde"))]
pub struct Error<T> {
    code: T,
    msg: String,
    source: Option<Box<(dyn std::error::Error + 'static + Send + Sync)>>,
    backtrace: Option<Backtrace>,
}

pub type Result<T, C> = std::result::Result<T, Error<C>>;

impl<T: Debug + Copy + Sync + Send + 'static> Error<T> {
    pub fn new(code: T, msg: String) -> Self {
        #[cfg(feature = "backtrace")]
        let backtrace = Some(Backtrace::force_capture());

        #[cfg(not(feature = "backtrace"))]
        let backtrace = None;

        Self {
            code,
            msg,
            source: None,
            backtrace,
        }
    }

    pub fn code(&self) -> T {
        self.code
    }

    pub fn msg(&self) -> &str {
        &self.msg
    }

    #[cfg(feature = "backtrace")]
    pub fn backtrace(&self) -> Option<&Backtrace> {
        self.backtrace.as_ref()
    }
}

impl<T: Debug + Clone + Copy> std::error::Error for Error<T> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as _)
    }
}

impl<T: Debug> Debug for Error<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{:?}", type_name::<T>(), self.code)?;
        if !self.msg.is_empty() {
            write!(f, ", msg:{}", self.msg)?;
        }
        if self.source.is_some() {
            write!(f, "\nCaused by: {:?}", self.source.as_ref().unwrap())?;
        }
        if let Some(backtrace) = &self.backtrace {
            if let BacktraceStatus::Captured = backtrace.status() {
                let mut backtrace = backtrace.to_string();
                write!(f, "\n")?;
                if backtrace.starts_with("stack backtrace:") {
                    // Capitalize to match "Caused by:"
                    backtrace.replace_range(0..1, "S");
                } else {
                    // "stack backtrace:" prefix was removed in
                    // https://github.com/rust-lang/backtrace-rs/pull/286
                    writeln!(f, "Stack backtrace:")?;
                }
                backtrace.truncate(backtrace.trim_end().len());
                write!(f, "{}", backtrace)?;
            }
        }
        Ok(())
    }
}

impl<T: Debug> Display for Error<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{:?}", type_name::<T>(), self.code)?;
        if !self.msg.is_empty() {
            write!(f, ", msg:{}", self.msg)?;
        }
        if self.source.is_some() {
            write!(f, "\nCaused by: {:?}", self.source.as_ref().unwrap())?;
        }
        if let Some(backtrace) = &self.backtrace {
            if let BacktraceStatus::Captured = backtrace.status() {
                let mut backtrace = backtrace.to_string();
                write!(f, "\n")?;
                if backtrace.starts_with("stack backtrace:") {
                    // Capitalize to match "Caused by:"
                    backtrace.replace_range(0..1, "S");
                } else {
                    // "stack backtrace:" prefix was removed in
                    // https://github.com/rust-lang/backtrace-rs/pull/286
                    writeln!(f, "Stack backtrace:")?;
                }
                backtrace.truncate(backtrace.trim_end().len());
                write!(f, "{}", backtrace)?;
            }
        }
        Ok(())
    }
}

impl<T: Default> From<String> for Error<T> {
    fn from(value: String) -> Self {
        #[cfg(feature = "backtrace")]
            let backtrace = Some(Backtrace::force_capture());

        #[cfg(not(feature = "backtrace"))]
            let backtrace = None;
        Self {
            code: Default::default(),
            msg: value,
            source: None,
            backtrace,
        }
    }
}

impl<T, E: std::error::Error + 'static + Send + Sync> From<(T, String, E)> for Error<T> {
    fn from(value: (T, String, E)) -> Self {
        #[cfg(feature = "backtrace")]
            let backtrace = Some(Backtrace::force_capture());

        #[cfg(not(feature = "backtrace"))]
            let backtrace = None;
        Self {
            code: value.0,
            msg: value.1,
            source: Some(Box::new(value.2)),
            backtrace,
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
            sfo_result::Error::from(($err, "".to_string(), e))
        }
    };
    ($err: expr, $($arg:tt)*) => {
        |e| {
            #[cfg(feature = "log")]
            log::error!("{} err:{:?}", format!($($arg)*), e);
            sfo_result::Error::from($err, format!("{} err {}", format!($($arg)*), e))
        }
    };
}

#[cfg(test)]
mod test {
    #[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
    pub enum TestCode {
        #[default]
        Test1,
        Test2,
    }
    pub type Error = super::Error<TestCode>;

    #[test]
    fn test() {
        use crate as sfo_result;
        let error = sfo_result::Error::new(1, "test".to_string());
        println!("{:?}", error);

        let error = err!(1, "test");
        println!("{:?}", error);

        let error = Error::from((TestCode::Test1, "test".to_string(), error));
        println!("{:?}", error);

        // assert_eq!(format!("{:?}", error), "Error: 1, msg: test");
        // assert_eq!(format!("{}", error), "Error: 1, msg: test");
    }
}
