//! Recovery policy only: does not restart Xray or change Windows/Proxifier routes.
//! The controller must check authenticated session and server eligibility before every retry.
use std::time::Duration;

pub const MAX_RETRIES: u32 = 5;
pub const STABLE_RESET_SECS: u64 = 120;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryAction {
    RetryAfter(Duration),
    GiveUp,
}

/// Exponential backoff with a deterministic cap. No tight restart loops.
pub fn next_action(consecutive_failures: u32) -> RecoveryAction {
    if consecutive_failures >= MAX_RETRIES {
        return RecoveryAction::GiveUp;
    }
    let seconds = 2u64.saturating_pow(consecutive_failures.min(5)).min(30);
    RecoveryAction::RetryAfter(Duration::from_secs(seconds))
}

/// Only a stable, verified session may clear the failure counter.
pub fn should_reset_failures(verified_duration: Duration) -> bool {
    verified_duration >= Duration::from_secs(STABLE_RESET_SECS)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retry_delay_is_bounded() {
        assert_eq!(next_action(0), RecoveryAction::RetryAfter(Duration::from_secs(1)));
        assert_eq!(next_action(1), RecoveryAction::RetryAfter(Duration::from_secs(2)));
        assert_eq!(next_action(4), RecoveryAction::RetryAfter(Duration::from_secs(16)));
        assert_eq!(next_action(5), RecoveryAction::GiveUp);
    }
    #[test]
    fn failures_reset_only_after_stable_verification() {
        assert!(!should_reset_failures(Duration::from_secs(119)));
        assert!(should_reset_failures(Duration::from_secs(120)));
    }
}
