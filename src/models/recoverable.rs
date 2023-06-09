pub enum Recoverable<T, E> {
    Ok(T),
    Unrecoverable(E),
    /// indicates that the error has occurred, but the program can continue
    /// (e.g. a field in a parsed string is not present, but the program can continue du to a default value)
    Recoverable(T),
}

impl<T, E> Recoverable<T, E>
where
    E: Default,
{
    pub fn to_lenient_result(self) -> Result<T, E> {
        match self {
            Recoverable::Ok(value) => Ok(value),
            Recoverable::Unrecoverable(error) => Err(error),
            Recoverable::Recoverable(value) => Ok(value),
        }
    }

    pub fn to_result(self) -> Result<T, E> {
        match self {
            Recoverable::Ok(value) => Ok(value),
            Recoverable::Unrecoverable(error) => Err(error),
            Recoverable::Recoverable(_value) => Err(E::default()),
        }
    }
}

#[macro_export]
macro_rules! unrecoverable {
    ($msg:literal $(,)?) => {
        return $crate::models::unrecoverable::Unrecoverable($msg);
    };
    ($err:expr $(,)?) => {
        return $crate::models::unrecoverable::Unrecoverable($err);
    };
    ($fmt:expr, $($arg:tt)*) => {
        return $crate::models::unrecoverable::Unrecoverable(format!($fmt, $($arg)*));
    };
}
