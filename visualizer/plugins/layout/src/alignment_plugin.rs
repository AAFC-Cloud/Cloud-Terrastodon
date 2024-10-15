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

pub struct AlignmentPlugin;

impl Plugin for AlignmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, left_to_right_flow);
        app.add_systems(Update, vertical_alignment);
        app.add_systems(Update, vertical_separation);
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

fn vertical_separation(
    mut lf_joint_query: Query<&DistanceJoint, With<LeaderFollowerJoint>>,
    mut phys_query: Query<(Entity, &Position, &mut LinearVelocity)>,
) {
    let mut y_values = HashMap::new();
    for (entity, pos, _) in phys_query.iter() {
        y_values.insert(entity, pos.y);
    }

    let neighbourhoods = get_neighbourhoods(lf_joint_query.as_query_lens());
    let neighbourhoods = neighbourhoods
        .into_iter()
        .sorted_by_cached_key(|(key, _)| (y_values.get(key).unwrap_or(&0.0) * 1000.0) as i32)
        .map(|(key, value)| {
            let value = value
                .into_iter()
                .sorted_by_cached_key(|e| (y_values.get(e).unwrap_or(&0.0) * 1000.0) as i32)
                .rev()
                .collect::<IndexSet<Entity>>();
            (key, value)
        })
        .rev()
        .collect::<IndexMap<Entity, IndexSet<Entity>>>();

    for ((_, above_followers), (below_leader, _)) in neighbourhoods.iter().tuple_windows() {
        // the above followers may be above the below leader
        // if so, we want to apply an upwards force to the above followers
        let below_leader_y = y_values.get(below_leader).cloned().unwrap_or_default();
        let last_above_follower_y = above_followers
            .last()
            .and_then(|e| y_values.get(e))
            .cloned()
            .unwrap_or_default();
        let diff = below_leader_y - last_above_follower_y;
        if diff > 0.0 {
            for above_follower in above_followers {
                let Ok((_, _, mut vel)) = phys_query.get_mut(*above_follower) else {
                    continue;
                };
                vel.y += diff;
            }
        }
    }
}
