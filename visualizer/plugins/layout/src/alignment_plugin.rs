use crate::prelude::LeaderFollowerJoint;
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
use std::cmp::max;
use std::cmp::min;
use std::ops::RangeInclusive;

pub struct AlignmentPlugin;

impl Plugin for AlignmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, left_to_right_flow);
        app.add_systems(Update, vertical_alignment);
        app.add_systems(Update, vertical_placement);
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

        // if the leader is to the right of the follower
        if leader_pos.x + 500.0 >= follower_pos.x {
            // add velocity so that the follower goes to the right
            // this will compound each tick
            follower_vel.0 += Vec2::X * 200.0;
            leader_vel.0 += -Vec2::X * 200.0;
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
    for children in neighbourhoods.values() {
        // the vertical span of each child node should not overlap with the vertical span of any other child
        let children_vertical_spans = children
            .iter()
            .filter_map(|child| {
                // get the range of y values of the children of the child
                let Some(child_children) = neighbourhoods.get(child) else {
                    return None;
                };
                let y_range = child_children
                    .iter()
                    .map(|child| y_values.get(child).unwrap())
                    .fold(i32::MAX..=i32::MIN, |acc, &y| {
                        let start: i32 = *acc.start().min(&(y as i32));
                        let end: i32 = *acc.end().max(&(y as i32));
                        start..=end
                    });
                Some((*child, y_range, child_children))
            })
            .collect_vec();
        for (parent, span, children) in children_vertical_spans.iter() {
            // the span of children should be centered on the parent
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
        for ((_, above_span, above_children), (_, below_span, below_children)) in
            children_vertical_spans.iter().tuple_windows()
        {
            // apply force to overlapping regions
            let overlap = overlap_size(&above_span, &below_span);
            if overlap > 0 {
                // a force should be applied such that the children spread apart
                // the bottom-most above-child should receive an upwards force
                // the top-most below-child should receive a downwards force
                let bottom_most_above_child = above_children.last().unwrap();
                let top_most_below_child = below_children.first().unwrap();
                if let Ok((_, _, mut vel)) = phys_query.get_mut(*bottom_most_above_child) {
                    vel.y += overlap as f32 * 20.0;
                }
                if let Ok((_, _, mut vel)) = phys_query.get_mut(*top_most_below_child) {
                    vel.y -= overlap as f32 * 20.0;
                }
                debug!("Found overlap!");
            }
        }
    }
}
