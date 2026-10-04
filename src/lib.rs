use std::time::{Duration, Instant};

use android_activity::input::{InputEvent, MotionAction};
use android_activity::{AndroidApp, InputStatus, MainEvent, PollEvent};
use ndk::hardware_buffer_format::HardwareBufferFormat;
use ndk::native_window::NativeWindow;

// Simple 8x8 ASCII Bitmap Font (0-9, A-Z, spasi, simbol dasar)
fn get_char_bytes(c: char) -> &'static [u8; 8] {
    match c {
        '0' => &[0x3E, 0x51, 0x49, 0x45, 0x3E, 0x00, 0x00, 0x00],
        '1' => &[0x00, 0x42, 0x7F, 0x40, 0x00, 0x00, 0x00, 0x00],
        '2' => &[0x42, 0x61, 0x51, 0x49, 0x46, 0x00, 0x00, 0x00],
        '3' => &[0x21, 0x41, 0x45, 0x4B, 0x31, 0x00, 0x00, 0x00],
        '4' => &[0x18, 0x14, 0x12, 0x7F, 0x10, 0x00, 0x00, 0x00],
        '5' => &[0x27, 0x45, 0x45, 0x45, 0x39, 0x00, 0x00, 0x00],
        '6' => &[0x3C, 0x4A, 0x49, 0x49, 0x30, 0x00, 0x00, 0x00],
        '7' => &[0x01, 0x71, 0x09, 0x05, 0x03, 0x00, 0x00, 0x00],
        '8' => &[0x36, 0x49, 0x49, 0x49, 0x36, 0x00, 0x00, 0x00],
        '9' => &[0x06, 0x49, 0x49, 0x29, 0x1E, 0x00, 0x00, 0x00],
        'A' => &[0x7C, 0x12, 0x11, 0x12, 0x7C, 0x00, 0x00, 0x00],
        'B' => &[0x7F, 0x49, 0x49, 0x49, 0x36, 0x00, 0x00, 0x00],
        'C' => &[0x3E, 0x41, 0x41, 0x41, 0x22, 0x00, 0x00, 0x00],
        'D' => &[0x7F, 0x41, 0x41, 0x22, 0x1C, 0x00, 0x00, 0x00],
        'E' => &[0x7F, 0x49, 0x49, 0x49, 0x41, 0x00, 0x00, 0x00],
        'F' => &[0x7F, 0x09, 0x09, 0x09, 0x01, 0x00, 0x00, 0x00],
        'G' => &[0x3E, 0x41, 0x49, 0x49, 0x7A, 0x00, 0x00, 0x00],
        'H' => &[0x7F, 0x08, 0x08, 0x08, 0x7F, 0x00, 0x00, 0x00],
        'I' => &[0x00, 0x41, 0x7F, 0x41, 0x00, 0x00, 0x00, 0x00],
        'J' => &[0x20, 0x40, 0x41, 0x3F, 0x01, 0x00, 0x00, 0x00],
        'K' => &[0x7F, 0x08, 0x14, 0x22, 0x41, 0x00, 0x00, 0x00],
        'L' => &[0x7F, 0x40, 0x40, 0x40, 0x40, 0x00, 0x00, 0x00],
        'M' => &[0x7F, 0x02, 0x0C, 0x02, 0x7F, 0x00, 0x00, 0x00],
        'N' => &[0x7F, 0x04, 0x08, 0x10, 0x7F, 0x00, 0x00, 0x00],
        'O' => &[0x3E, 0x41, 0x41, 0x41, 0x3E, 0x00, 0x00, 0x00],
        'P' => &[0x7F, 0x09, 0x09, 0x09, 0x06, 0x00, 0x00, 0x00],
        'Q' => &[0x3E, 0x41, 0x51, 0x21, 0x5E, 0x00, 0x00, 0x00],
        'R' => &[0x7F, 0x09, 0x19, 0x29, 0x46, 0x00, 0x00, 0x00],
        'S' => &[0x26, 0x49, 0x49, 0x49, 0x32, 0x00, 0x00, 0x00],
        'T' => &[0x01, 0x01, 0x7F, 0x01, 0x01, 0x00, 0x00, 0x00],
        'U' => &[0x3F, 0x40, 0x40, 0x40, 0x3F, 0x00, 0x00, 0x00],
        'V' => &[0x1F, 0x20, 0x40, 0x20, 0x1F, 0x00, 0x00, 0x00],
        'W' => &[0x3F, 0x40, 0x38, 0x40, 0x3F, 0x00, 0x00, 0x00],
        'X' => &[0x63, 0x14, 0x08, 0x14, 0x63, 0x00, 0x00, 0x00],
        'Y' => &[0x07, 0x08, 0x70, 0x08, 0x07, 0x00, 0x00, 0x00],
        'Z' => &[0x61, 0x51, 0x49, 0x45, 0x43, 0x00, 0x00, 0x00],
        ':' => &[0x00, 0x36, 0x36, 0x00, 0x00, 0x00, 0x00, 0x00],
        '.' => &[0x00, 0x60, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00],
        '(' => &[0x00, 0x1C, 0x22, 0x41, 0x00, 0x00, 0x00, 0x00],
        ')' => &[0x00, 0x41, 0x22, 0x1C, 0x00, 0x00, 0x00, 0x00],
        '/' => &[0x20, 0x10, 0x08, 0x04, 0x02, 0x00, 0x00, 0x00],
        '>' => &[0x41, 0x22, 0x14, 0x08, 0x00, 0x00, 0x00, 0x00],
        '<' => &[0x08, 0x14, 0x22, 0x41, 0x00, 0x00, 0x00, 0x00],
        '=' => &[0x14, 0x14, 0x14, 0x14, 0x14, 0x00, 0x00, 0x00],
        _ => &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], // Space or unknown
    }
}

