use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use simula_net::{replicate, Proxy, Replicate};

pub struct NetAuthorityPlugin;

#[derive(Reflect, Default, Debug, Component, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Authority {
    pub count: u32,
}

#[derive(Reflect, Default, Debug, Component, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Minion {
    pub count: u32,
}

#[derive(Reflect, Default, Debug, Component, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Worker {
    pub count: u32,
}

impl Plugin for NetAuthorityPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Authority>()
            .register_type::<Minion>()
            .register_type::<Worker>()
            .register_type::<Replicate<Authority>>()
            .register_type::<Replicate<Minion>>()
            .register_type::<Replicate<Worker>>()
            .add_startup_system(setup)
            .add_system(run_authorities)
            .add_system(run_minions)
            .add_system(run_workers)
            .add_system(replicate::<Authority>)
            .add_system(replicate::<Minion>)
            .add_system(replicate::<Worker>)
            .add_system(setup_minions_replication)
            .add_system(reset_minions_replication);
    }
}

fn setup(mut _commands: Commands) {}

fn run_authorities(mut authorities: Query<&mut Authority, With<Replicate<Authority>>>) {
    for mut authority in authorities.iter_mut() {
        authority.count += 1;
    }
}

fn run_minions(mut minions: Query<&mut Minion, With<Replicate<Minion>>>) {
    for mut minion in minions.iter_mut() {
        minion.count += 1;
    }
}

fn run_workers(mut workers: Query<&mut Worker, With<Replicate<Worker>>>) {
    for mut worker in workers.iter_mut() {
        worker.count += 1;
    }
}

fn setup_minions_replication(
    mut commands: Commands,
    authorities: Query<&Proxy, With<Authority>>,
    minions: Query<Entity, (With<Minion>, Without<Replicate<Minion>>, Without<Proxy>)>,
) {
    // once there is an authority, replicate minions to it
    // if there are more than one authorities, we don't know which one to replicate to
    // lets wait for network to figure it out
    if authorities.iter().count() == 1 {
        let authority_proxy = authorities.iter().next().unwrap();
        for minion_entity in minions.iter() {
            commands.entity(minion_entity).insert(Replicate::<Minion> {
                target: Some(authority_proxy.sender.clone()),
                ..default()
            });
        }
    }
}

fn reset_minions_replication(
    mut commands: Commands,
    authorities: Query<&Proxy, With<Authority>>,
    minions: Query<Entity, (With<Minion>, With<Replicate<Minion>>)>,
) {
    // if authorities are gone or changed, remove replication from minions
    // so it can be setup again
    if authorities.iter().count() != 1 {
        for minion_entity in minions.iter() {
            commands.entity(minion_entity).remove::<Replicate<Minion>>();
        }
    }
}
