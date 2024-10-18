use crate::prelude::LeaderFollowerJoint;
use crate::prelude::OrganizerAction;
use avian2d::dynamics::solver::xpbd::XpbdConstraint;
use avian2d::prelude::DistanceJoint;
use avian2d::prelude::LinearVelocity;
use avian2d::prelude::Position;
use bevy::ecs::system::QueryLens;
use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use bevy::utils::HashSet;
use indexmap::IndexMap;
use indexmap::IndexSet;
use itertools::Itertools;
use leafwing_input_manager::prelude::ActionState;
use std::cmp::max;
use std::cmp::min;
use std::ops::RangeInclusive;

pub struct AlignmentPlugin;

impl Plugin for AlignmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (left_to_right_flow, vertical_alignment, vertical_placement).run_if(
                |actions_query: Query<&ActionState<OrganizerAction>>| {
                    actions_query
                        .get_single()
                        .map(|a| a.pressed(&OrganizerAction::Horizontal))
                        .unwrap_or_default()
                },
            ),
        );
    }
}

fn left_to_right_flow(
    joint_query: Query<&DistanceJoint, With<LeaderFollowerJoint>>,
    mut phys_query: Query<(&Position, &mut LinearVelocity)>,
) {
    for joint in joint_query.iter() {
        let [leader_entity, follower_entity] = joint.entities();
        let Ok([leader, follower]) = phys_query.get_many_mut([leader_entity, follower_entity])
        else {
            warn!("Failed to get leader-follower position data for {leader_entity} and {follower_entity}");
            continue;
        };
        let (leader_pos, mut leader_vel) = leader;
        let (follower_pos, mut follower_vel) = follower;
        let x_dist = follower_pos.x - leader_pos.x;

        // if the follower isn't to the right of the leader
        if x_dist < 500.0 {
            follower_vel.x += 200.0;
            leader_vel.x += -200.0;
        }

        // if the follower is very far to the right of the leader
        if x_dist > 3000.0 {
            follower_vel.x -= 100.0;
            leader_vel.x += 100.0;
        }
    }
}

fn get_neighbourhoods(
    mut lf_joint_query: QueryLens<&DistanceJoint>,
) -> HashMap<Entity, HashSet<Entity>> {
    let mut neighbourhoods = HashMap::new();
    for joint in lf_joint_query.query().iter() {
        let [leader, follower] = joint.entities();
        let neighbourhood = neighbourhoods.entry(leader).or_insert_with(HashSet::new);
        neighbourhood.insert(follower);
    }
    neighbourhoods
}

fn sort_neighbourhoods(
    neighbourhoods: &HashMap<Entity, HashSet<Entity>>,
    y_values: &HashMap<Entity, f32>,
) -> IndexMap<Entity, IndexSet<Entity>> {
    neighbourhoods
        .into_iter()
        .sorted_by_cached_key(|(key, _)| (y_values.get(*key).unwrap_or(&0.0) * 1024.0) as i32)
        .map(|(key, value)| {
            let value = value
                .iter()
                .sorted_by_cached_key(|e| (y_values.get(*e).unwrap_or(&0.0) * 1024.0) as i32)
                .rev()
                .cloned()
                .collect::<IndexSet<Entity>>();
            (*key, value)
        })
        .rev()
        .collect()
}

fn vertical_alignment(
    mut lf_joint_query: Query<&DistanceJoint, With<LeaderFollowerJoint>>,
    mut phys_query: Query<(&Position, &mut LinearVelocity)>,
) {
    // we want the followers to converge on a common X value
    let neighbourhoods = get_neighbourhoods(lf_joint_query.as_query_lens());
    for neighbourhood in neighbourhoods.into_values() {
        // Get the mean x value
        let mean_x = neighbourhood
            .iter()
            .filter_map(|&e| phys_query.get(e).ok())
            .map(|x| x.0.x)
            .sum::<f32>()
            / neighbourhood.len() as f32;

        // Influence the members towards the mean x value
        for member_entity in neighbourhood {
            let Ok((member_pos, mut member_vel)) = phys_query.get_mut(member_entity) else {
                warn!("Failed to get member pos and vel for {member_entity}");
                continue;
            };
            let diff = mean_x - member_pos.x;
            member_vel.x += diff;
        }
    }
}

fn overlap_size(range1: &RangeInclusive<i32>, range2: &RangeInclusive<i32>) -> i32 {
    let start = max(*range1.start(), *range2.start());
    let end = min(*range1.end(), *range2.end());

    if start <= end {
        end - start + 1 // +1 because RangeInclusive includes the end value
    } else {
        0 // No overlap
    }
}

