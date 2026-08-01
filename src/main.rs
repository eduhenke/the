use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::*;

// Load an animation from an aseprite file
fn spawn_demo_animation(mut cmd : Commands, server : Res<AssetServer>){
    cmd.spawn((
        AseAnimation {
            aseprite: server.load("pixel-crawler/Entities/Characters/New_Version/Idle/Idle_Down.aseprite"),
            animation: Animation::tag("walk-right")
                .with_repeat(AnimationRepeat::Count(1))
                .with_speed(2.)
                // Aseprite provides a repeat config per tag, which is beeing ignored on purpose.
                .with_repeat(AnimationRepeat::Count(42))
                // The direction is provided by the asperite config for the tag, but can be overwritten.
                .with_direction(AnimationDirection::PingPong)
                // you can also chain finite animations, loop animations will never finish
                .with_then("walk-left", AnimationRepeat::Count(4))
                .with_then("walk-up", AnimationRepeat::Loop),
        },
        // The Render target. There are default impls for Sprite, Ui and 3D.
        // You may also define your own. Checkout the examples.
        Sprite {
            flip_x: true,
            ..default()
        },
    ));
}


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_asset::<Aseprite>()
        .add_plugins(AsepriteUltraPlugin)
        .add_systems(Startup, spawn_demo_animation)
        // .add_plugins((PlayerPlugin, EnemyPlugin, WorldPlugin))
        .run();
}