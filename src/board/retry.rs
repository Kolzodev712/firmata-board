use std::time::Duration;

use backoff::ExponentialBackoff;

use crate::error::{Error, Result};

pub fn default_backoff() -> ExponentialBackoff {
    ExponentialBackoff {
        max_interval: Duration::from_millis(5_000),
        ..Default::default()
    }
}

pub fn retry_with_backoff<F, T>(mut operation: F) -> Result<T>
where
    F: FnMut() -> Result<T>,
{
    backoff::retry(default_backoff(), || {
        operation().map_err(|err| {
            if err.is_transient() {
                backoff::Error::transient(err)
            } else {
                backoff::Error::permanent(err)
            }
        })
    })
    .map_err(Error::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU8, Ordering};

    #[test]
    fn retries_transient_errors() {
        let attempts = AtomicU8::new(0);
        let result = retry_with_backoff(|| {
            let n = attempts.fetch_add(1, Ordering::SeqCst);
            if n < 2 {
                Err(Error::StdIoError {
                    source: std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout"),
                })
            } else {
                Ok(42)
            }
        });
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn does_not_retry_permanent_errors() {
        let attempts = AtomicU8::new(0);
        let result: Result<()> = retry_with_backoff(|| {
            attempts.fetch_add(1, Ordering::SeqCst);
            Err(Error::BadByte { byte: 0x00 })
        });
        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }
}