fn range_width(range: &RangeInclusive<i32>) -> i32 {
    range.end() - range.start()
}

fn vertical_placement(
    mut lf_joint_query: Query<&DistanceJoint, With<LeaderFollowerJoint>>,
    mut phys_query: Query<(Entity, &Position, &mut LinearVelocity)>,
) {
    let mut y_values = HashMap::new();
    for (entity, pos, _) in phys_query.iter() {
        y_values.insert(entity, pos.y);
    }

    let neighbourhoods = get_neighbourhoods(lf_joint_query.as_query_lens());
    let neighbourhoods = sort_neighbourhoods(&neighbourhoods, &y_values);
    for parents in neighbourhoods.values() {
        // get the vertical spans of the parents by looking at the children
        let vertical_spans = parents
            .iter()
            .filter_map(|parent| {
                // get the range of y values of the children of the child
                let Some(children) = neighbourhoods.get(parent) else {
                    return None;
                };
                let y_range = children
                    .iter()
                    .map(|child| y_values.get(child).unwrap()) // todo: return Vec<(child, y)> instead of Vec<child> to reduce redundant work
                    .fold(i32::MAX..=i32::MIN, |acc, &y| {
                        let margin = 50;
                        let start: i32 = *acc.start().min(&(y as i32)) - margin;
                        let end: i32 = *acc.end().max(&(y as i32)) + margin;
                        start..=end
                    });
                Some((*parent, y_range, children))
            })
            .collect_vec();

        // the vertical spans should be horizontally aligned with the parent
        for (parent, span, children) in vertical_spans.iter() {
            let span_center = span.start() + (span.end() - span.start()) / 2;
            let diff = span_center as f32 - y_values.get(parent).unwrap();
            if diff.abs() > 2.0 {
                // apply a force to all children to move the span
                for child in children.iter() {
                    if let Ok((_, _, mut vel)) = phys_query.get_mut(*child) {
                        vel.y += -diff as f32 * 0.05;
                    }
                }
            }
        }

        // apply a contractive force between children
        for (_parent, _span, children) in vertical_spans.iter() {
            let desired_dist = 80.0; // todo: adjust based on the size of the child
            for (c1, c2) in children.iter().tuple_windows() {
                let c1y = y_values.get(c1).unwrap();
                let c2y = y_values.get(c2).unwrap();
                let dist = c1y - c2y;
                let error = desired_dist - dist;
                if error.abs() > 5.0 {
                    if let Ok((_, _, mut vel)) = phys_query.get_mut(*c1) {
                        vel.y += error as f32 * 0.05;
                    }
                    if let Ok((_, _, mut vel)) = phys_query.get_mut(*c2) {
                        vel.y -= error as f32 * 0.05;
                    }
                }
            }
        }

        // the vertical spans should not vertically overlap
        for ((above, above_span, above_children), (below, below_span, below_children)) in
            vertical_spans.iter().tuple_windows()
        {
            // check if overlap exists
            let overlap = overlap_size(&above_span, &below_span);
            // add some margin
            // let overlap = overlap + 50;
            if overlap > 0 {
                // force the children apart by moving away the closest children
                let bottom_most_above_child = above_children.last().unwrap();
                if let Ok((_, _, mut vel)) = phys_query.get_mut(*bottom_most_above_child) {
                    vel.y += overlap as f32 * 20.0;
                }
                let top_most_below_child = below_children.first().unwrap();
                if let Ok((_, _, mut vel)) = phys_query.get_mut(*top_most_below_child) {
                    vel.y -= overlap as f32 * 20.0;
                }
                debug!("Found overlap!");
            }

            // distance the parents based on the span sizes
            let above_y = y_values.get(above).unwrap();
            let below_y = y_values.get(below).unwrap();
            let distance = (above_y - below_y).abs();
            // between the parents there should be half of each span
            let expected = range_width(&above_span) / 2 + range_width(&below_span) / 2;
            // plus a bit more
            let expected = expected as f32 + 50.0;
            let error = expected - distance;
            if error > 0.0 {
                // apply a separating force to the parents
                if let Ok((_, _, mut vel)) = phys_query.get_mut(*above) {
                    vel.y += error * 2.0;
                }
                if let Ok((_, _, mut vel)) = phys_query.get_mut(*below) {
                    vel.y -= error * 2.0;
                }
            }
        }
    }
}
