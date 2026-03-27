use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::RoleDefinition;
use cloud_terrastodon_azure::RolePermissionAction;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_role_definitions_and_assignments;
use eyre::Result;
use serde::Serialize;
use std::collections::HashSet;
use std::io::Write;
use tracing::info;

/// Find role definitions and role assignments that satisfy an action or data action.
#[derive(Args, Debug, Clone)]
pub struct AzureRoleDefinitionFindArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,

    /// Required action or data action to search for.
    pub action: RolePermissionAction,
}

impl AzureRoleDefinitionFindArgs {
    pub async fn invoke(self) -> Result<()> {
        info!(action = %self.action, "Fetching Azure role definitions and role assignments");
        let rbac = fetch_all_role_definitions_and_assignments(self.tenant.resolve().await?).await?;

        let fallback_chain = build_fallback_chain(&self.action);
        let literal_match_counts = fallback_chain
            .iter()
            .map(|candidate| LiteralMatchCount {
                candidate: candidate.to_string(),
                role_definition_count: rbac
                    .role_definitions
                    .values()
                    .filter(|rd| role_definition_has_literal(rd, candidate))
                    .count(),
            })
            .collect::<Vec<_>>();
        let first_literal_hit = literal_match_counts
            .iter()
            .position(|item| item.role_definition_count > 0)
            .map(|idx| fallback_chain[idx].to_string());

        let mut role_definition_matches = rbac
            .role_definitions
            .values()
            .filter_map(|rd| evaluate_role_definition(rd, &self.action, &fallback_chain))
            .collect::<Vec<_>>();
        role_definition_matches.sort_by(role_definition_match_cmp);

        let role_definition_match_lookup = role_definition_matches
            .iter()
            .map(|m| (m.role_definition_id.clone(), m.clone()))
            .collect::<std::collections::HashMap<_, _>>();

        let mut assignment_matches = rbac
            .role_assignments
            .values()
            .filter_map(|assignment| {
                let role_match =
                    role_definition_match_lookup.get(&assignment.role_definition_id)?;
                Some(RoleAssignmentMatch {
                    role_assignment_id: assignment.id.clone(),
                    role_definition_id: role_match.role_definition_id.clone(),
                    role_definition_name: role_match.role_definition_name.clone(),
                    principal_id: assignment.principal_id,
                    scope: assignment.scope.expanded_form(),
                    scope_specificity: assignment.scope.expanded_form().len(),
                    specificity_cost: role_match.specificity_cost,
                    match_source: role_match.match_source,
                    matched_permission: role_match.matched_permission.clone(),
                    literal_fallback_rank: role_match.literal_fallback_rank,
                })
            })
            .collect::<Vec<_>>();
        assignment_matches.sort_by(role_assignment_match_cmp);

        let output = RoleDefinitionFindOutput {
            query_action: self.action.to_string(),
            fallback_chain: fallback_chain.iter().map(ToString::to_string).collect(),
            literal_match_counts,
            first_literal_hit,
            role_definition_matches,
            role_assignment_matches: assignment_matches,
        };

        info!(
            action = %self.action,
            definition_matches = output.role_definition_matches.len(),
            assignment_matches = output.role_assignment_matches.len(),
            "Completed role definition find"
        );

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &output)?;
        handle.write_all(b"\n")?;

        Ok(())
    }
}

#[derive(Debug, Serialize)]
struct RoleDefinitionFindOutput {
    query_action: String,
    fallback_chain: Vec<String>,
    literal_match_counts: Vec<LiteralMatchCount>,
    first_literal_hit: Option<String>,
    role_definition_matches: Vec<RoleDefinitionMatch>,
    role_assignment_matches: Vec<RoleAssignmentMatch>,
}

#[derive(Debug, Serialize)]
struct LiteralMatchCount {
    candidate: String,
    role_definition_count: usize,
}

#[derive(Debug, Serialize, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "snake_case")]
enum MatchSource {
    Action,
    DataAction,
}

#[derive(Debug, Serialize, Clone)]
struct RoleDefinitionMatch {
    role_definition_id: cloud_terrastodon_azure::RoleDefinitionId,
    role_definition_name: String,
    specificity_cost: u64,
    match_source: MatchSource,
    matched_permission: String,
    literal_fallback_rank: Option<usize>,
}

#[derive(Debug, Serialize)]
struct RoleAssignmentMatch {
    role_assignment_id: cloud_terrastodon_azure::RoleAssignmentId,
    role_definition_id: cloud_terrastodon_azure::RoleDefinitionId,
    role_definition_name: String,
    principal_id: cloud_terrastodon_azure::PrincipalId,
    scope: String,
    scope_specificity: usize,
    specificity_cost: u64,
    match_source: MatchSource,
    matched_permission: String,
    literal_fallback_rank: Option<usize>,
}

fn evaluate_role_definition(
    role_definition: &RoleDefinition,
    query_action: &RolePermissionAction,
    fallback_chain: &[RolePermissionAction],
) -> Option<RoleDefinitionMatch> {
    let mut best: Option<RoleDefinitionMatch> = None;

    for permission in &role_definition.permissions {
        if permission.satisfies(std::slice::from_ref(query_action), &[]) {
            for action in &permission.actions {
                if !action.satisfies(query_action) {
                    continue;
                }
                let candidate = RoleDefinitionMatch {
                    role_definition_id: role_definition.id.clone(),
                    role_definition_name: role_definition.display_name.clone(),
                    specificity_cost: action_specificity_cost(action),
                    match_source: MatchSource::Action,
                    matched_permission: action.to_string(),
                    literal_fallback_rank: fallback_rank(action, fallback_chain),
                };
                if best
                    .as_ref()
                    .is_none_or(|current| role_definition_match_cmp(&candidate, current).is_lt())
                {
                    best = Some(candidate);
                }
            }
        }

        if permission.satisfies(&[], std::slice::from_ref(query_action)) {
            for action in &permission.data_actions {
                if !action.satisfies(query_action) {
                    continue;
                }
                let candidate = RoleDefinitionMatch {
                    role_definition_id: role_definition.id.clone(),
                    role_definition_name: role_definition.display_name.clone(),
                    specificity_cost: action_specificity_cost(action),
                    match_source: MatchSource::DataAction,
                    matched_permission: action.to_string(),
                    literal_fallback_rank: fallback_rank(action, fallback_chain),
                };
                if best
                    .as_ref()
                    .is_none_or(|current| role_definition_match_cmp(&candidate, current).is_lt())
                {
                    best = Some(candidate);
                }
            }
        }
    }

    best
}

