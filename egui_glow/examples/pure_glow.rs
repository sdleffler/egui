//! Example how to use pure `egui_glow` without [`epi`].

fn create_display(
    event_loop: &glutin::event_loop::EventLoop<()>,
) -> (
    glutin::WindowedContext<glutin::PossiblyCurrent>,
    glow::Context,
) {
    let window_builder = glutin::window::WindowBuilder::new()
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize {
            width: 800.0,
            height: 600.0,
        })
        .with_title("egui_glow example");

    let gl_window = unsafe {
        glutin::ContextBuilder::new()
            .with_depth_buffer(0)
            .with_srgb(true)
            .with_stencil_buffer(0)
            .with_vsync(true)
            .build_windowed(window_builder, event_loop)
            .unwrap()
            .make_current()
            .unwrap()
    };

    let gl = unsafe { glow::Context::from_loader_function(|s| gl_window.get_proc_address(s)) };

    unsafe {
        use glow::HasContext as _;
        gl.enable(glow::FRAMEBUFFER_SRGB);
    }

    (gl_window, gl)
}

fn main() {
    let event_loop = glutin::event_loop::EventLoop::with_user_event();
    let (gl_window, gl) = create_display(&event_loop);

    let mut egui_glow = egui_glow::EguiGlow::new(&gl_window, &gl);

    event_loop.run(move |event, _, control_flow| {
        let mut redraw = || {
            let mut quit = false;

            let (needs_repaint, shapes) = egui_glow.run(gl_window.window(), |egui_ctx| {
                egui::SidePanel::left("my_side_panel").show(egui_ctx, |ui| {
                    ui.heading("Hello World!");
                    if ui.button("Quit").clicked() {
                        quit = true;
                    }
                });
            });

            *control_flow = if quit {
                glutin::event_loop::ControlFlow::Exit
            } else if needs_repaint {
                gl_window.window().request_redraw();
                glutin::event_loop::ControlFlow::Poll
            } else {
                glutin::event_loop::ControlFlow::Wait
            };

            {
                let color = egui::Rgba::from_rgb(0.1, 0.3, 0.2);
                unsafe {
                    use glow::HasContext as _;
                    gl.clear_color(color[0], color[1], color[2], color[3]);
                    gl.clear(glow::COLOR_BUFFER_BIT);
                }

                // draw things behind egui here

                egui_glow.paint(&gl_window, &gl, shapes);

                // draw things on top of egui here

                gl_window.swap_buffers().unwrap();
            }
        };

        match event {
            // Platform-dependent event handlers to workaround a winit bug
            // See: https://github.com/rust-windowing/winit/issues/987
            // See: https://github.com/rust-windowing/winit/issues/1619
            glutin::event::Event::RedrawEventsCleared if cfg!(windows) => redraw(),
            glutin::event::Event::RedrawRequested(_) if !cfg!(windows) => redraw(),

            glutin::event::Event::WindowEvent { event, .. } => {
                use glutin::event::WindowEvent;
                if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                }

                if let glutin::event::WindowEvent::Resized(physical_size) = event {
                    gl_window.resize(physical_size);
                }

                egui_glow.on_event(&event);

                gl_window.window().request_redraw(); // TODO: ask egui if the events warrants a repaint instead
            }
            glutin::event::Event::LoopDestroyed => {
                egui_glow.destroy(&gl);
            }

            _ => (),
        }
    });
}