fn draw_pixel(bytes: &mut [u8], stride: usize, width: usize, height: usize, x: usize, y: usize, color: [u8; 3]) {
    if x < width && y < height {
        let idx = (y * stride + x) * 4;
        if idx + 3 < bytes.len() {
            bytes[idx] = color[0];
            bytes[idx + 1] = color[1];
            bytes[idx + 2] = color[2];
            bytes[idx + 3] = 255;
        }
    }
}

fn draw_rect(bytes: &mut [u8], stride: usize, width: usize, height: usize, x: usize, y: usize, w: usize, h: usize, color: [u8; 3]) {
    for py in y..(y + h) {
        for px in x..(x + w) {
            draw_pixel(bytes, stride, width, height, px, py, color);
        }
    }
}

fn draw_text(bytes: &mut [u8], stride: usize, width: usize, height: usize, text: &str, x: usize, y: usize, scale: usize, color: [u8; 3]) {
    let mut cur_x = x;
    for c in text.chars() {
        let bitmap = get_char_bytes(c.to_ascii_uppercase());
        for col in 0..8 {
            let row_byte = bitmap[col];
            for row in 0..8 {
                if (row_byte & (1 << row)) != 0 {
                    for sy in 0..scale {
                        for sx in 0..scale {
                            draw_pixel(
                                bytes,
                                stride,
                                width,
                                height,
                                cur_x + col * scale + sx,
                                y + row * scale + sy,
                                color,
                            );
                        }
                    }
                }
            }
        }
        cur_x += 7 * scale;
    }
}

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    let mut running = true;
    let mut window: Option<NativeWindow> = None;

    // State Pengujian
    let mut duration_limit: u64 = 30; // Default 30 Detik
    let mut total_clicks: u64 = 0;
    let mut clicks_this_second: u32 = 0;
    let mut current_cps: u32 = 0;
    let mut test_started = false;
    let mut test_finished = false;

    let mut start_time = Instant::now();
    let mut last_second = Instant::now();

    while running {
        app.poll_events(Some(Duration::from_millis(16)), |event| {
            if let PollEvent::Main(main_event) = event {
                match main_event {
                    MainEvent::InitWindow { .. } => {
                        window = app.native_window();
                        if let Some(w) = &window {
                            let _ = w.set_buffers_geometry(
                                0,
                                0,
                                Some(HardwareBufferFormat::R8G8B8A8_UNORM),
                            );
                        }
                    }
                    MainEvent::TerminateWindow { .. } => window = None,
                    MainEvent::Destroy => running = false,
                    _ => {}
                }
            }
        });

        // Waktu & Hitung Mundur
        let mut time_left = duration_limit;
        if test_started && !test_finished {
            let elapsed = start_time.elapsed().as_secs();

            if elapsed >= duration_limit {
                time_left = 0;
                test_finished = true;
            } else {
                time_left = duration_limit - elapsed;
            }

            // Update CPS per detik
            if last_second.elapsed() >= Duration::from_secs(1) {
                current_cps = clicks_this_second;
                clicks_this_second = 0;
                last_second = Instant::now();
            }
        }

        // Tangkap Touch / Klik
        if let Ok(mut it) = app.input_events_iter() {
            while it.next(|ev| {
                if let InputEvent::MotionEvent(m) = ev {
                    if m.action() == MotionAction::Down {
                        let touch_x = m.pointer_at_index(0).x();
                        let touch_y = m.pointer_at_index(0).y();

                        if let Some(w) = &window {
                            let win_h = w.height() as f32;
                            let win_w = w.width() as f32;

                            // Tombol RESET (Area Bawah Layar)
                            if touch_y > win_h - 200.0 {
                                total_clicks = 0;
                                clicks_this_second = 0;
                                current_cps = 0;
                                test_started = false;
                                test_finished = false;
                                return InputStatus::Handled;
                            }

                            // Tombol Mode (30s / 60s) saat belum mulai
                            if !test_started && touch_y < 250.0 {
                                if touch_x < win_w / 2.0 {
                                    duration_limit = 30;
                                } else {
                                    duration_limit = 60;
                                }
                                return InputStatus::Handled;
                            }
                        }

                        // Jika tes belum selesai, hitung klik
                        if !test_finished {
                            if !test_started {
                                test_started = true;
                                start_time = Instant::now();
                                last_second = Instant::now();
                            }

                            total_clicks += 1;
                            clicks_this_second += 1;
                        }
                    }
                }
                InputStatus::Handled
            }) {}
        }

        // Render Layar
        if let Some(w) = &window {
            if let Ok(mut guard) = w.lock(None) {
                let win_w = guard.width() as usize;
                let win_h = guard.height() as usize;
                let stride = guard.stride() as usize;

                if let Some(bytes) = guard.bytes() {
                    // Background Utama (Gelap / Catppuccin Base)
                    let bg_color = if test_finished { [40, 20, 30] } else { [30, 30, 46] };
                    for y in 0..win_h {
                        for x in 0..stride {
                            draw_pixel(bytes, stride, win_w, win_h, x, y, bg_color);
                        }
                    }

                    // 1. Header Mode Selector (Top)
                    let color_30s = if duration_limit == 30 { [166, 227, 161] } else { [100, 100, 120] };
                    let color_60s = if duration_limit == 60 { [166, 227, 161] } else { [100, 100, 120] };
                    draw_text(bytes, stride, win_w, win_h, "MODE: < 30S >", 40, 60, 3, color_30s);
                    draw_text(bytes, stride, win_w, win_h, "< 60S >", win_w - 280, 60, 3, color_60s);

                    // 2. Timer Count Down (Besar di Tengah Atas)
                    let timer_str = format!("TIME: {}S", time_left);
                    let timer_color = if time_left <= 5 && test_started { [243, 139, 168] } else { [250, 179, 135] };
                    draw_text(bytes, stride, win_w, win_h, &timer_str, 40, 180, 5, timer_color);

                    // 3. Stats Display
                    let clicks_str = format!("CLICKS : {}", total_clicks);
                    draw_text(bytes, stride, win_w, win_h, &clicks_str, 40, 340, 4, [205, 214, 244]);

                    let cps_str = format!("LIVE CPS: {}", current_cps);
                    draw_text(bytes, stride, win_w, win_h, &cps_str, 40, 430, 4, [137, 180, 250]);

                    if test_finished {
                        let avg_cps = total_clicks as f64 / duration_limit as f64;
                        let avg_str = format!("AVG CPS : {:.1}", avg_cps);
                        draw_text(bytes, stride, win_w, win_h, "TEST SELESAI!", 40, 540, 4, [166, 227, 161]);
                        draw_text(bytes, stride, win_w, win_h, &avg_str, 40, 610, 4, [249, 226, 175]);
                    } else if !test_started {
                        draw_text(bytes, stride, win_w, win_h, "TAP / KLIK UNTUK MULAI", 40, 550, 3, [147, 153, 178]);
                    }

                    // 4. Tombol RESET (Bawah)
                    draw_rect(bytes, stride, win_w, win_h, 0, win_h - 180, win_w, 180, [243, 139, 168]);
                    draw_text(bytes, stride, win_w, win_h, "RESET", win_w / 2 - 80, win_h - 120, 5, [17, 17, 27]);
                }
            }
        }
    }
}
