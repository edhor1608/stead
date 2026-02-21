use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EndpointLease {
    pub name: String,
    pub owner: String,
    pub port: u16,
}

impl EndpointLease {
    pub fn url(&self) -> String {
        format!("http://{}.localhost:{}", self.name, self.port)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointConflict {
    pub name: String,
    pub requested_port: u16,
    pub held_by: Option<EndpointLease>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EndpointClaimResult {
    Claimed(EndpointLease),
    Negotiated {
        requested_port: u16,
        assigned: EndpointLease,
        held_by: EndpointLease,
    },
    Conflict(EndpointConflict),
}

impl EndpointClaimResult {
    pub fn unwrap_claimed(self) -> EndpointLease {
        match self {
            Self::Claimed(lease) => lease,
            other => panic!("expected claimed result, got {other:?}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EndpointEvent {
    RangeExhausted {
        name: String,
        owner: String,
        requested_port: u16,
        reason: &'static str,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EndpointError {
    NotFound {
        name: String,
    },
    NotOwner {
        name: String,
        expected_owner: String,
        attempted_by: String,
    },
}

impl EndpointError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound { .. } => "not_found",
            Self::NotOwner { .. } => "not_owner",
        }
    }
}

#[derive(Debug)]
pub struct EndpointRegistry {
    leases_by_name: HashMap<String, EndpointLease>,
    port_range: (u16, u16),
    events: Vec<EndpointEvent>,
}

impl Default for EndpointRegistry {
    fn default() -> Self {
        Self::with_port_range(4100, 4999)
    }
}

impl EndpointRegistry {
    pub fn with_port_range(start: u16, end: u16) -> Self {
        assert!(start <= end, "invalid endpoint port range");
        Self {
            leases_by_name: HashMap::new(),
            port_range: (start, end),
            events: Vec::new(),
        }
    }

    pub fn claim(
        &mut self,
        name: impl Into<String>,
        owner: impl Into<String>,
        requested_port: Option<u16>,
    ) -> EndpointClaimResult {
        let name = name.into();
        let owner = owner.into();
        let (start, _) = self.port_range;
        let requested_port = requested_port.unwrap_or(start);

        if let Some(existing) = self.leases_by_name.get(&name).cloned() {
            if existing.owner == owner {
                return EndpointClaimResult::Claimed(existing);
            }

            return EndpointClaimResult::Conflict(EndpointConflict {
                name,
                requested_port,
                held_by: Some(existing),
            });
        }

        if self.is_port_free(requested_port) {
            let lease = EndpointLease {
                name: name.clone(),
                owner,
                port: requested_port,
            };
            self.leases_by_name.insert(name, lease.clone());
            return EndpointClaimResult::Claimed(lease);
        }

        if let Some(assigned_port) = self.next_available_port_after(requested_port) {
            let lease = EndpointLease {
                name: name.clone(),
                owner,
                port: assigned_port,
            };
            let held_by = self
                .lease_for_port(requested_port)
                .expect("requested port was occupied");
            self.leases_by_name.insert(name, lease.clone());
            return EndpointClaimResult::Negotiated {
                requested_port,
                assigned: lease,
                held_by,
            };
        }

        self.events.push(EndpointEvent::RangeExhausted {
            name: name.clone(),
            owner,
            requested_port,
            reason: "endpoint_range_exhausted",
        });

        EndpointClaimResult::Conflict(EndpointConflict {
            name,
            requested_port,
            held_by: self.lease_for_port(requested_port),
        })
    }

    pub fn release(
        &mut self,
        name: impl AsRef<str>,
        owner: impl Into<String>,
    ) -> Result<EndpointLease, EndpointError> {
        let name = name.as_ref();
        let owner = owner.into();
        let Some(lease) = self.leases_by_name.get(name).cloned() else {
            return Err(EndpointError::NotFound {
                name: name.to_string(),
            });
        };

        if lease.owner != owner {
            return Err(EndpointError::NotOwner {
                name: name.to_string(),
                expected_owner: lease.owner,
                attempted_by: owner,
            });
        }

        Ok(self
            .leases_by_name
            .remove(name)
            .expect("lease checked before remove"))
    }

    pub fn list(&self) -> Vec<EndpointLease> {
        let mut leases: Vec<_> = self.leases_by_name.values().cloned().collect();
        leases.sort_by(|left, right| left.name.cmp(&right.name));
        leases
    }

    pub fn drain_events(&mut self) -> Vec<EndpointEvent> {
        std::mem::take(&mut self.events)
    }

    pub fn export_leases(&self) -> Vec<EndpointLease> {
        self.list()
    }

    pub fn import_leases(&mut self, leases: Vec<EndpointLease>) {
        self.leases_by_name.clear();
        for lease in leases {
            self.leases_by_name.insert(lease.name.clone(), lease);
        }
    }

    fn lease_for_port(&self, port: u16) -> Option<EndpointLease> {
        self.leases_by_name
            .values()
            .find(|lease| lease.port == port)
            .cloned()
    }

    fn is_port_free(&self, port: u16) -> bool {
        self.port_in_range(port)
            && self
                .leases_by_name
                .values()
                .all(|existing| existing.port != port)
    }

    fn port_in_range(&self, port: u16) -> bool {
        let (start, end) = self.port_range;
        (start..=end).contains(&port)
    }

    fn next_available_port_after(&self, requested: u16) -> Option<u16> {
        if !self.port_in_range(requested) {
            return None;
        }

        let (start, end) = self.port_range;
        ((requested.saturating_add(1))..=end)
            .chain(start..requested)
            .find(|candidate| self.is_port_free(*candidate))
    }
}

pub fn crate_identity() -> &'static str {
    "stead-endpoints"
}
