//! Skill policy evaluation: a pure, independently testable component.
//!
//! Mirrors the upstream opt-in restriction model (§9.10): a skill is open to
//! authenticated company agents by default; a restriction policy may
//! allow-list agent ids/roles or deny specific agents.

use serde::{Deserialize, Serialize};

/// Restriction policy for a skill (JSON shape stored on the skill row).
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SkillRestrictionPolicy {
    /// Agents allowed to use the skill (empty = all).
    #[serde(default)]
    pub allowed_agent_ids: Vec<String>,
    /// Roles allowed to use the skill (empty = all).
    #[serde(default)]
    pub allowed_roles: Vec<String>,
    /// Agents explicitly denied.
    #[serde(default)]
    pub deny_agent_ids: Vec<String>,
}

/// Facts about the agent requesting the skill.
#[derive(Debug, Clone)]
pub struct AgentFacts {
    /// Agent id.
    pub agent_id: String,
    /// Owning company id.
    pub company_id: String,
    /// Agent role.
    pub role: String,
    /// Agent status.
    pub status: String,
}

/// Facts about the skill being requested.
#[derive(Debug, Clone)]
pub struct SkillFacts {
    /// Owning company id.
    pub company_id: String,
    /// Skill name.
    pub name: String,
    /// Skill status.
    pub status: String,
    /// Restriction policy.
    pub policy: SkillRestrictionPolicy,
}

/// Evaluation outcome.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SkillEvaluation {
    /// Whether the agent may use the skill.
    pub allowed: bool,
    /// Human-readable reason.
    pub reason: String,
}

/// Evaluates whether `agent` may use `skill`.
///
/// # Errors
///
/// This function is pure and never fails; it returns a decision with a
/// reason.
#[must_use]
pub fn evaluate_skill(agent: &AgentFacts, skill: &SkillFacts) -> SkillEvaluation {
    if agent.company_id != skill.company_id {
        return SkillEvaluation {
            allowed: false,
            reason: "skill belongs to a different company".to_owned(),
        };
    }
    if skill.status != "active" {
        return SkillEvaluation {
            allowed: false,
            reason: "skill is disabled".to_owned(),
        };
    }
    if agent.status != "active" {
        return SkillEvaluation {
            allowed: false,
            reason: "agent is not active".to_owned(),
        };
    }
    if skill.policy.deny_agent_ids.contains(&agent.agent_id) {
        return SkillEvaluation {
            allowed: false,
            reason: "agent is denied by policy".to_owned(),
        };
    }
    if !skill.policy.allowed_agent_ids.is_empty()
        && !skill.policy.allowed_agent_ids.contains(&agent.agent_id)
    {
        return SkillEvaluation {
            allowed: false,
            reason: "agent is not allow-listed".to_owned(),
        };
    }
    if !skill.policy.allowed_roles.is_empty() && !skill.policy.allowed_roles.contains(&agent.role) {
        return SkillEvaluation {
            allowed: false,
            reason: "role is not allow-listed".to_owned(),
        };
    }
    SkillEvaluation {
        allowed: true,
        reason: "allowed".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn agent(role: &str) -> AgentFacts {
        AgentFacts {
            agent_id: "a1".to_owned(),
            company_id: "c1".to_owned(),
            role: role.to_owned(),
            status: "active".to_owned(),
        }
    }

    fn skill(policy: SkillRestrictionPolicy) -> SkillFacts {
        SkillFacts {
            company_id: "c1".to_owned(),
            name: "s".to_owned(),
            status: "active".to_owned(),
            policy,
        }
    }

    #[test]
    fn allows_by_default() {
        let result = evaluate_skill(&agent("engineer"), &skill(Default::default()));
        assert!(result.allowed);
    }

    #[test]
    fn rejects_cross_company() {
        let mut facts = agent("engineer");
        facts.company_id = "c2".to_owned();
        let result = evaluate_skill(&facts, &skill(Default::default()));
        assert!(!result.allowed);
        assert!(result.reason.contains("different company"));
    }

    #[test]
    fn rejects_disabled_skill_and_inactive_agent() {
        let mut s = skill(Default::default());
        s.status = "disabled".to_owned();
        assert!(!evaluate_skill(&agent("engineer"), &s).allowed);

        let mut a = agent("engineer");
        a.status = "paused".to_owned();
        assert!(!evaluate_skill(&a, &skill(Default::default())).allowed);
    }

    #[test]
    fn enforces_allow_lists_and_deny_lists() {
        let allow = evaluate_skill(
            &agent("engineer"),
            &skill(SkillRestrictionPolicy {
                allowed_agent_ids: vec!["a2".to_owned()],
                ..Default::default()
            }),
        );
        assert!(!allow.allowed);
        assert!(allow.reason.contains("allow-list"));

        let allow_role = evaluate_skill(
            &agent("ceo"),
            &skill(SkillRestrictionPolicy {
                allowed_roles: vec!["ceo".to_owned()],
                ..Default::default()
            }),
        );
        assert!(allow_role.allowed);

        let deny = evaluate_skill(
            &agent("engineer"),
            &skill(SkillRestrictionPolicy {
                deny_agent_ids: vec!["a1".to_owned()],
                ..Default::default()
            }),
        );
        assert!(!deny.allowed);
        assert!(deny.reason.contains("denied"));
    }
}