fn role_definition_has_literal(
    role_definition: &RoleDefinition,
    candidate: &RolePermissionAction,
) -> bool {
    role_definition.permissions.iter().any(|permission| {
        permission
            .actions
            .iter()
            .any(|a| eq_ignore_case(a, candidate))
            || permission
                .data_actions
                .iter()
                .any(|a| eq_ignore_case(a, candidate))
    })
}

fn fallback_rank(
    action: &RolePermissionAction,
    fallback_chain: &[RolePermissionAction],
) -> Option<usize> {
    fallback_chain
        .iter()
        .position(|candidate| eq_ignore_case(action, candidate))
}

fn eq_ignore_case(left: &RolePermissionAction, right: &RolePermissionAction) -> bool {
    left.to_string().eq_ignore_ascii_case(&right.to_string())
}

fn action_specificity_cost(action: &RolePermissionAction) -> u64 {
    const ACTION_BASE_COST: u64 = 1_000;
    const WILDCARD_COST: u64 = 1_000_000;
    const EARLY_WILDCARD_COST: u64 = 10_000;
    const MAX_WILDCARD_DEPTH: u64 = 16;

    let mut cost = ACTION_BASE_COST;
    if let Some(idx) = action.find('*') {
        cost = cost.saturating_add(WILDCARD_COST);
        let prefix = &action[..idx];
        let segments_before_wildcard = prefix
            .split('/')
            .filter(|segment| !segment.is_empty())
            .count() as u64;
        let depth_penalty = MAX_WILDCARD_DEPTH
            .saturating_sub(segments_before_wildcard)
            .saturating_mul(EARLY_WILDCARD_COST);
        cost = cost.saturating_add(depth_penalty);
    }

    cost
}

fn role_definition_match_cmp(
    left: &RoleDefinitionMatch,
    right: &RoleDefinitionMatch,
) -> std::cmp::Ordering {
    left.specificity_cost
        .cmp(&right.specificity_cost)
        .then_with(|| left.match_source.cmp(&right.match_source))
        .then_with(|| left.role_definition_name.cmp(&right.role_definition_name))
        .then_with(|| left.matched_permission.cmp(&right.matched_permission))
}

fn role_assignment_match_cmp(
    left: &RoleAssignmentMatch,
    right: &RoleAssignmentMatch,
) -> std::cmp::Ordering {
    left.specificity_cost
        .cmp(&right.specificity_cost)
        .then_with(|| right.scope_specificity.cmp(&left.scope_specificity))
        .then_with(|| left.role_definition_name.cmp(&right.role_definition_name))
        .then_with(|| left.scope.cmp(&right.scope))
}

fn build_fallback_chain(action: &RolePermissionAction) -> Vec<RolePermissionAction> {
    let raw = action.to_string();
    let segments = raw
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    if segments.is_empty() {
        return vec![RolePermissionAction::new(raw)];
    }

    let mut seen = HashSet::<String>::new();
    let mut chain = Vec::new();

    let exact = raw.clone();
    if seen.insert(exact.clone()) {
        chain.push(RolePermissionAction::new(exact));
    }

    for keep_count in (1..segments.len()).rev() {
        let candidate = format!("{}/*", segments[..keep_count].join("/"));
        if seen.insert(candidate.clone()) {
            chain.push(RolePermissionAction::new(candidate));
        }
    }

    chain
}

#[cfg(test)]
mod tests {
    use super::action_specificity_cost;
    use super::build_fallback_chain;
    use cloud_terrastodon_azure::RolePermissionAction;

    #[test]
    fn fallback_chain_progressively_generalizes() {
        let chain = build_fallback_chain(&RolePermissionAction::new(
            "Microsoft.ContainerInstance/containerGroups/containers/exec/action",
        ));
        let values = chain.iter().map(ToString::to_string).collect::<Vec<_>>();

        assert_eq!(
            values,
            vec![
                "Microsoft.ContainerInstance/containerGroups/containers/exec/action",
                "Microsoft.ContainerInstance/containerGroups/containers/exec/*",
                "Microsoft.ContainerInstance/containerGroups/containers/*",
                "Microsoft.ContainerInstance/containerGroups/*",
                "Microsoft.ContainerInstance/*",
            ]
        );
    }

    #[test]
    fn exact_is_more_specific_than_wildcards() {
        let exact = RolePermissionAction::new(
            "Microsoft.ContainerInstance/containerGroups/containers/exec/action",
        );
        let narrower_wildcard =
            RolePermissionAction::new("Microsoft.ContainerInstance/containerGroups/containers/*");
        let broader_wildcard = RolePermissionAction::new("Microsoft.ContainerInstance/*");

        assert!(action_specificity_cost(&exact) < action_specificity_cost(&narrower_wildcard));
        assert!(
            action_specificity_cost(&narrower_wildcard)
                < action_specificity_cost(&broader_wildcard)
        );
    }
}
