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
    file: Option<String>,
    line: Option<u32>,
}

#[cfg(not(feature = "serde"))]
pub struct Error<T> {
    code: T,
    msg: String,
    source: Option<Box<(dyn std::error::Error + 'static + Send + Sync)>>,
    backtrace: Option<Backtrace>,
    file: Option<String>,
    line: Option<u32>,
}

pub type Result<T, C> = std::result::Result<T, Error<C>>;

impl<T: Debug + Copy + Sync + Send + 'static> Error<T> {
    pub fn new(code: T, msg: String, file: &str, line: u32) -> Self {
        #[cfg(feature = "backtrace")]
        let backtrace = Some(Backtrace::force_capture());

        #[cfg(not(feature = "backtrace"))]
        let backtrace = None;

        Self {
            code,
            msg,
            source: None,
            backtrace,
            file: Some(file.to_string()),
            line: Some(line),
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

        if self.file.is_some() && self.line.is_some() {
            write!(f, " at:[{}:{}]", self.file.as_ref().unwrap(), self.line.as_ref().unwrap())?;
        }

        if !self.msg.is_empty() {
            write!(f, ", msg:{}", self.msg)?;
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
        if self.source.is_some() {
            write!(f, "\nCaused by: {:?}", self.source.as_ref().unwrap())?;
        }
        Ok(())
    }
}

impl<T: Debug> Display for Error<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{:?}", type_name::<T>(), self.code)?;

        if self.file.is_some() && self.line.is_some() {
            write!(f, " at:[{}:{}]", self.file.as_ref().unwrap(), self.line.as_ref().unwrap())?;
        }

        if !self.msg.is_empty() {
            write!(f, ", msg:{}", self.msg)?;
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
        if self.source.is_some() {
            write!(f, "\nCaused by: {:?}", self.source.as_ref().unwrap())?;
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
            file: None,
            line: None,
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
            file: None,
            line: None,
        }
    }
}

impl<T, E: std::error::Error + 'static + Send + Sync> From<(T, String, E, &str, u32)> for Error<T> {
    fn from(value: (T, String, E, &str, u32)) -> Self {
        #[cfg(feature = "backtrace")]
        let backtrace = Some(Backtrace::force_capture());

        #[cfg(not(feature = "backtrace"))]
        let backtrace = None;

        Self {
            code: value.0,
            msg: value.1,
            source: Some(Box::new(value.2)),
            backtrace,
            file: Some(value.3.to_string()),
            line: Some(value.4),
        }
    }
}

impl<T, E: std::error::Error + 'static + Send + Sync> From<(T, &str, E, &str, u32)> for Error<T> {
    fn from(value: (T, &str, E, &str, u32)) -> Self {
        #[cfg(feature = "backtrace")]
            let backtrace = Some(Backtrace::force_capture());

        #[cfg(not(feature = "backtrace"))]
            let backtrace = None;
        Self {
            code: value.0,
            msg: value.1.to_string(),
            source: Some(Box::new(value.2)),
            backtrace,
            file: Some(value.3.to_string()),
            line: Some(value.4),
        }
    }
}

#[cfg(feature = "log")]
pub use log::error as serror;

#[cfg(feature = "log")]
#[macro_export]
macro_rules! error {
    // error!(target: "my_target", key1 = 42, key2 = true; "a {} event", "log")
    // error!(target: "my_target", "a {} event", "log")
    (target: $target:expr, $($arg:tt)+) => ($crate::serror!($target, $($arg)+));

    // error!("a {} event", "log")
    ($($arg:tt)+) => ($crate::serror!($($arg)+))
}
#[cfg(not(feature = "log"))]
#[macro_export]
macro_rules! error {
    // error!(target: "my_target", key1 = 42, key2 = true; "a {} event", "log")
    // error!(target: "my_target", "a {} event", "log")
    (target: $target:expr, $($arg:tt)+) => ();

    // error!("a {} event", "log")
    ($($arg:tt)+) => ()
}

#[macro_export]
macro_rules! err {
    ( $err: expr) => {
        {
            $crate::error!("{:?}", $err);
            $crate::Error::new($err, "".to_string(), file!(), line!())
        }
    };
    ( $err: expr, $($arg:tt)*) => {
        {
            $crate::error!("{}", format!($($arg)*));
            $crate::Error::new($err, format!("{}", format!($($arg)*)), file!(), line!())
        }
    };
}

#[macro_export]
macro_rules! into_err {
    ($err: expr) => {
        |e| {
            $crate::error!("err:{:?}", e);
            $crate::Error::from(($err, "".to_string(), e, file!(), line!()))
        }
    };
    ($err: expr, $($arg:tt)*) => {
        |e| {
            $crate::error!("{} err:{:?}", format!($($arg)*), e);
            $crate::Error::from(($err, format!($($arg)*), e, file!(), line!()))
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
        let error = sfo_result::Error::new(1, "test".to_string(), file!(), line!());
        println!("{:?}", error);

        let error = err!(1, "test");
        println!("{:?}", error);

        let error = Error::from((TestCode::Test1, "test".to_string(), error));
        println!("{:?}", error);

        // assert_eq!(format!("{:?}", error), "Error: 1, msg: test");
        // assert_eq!(format!("{}", error), "Error: 1, msg: test");
    }
}
