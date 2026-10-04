use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy_svg::prelude::*;

// Setengah ukuran SVG (200x200) untuk hit-test
const HALF_SIZE: f32 = 100.0;

#[derive(Component)]
struct GdButton {
    base_scale: f32,
    target: f32,
    scale: f32,
    velocity: f32,
    pressed: bool,
}

#[bevy_main]
pub fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.08, 0.08, 0.22)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "GD Vector Button".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(SvgPlugin)
        .add_systems(Startup, (setup, setup_diag))
        .add_systems(Update, (button_touch_system, button_spring_system))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let svg_handle: Handle<Svg> = asset_server.load("btn_play.svg");
    let base = 1.2;

    commands.spawn((
        Svg2d(svg_handle),
        Origin::Center,
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(base)),
        GdButton {
            base_scale: base,
            target: base,
            scale: base,
            velocity: 0.0,
            pressed: false,
        },
    ));
}

fn button_touch_system(
    touches: Res<Touches>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut btn_q: Query<(&Transform, &mut GdButton)>,
) {
    let Ok(_window) = windows.single() else { return };
    let Ok((camera, cam_tf)) = camera_q.single() else { return };

    for touch in touches.iter_just_pressed() {
        let Ok(world) = camera.viewport_to_world_2d(cam_tf, touch.position()) else {
            continue;
        };
        for (tf, mut btn) in btn_q.iter_mut() {
            let d = (world - tf.translation.truncate()).abs();
            let half = HALF_SIZE * btn.base_scale;
            if d.x <= half && d.y <= half {
                btn.pressed = true;
                btn.target = btn.base_scale * 0.85;
                info!("[GD] tombol ditekan");
            }
        }
    }

    let released = touches.iter_just_released().count() + touches.iter_just_canceled().count();
    if released > 0 {
        for (_, mut btn) in btn_q.iter_mut() {
            if btn.pressed {
                btn.pressed = false;
                btn.target = btn.base_scale;
                // dorongan ke atas -> overshoot lalu memantul ala GD
                btn.velocity = 9.0;
                info!("[GD] aksi dijalankan");
            }
        }
    }
}

fn button_spring_system(time: Res<Time>, mut btn_q: Query<(&mut Transform, &mut GdButton)>) {
    let dt = time.delta_secs().min(1.0 / 30.0);
    let stiffness = 420.0;
    let damping = 15.0; // rasio redaman ~0.37 -> memantul

    for (mut tf, mut btn) in btn_q.iter_mut() {
        let accel = stiffness * (btn.target - btn.scale) - damping * btn.velocity;
        btn.velocity += accel * dt;
        btn.scale += btn.velocity * dt;
        tf.scale = Vec3::splat(btn.scale);
    }
}

#[derive(Component)]
struct StatusText;

fn setup_diag(mut commands: Commands) {
    commands.spawn((
        Sprite::from_color(Color::srgb(0.9, 0.2, 0.2), Vec2::splat(60.0)),
        Transform::from_xyz(0.0, -220.0, 1.0),
    ));
    commands.spawn((
        Text2d::new("svg: menunggu..."),
        TextFont {
            font_size: bevy::bevy_text::FontSize::Px(26.0),
            ..default()
        },
        Transform::from_xyz(0.0, 220.0, 1.0),
        StatusText,
    ));
}

fn diag_status(
    asset_server: Res<AssetServer>,
    svgs: Query<&Svg2d>,
    mut text: Query<&mut Text2d, With<StatusText>>,
) {
    let (Some(svg), Ok(mut t)) = (svgs.iter().next(), text.single_mut()) else {
        return;
    };
    let raw = match asset_server.load_state(&svg.0) {
        LoadState::NotLoaded => "svg: not loaded".to_string(),
        LoadState::Loading => "svg: loading".to_string(),
        LoadState::Loaded => "svg: loaded OK".to_string(),
        LoadState::Failed(e) => format!("svg: FAILED {}", e),
    };
    let msg: String = raw
        .chars()
        .take(200)
        .collect::<Vec<_>>()
        .chunks(28)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n");
    if t.0 != msg {
        t.0 = msg;
    }
}
