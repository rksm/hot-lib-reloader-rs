use std::time::{Duration, Instant};

/// Records the time when [`TimedLimiter::too_frequent`] was called. If this
/// gets called `N` times with a duration of less then `delay`, this method will
/// return `true`, otherwise `false`.
///
/// This is useful to make sure something does not get invoked too frequently.
pub(crate) struct TimedLimiter<const N: usize> {
    delay: Duration,
    last_changes: [Option<Instant>; N],
}

impl<const N: usize> TimedLimiter<N> {
    pub(crate) fn new(delay: Duration) -> Self {
        Self {
            delay,
            last_changes: [None; N],
        }
    }

    /// See struct doc.
    pub(crate) fn too_frequent(&mut self) -> bool {
        self.last_changes.rotate_right(1);
        let now = Instant::now();
        self.last_changes[0] = Some(now);

        for i in 0..N - 1 {
            match (self.last_changes[i], self.last_changes[i + 1]) {
                (Some(a), Some(b)) if a - b < self.delay => {
                    continue;
                }
                _ => return false,
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::util::TimedLimiter;
    #[test]
    fn test_timed_limiter() {
        let mut l = TimedLimiter::<3>::new(Duration::from_millis(5));
        assert!(!l.too_frequent());
        std::thread::sleep(std::time::Duration::from_millis(1));
        assert!(!l.too_frequent());
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(!l.too_frequent());
        std::thread::sleep(std::time::Duration::from_millis(1));
        assert!(!l.too_frequent());
        std::thread::sleep(std::time::Duration::from_millis(1));
        assert!(l.too_frequent());
    }
}
