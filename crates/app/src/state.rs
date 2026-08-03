//! Shared application state, registered on the Topcoat app context.

use std::sync::Arc;

use staple_data::{
    ActivityRepository, AgentRepository, ApiKeyRepository, ApprovalRepository, AssetRepository,
    CompanyRepository, CostRepository, DecisionRepository, DocumentRepository,
    EnvironmentRepository, ExternalObjectRepository, GoalRepository, HeartbeatRepository,
    IssueCommentRepository, IssueRelationRepository, IssueRepository, IssueStructureRepository,
    LabelRepository, PermissionGrantRepository, ProjectRepository, RoutineRepository,
    SecretRepository, SkillRepository, WorkProductRepository, WorkspaceRepository,
};

use crate::storage::LocalStorage;
use staple_adapters::{AdapterRegistry, PluginReport};

/// Application-wide dependencies for route handlers.
#[derive(Clone)]
pub struct AppState {
    /// Companies repository.
    pub companies: Arc<dyn CompanyRepository>,
    /// Agents repository (org hierarchy, subordinate budgets).
    pub agents: Arc<dyn AgentRepository>,
    /// Principal permission grants repository.
    pub permission_grants: Arc<dyn PermissionGrantRepository>,
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
    /// Environments repository.
    pub environments: Arc<dyn EnvironmentRepository>,
    /// Workspaces repository.
    pub workspaces: Arc<dyn WorkspaceRepository>,
    /// Labels repository.
    pub labels: Arc<dyn LabelRepository>,
    /// Issue structure repository.
    pub issue_structure: Arc<dyn IssueStructureRepository>,
    /// Routines repository.
    pub routines: Arc<dyn RoutineRepository>,
    /// Adapter registry.
    pub adapters: Arc<AdapterRegistry>,
    /// Plugin load diagnostics.
    pub plugin_reports: Vec<PluginReport>,
}
