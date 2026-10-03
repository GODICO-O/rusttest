use std::time::Duration;

use android_activity::input::{InputEvent, MotionAction};
use android_activity::{AndroidApp, InputStatus, MainEvent, PollEvent};
use log::{info, LevelFilter};
use ndk::native_window::NativeWindow;

const COLORS: [[u8; 3]; 4] = [
    [30, 30, 46],
    [243, 139, 168],
    [166, 227, 161],
    [137, 180, 250],
];

fn draw(win: &NativeWindow, color: [u8; 3]) {
    if let Ok(mut guard) = win.lock(None) {
        let w = guard.width() as usize;
        let h = guard.height() as usize;
        let stride = guard.stride() as usize;
        
        if let Some(bytes) = guard.bytes() {
            // Loop penuh melewati stride agar seluruh buffer baris terisi bersih
            for y in 0..h {
                let row_start = y * stride * 4;
                for x in 0..w {
                    let i = row_start + (x * 4);
                    if i + 3 < bytes.len() {
                        bytes[i] = color[0];     // Red
                        bytes[i + 1] = color[1]; // Green
                        bytes[i + 2] = color[2]; // Blue
                        bytes[i + 3] = 255;      // Alpha
                    }
                }
            }
        }
    }
}

#[no_mangle]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(LevelFilter::Info)
            .with_tag("RustTest"),
    );
    info!("Halo dari Rust via GameActivity!");

    let mut running = true;
    let mut window: Option<NativeWindow> = None;
    let mut color_idx = 0usize;
    let mut dirty = true;

    while running {
        app.poll_events(Some(Duration::from_millis(16)), |event| {
            if let PollEvent::Main(main_event) = event {
                match main_event {
                    MainEvent::InitWindow { .. } => {
                        if let Some(win) = app.native_window() {
                            // Paksa set format buffer ke RGBA_8888 (4 byte/pixel)
                            let _ = win.set_buffers_geometry(0, 0, Some(ndk::hardware_buffer::HardwareBufferFormat::R8G8B8A8_UNORM));
                            window = Some(win);
                        }
                        dirty = true;
                    }
                    MainEvent::TerminateWindow { .. } => window = None,
                    MainEvent::RedrawNeeded { .. } => dirty = true,
                    MainEvent::Destroy => running = false,
                    _ => {}
                }
            }
        });

        if let Ok(mut it) = app.input_events_iter() {
            while it.next(|ev| {
                if let InputEvent::MotionEvent(m) = ev {
                    if m.action() == MotionAction::Down {
                        color_idx = (color_idx + 1) % COLORS.len();
                        dirty = true;
                        info!("Tap! warna ke-{}", color_idx);
                    }
                }
                InputStatus::Handled
            }) {}
        }

        if dirty {
            if let Some(w) = &window {
                draw(w, COLORS[color_idx]);
                dirty = false;
            }
        }
    }
}
