#![feature(trivial_bounds)]

use azure::prelude::ScopeImpl;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy::utils::HashSet;

pub(crate) struct RelationshipPlugin;
impl Plugin for RelationshipPlugin {
    fn build(&self, app: &mut App) {
    }
}

#[derive(Component, Reflect, Debug)]
pub struct AzureRelationship {
    pub parent: ScopeImpl,
    pub child: ScopeImpl,
}
