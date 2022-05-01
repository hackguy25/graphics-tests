use nalgebra::Complex;
use pixels::{Pixels, SurfaceTexture};
use rayon::prelude::*;
use std::ops::Index;
use std::time::Instant;
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    // window::Fullscreen,
};
use winit_input_helper::WinitInputHelper;

// raytracing: 240x240
// rasterization: 600x600
const WIDTH: usize = 1200;
const HEIGHT: usize = 1200;
mod cgfs_rasterization;
mod cgfs_raytracing;
mod cgfs_scene;
mod mandel;

/// Draws the raytracing scene.
/// Based on the chapters 2 through 5 of the book Computer Graphics from Scratch.
fn draw_scene_raytracing(frame: &mut [u8], start: &Instant) {
    let _time = start.elapsed().as_secs_f64();
    // the camera's position
    let o = (0., 0., 0.);
    // let o = (_time.sin(), _time.cos() + 1., 0.);
    // the viewport data: (width, height, distance from camera)
    let v = (1., 1., 1.);
    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
        let x = (i % WIDTH) as f64 - WIDTH as f64 / 2.;
        let y = HEIGHT as f64 / 2. - (i / WIDTH) as f64;

        let d = cgfs_raytracing::canvas_to_viewport(&v, x, y, WIDTH as f64, HEIGHT as f64);
        let color = cgfs_raytracing::trace_ray(&o, &d, 1., f64::INFINITY, 5);
        let color = (
            color.0.clamp(0., 1.),
            color.1.clamp(0., 1.),
            color.2.clamp(0., 1.),
        );

        let rgba = [
            (255.99 * color.0) as u8,
            (255.99 * color.1) as u8,
            (255.99 * color.2) as u8,
            0xff,
        ];
        pixel.copy_from_slice(&rgba);
    }
}

/// Colors the pixel (x, y) on the canvas with the goven color.
fn put_pixel(x: i64, y: i64, color: &(f64, f64, f64), canvas: &mut [u8]) {
    let x = (x + WIDTH as i64 / 2).clamp(0, WIDTH as i64 - 1) as usize;
    let y = (HEIGHT as i64 / 2 - y).clamp(0, HEIGHT as i64 - 1) as usize;
    let start = 4 * (y * WIDTH + x);
    canvas[start] = (255.99 * color.0.clamp(0., 1.)) as u8;
    canvas[start + 1] = (255.99 * color.1.clamp(0., 1.)) as u8;
    canvas[start + 2] = (255.99 * color.2.clamp(0., 1.)) as u8;
}

/// Colors the pixel (x, y) on the canvas with the goven color, if the new depth is closer than the old one.
fn put_pixel_depth(
    x: i64,
    y: i64,
    z_inv: f64,
    color: &(f64, f64, f64),
    canvas: &mut [u8],
    depth_buffer: &mut [f64],
) {
    let x = (x + WIDTH as i64 / 2).clamp(0, WIDTH as i64 - 1) as usize;
    let y = (HEIGHT as i64 / 2 - y).clamp(0, HEIGHT as i64 - 1) as usize;
    let pos = y * WIDTH + x;
    if depth_buffer[pos] < z_inv {
        let start = 4 * pos;
        canvas[start] = (255.99 * color.0.clamp(0., 1.)) as u8;
        canvas[start + 1] = (255.99 * color.1.clamp(0., 1.)) as u8;
        canvas[start + 2] = (255.99 * color.2.clamp(0., 1.)) as u8;
        depth_buffer[pos] = z_inv;
    }
}

