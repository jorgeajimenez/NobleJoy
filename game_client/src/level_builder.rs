use crate::components::{EntityRole, GameSessionState, StudioEntity};
use crate::scene_parser::{HotfixDelta, LevelContract};
use bevy::prelude::*;

pub fn spawn_level_from_contract(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    existing_entities: &Query<Entity, With<StudioEntity>>,
    session: &mut ResMut<GameSessionState>,
    contract: &LevelContract,
) {
    for entity in existing_entities.iter() {
        commands.entity(entity).despawn_recursive();
    }

    session.active_project_id = contract.project_id.clone();
    session.theme = contract.theme.clone();
    session.victory = false;
    session.status_banner = format!(
        "Loaded [{}] Theme: {} | Press [Q] for Visual QA Audit | [WASD] Move",
        contract.project_id, contract.theme
    );

    for spec in &contract.entities {
        let (role, mesh_handle, material_handle, half_extents) = match spec.asset_key.as_str() {
            "player" => (
                EntityRole::Player,
                meshes.add(Capsule3d::new(0.45 * spec.scale, 0.9 * spec.scale)),
                materials.add(StandardMaterial {
                    base_color: Color::srgb(0.15, 0.85, 1.0),
                    emissive: LinearRgba::rgb(0.05, 0.4, 0.6),
                    ..default()
                }),
                Vec3::splat(0.5 * spec.scale),
            ),
            "goal" => (
                EntityRole::Goal,
                meshes.add(Torus::new(0.35 * spec.scale, 0.75 * spec.scale)),
                materials.add(StandardMaterial {
                    base_color: Color::srgb(0.1, 1.0, 0.45),
                    emissive: LinearRgba::rgb(0.1, 1.2, 0.35),
                    ..default()
                }),
                Vec3::splat(0.8 * spec.scale),
            ),
            _ => (
                EntityRole::Obstacle,
                meshes.add(Cuboid::new(
                    1.2 * spec.scale,
                    1.2 * spec.scale,
                    1.2 * spec.scale,
                )),
                materials.add(StandardMaterial {
                    base_color: Color::srgb(0.92, 0.24, 0.24),
                    metallic: 0.5,
                    perceptual_roughness: 0.4,
                    ..default()
                }),
                Vec3::splat(0.6 * spec.scale),
            ),
        };

        commands.spawn((
            PbrBundle {
                mesh: mesh_handle,
                material: material_handle,
                transform: Transform::from_xyz(spec.x, spec.y, spec.z),
                ..default()
            },
            StudioEntity {
                id: spec.id.clone(),
                role,
                solid: spec.solid,
                half_extents,
            },
        ));
    }
}

pub fn apply_hotfix_deltas(
    query: &mut Query<(&StudioEntity, &mut Transform)>,
    hotfixes: &[HotfixDelta],
) {
    for delta in hotfixes {
        if delta.action == "reposition" {
            for (studio_ent, mut transform) in query.iter_mut() {
                if studio_ent.id == delta.id {
                    transform.translation = Vec3::new(delta.x, delta.y, delta.z);
                }
            }
        }
    }
}
