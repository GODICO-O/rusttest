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
            for y in 0..h {
                // Posisi awal byte untuk baris ke-y (stride dihitung dalam piksel, 1 piksel = 4 byte)
                let row_bytes_offset = y * stride * 4;

                for x in 0..w {
                    let pixel_offset = row_bytes_offset + (x * 4);

                    if pixel_offset + 3 < bytes.len() {
                        bytes[pixel_offset].write(color[0]);     // R
                        bytes[pixel_offset + 1].write(color[1]); // G
                        bytes[pixel_offset + 2].write(color[2]); // B
                        bytes[pixel_offset + 3].write(255);      // A
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
                        window = app.native_window();
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
