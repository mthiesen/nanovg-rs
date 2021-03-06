extern crate glutin;
extern crate gl;
extern crate nanovg;

use std::time::Instant;
use std::f32::consts::PI;
use glutin::GlContext;
use nanovg::{FillStyle, StrokeStyle, ColoringStyle, Color, Paint, CompositeOperation, BasicCompositeOperation, PathOptions, Scissor};

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("nanovg Test")
        .with_dimensions(1024, 720);
    let context = glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4)
        .with_srgb(true);
    let gl_window = glutin::GlWindow::new(window, context, &events_loop).unwrap();

    unsafe {
        gl_window.make_current().unwrap();
        gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    }
    
    let context = nanovg::Context::with_gl3(nanovg::CreateFlags::new().stencil_strokes()).unwrap();
    let img = nanovg::Image::new(&context)
        .repeat_y()
        .repeat_x()
        .build_from_file("resources/lenna.png")
        .expect("Couldn't load image");

    let start_time = Instant::now();
    let mut running = true;

    while running {
        events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::Closed => running = false,
                glutin::WindowEvent::Resized(w, h) => gl_window.resize(w, h),
                _ => {},
            },
            _ => {},
        });

        let (width, height) = gl_window.get_inner_size_pixels().unwrap();
        let (width, height) = (width as i32, height as i32);

        unsafe {
            gl::Viewport(0, 0, width, height);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
        }

        let elapsed = {
            let elapsed = start_time.elapsed();
            elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9
        } as f32;

        let ratio = width as f32 / height as f32;

        // Let's draw a frame!

        context.frame((width, height), ratio, |frame| {
            // Draw red-filled rectangle.
            frame.path(|path| {
                path.rect((100.0, 100.0), (300.0, 300.0));
                path.fill(FillStyle {
                    coloring_style: ColoringStyle::Paint(Paint::with_linear_gradient(&context, (100.0, 100.0), (400.0, 400.0), Color::from_rgb(0xAA, 0x6C, 0x39), Color::from_rgb(0x88, 0x2D, 0x60))),
                    .. Default::default()
                });
            }, Default::default());

            // Draw custom yellow/blue shape.
            frame.path(|path| {
                let origin = (150.0, 140.0);
                path.circle(origin, 64.0);
                path.sub_path(origin, |sp| {
                    sp.line_to((origin.0 + 300.0, origin.1 - 50.0));
                    sp.quad_bezier_to((origin.0 + 500.0, origin.1 + 100.0), (300.0, 100.0));
                    sp.close();
                });
                path.stroke(StrokeStyle {
                    coloring_style: ColoringStyle::Color(Color::new(1.0, 1.0, 0.0, 1.0)),
                    width: 3.0,
                    .. Default::default()
                });
                path.fill(FillStyle {
                    coloring_style: ColoringStyle::Color(Color::new(0.2, 0.0, 0.8, 1.0)),
                    .. Default::default()
                });
            }, PathOptions {
                composite_operation: CompositeOperation::Basic(BasicCompositeOperation::Lighter),
                alpha: elapsed.cos() * 0.5 + 0.5,
                .. Default::default()
            });

            // Draw rolling image (with scissors)
            frame.path(|path| {
                let radius = 100.0;
                let distance = 500.0; // Distance to roll
                let rolled = ((elapsed / 5.0).sin() * 0.5 + 0.5) * distance; // Distance currently rolled
                let origin = (rolled + 100.0, 600.0);
                let paint = Paint::with_image_pattern(&context, &img, origin, (100.0, 100.0), rolled / (2.0 * PI * radius) * 2.0 * PI, 1.0);
                path.circle(origin, radius);
                path.fill(FillStyle {
                    coloring_style: ColoringStyle::Paint(paint),
                    .. Default::default()
                })
            }, PathOptions {
                scissor: Some(Scissor::Rect { x: 150.0, y: 600.0, width: 1000.0, height: 200.0 }),
                .. Default::default()
            });

            // Draw stroked rectangle.
            frame.path(|path| {
                path.rect((300.0, 310.0), (300.0, 300.0));
                let color = Color::lerp(Color::from_rgb(0x2e, 0x50, 0x77), Color::from_rgb(0xff, 0xca, 0x77), elapsed.sin() * 0.5 + 0.5);
                path.fill(FillStyle {
                    coloring_style: ColoringStyle::Color(Color::new(0.2, 0.2, 0.2, 0.7)),
                    .. Default::default()
                });
                path.stroke(StrokeStyle {
                    coloring_style: ColoringStyle::Color(color),
                    width: 5.0,
                    .. Default::default()
                });
            }, Default::default());
        });

        gl_window.swap_buffers().unwrap();
    }
}
