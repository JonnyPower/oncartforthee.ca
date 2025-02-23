use bevy::asset::ErasedAssetLoader;
use bevy::math::Vec3;
use bevy::prelude::{
    info, AssetServer, BuildChildren, ChildBuild, Commands, Entity, Quat, Res, Transform, Vec2,
    Vec3Swizzles,
};
use bevy::scene::SceneRoot;
use bevy_rapier3d::prelude::Collider;
use std::f32::consts::PI;

const WALL_SEGMENT_WIDTH: f32 = 2.5;
const WALL_SEGMENT_HEIGHT: f32 = 3.01;
const WALL_SEGMENT_DEPTH: f32 = 0.225;

pub fn spawn_walls(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    start: Vec3,
    end: Vec3,
) -> Result<Vec<Entity>, String> {
    if start.y != end.y {
        Err(format!(
            "Wall start & end must be same y, got {} vs {}",
            start.y, end.y
        ))
    } else {
        let start_xz = start.xz();
        let end_xz = end.xz();
        let length = (end_xz - start_xz).abs();
        let direction = (end_xz - start_xz).normalize();
        let horizontal = Vec3::new(direction.x, 0.0, direction.y).normalize();
        let angle = horizontal.x.atan2(horizontal.z);
        let rotation = Quat::from_rotation_y(angle + PI / 2.);
        let num_segments = (length.length() / WALL_SEGMENT_WIDTH).ceil() as i32;
        info!("length; {}", length);
        info!("segments; {}", num_segments);
        let mut entities = vec![];
        commands
            .spawn(Transform::from_translation(start))
            .with_children(|parent| {
                for i in 1..=num_segments {
                    let pos = direction * i as f32 * WALL_SEGMENT_WIDTH;
                    info!("segment {}; pos: {}", i, pos);
                    info!("global pos: {}", pos + start_xz);
                    entities.push(
                        parent
                            .spawn((
                                SceneRoot(
                                    asset_server.load("models/SM_Bld_Base_Wall_01.glb#Scene0"),
                                ),
                                Transform::from_xyz(pos.x, start.y, pos.y).with_rotation(rotation),
                            ))
                            .id(),
                    );
                }
                let mid_point = direction * length / 2.0;
                parent.spawn((
                    get_cuboid(length, num_segments),
                    Transform::from_xyz(
                        mid_point.x,
                        start.y + WALL_SEGMENT_HEIGHT / 2.0,
                        mid_point.y,
                    ),
                ));
            });
        Ok(entities)
    }
}

fn get_cuboid(length: Vec2, num_segments: i32) -> Collider {
    let wall_width = num_segments as f32 * WALL_SEGMENT_WIDTH;
    if length.x > length.y {
        Collider::cuboid(
            wall_width / 2.0,
            WALL_SEGMENT_HEIGHT / 2.0,
            length.y / 2.0 + (WALL_SEGMENT_DEPTH / 2.0),
        )
    } else {
        Collider::cuboid(
            length.x / 2.0 + (WALL_SEGMENT_DEPTH / 2.0),
            WALL_SEGMENT_HEIGHT / 2.0,
            wall_width / 2.0,
        )
    }
}
