use cloud_terrastodon_registry::DiscoveryPlan;
use eyre::Result;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

/// Return longest-path layers, feedback-edge indices, and structural preorder.
/// Unlike compiler graphs, discovery graphs have no source-language loop headers.
/// Stable iterative DFS identifies only the edges needed to break cycles; it does
/// not infer nesting. Its preorder keeps a branch's descendants together even
/// when target IDs were allocated before their dependencies.
#[expect(
    clippy::type_complexity,
    reason = "layout returns its layer map, feedback-edge set, and structural node order together"
)]
pub(crate) fn layers(
    plan: &DiscoveryPlan,
) -> Result<(BTreeMap<usize, usize>, BTreeSet<usize>, Vec<usize>)> {
    let mut successors = plan
        .nodes
        .iter()
        .map(|node| (node.id, Vec::new()))
        .collect::<BTreeMap<_, _>>();
    for (index, edge) in plan.edges.iter().enumerate() {
        if successors.contains_key(&edge.target)
            && let Some(outgoing) = successors.get_mut(&edge.source)
        {
            outgoing.push((edge.target, index));
        }
    }
    for outgoing in successors.values_mut() {
        outgoing.sort_by(|&(target_a, edge_a), &(target_b, edge_b)| {
            target_a
                .cmp(&target_b)
                .then_with(|| plan.edges[edge_a].label.cmp(&plan.edges[edge_b].label))
                .then_with(|| edge_a.cmp(&edge_b))
        });
    }
    let incoming = successors
        .values()
        .flatten()
        .map(|&(target, _)| target)
        .collect::<BTreeSet<_>>();
    let mut roots = plan.nodes.iter().collect::<Vec<_>>();
    roots.sort_by_key(|node| {
        let priority = if node.seed {
            0
        } else if !incoming.contains(&node.id) {
            1
        } else {
            2
        };
        (priority, node.id)
    });
    let mut colors = successors
        .keys()
        .map(|&id| (id, 0_u8))
        .collect::<BTreeMap<_, _>>();
    let mut feedback = BTreeSet::new();
    let mut preorder = Vec::with_capacity(roots.len());
    for root in roots {
        if colors[&root.id] != 0 {
            continue;
        }
        colors.insert(root.id, 1);
        preorder.push(root.id);
        let mut stack = vec![(root.id, 0)];
        while let Some((node, next)) = stack.last_mut() {
            let node = *node;
            if let Some(&(target, edge)) = successors[&node].get(*next) {
                *next += 1;
                match colors[&target] {
                    0 => {
                        colors.insert(target, 1);
                        preorder.push(target);
                        stack.push((target, 0));
                    }
                    1 => {
                        feedback.insert(edge);
                    }
                    _ => {}
                }
            } else {
                colors.insert(node, 2);
                stack.pop();
            }
        }
    }

    let mut indegrees = successors
        .keys()
        .map(|&id| (id, 0_usize))
        .collect::<BTreeMap<_, _>>();
    for outgoing in successors.values() {
        for &(target, edge) in outgoing {
            if !feedback.contains(&edge) {
                *indegrees.get_mut(&target).ok_or_else(|| {
                    eyre::eyre!("missing incoming-edge count for node {target}")
                })? += 1;
            }
        }
    }
    let mut ready = indegrees
        .iter()
        .filter_map(|(&id, &count)| (count == 0).then_some(id))
        .collect::<BTreeSet<_>>();
    let mut layers = successors
        .keys()
        .map(|&id| (id, 0_usize))
        .collect::<BTreeMap<_, _>>();
    while let Some(node) = ready.pop_first() {
        for &(target, edge) in &successors[&node] {
            if feedback.contains(&edge) {
                continue;
            }
            let layer = layers[&node] + 1;
            let target_layer = layers
                .get_mut(&target)
                .ok_or_else(|| eyre::eyre!("missing layout layer for node {target}"))?;
            *target_layer = (*target_layer).max(layer);
            let count = indegrees
                .get_mut(&target)
                .ok_or_else(|| eyre::eyre!("missing incoming-edge count for node {target}"))?;
            *count -= 1;
            if *count == 0 {
                ready.insert(target);
            }
        }
    }
    Ok((layers, feedback, preorder))
}
