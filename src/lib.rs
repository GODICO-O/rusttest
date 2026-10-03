use std::time::Duration;

use android_activity::input::{InputEvent, MotionAction};
use android_activity::{AndroidApp, InputStatus, MainEvent, PollEvent};
use log::{info, LevelFilter};
use ndk::hardware_buffer_format::HardwareBufferFormat;
use ndk::native_window::NativeWindow;

const COLORS: [[u8; 3]; 4] = [
    [30, 30, 46],
    [243, 139, 168],
    [166, 227, 161],
    [137, 180, 250],
];

fn draw(win: &NativeWindow, c: [u8; 3]) {
    if let Ok(mut guard) = win.lock(None) {
        let h = guard.height() as usize;
        let stride = guard.stride() as usize;
        let fmt = guard.format();
        if let Some(bytes) = guard.bytes() {
            match fmt {
                HardwareBufferFormat::R5G6B5_UNORM => {
                    let px: u16 = (((c[0] as u16) >> 3) << 11)
                        | (((c[1] as u16) >> 2) << 5)
                        | ((c[2] as u16) >> 3);
                    let lo = (px & 0xff) as u8;
                    let hi = (px >> 8) as u8;
                    for y in 0..h {
                        for x in 0..stride {
                            let i = (y * stride + x) * 2;
                            if i + 1 < bytes.len() {
                                bytes[i].write(lo);
                                bytes[i + 1].write(hi);
                            }
                        }
                    }
                }
                _ => {
                    // RGBA_8888 / RGBX_8888
                    for y in 0..h {
                        for x in 0..stride {
                            let i = (y * stride + x) * 4;
                            if i + 3 < bytes.len() {
                                bytes[i].write(c[0]);
                                bytes[i + 1].write(c[1]);
                                bytes[i + 2].write(c[2]);
                                bytes[i + 3].write(255);
                            }
                        }
                    }
                }
            }
        }
        info!("draw: format={:?} stride={} h={}", fmt, stride, h);
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
                        if let Some(w) = &window {
                            // 0,0 = ukuran asli window; paksa format RGBA 8-bit
                            let _ = w.set_buffers_geometry(
                                0,
                                0,
                                Some(HardwareBufferFormat::R8G8B8A8_UNORM),
                            );
                        }
                        dirty = true;
                    }
                    MainEvent::TerminateWindow { .. } => window = None,
                    MainEvent::WindowResized { .. } | MainEvent::RedrawNeeded { .. } => {
                        dirty = true;
                    }
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
