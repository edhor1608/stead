use std::fs;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use stead_contracts::{AttentionTier, Contract, ContractStatus, SqliteContractStore};
use stead_resources::{ClaimResult, ResourceEvent, ResourceKey, ResourceLease, ResourceRegistry};

pub const API_VERSION: &str = "v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiEnvelope<T> {
    pub version: &'static str,
    pub data: T,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiRequest {
    Health,
    CreateContract {
        id: String,
        blocked_by: Vec<String>,
    },
    ListContracts,
    AttentionStatus,
    TransitionContract {
        id: String,
        to: ContractStatus,
    },
    GetContract {
        id: String,
    },
    ClaimResource {
        resource: ResourceKey,
        owner: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiResponse {
    Health { status: String },
    ContractState(Contract),
    Contracts(Vec<Contract>),
    Attention(AttentionCounts),
    ResourceClaim(ClaimResult),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AttentionCounts {
    pub needs_decision: usize,
    pub anomaly: usize,
    pub completed: usize,
    pub running: usize,
    pub queued: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ApiError {
    pub code: &'static str,
    pub message: String,
}

impl ApiError {
    fn not_found(message: impl Into<String>) -> Self {
        Self {
            code: "not_found",
            message: message.into(),
        }
    }

    fn invalid_transition(message: impl Into<String>) -> Self {
        Self {
            code: "invalid_transition",
            message: message.into(),
        }
    }

    fn storage(message: impl Into<String>) -> Self {
        Self {
            code: "storage_error",
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonEvent {
    pub cursor: u64,
    pub kind: DaemonEventKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonEventKind {
    ContractCreated {
        id: String,
    },
    ContractTransitioned {
        id: String,
        from: ContractStatus,
        to: ContractStatus,
    },
    ResourceConflictEscalated {
        resource: ResourceKey,
        requested_by: String,
        held_by: String,
        reason: &'static str,
    },
}

#[derive(Debug, Default)]
struct EventState {
    next_cursor: u64,
    history: Vec<DaemonEvent>,
    subscribers: Vec<Sender<DaemonEvent>>,
}

#[derive(Debug, Clone)]
pub struct Daemon {
    store: SqliteContractStore,
    resources_path: std::path::PathBuf,
    resources: Arc<Mutex<ResourceRegistry>>,
    events: Arc<Mutex<EventState>>,
}

impl Daemon {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, ApiError> {
        Self::with_port_range(path, 3000, 4999)
    }

    pub fn with_port_range(path: impl AsRef<Path>, start: u16, end: u16) -> Result<Self, ApiError> {
        let db_path = path.as_ref().to_path_buf();
        let store =
            SqliteContractStore::open(&db_path).map_err(|e| ApiError::storage(e.to_string()))?;
        let resources_path = db_path.with_file_name("resources.json");
        let mut registry = ResourceRegistry::with_port_range(start, end);
        registry.import_leases(load_resource_leases(&resources_path));

        Ok(Self {
            store,
            resources_path,
            resources: Arc::new(Mutex::new(registry)),
            events: Arc::new(Mutex::new(EventState::default())),
        })
    }

    pub fn handle(&self, req: ApiRequest) -> Result<ApiEnvelope<ApiResponse>, ApiError> {
        let data = match req {
            ApiRequest::Health => ApiResponse::Health {
                status: "ok".to_string(),
            },
            ApiRequest::CreateContract { id, blocked_by } => {
                let contract = Contract::new(id.clone(), blocked_by);
                self.store
                    .save_contract(&contract)
                    .map_err(|e| ApiError::storage(e.to_string()))?;

                self.publish(DaemonEventKind::ContractCreated { id });
                ApiResponse::ContractState(contract)
            }
            ApiRequest::ListContracts => {
                let contracts = self
                    .store
                    .list_contracts()
                    .map_err(|e| ApiError::storage(e.to_string()))?;
                ApiResponse::Contracts(contracts)
            }
            ApiRequest::AttentionStatus => {
                let counts = AttentionCounts {
                    needs_decision: self
                        .store
                        .list_by_attention_tier(AttentionTier::NeedsDecision)
                        .map_err(|e| ApiError::storage(e.to_string()))?
                        .len(),
                    anomaly: self
                        .store
                        .list_by_attention_tier(AttentionTier::Anomaly)
                        .map_err(|e| ApiError::storage(e.to_string()))?
                        .len(),
                    completed: self
                        .store
                        .list_by_attention_tier(AttentionTier::Completed)
                        .map_err(|e| ApiError::storage(e.to_string()))?
                        .len(),
                    running: self
                        .store
                        .list_by_attention_tier(AttentionTier::Running)
                        .map_err(|e| ApiError::storage(e.to_string()))?
                        .len(),
                    queued: self
                        .store
                        .list_by_attention_tier(AttentionTier::Queued)
                        .map_err(|e| ApiError::storage(e.to_string()))?
                        .len(),
                };
                ApiResponse::Attention(counts)
            }
            ApiRequest::TransitionContract { id, to } => {
                let mut contract = self
                    .store
                    .load_contract(&id)
                    .map_err(|e| ApiError::storage(e.to_string()))?
                    .ok_or_else(|| ApiError::not_found(format!("contract not found: {id}")))?;

                let event = contract
                    .transition_to(to)
                    .map_err(|e| ApiError::invalid_transition(e.to_string()))?;

                self.store
                    .record_transition(&contract, &event)
                    .map_err(|e| ApiError::storage(e.to_string()))?;

                self.publish(DaemonEventKind::ContractTransitioned {
                    id: event.contract_id,
                    from: event.from,
                    to: event.to,
                });

                ApiResponse::ContractState(contract)
            }
            ApiRequest::GetContract { id } => {
                let contract = self
                    .store
                    .load_contract(&id)
                    .map_err(|e| ApiError::storage(e.to_string()))?
                    .ok_or_else(|| ApiError::not_found(format!("contract not found: {id}")))?;

                ApiResponse::ContractState(contract)
            }
            ApiRequest::ClaimResource { resource, owner } => {
                let (claim, resource_events, leases) = {
                    let mut registry = self.resources.lock().expect("resource lock poisoned");
                    let claim = registry.claim(resource, owner);
                    let events = registry.drain_events();
                    let leases = registry.export_leases();
                    (claim, events, leases)
                };

                self.persist_resource_leases(&leases)?;

                for event in resource_events {
                    self.publish_resource_event(event);
                }

                ApiResponse::ResourceClaim(claim)
            }
        };

        Ok(ApiEnvelope {
            version: API_VERSION,
            data,
        })
    }

    pub fn subscribe(&self) -> Receiver<DaemonEvent> {
        let (tx, rx) = mpsc::channel();
        let mut state = self.events.lock().expect("event lock poisoned");
        state.subscribers.push(tx);
        rx
    }

    pub fn replay_from(&self, cursor: u64) -> Vec<DaemonEvent> {
        let state = self.events.lock().expect("event lock poisoned");
        state
            .history
            .iter()
            .filter(|event| event.cursor > cursor)
            .cloned()
            .collect()
    }

    fn publish(&self, kind: DaemonEventKind) {
        let mut state = self.events.lock().expect("event lock poisoned");
        state.next_cursor += 1;

        let event = DaemonEvent {
            cursor: state.next_cursor,
            kind,
        };

        state.history.push(event.clone());
        state
            .subscribers
            .retain(|sender| sender.send(event.clone()).is_ok());
    }

    fn publish_resource_event(&self, event: ResourceEvent) {
        match event {
            ResourceEvent::ConflictEscalated {
                requested,
                requested_by,
                held_by,
                reason,
            } => {
                self.publish(DaemonEventKind::ResourceConflictEscalated {
                    resource: requested,
                    requested_by,
                    held_by,
                    reason,
                });
            }
        }
    }

    fn persist_resource_leases(&self, leases: &[ResourceLease]) -> Result<(), ApiError> {
        let data = serde_json::to_string(leases).map_err(|e| ApiError::storage(e.to_string()))?;
        fs::write(&self.resources_path, data).map_err(|e| ApiError::storage(e.to_string()))
    }
}

fn load_resource_leases(path: &Path) -> Vec<ResourceLease> {
    let Ok(raw) = fs::read_to_string(path) else {
        return Vec::new();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

pub fn crate_identity() -> &'static str {
    "stead-daemon"
}
