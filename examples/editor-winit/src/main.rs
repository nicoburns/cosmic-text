// SPDX-License-Identifier: MIT OR Apache-2.0

use cosmic_text::{
    Action, Attrs, BorrowedWithFontSystem, Buffer, CacheKeyFlags, Color, Edit, Family, FontSystem,
    LineHeight, Motion, Scroll, Shaping, Style, SwashCache, SyntaxEditor, SyntaxSystem, Weight,
};
use std::{env, num::NonZeroU32, rc::Rc, slice};
use tiny_skia::{Paint, PixmapMut, Rect, Transform};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, Event, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::WindowBuilder,
};

fn main() {
    env_logger::init();

    let path = env::args().nth(1).unwrap_or(String::new());

    let event_loop = EventLoop::new().unwrap();
    let window = Rc::new(WindowBuilder::new().build(&event_loop).unwrap());
    let context = softbuffer::Context::new(window.clone()).unwrap();
    let mut surface = softbuffer::Surface::new(&context, window.clone()).unwrap();
    let mut font_system = FontSystem::new();
    let syntax_system = SyntaxSystem::new();
    let mut swash_cache = SwashCache::new();

    let font_sizes = [
        10.0, // Metrics::new(10.0, 14.0).scale(display_scale), // Caption
        14.0, // Metrics::new(14.0, 20.0).scale(display_scale), // Body
        20.0, // Metrics::new(20.0, 28.0).scale(display_scale), // Title 4
        24.0, // Metrics::new(24.0, 32.0).scale(display_scale), // Title 3
        28.0, // Metrics::new(28.0, 36.0).scale(display_scale), // Title 2
        32.0, // Metrics::new(32.0, 44.0).scale(display_scale), // Title 1
    ];
    let font_size_default = 1; // Body
    let mut font_size_i = font_size_default;

    let line_x = 8.0 * (window.scale_factor() as f32);

    let mut editor = SyntaxEditor::new(
        Buffer::new(&mut font_system),
        &syntax_system,
        "base16-eighties.dark",
    )
    .unwrap();
    let mut editor = editor.borrow_with(&mut font_system);

    let attrs = Attrs::new()
        .size(font_sizes[font_size_i] * (window.scale_factor() as f32))
        .line_height(LineHeight::Proportional(1.2))
        .family(Family::Monospace);

    match editor.load_text(&path, attrs) {
        Ok(()) => (),
        Err(err) => {
            log::error!("failed to load {:?}: {}", path, err);
        }
    }

    let mut ctrl_pressed = false;
    let mut mouse_x = 0.0;
    let mut mouse_y = 0.0;
    let mut mouse_left = ElementState::Released;

    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Wait);

            match event {
                Event::WindowEvent {
                    window_id,
                    event: WindowEvent::ScaleFactorChanged { scale_factor, .. },
                } => {
                    // set_text(&mut buffer, scale_factor as f32);
                    // TODO: Update scale factor for editor
                    log::info!("Updated scale factor for {window_id:?}");
                    window.request_redraw();
                }
                Event::WindowEvent {
                    window_id: _,
                    event: WindowEvent::RedrawRequested,
                } => {
                    let (width, height) = {
                        let size = window.inner_size();
                        (size.width, size.height)
                    };

                    dbg!(width, height);

                    surface
                        .resize(
                            NonZeroU32::new(width).unwrap(),
                            NonZeroU32::new(height).unwrap(),
                        )
                        .unwrap();

                    let mut surface_buffer = surface.buffer_mut().unwrap();
                    let surface_buffer_u8 = unsafe {
                        slice::from_raw_parts_mut(
                            surface_buffer.as_mut_ptr() as *mut u8,
                            surface_buffer.len() * 4,
                        )
                    };
                    let mut pixmap =
                        PixmapMut::from_bytes(surface_buffer_u8, width, height).unwrap();
                    pixmap.fill(tiny_skia::Color::from_rgba8(0, 0, 0, 0xFF));

                    editor.with_buffer_mut(|buffer| {
                        buffer.set_size(width as f32 - line_x * 2.0, height as f32)
                    });

                    // // Set scroll to view scroll
                    // buffer.set_scroll(scroll);
                    // // Set size, will relayout and shape until scroll if changed
                    // buffer.set_size(width as f32, height as f32);
                    // // Shape until scroll, ensures scroll is clamped
                    // //TODO: ability to prune with multiple views?
                    // buffer.shape_until_scroll(true);
                    // // Update scroll after buffer clamps it
                    // scroll = buffer.scroll();

                    let mut paint = Paint::default();
                    paint.anti_alias = false;
                    editor.draw(&mut swash_cache, |x, y, w, h, color| {
                        // Note: due to softbuffer and tiny_skia having incompatible internal color representations we swap
                        // the red and blue channels here
                        paint.set_color_rgba8(color.b(), color.g(), color.r(), color.a());
                        pixmap.fill_rect(
                            Rect::from_xywh(x as f32, y as f32, w as f32, h as f32).unwrap(),
                            &paint,
                            Transform::identity(),
                            None,
                        );
                    });

                    // TODO: Draw scrollbar

                    surface_buffer.present().unwrap();
                }
                Event::WindowEvent {
                    event: WindowEvent::ModifiersChanged(modifiers),
                    ..
                } => ctrl_pressed = modifiers.state().control_key(),
                Event::WindowEvent {
                    event:
                        WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    logical_key,
                                    text,
                                    state,
                                    ..
                                },
                            ..
                        },
                    ..
                } => {
                    println!("{:?} {:?} {:?}", &logical_key, text, state);

                    if state == ElementState::Pressed {
                        match logical_key {
                            Key::Named(NamedKey::ArrowLeft) => {
                                editor.action(Action::Motion(Motion::Left))
                            }
                            Key::Named(NamedKey::ArrowRight) => {
                                editor.action(Action::Motion(Motion::Right))
                            }
                            Key::Named(NamedKey::ArrowUp) => {
                                editor.action(Action::Motion(Motion::Up))
                            }
                            Key::Named(NamedKey::ArrowDown) => {
                                editor.action(Action::Motion(Motion::Down))
                            }
                            Key::Named(NamedKey::Home) => {
                                editor.action(Action::Motion(Motion::Home))
                            }
                            Key::Named(NamedKey::End) => editor.action(Action::Motion(Motion::End)),
                            Key::Named(NamedKey::PageUp) => {
                                editor.action(Action::Motion(Motion::PageUp))
                            }
                            Key::Named(NamedKey::PageDown) => {
                                editor.action(Action::Motion(Motion::PageDown))
                            }
                            Key::Named(NamedKey::Escape) => editor.action(Action::Escape),
                            Key::Named(NamedKey::Enter) => editor.action(Action::Enter),
                            Key::Named(NamedKey::Backspace) => editor.action(Action::Backspace),
                            Key::Named(NamedKey::Delete) => editor.action(Action::Delete),
                            Key::Character(c) => {
                                if ctrl_pressed {
                                    match &*c {
                                        "0" => {
                                            font_size_i = font_size_default;
                                            //editor
                                            //    .with_buffer_mut(|buffer| buffer.set_metrics(font_sizes[font_size_i]));
                                        }
                                        "-" => {
                                            if font_size_i > 0 {
                                                font_size_i -= 1;
                                                // editor.with_buffer_mut(|buffer| {
                                                //     buffer.set_metrics(font_sizes[font_size_i])
                                                // });
                                            }
                                        }
                                        "=" => {
                                            if font_size_i + 1 < font_sizes.len() {
                                                font_size_i += 1;
                                                // editor.with_buffer_mut(|buffer| {
                                                //     buffer.set_metrics(font_sizes[font_size_i])
                                                // });
                                            }
                                        }
                                        _ => {}
                                    }
                                } else {
                                    for c in c.chars() {
                                        editor.action(Action::Insert(c));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    window.request_redraw();
                }
                Event::WindowEvent {
                    event:
                        WindowEvent::CursorMoved {
                            device_id,
                            position,
                        },
                    window_id: _,
                } => {
                    mouse_x = position.x;
                    mouse_y = position.y;
                }
                Event::WindowEvent {
                    event:
                        WindowEvent::MouseInput {
                            device_id,
                            state,
                            button,
                        },
                    window_id: _,
                } => {
                    if button == MouseButton::Left {
                        if state == ElementState::Released && mouse_left == ElementState::Pressed {
                            println!("CLICK ({mouse_x}) ({mouse_y})");
                            editor.action(Action::Click {
                                x: mouse_x /*- line_x*/ as i32,
                                y: mouse_y as i32,
                            });
                        }
                        mouse_left = state;
                    }
                }
                Event::WindowEvent {
                    window_id,
                    event:
                        WindowEvent::MouseWheel {
                            device_id,
                            delta,
                            phase,
                        },
                } => {
                    dbg!(delta);
                    match delta {
                        MouseScrollDelta::LineDelta(x, y) => {
                            editor.action(Action::Scroll { lines: y as i32 })
                        }
                        MouseScrollDelta::PixelDelta(PhysicalPosition { x, y }) => {
                            editor.action(Action::Scroll {
                                lines: (y / 20.0).floor() as i32,
                            })
                        }
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id: _,
                } => {
                    //TODO: just close one window
                    elwt.exit();
                }
                _ => {}
            }
        })
        .unwrap();
}
