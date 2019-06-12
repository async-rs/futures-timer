use std::fmt;

/// Errors that can occur from the timer implementation
///
/// Currently, there is only one possible error which occurs when the timer
/// instance is dropped.
#[derive(Debug)]
pub struct Error(ErrorKind);

#[derive(Debug)]
enum ErrorKind {
    TimerDropped,
}

impl Error {
    pub(crate) fn new_timer_dropped() -> Self {
        Self(ErrorKind::TimerDropped)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            ErrorKind::TimerDropped => f.write_str("timer was dropped"),
        }
    }
}

impl ::std::error::Error for Error {}

/// Error returned by the `TryFuture`/`TryStream` timeout extension traits.
#[derive(Debug)]
pub struct TimeoutError<E>(TimeoutErrorKind<E>);

#[derive(Debug)]
enum TimeoutErrorKind<E> {
    Inner(E),
    Timer(Error),
    Elapsed,
}

impl<E> TimeoutError<E> {
    /// Creates a new `TimeoutError` representing that the inner `TryFuture`
    /// or `TryStream` completed with an error.
    pub fn new_inner(inner_err: E) -> Self {
        Self(TimeoutErrorKind::Inner(inner_err))
    }

    /// Returns `true` if this error was caused by the inner `TryFuture` or
    /// `TryStream` completing with an error.
    pub fn is_inner(&self) -> bool {
        match self.0 {
            TimeoutErrorKind::Inner(_) => true,
            _ => false,
        }
    }

    /// Attempts to view this error as the error caused by the inner
    /// `TryFuture` or `TryStream`.
    ///
    /// Returns `None` if the error was not caused by the inner `TryFuture` or
    /// `TryStream`.
    pub fn try_as_inner(&self) -> Option<&E> {
        match self.0 {
            TimeoutErrorKind::Inner(ref err) => Some(err),
            _ => None,
        }
    }

    /// Consumes this error, attempting to view it as the error caused by the
    /// inner `TryFuture` of `TryStream`.
    ///
    /// Returns `None` if the error was not caused by the inner `TryFuture` or
    /// `TryStream`.
    pub fn try_into_inner(self) -> Option<E> {
        match self.0 {
            TimeoutErrorKind::Inner(err) => Some(err),
            _ => None,
        }
    }

    /// Creates a new `TimeoutError` representing that the timer implementation
    /// encountered an error.
    pub fn new_timer(timer_err: Error) -> Self {
        Self(TimeoutErrorKind::Timer(timer_err))
    }

    /// Returns `true` if this error was caused by the timer implementation
    /// encountering an error.
    pub fn is_timer(&self) -> bool {
        match self.0 {
            TimeoutErrorKind::Timer(_) => true,
            _ => false,
        }
    }

    /// Attempts to view this error as the error caused by the timer
    /// implementation.
    ///
    /// Returns `None` if the error was not caused by the timer implementation.
    pub fn try_as_timer(&self) -> Option<&Error> {
        match self.0 {
            TimeoutErrorKind::Timer(ref err) => Some(err),
            _ => None,
        }
    }

    /// Consumes this error, attempting to view it as an error caused by the
    /// timer implementation.
    ///
    /// Returns `None` if the error was not caused by the timer implementation.
    pub fn try_into_timer(self) -> Option<Error> {
        match self.0 {
            TimeoutErrorKind::Timer(err) => Some(err),
            _ => None,
        }
    }

    /// Creates a new `TimeoutError` representing that the item produced by the
    /// inner `TryFuture` or `TryStream` did not complete before the deadline.
    pub fn new_elapsed() -> Self {
        Self(TimeoutErrorKind::Elapsed)
    }

    /// Returns `true` if this error was caused by the item produced by the
    /// inner `TryFuture` or `TryStream` failing to complete before the
    /// deadline.
    pub fn is_elapsed(&self) -> bool {
        match self.0 {
            TimeoutErrorKind::Elapsed => true,
            _ => false,
        }
    }
}

impl<E: fmt::Display> fmt::Display for TimeoutError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            TimeoutErrorKind::Inner(ref err) => write!(f, "inner timeout error: {}", err),
            TimeoutErrorKind::Timer(ref err) => write!(f, "timer timeout error: {}", err),
            TimeoutErrorKind::Elapsed => f.write_str("timeout elapsed"),
        }
    }
}

impl<E: ::std::error::Error> ::std::error::Error for TimeoutError<E> {}
