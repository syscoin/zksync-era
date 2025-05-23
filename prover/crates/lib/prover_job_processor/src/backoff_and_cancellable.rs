use std::{ops::Mul, time::Duration};

use tokio_util::sync::CancellationToken;

/// Utility struct that provides cancellation awareness & backoff capabilities.
/// They usually go hand in hand, having a wrapper over both simplifies implementation.
#[derive(Debug, Clone)]
pub struct BackoffAndCancellable {
    pub(crate) backoff: Backoff,
    pub(crate) cancellation_token: CancellationToken,
}

impl BackoffAndCancellable {
    pub fn new(backoff: Backoff, cancellation_token: CancellationToken) -> Self {
        Self {
            backoff,
            cancellation_token,
        }
    }
}

/// Backoff - convenience structure that takes care of backoff timings.
#[derive(Debug, Clone)]
pub struct Backoff {
    base_delay: Duration,
    current_delay: Duration,
    max_delay: Duration,
}
impl Backoff {
    /// The delay multiplication coefficient.
    // Currently it's hardcoded, but could be provided in the constructor.
    const DELAY_MULTIPLIER: u32 = 2;
    /// Create a backoff with base_delay (first delay) and max_delay (maximum delay possible).
    pub fn new(base_delay: Duration, max_delay: Duration) -> Self {
        Backoff {
            base_delay,
            current_delay: base_delay,
            max_delay,
        }
    }

    /// Get current delay, handling future delays if needed
    pub fn delay(&mut self) -> Duration {
        let delay = self.current_delay;
        self.current_delay = self
            .current_delay
            .mul(Self::DELAY_MULTIPLIER)
            .min(self.max_delay);
        delay
    }

    /// Reset the backoff time for to base delay
    pub fn reset(&mut self) {
        self.current_delay = self.base_delay;
    }
}

impl Default for Backoff {
    /// Sensible database specific delays.
    fn default() -> Self {
        Self::new(Duration::from_secs(1), Duration::from_secs(5))
    }
}
