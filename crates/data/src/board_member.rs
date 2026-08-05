//! Built-in board-member skill.
//!
//! The board-member skill powers the board chat surface ("Board Concierge"):
//! it is a system prompt plus an allowed-action contract that the later
//! streaming chat endpoint (`/api/board/chat/stream`) will feed to the model.
//! It mirrors the upstream Board Chat surface, which is "powered by the
//! board-member skill" (see `ui/src/pages/BoardChat.tsx`).

use crate::skills::{
    AgentFacts, SkillEvaluation, SkillFacts, SkillRestrictionPolicy, evaluate_skill,
};

/// Name of the built-in board-member skill.
pub const BOARD_MEMBER_SKILL_NAME: &str = "board-member";

/// The built-in board-member skill definition.
#[derive(Debug, Clone, PartialEq)]
pub struct BoardMemberSkill {
    /// Skill name.
    pub name: &'static str,
    /// Skill description.
    pub description: &'static str,
    /// Restriction policy. The policy is open to all active agents of the
    /// company (the shared evaluator still enforces company scoping, active
    /// status, and any allow/deny lists); the prompt spells out the
    /// read-only-by-default and explicit-mutation authorization model.
    pub restriction_policy: SkillRestrictionPolicy,
    /// System prompt: role, allowed queries/actions, company-scoped
    /// authorization boundaries, and output format.
    pub system_prompt: &'static str,
}

/// System prompt for the board-member skill.
const SYSTEM_PROMPT: &str = r#"You are the Board Concierge, a board-level assistant that manages the company board on behalf of the operator. The user talks to you conversationally; translate natural language into board capabilities and present results clearly.

## Allowed queries (read-only, default)
- List companies
- List and get projects for the active company
- List, search, and get issues for the active company
- List and get agents for the active company
- Read cost summaries for the active company (monthly spend, budget, utilization)

## Allowed actions (explicit, only on an explicit user request)
- Change an issue's status
- Create a comment on an issue

No other mutations are allowed. If the user asks for a mutation outside this list, decline and explain that the action is not available to the board-member skill.

## Company scoping and authorization
- All reads and writes are scoped to the active company. Never access or expose data from any other company.
- Only act on resources that belong to the active company; verify ownership before reading or mutating.
- The board context is a full-control operator context. Agent identity must be active and belong to the same company.
- Never bypass company boundaries, and never share data across companies.

## Output format
- Stream responses as markdown.
- Use markdown tables for lists (issues, agents, costs) and bold for status values.
- Keep responses concise and surface what needs attention first.
- Optional: emit structured action metadata as `%%ACTIONS%%{...}%%/ACTIONS%%` for the UI observer layer; this metadata must never appear in a durable comment body.
"#;

/// Returns the built-in board-member skill.
#[must_use]
pub fn board_member_skill() -> BoardMemberSkill {
    BoardMemberSkill {
        name: BOARD_MEMBER_SKILL_NAME,
        description: "Board Concierge: manage the company board on behalf of the operator",
        restriction_policy: SkillRestrictionPolicy::default(),
        system_prompt: SYSTEM_PROMPT,
    }
}

/// Evaluates whether an agent may use the board-member skill in `company_id`.
///
/// Reuses the shared skill evaluator with the built-in skill's restriction
/// policy and an `active` status.
///
/// # Errors
///
/// This function is pure and never fails; it returns a decision with a
/// reason.
#[must_use]
pub fn evaluate_board_member_skill(agent: &AgentFacts, company_id: &str) -> SkillEvaluation {
    let skill = board_member_skill();
    evaluate_skill(
        agent,
        &SkillFacts {
            company_id: company_id.to_owned(),
            name: skill.name.to_owned(),
            status: "active".to_owned(),
            policy: skill.restriction_policy,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn agent(company_id: &str, status: &str) -> AgentFacts {
        AgentFacts {
            agent_id: "a1".to_owned(),
            company_id: company_id.to_owned(),
            role: "engineer".to_owned(),
            status: status.to_owned(),
        }
    }

    #[test]
    fn loads_with_expected_identity_and_prompt() {
        let skill = board_member_skill();
        assert_eq!(skill.name, BOARD_MEMBER_SKILL_NAME);
        assert!(skill.description.contains("Board Concierge"));
        assert!(!skill.system_prompt.is_empty());
        assert!(
            skill
                .system_prompt
                .contains("Company scoping and authorization")
        );
    }

    #[test]
    fn prompt_contains_company_scope_boundary() {
        let prompt = board_member_skill().system_prompt;
        assert!(prompt.contains("scoped to the active company"));
        assert!(prompt.contains("Never access or expose data from any other company"));
    }

    #[test]
    fn prompt_lists_allowed_queries_and_mutations() {
        let prompt = board_member_skill().system_prompt;
        for expected in [
            "List companies",
            "projects",
            "issues",
            "agents",
            "cost summaries",
            "Change an issue's status",
            "Create a comment on an issue",
        ] {
            assert!(prompt.contains(expected), "prompt missing {expected:?}");
        }
    }

    #[test]
    fn restriction_policy_is_open_and_read_only_default() {
        let skill = board_member_skill();
        assert_eq!(skill.restriction_policy, SkillRestrictionPolicy::default());
        assert!(skill.restriction_policy.allowed_agent_ids.is_empty());
        assert!(skill.restriction_policy.allowed_roles.is_empty());
        assert!(skill.restriction_policy.deny_agent_ids.is_empty());
        assert!(skill.system_prompt.contains("read-only, default"));
    }

    #[test]
    fn evaluate_allows_same_company_active_agent() {
        let result = evaluate_board_member_skill(&agent("c1", "active"), "c1");
        assert!(result.allowed);
        assert_eq!(result.reason, "allowed");
    }

    #[test]
    fn evaluate_rejects_cross_company_and_inactive_agent() {
        let cross = evaluate_board_member_skill(&agent("c2", "active"), "c1");
        assert!(!cross.allowed);
        assert!(cross.reason.contains("different company"));

        let paused = evaluate_board_member_skill(&agent("c1", "paused"), "c1");
        assert!(!paused.allowed);
        assert!(paused.reason.contains("not active"));
    }
}