/// Draws the first rasterization scene.
/// Based on the chapters 6 through 9 of the book Computer Graphics from Scratch.
fn draw_scene_rasterization(frame: &mut [u8], start: &Instant) {
    let _time = start.elapsed().as_secs_f64();
    frame.fill(0xff);
    if false {
        if false {
            cgfs_rasterization::draw_filled_triangle(
                cgfs_rasterization::TRIANGLE_POINTS.index(0),
                cgfs_rasterization::TRIANGLE_POINTS.index(1),
                cgfs_rasterization::TRIANGLE_POINTS.index(2),
                &(0.4, 1., 0.2),
                |x, y, c| {
                    put_pixel(x, y, c, frame);
                },
            );
        } else {
            cgfs_rasterization::draw_shaded_triangle(
                cgfs_rasterization::TRIANGLE_POINTS.index(0),
                cgfs_rasterization::TRIANGLE_POINTS.index(1),
                cgfs_rasterization::TRIANGLE_POINTS.index(2),
                &(0.4, 1., 0.2),
                &(1., 0.5, 0.),
                |x, y, c| {
                    put_pixel(x, y, c, frame);
                },
            );
        }
        for (p0, p1) in cgfs_rasterization::TRIANGLE {
            cgfs_rasterization::draw_line(
                cgfs_rasterization::TRIANGLE_POINTS.index(*p0),
                cgfs_rasterization::TRIANGLE_POINTS.index(*p1),
                &(0., 0., 0.),
                |x, y, c| {
                    put_pixel(x, y, c, frame);
                },
            )
        }
    } else {
        let o = (0., 0., 0.);
        // let o = (_time.sin(), _time.cos() + 1., 0.);
        let v = (1., 1., 1.);
        for ((p0, p1), c) in cgfs_rasterization::CUBE {
            cgfs_rasterization::draw_line(
                &cgfs_rasterization::project_vertex(
                    &v,
                    &o,
                    WIDTH as f64,
                    HEIGHT as f64,
                    cgfs_rasterization::CUBE_POINTS.index(*p0),
                ),
                &cgfs_rasterization::project_vertex(
                    &v,
                    &o,
                    WIDTH as f64,
                    HEIGHT as f64,
                    cgfs_rasterization::CUBE_POINTS.index(*p1),
                ),
                c,
                |x, y, c| {
                    put_pixel(x, y, c, frame);
                },
            )
        }
    }
}

/// Draws the second rasterization scene.
/// Based on the chapters 10 through 15 of the book Computer Graphics from Scratch.
fn draw_scene_rasterization_scene(frame: &mut [u8], start: &Instant, _mul: i64, _scale: i32) {
    // reset frame to white
    frame.fill(0xff);
    // create depth buffer with infinite distance
    // (depth buffer holds inverse of distance)
    let mut depth_buffer = vec![0.; WIDTH * HEIGHT];
    // create and position the camera
    let mut camera = cgfs_scene::Camera::default();
    let rot = 0.3 * (start.elapsed().as_secs_f64() * 0.1).sin();
    camera.rotation = cgfs_scene::homogeneous_rotation(0., rot, 0.);
    // precompute the inverse of the camera transform
    let camera_m_inv = camera.inverse_transform();
    // extract viewport data from the camera
    let (v_w, v_h, d) = (
        camera.perspective[0],
        camera.perspective[1],
        camera.perspective[2],
    );
    // select a scene to render
    let scene = match 1 {
        0 => cgfs_scene::simple_scene(),
        _ => cgfs_scene::cube_checkerboard(6),
    };
    // render all instances in the scene
    for instance in scene {
        for (p0, p1, p2, color) in
            instance.project_and_clip(&camera_m_inv, v_w, v_h, d, WIDTH as f64, HEIGHT as f64)
        {
            // cull triangle
            if cgfs_scene::cull_triangle(&p0, &p1, &p2) {
                continue;
            }
            cgfs_rasterization::draw_filled_triangle_with_depth(
                &p0,
                &p1,
                &p2,
                &color,
                |x, y, z, c| {
                    put_pixel_depth(x, y, z, c, frame, &mut depth_buffer);
                },
            )
        }
    }
}

/// Draws the mandelbrot set with a naive approach.
fn draw_mandelbrot_naive(frame: &mut [u8], max_iters: i64) {
    let max_iters = max_iters.max(0) as u64;
    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
        let x = 4. * (i % WIDTH) as f64 / WIDTH as f64 - 2.;
        let y = 4. * (i / WIDTH) as f64 / HEIGHT as f64 - 2.;
        let iters = mandel::mandelbrot_naive(Complex::new(x, y), max_iters);
        let rgba = mandel::gradient_bw(iters as f64, max_iters as f64);
        pixel.copy_from_slice(&rgba);
    }
}

fn mandelbrot_smooth_calculate_pixel(
    i: usize,
    pixel: &mut [u8],
    max_iters: u64,
    pos: &[f64; 3],
    samples: i32,
) {
    if samples <= 1 {
        // determine the complex number represented by the pixel
        let x = (4. * ((i % WIDTH) as f64 + 0.5) / WIDTH as f64 - 2.) * 2_f64.powf(pos[2]) + pos[0];
        let y = (4. * ((i / WIDTH) as f64 + 1. / 3.) / HEIGHT as f64 - 2.) * 2_f64.powf(pos[2])
            + pos[1];
        // calculate the smoothed number of iterations
        let iters = mandel::mandelbrot_smooth(Complex::new(x, y), max_iters);
        // assign a color to the number of iterations
        let rgba = mandel::gradient_bbmw(iters, max_iters as f64);
        pixel.copy_from_slice(&rgba);
    } else {
        let mut iters = 0.;
        // calculate the smoothed number of iteration for many samples
        for (dx, dy) in mandel::halton_2d::<2, 3>().take(samples as usize) {
            let x =
                (4. * ((i % WIDTH) as f64 + dx) / WIDTH as f64 - 2.) * 2_f64.powf(pos[2]) + pos[0];
            let y =
                (4. * ((i / WIDTH) as f64 + dy) / HEIGHT as f64 - 2.) * 2_f64.powf(pos[2]) + pos[1];
            iters += mandel::mandelbrot_smooth(Complex::new(x, y), max_iters);
        }
        // assign a color to the number of iterations
        let rgba = mandel::gradient_bbmw(iters / samples as f64, max_iters as f64);
        pixel.copy_from_slice(&rgba);
    }
}

