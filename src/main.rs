use std::collections::HashMap;

use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::*;
use bevy_ecs_tilemap::prelude::*;

#[derive(Component)]
struct Player;

#[derive(Component, PartialEq, Eq, Clone, Copy, Default, Hash, strum_macros::Display)]
enum Facing {
    Up,
    #[default]
    Down,
    Right,
    Left,
}

#[derive(Component, PartialEq, Eq, Clone, Copy, Default, Hash, strum_macros::Display)]
enum AnimationState {
    #[default]
    Idle,
    Walk,
    #[allow(dead_code)]
    Pierce,
}

// #[derive(Component)]
struct AnimationClip {
    anim: AseAnimation,
    flip_x: bool,
}

#[derive(Component)]
struct AnimationMap(HashMap<(Facing, AnimationState), AnimationClip>);

const BASE: &str = "pixel-crawler/Entities/Characters/Body_A/Animations";
const SPEED: f32 = 300.0; // pixels per second

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let texture_handle: Handle<Image> = asset_server.load("pixel-crawler/Environment/Tilesets//Dungeon_Tiles.png");

    let map_size = TilemapSize { x: 320, y: 320 };
    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();

    for x in 0..320u32 {
        for y in 0..320u32 {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn((
                    TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(tilemap_entity),
                        texture_index: TileTextureIndex(30),
                        ..Default::default()
                    },
                    // PostUpdate::default(),
                ))
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        anchor: TilemapAnchor::Center,
        transform: Transform::from_scale(Vec3::splat(2.0)),
        ..Default::default()
    });


    commands.spawn((
        Player,
        Transform::from_xyz(0.0, 0.0, 1.0),
        Facing::Down,
        AnimationState::Idle,
        AnimationMap({
            let mut map = HashMap::new();
            for facing in [Facing::Up, Facing::Down, Facing::Right, Facing::Left] {
                for state in [
                    AnimationState::Idle,
                    AnimationState::Walk,
                    AnimationState::Pierce,
                ] {
                    let sprite_name = match facing {
                        Facing::Left | Facing::Right => "Side",
                        Facing::Down => "Down",
                        Facing::Up => "Up",
                    };
                    let animation = match state {
                        AnimationState::Pierce => Animation::default()
                            .with_repeat(AnimationRepeat::Count(0))
                            .with_speed(2.0),
                        AnimationState::Idle | AnimationState::Walk => Animation::default(),
                    };
                    map.insert(
                        (facing, state),
                        AnimationClip {
                            anim: AseAnimation {
                                aseprite: asset_server.load(format!(
                                    "{BASE}/{state}_Base/{state}_{sprite_name}.aseprite"
                                )),
                                animation,
                            },
                            flip_x: facing == Facing::Left,
                        },
                    );
                }
            }
            map
        }),
        AseAnimation {
            aseprite: asset_server.load(format!("{BASE}/Idle_Base/Idle_Down.aseprite")),
            animation: Animation::default(),
        },
        Sprite {
            custom_size: Some(Vec2::splat(128.0)),
            ..default()
        },
    ));
}

fn control_player(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Facing, &mut AnimationState, &mut Transform), With<Player>>,
) {
    for (mut facing, mut anim_state, mut transform) in &mut query {
        if *anim_state == AnimationState::Pierce {
            continue;
        }

        let mut dir = Vec2::ZERO;

        if keyboard.pressed(KeyCode::KeyW) {
            dir.y += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            dir.y -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            dir.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            dir.x += 1.0;
        }

        let desired = if keyboard.pressed(KeyCode::KeyW) {
            Some(Facing::Up)
        } else if keyboard.pressed(KeyCode::KeyS) {
            Some(Facing::Down)
        } else if keyboard.pressed(KeyCode::KeyA) {
            Some(Facing::Left)
        } else if keyboard.pressed(KeyCode::KeyD) {
            Some(Facing::Right)
        } else {
            None
        };

        // 2. move — normalize so diagonals aren't faster, scale by delta time
        if dir != Vec2::ZERO {
            let step = dir.normalize() * SPEED * time.delta_secs();
            transform.translation += step.extend(0.0);
        }

        if let Some(desired) = desired {
            *facing = desired;
            *anim_state = AnimationState::Walk;
        } else if keyboard.just_pressed(KeyCode::KeyX) {
            *anim_state = AnimationState::Pierce;
        } else {
            *anim_state = AnimationState::Idle;
        }
    }
}

fn end_animation(
    mut events: MessageReader<AnimationEvents>,
    mut query: Query<&mut AnimationState>,
) {
    for event in events.read() {
        let AnimationEvents::Finished(entity) = event else {
            continue;
        };
        let Ok(mut anim_state) = query.get_mut(*entity) else {
            continue;
        };
        *anim_state = AnimationState::Idle;
    }
}

fn control_animation(
    mut query: Query<
        (
            &Facing,
            &AnimationState,
            &mut AseAnimation,
            &mut Sprite,
            &AnimationMap,
        ),
        Changed<AnimationState>,
    >,
) {
    for (facing, anim_state, mut ase, mut sprite, map) in &mut query {
        let Some(clip) = map.0.get(&(facing.clone(), anim_state.clone())) else {
            continue;
        };

        *ase = clip.anim.clone();
        sprite.flip_x = clip.flip_x;
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(AsepriteUltraPlugin)
        .add_plugins(TilemapPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (end_animation, control_player, control_animation).chain(),
        )
        .run();
}
