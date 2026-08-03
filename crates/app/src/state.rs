//! Shared application state, registered on the Topcoat app context.

use std::sync::Arc;

use staple_data::{
    ActivityRepository, ApiKeyRepository, ApprovalRepository, AssetRepository, CompanyRepository,
    CostRepository, DecisionRepository, DocumentRepository, ExternalObjectRepository,
    GoalRepository, HeartbeatRepository, IssueCommentRepository, IssueRelationRepository,
    IssueRepository, ProjectRepository, SecretRepository, SkillRepository, WorkProductRepository,
};

use crate::storage::LocalStorage;

/// Application-wide dependencies for route handlers.
#[derive(Clone)]
pub struct AppState {
    /// Companies repository.
    pub companies: Arc<dyn CompanyRepository>,
    /// Goals repository.
    pub goals: Arc<dyn GoalRepository>,
    /// Projects repository.
    pub projects: Arc<dyn ProjectRepository>,
    /// Issues repository.
    pub issues: Arc<dyn IssueRepository>,
    /// Issue comments repository.
    pub comments: Arc<dyn IssueCommentRepository>,
    /// Documents repository.
    pub documents: Arc<dyn DocumentRepository>,
    /// Assets repository.
    pub assets: Arc<dyn AssetRepository>,
    /// Issue relations (blockers) repository.
    pub relations: Arc<dyn IssueRelationRepository>,
    /// Local disk attachment storage.
    pub storage: LocalStorage,
    /// Issue work products repository.
    pub work_products: Arc<dyn WorkProductRepository>,
    /// Heartbeat runs repository.
    pub heartbeat: Arc<dyn HeartbeatRepository>,
    /// Costs/budget repository.
    pub costs: Arc<dyn CostRepository>,
    /// Approvals repository.
    pub approvals: Arc<dyn ApprovalRepository>,
    /// Activity log repository.
    pub activity: Arc<dyn ActivityRepository>,
    /// Secrets repository.
    pub secrets: Arc<dyn SecretRepository>,
    /// Agent API key repository (also used by the auth layer).
    pub api_keys: Arc<dyn ApiKeyRepository>,
    /// Decision desk repository.
    pub decisions: Arc<dyn DecisionRepository>,
    /// External object repository.
    pub external_objects: Arc<dyn ExternalObjectRepository>,
    /// Skills repository.
    pub skills: Arc<dyn SkillRepository>,
}
