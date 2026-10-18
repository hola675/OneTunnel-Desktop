//! In-memory policy repository. All edits are validated before being published.
//! No automatic driver activation, persistence or network interception.
use crate::routing::{evaluate, validate, ConnectionMeta, RouteAction, RouteDecision, RouteRule};
use serde::Serialize;
use std::sync::RwLock;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicySnapshot {
    pub revision: u64,
    pub default_action: RouteAction,
    pub rules: Vec<RouteRule>,
    pub enforcement_active: bool,
}

pub struct PolicyStore {
    inner: RwLock<PolicySnapshot>,
}

impl Default for PolicyStore {
    fn default() -> Self {
        Self {
            inner: RwLock::new(PolicySnapshot {
                revision: 0,
                default_action: RouteAction::Direct,
                rules: Vec::new(),
                enforcement_active: false,
            }),
        }
    }
}

impl PolicyStore {
    pub fn snapshot(&self) -> Result<PolicySnapshot, String> {
        self.inner.read().map(|state| state.clone())
            .map_err(|_| "Policy store is unavailable".into())
    }

    pub fn replace(&self, rules: Vec<RouteRule>, default_action: RouteAction, expected_revision: u64)
        -> Result<PolicySnapshot, String>
    {
        validate(&rules)?;
        // There is no enforcement backend. Reject a default that could imply
        // system-wide tunnelling or blocking if an engine were accidentally added.
        if default_action != RouteAction::Direct {
            return Err("Default routing must remain direct until enforcement is certified".into());
        }
        let mut state = self.inner.write().map_err(|_| "Policy store is unavailable")?;
        if state.revision != expected_revision {
            return Err("Policy was modified; reload before saving".into());
        }
        state.revision = state.revision.checked_add(1).ok_or("Policy revision overflow")?;
        state.rules = rules;
        state.default_action = default_action;
        Ok(state.clone())
    }

    pub fn preview(&self, connection: &ConnectionMeta) -> Result<RouteDecision, String> {
        let state = self.inner.read().map_err(|_| "Policy store is unavailable")?;
        evaluate(&state.rules, state.default_action, connection)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_is_direct_and_not_enforced() {
        let store = PolicyStore::default();
        let snapshot = store.snapshot().unwrap();
        assert!(snapshot.rules.is_empty());
        assert!(!snapshot.enforcement_active);
        assert_eq!(snapshot.default_action, RouteAction::Direct);
    }
    #[test]
    fn rejects_stale_policy_revisions() {
        let store = PolicyStore::default();
        assert!(store.replace(vec![], RouteAction::Direct, 0).is_ok());
        assert!(store.replace(vec![], RouteAction::Direct, 0).is_err());
    }
    #[test]
    fn refuses_default_blocking() {
        let store = PolicyStore::default();
        assert!(store.replace(vec![], RouteAction::Block, 0).is_err());
        assert_eq!(store.snapshot().unwrap().revision, 0);
    }
}
