//! Shared application state, registered on the Topcoat app context.

use std::sync::Arc;

use staple_data::{
    ActivityRepository, AgentRepository, AgentRuntimeRepository, ApiKeyRepository,
    ApprovalRepository, AssetRepository, BoardKeyRepository, BudgetPolicyRepository,
    CaseRepository, CompanyRepository, CostRepository, DecisionActionRepository,
    DecisionRepository, DocumentRepository, EnvironmentRepository, ExternalObjectCatalogRepository,
    ExternalObjectRepository, GoalRepository, HeartbeatRepository, InfrastructureRepository,
    InviteRepository, IssueCommentRepository, IssueRelationRepository, IssueRepository,
    IssueStructureRepository, LabelRepository, MembershipRepository, PermissionGrantRepository,
    PipelineRepository, PluginRepository, PluginRuntimeRepository, PreferenceRepository,
    ProjectRepository, RoutineRepository, ScatteredRepository, SecretBindingRepository,
    SecretRepository, SkillCatalogRepository, SkillRepository, WorkProductRepository,
    WorkspaceRepository,
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
    /// Agent runtime (sessions/state/wakeups/recovery) repository.
    pub agent_runtime: Arc<dyn AgentRuntimeRepository>,
    /// Principal permission grants repository.
    pub permission_grants: Arc<dyn PermissionGrantRepository>,
    /// Company memberships / instance roles repository.
    pub memberships: Arc<dyn MembershipRepository>,
    /// Invites / join requests repository.
    pub invites: Arc<dyn InviteRepository>,
    /// Infrastructure repository (auth/settings/folders/watchdogs/events).
    pub infrastructure: Arc<dyn InfrastructureRepository>,
    /// Board API keys / CLI auth challenges repository.
    pub board_keys: Arc<dyn BoardKeyRepository>,
    /// Budget policies / incidents repository.
    pub budget_policies: Arc<dyn BudgetPolicyRepository>,
    /// Cases repository.
    pub cases: Arc<dyn CaseRepository>,
    /// Sidebar preferences / company logos repository.
    pub preferences: Arc<dyn PreferenceRepository>,
    /// Pipelines repository.
    pub pipelines: Arc<dyn PipelineRepository>,
    /// Plugin registry/config/settings/resources repository.
    pub plugins: Arc<dyn PluginRepository>,
    /// Plugin runtime (state/entities/jobs/logs/webhooks/db) repository.
    pub plugin_runtime: Arc<dyn PluginRuntimeRepository>,
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
    /// Decision action domain repository (bundles/decisions/effects/training).
    pub decision_actions: Arc<dyn DecisionActionRepository>,
    /// External object repository (legacy issue links).
    pub external_objects: Arc<dyn ExternalObjectRepository>,
    /// External-object catalog + mentions repository (upstream alignment).
    pub external_object_catalog: Arc<dyn ExternalObjectCatalogRepository>,
    /// Skill catalog repository (versions/policies/comments/stars/test runs).
    pub skill_catalog: Arc<dyn SkillCatalogRepository>,
    /// Secret binding repository (provider configs/bindings/user secrets).
    pub secret_bindings: Arc<dyn SecretBindingRepository>,
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
    /// Scattered domain repository (status cards/smoke/feedback/finance/annotations).
    pub scattered: Arc<dyn ScatteredRepository>,
    /// Adapter registry.
    pub adapters: Arc<AdapterRegistry>,
    /// Plugin load diagnostics.
    pub plugin_reports: Vec<PluginReport>,
}
