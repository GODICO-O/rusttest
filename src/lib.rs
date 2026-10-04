use bevy::prelude::*;
use bevy_svg::prelude::*;

#[derive(Component)]
struct GdButton {
    base_scale: Vec3,
    target_scale: Vec3,
    is_pressed: bool,
}

#[bevy_main]
pub fn main() {
    #[cfg(target_os = "android")]
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "GD Vector Button Test (Bevy 0.19)".into(),
                resolution: (800, 600).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(SvgPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (button_interaction_system, button_animate_system))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((Camera2d, Msaa::Sample4));

    let svg_handle: Handle<Svg> = asset_server.load("btn_play.svg");
    let base_scale = Vec3::splat(1.2);

    commands.spawn((
        Svg2d(svg_handle),
        Origin::Center,
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(base_scale),
        GdButton {
            base_scale,
            target_scale: base_scale,
            is_pressed: false,
        },
    ));
}

fn button_interaction_system(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut btn_q: Query<(&Transform, &mut GdButton)>,
) {
    let Ok(window) = windows.single() else { return };
    let Ok((camera, camera_transform)) = camera_q.single() else { return };

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
    {
        for (transform, mut btn) in btn_q.iter_mut() {
            let distance = transform.translation.truncate().distance(world_position);
            let is_hovered = distance <= 100.0 * transform.scale.x;

            if is_hovered && buttons.just_pressed(MouseButton::Left) {
                btn.is_pressed = true;
                btn.target_scale = btn.base_scale * 0.85;
                info!(">> [GD Vector] Tombol Ditekan!");
            }

            if buttons.just_released(MouseButton::Left) && btn.is_pressed {
                btn.is_pressed = false;
                btn.target_scale = btn.base_scale * 1.25;
                info!(">> [GD Vector] Action Executed!");
            }
        }
    }
}

fn button_animate_system(time: Res<Time>, mut btn_q: Query<(&mut Transform, &mut GdButton)>) {
    let delta = time.delta_secs();
    for (mut transform, mut btn) in btn_q.iter_mut() {
        if !btn.is_pressed && (btn.target_scale - btn.base_scale).length() > 0.01 {
            btn.target_scale = btn.target_scale.lerp(btn.base_scale, delta * 10.0);
        }
        transform.scale = transform.scale.lerp(btn.target_scale, delta * 25.0);
    }
}
