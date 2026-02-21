use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceKey {
    Port(u16),
}

impl ResourceKey {
    pub fn port(value: u16) -> Self {
        Self::Port(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceLease {
    pub resource: ResourceKey,
    pub owner: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceConflict {
    pub requested: ResourceKey,
    pub held_by: ResourceLease,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimResult {
    Claimed(ResourceLease),
    Negotiated {
        requested: ResourceKey,
        assigned: ResourceLease,
        held_by: ResourceLease,
    },
    Conflict(ResourceConflict),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceEvent {
    ConflictEscalated {
        requested: ResourceKey,
        requested_by: String,
        held_by: String,
        reason: &'static str,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceError {
    NotFound(ResourceKey),
    NotOwner {
        resource: ResourceKey,
        expected_owner: String,
        attempted_by: String,
    },
}

impl ResourceError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "not_found",
            Self::NotOwner { .. } => "not_owner",
        }
    }
}

#[derive(Debug)]
pub struct ResourceRegistry {
    leases: HashMap<ResourceKey, ResourceLease>,
    port_range: (u16, u16),
    events: Vec<ResourceEvent>,
}

impl Default for ResourceRegistry {
    fn default() -> Self {
        Self::with_port_range(3000, 4999)
    }
}

impl ResourceRegistry {
    pub fn with_port_range(start: u16, end: u16) -> Self {
        assert!(start <= end, "invalid port range");
        Self {
            leases: HashMap::new(),
            port_range: (start, end),
            events: Vec::new(),
        }
    }

    pub fn claim(&mut self, resource: ResourceKey, owner: impl Into<String>) -> ClaimResult {
        let owner = owner.into();

        if let Some(existing) = self.leases.get(&resource).cloned() {
            if existing.owner == owner {
                return ClaimResult::Claimed(existing);
            }

            if let Some(negotiated_resource) = self.next_available_port_after(&resource) {
                let assigned = ResourceLease {
                    resource: negotiated_resource.clone(),
                    owner,
                };
                self.leases.insert(negotiated_resource, assigned.clone());
                return ClaimResult::Negotiated {
                    requested: resource,
                    assigned,
                    held_by: existing,
                };
            }

            self.events.push(ResourceEvent::ConflictEscalated {
                requested: resource.clone(),
                requested_by: owner,
                held_by: existing.owner.clone(),
                reason: "port_range_exhausted",
            });

            return ClaimResult::Conflict(ResourceConflict {
                requested: resource,
                held_by: existing,
            });
        }

        let lease = ResourceLease {
            resource: resource.clone(),
            owner,
        };
        self.leases.insert(resource, lease.clone());
        ClaimResult::Claimed(lease)
    }

    pub fn release(
        &mut self,
        resource: ResourceKey,
        owner: impl Into<String>,
    ) -> Result<ResourceLease, ResourceError> {
        let owner = owner.into();
        let Some(lease) = self.leases.get(&resource) else {
            return Err(ResourceError::NotFound(resource));
        };

        if lease.owner != owner {
            return Err(ResourceError::NotOwner {
                resource: resource.clone(),
                expected_owner: lease.owner.clone(),
                attempted_by: owner,
            });
        }

        // Safe because we already checked existence and ownership above.
        Ok(self
            .leases
            .remove(&resource)
            .expect("lease checked before remove"))
    }

    fn next_available_port_after(&self, resource: &ResourceKey) -> Option<ResourceKey> {
        let ResourceKey::Port(requested) = resource;
        let (start, end) = self.port_range;
        let from = requested.saturating_add(1).max(start);

        (from..=end)
            .map(ResourceKey::Port)
            .find(|candidate| !self.leases.contains_key(candidate))
    }

    pub fn drain_events(&mut self) -> Vec<ResourceEvent> {
        std::mem::take(&mut self.events)
    }

    pub fn export_leases(&self) -> Vec<ResourceLease> {
        self.leases.values().cloned().collect()
    }

    pub fn import_leases(&mut self, leases: Vec<ResourceLease>) {
        self.leases.clear();
        for lease in leases {
            self.leases.insert(lease.resource.clone(), lease);
        }
    }
}

pub fn crate_identity() -> &'static str {
    "stead-resources"
}