/// Draws the mandelbrot set with smooth coloring and antialiasing.
/// Supports moving and scaling using the numpad.
fn draw_mandelbrot_smooth_moving(frame: &mut [u8], max_iters: i64, pos: &[f64; 3], samples: i32) {
    let max_iters = max_iters.max(0) as u64;
    frame
        .par_chunks_exact_mut(4)
        .enumerate()
        .map(|(i, pixel)| mandelbrot_smooth_calculate_pixel(i, pixel, max_iters, pos, samples))
        .collect()
}

fn main() {
    // main event loop
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    // window that contains the framebuffer
    let window = {
        let size = LogicalSize::new(WIDTH as u32, HEIGHT as u32);
        WindowBuilder::new()
            .with_title("Hello Pixels")
            .with_inner_size(size)
            // .with_min_inner_size(size)
            // .with_fullscreen(Some(Fullscreen::Borderless(None)))
            .build(&event_loop)
            .unwrap()
    };
    // framebuffer
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture).unwrap()
    };
    // multiple auxiliary parameters
    let start_time = Instant::now();
    let mut param = 0_i64;
    let mut param2 = [0., 0., 0.];
    let mut samples = 0_i32;
    let mut prev_time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        // draw a new frame
        if let Event::RedrawRequested(_) = event {
            // select scene to draw
            match 2 {
                0 => draw_scene_raytracing(pixels.get_frame(), &start_time),
                1 => draw_scene_rasterization(pixels.get_frame(), &start_time),
                2 => {
                    draw_scene_rasterization_scene(pixels.get_frame(), &start_time, param, samples)
                }
                3 => draw_mandelbrot_naive(pixels.get_frame(), param),
                4 => draw_mandelbrot_smooth_moving(pixels.get_frame(), param, &param2, samples),
                _ => (),
            }

            // display drawing time and frames per second
            if true {
                let elapsed = prev_time.elapsed();
                print!("{:?} ({}fps) -> ", elapsed, 1. / elapsed.as_secs_f64());
            }

            // render the frame, exit on error
            if pixels.render().is_err() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // display drawing and render time
            if true {
                let elapsed = prev_time.elapsed();
                println!("{:?} ({}fps)", elapsed, 1. / elapsed.as_secs_f64());
            }

            prev_time = Instant::now();
        }

        // process inputs
        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }
            if input.key_pressed(VirtualKeyCode::NumpadAdd) {
                param += 1;
                println!("param = {param}");
            }
            if input.key_pressed(VirtualKeyCode::NumpadSubtract) {
                param -= 1;
                println!("param = {param}");
            }
            if input.key_pressed(VirtualKeyCode::Numpad4) {
                param2[0] -= 0.1 * (2_f64).powf(param2[2]);
                println!("x = {}", param2[0]);
            }
            if input.key_pressed(VirtualKeyCode::Numpad6) {
                param2[0] += 0.1 * (2_f64).powf(param2[2]);
                println!("x = {}", param2[0]);
            }
            if input.key_pressed(VirtualKeyCode::Numpad8) {
                param2[1] -= 0.1 * (2_f64).powf(param2[2]);
                println!("y = {}", param2[1]);
            }
            if input.key_pressed(VirtualKeyCode::Numpad5) {
                param2[1] += 0.1 * (2_f64).powf(param2[2]);
                println!("y = {}", param2[1]);
            }
            if input.key_pressed(VirtualKeyCode::Numpad7) {
                param2[2] += 0.1;
                println!("scale = {}", param2[2]);
            }
            if input.key_pressed(VirtualKeyCode::Numpad9) {
                param2[2] -= 0.1;
                println!("scale = {}", param2[2]);
            }
            if input.key_pressed(VirtualKeyCode::Numpad1) {
                samples -= 1;
                println!("samples = {}", samples);
            }
            if input.key_pressed(VirtualKeyCode::Numpad2) {
                samples += 1;
                println!("samples = {}", samples);
            }
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }
            // input was detected => redraw the window
            window.request_redraw();
        }
    });
}
