// SPDX-License-Identifier: MIT OR Apache-2.0

use cosmic_text::{
    Action, Attrs, BorrowedWithFontSystem, Buffer, CacheKeyFlags, Color, Edit, Family, FontSystem, LineHeight, Scroll, Shaping, Style, SwashCache, Weight
};
use std::{collections::HashMap, env, fs, num::NonZeroU32, rc::Rc, slice};
use tiny_skia::{Paint, PixmapMut, Rect, Transform};
use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window as WinitWindow, WindowBuilder},
};

fn main() {
    env_logger::init();

    let path = if let Some(arg) = env::args().nth(1) {
        arg
    } else {
        "../../sample/hello.txt".to_string()
    };

    let mut font_system = FontSystem::new();

    let mut swash_cache = SwashCache::new();

    let mut buffer = Buffer::new_empty();
    let mut buffer = buffer.borrow_with(&mut font_system);

    let mut attrs = Attrs::new()
        .family(Family::Monospace)
        .size(14.0)
        .line_height(LineHeight::Proportional(1.2));
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) => {
            panic!("failed to load {:?}: {}", path, err);
        }
    };

    let event_loop = EventLoop::new().unwrap(); 
    let mut window = Rc::new(WindowBuilder::new().build(&event_loop).unwrap());
    let mut context = softbuffer::Context::new(window.clone()).unwrap();
    let mut surface = softbuffer::Surface::new(&context, window.clone()).unwrap();
    let mut scroll = Scroll::default();

    // buffer.set_text(&text, attrs.scale(window.scale_factor() as f32), Shaping::Advanced);

    fn set_text<'a>(buffer: &mut BorrowedWithFontSystem<'a, Buffer>, scale_factor: f32) {

        let attrs = Attrs::new()
            .size(32.0)
            .line_height(LineHeight::Absolute(44.0))
            .scale(scale_factor);
        let serif_attrs = attrs.family(Family::Serif);
        let mono_attrs = attrs.family(Family::Monospace);
        let comic_attrs = attrs.family(Family::Name("Comic Neue"));

        let spans: &[(&str, Attrs)] = &[
            ("B", attrs.weight(Weight::BOLD)),
            ("old ", attrs),
            ("I", attrs.style(Style::Italic)),
            ("talic ", attrs),
            ("f", attrs),
            ("i ", attrs),
            ("f", attrs.weight(Weight::BOLD)),
            ("i ", attrs),
            ("f", attrs.style(Style::Italic)),
            ("i \n", attrs),
            ("Sans-Serif Normal ", attrs),
            ("Sans-Serif Bold ", attrs.weight(Weight::BOLD)),
            ("Sans-Serif Italic ", attrs.style(Style::Italic)),
            (
                "Sans-Serif Fake Italic ",
                attrs.cache_key_flags(CacheKeyFlags::FAKE_ITALIC),
            ),
            (
                "Sans-Serif Bold Italic\n",
                attrs.weight(Weight::BOLD).style(Style::Italic),
            ),
            ("Serif Normal ", serif_attrs),
            ("Serif Bold ", serif_attrs.weight(Weight::BOLD)),
            ("Serif Italic ", serif_attrs.style(Style::Italic)),
            (
                "Serif Bold Italic\n",
                serif_attrs.weight(Weight::BOLD).style(Style::Italic),
            ),
            ("Mono Normal ", mono_attrs),
            ("Mono Bold ", mono_attrs.weight(Weight::BOLD)),
            ("Mono Italic ", mono_attrs.style(Style::Italic)),
            (
                "Mono Bold Italic\n",
                mono_attrs.weight(Weight::BOLD).style(Style::Italic),
            ),
            ("Comic Normal ", comic_attrs),
            ("Comic Bold ", comic_attrs.weight(Weight::BOLD)),
            ("Comic Italic ", comic_attrs.style(Style::Italic)),
            (
                "Comic Bold Italic\n",
                comic_attrs.weight(Weight::BOLD).style(Style::Italic),
            ),
            ("R", attrs.color(Color::rgb(0xFF, 0x00, 0x00))),
            ("A", attrs.color(Color::rgb(0xFF, 0x7F, 0x00))),
            ("I", attrs.color(Color::rgb(0xFF, 0xFF, 0x00))),
            ("N", attrs.color(Color::rgb(0x00, 0xFF, 0x00))),
            ("B", attrs.color(Color::rgb(0x00, 0x00, 0xFF))),
            ("O", attrs.color(Color::rgb(0x4B, 0x00, 0x82))),
            ("W ", attrs.color(Color::rgb(0x94, 0x00, 0xD3))),
            (
                "Red ",
                attrs
                    .color(Color::rgb(0xFF, 0x00, 0x00))
                    .size(attrs.font_size * 1.9)
                    .line_height(LineHeight::Proportional(0.9)),
            ),
            (
                "Orange ",
                attrs
                    .color(Color::rgb(0xFF, 0x7F, 0x00))
                    .size(attrs.font_size * 1.6)
                    .line_height(LineHeight::Proportional(1.0)),
            ),
            (
                "Yellow ",
                attrs
                    .color(Color::rgb(0xFF, 0xFF, 0x00))
                    .size(attrs.font_size * 1.3)
                    .line_height(LineHeight::Proportional(1.1)),
            ),
            (
                "Green ",
                attrs
                    .color(Color::rgb(0x00, 0xFF, 0x00))
                    .size(attrs.font_size * 1.0)
                    .line_height(LineHeight::Proportional(1.2)),
            ),
            (
                "Blue ",
                attrs
                    .color(Color::rgb(0x00, 0x00, 0xFF))
                    .size(attrs.font_size * 0.8)
                    .line_height(LineHeight::Proportional(1.3)),
            ),
            (
                "Indigo ",
                attrs
                    .color(Color::rgb(0x4B, 0x00, 0x82))
                    .size(attrs.font_size * 0.6)
                    .line_height(LineHeight::Proportional(1.4)),
            ),
            (
                "Violet ",
                attrs
                    .color(Color::rgb(0x94, 0x00, 0xD3))
                    .size(attrs.font_size * 0.4)
                    .line_height(LineHeight::Proportional(1.5)),
            ),
            ("U", attrs.color(Color::rgb(0x94, 0x00, 0xD3))),
            ("N", attrs.color(Color::rgb(0x4B, 0x00, 0x82))),
            ("I", attrs.color(Color::rgb(0x00, 0x00, 0xFF))),
            ("C", attrs.color(Color::rgb(0x00, 0xFF, 0x00))),
            ("O", attrs.color(Color::rgb(0xFF, 0xFF, 0x00))),
            ("R", attrs.color(Color::rgb(0xFF, 0x7F, 0x00))),
            ("N\n", attrs.color(Color::rgb(0xFF, 0x00, 0x00))),
            (
                "生活,삶,जिंदगी 😀 FPS\n",
                attrs.color(Color::rgb(0xFF, 0x00, 0x00)),
            ),
        ];
        buffer.set_rich_text(spans.iter().copied(), attrs, Shaping::Advanced)
    }

    set_text(&mut buffer, window.scale_factor() as f32);

    

    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Wait);

            match event {
                Event::WindowEvent { window_id, event: WindowEvent::ScaleFactorChanged { scale_factor, .. } } => {
                    set_text(&mut buffer, scale_factor as f32);
                    log::info!("Updated scale factor for {window_id:?}");
                    window.request_redraw();
                }
                Event::WindowEvent {
                    window_id,
                    event: WindowEvent::RedrawRequested,
                } => {
                    let (width, height) = {
                        let size = window.inner_size();
                        (size.width, size.height)
                    };
                    
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

                    // Set scroll to view scroll
                    buffer.set_scroll(scroll);
                    // Set size, will relayout and shape until scroll if changed
                    buffer.set_size(width as f32, height as f32);
                    // Shape until scroll, ensures scroll is clamped
                    //TODO: ability to prune with multiple views?
                    buffer.shape_until_scroll(true);
                    // Update scroll after buffer clamps it
                    scroll = buffer.scroll();

                    let mut paint = Paint::default();
                    paint.anti_alias = false;
                    let transform = Transform::identity();
                    buffer.draw(
                        &mut swash_cache,
                        cosmic_text::Color::rgb(0xFF, 0xFF, 0xFF),
                        |x, y, w, h, color| {
                            paint.set_color_rgba8(color.r(), color.g(), color.b(), color.a());
                            pixmap.fill_rect(
                                Rect::from_xywh(x as f32, y as f32, w as f32, h as f32)
                                    .unwrap(),
                                &paint,
                                transform,
                                None,
                            );
                        },
                    );

                    surface_buffer.present().unwrap();
                }
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
                    if state == ElementState::Pressed {
                        match logical_key {
                            Key::Named(NamedKey::ArrowDown) => {
                                scroll.layout += 1;
                            }
                            Key::Named(NamedKey::ArrowUp) => {
                                scroll.layout -= 1;
                            }
                            Key::Named(NamedKey::PageDown) => {
                                scroll.layout += buffer.visible_lines();
                            }
                            Key::Named(NamedKey::PageUp) => {
                                scroll.layout -= buffer.visible_lines();
                            }
                            _ => {}
                        }
                    }
                    println!("{:?} {:?} {:?}", logical_key, text, state);
                    window.request_redraw();
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
