use pixels::{Pixels, SurfaceTexture};
use rand::prelude::*;
use rayon::prelude::*;
// use std::ops::Index;
use std::time::Instant;
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    // window::Fullscreen,
};
use winit_input_helper::WinitInputHelper;

const WIDTH: usize = 256;
const HEIGHT: usize = 128;
const START_X: i32 = WIDTH as i32 / 2;
const START_Y: i32 = HEIGHT as i32 / 2;
const BITS_PER_CHANNEL: u8 = 5;
const COLORS_PER_CHANNEL: u8 = 1 << BITS_PER_CHANNEL;
const AVERAGE: bool = true;
const _: () = assert!(WIDTH * HEIGHT == 1 << (3 * BITS_PER_CHANNEL), "number of pixels must match number of colors!");

#[derive(Clone, Hash, PartialEq, Eq)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn get_neighbors(&self) -> Vec<Point> {
        let mut ret = vec![];
        ret.reserve(8);
        for dy in -1..=1 {
            if self.y + dy < 0 || self.y + dy >= HEIGHT as i32 {
                continue;
            }
            for dx in -1..=1 {
                if self.x + dx < 0 || self.x + dx >= WIDTH as i32 {
                    continue;
                }
                ret.push(Point {
                    x: self.x + dx,
                    y: self.y + dy,
                });
            }
        }
        ret
    }
}

struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    fn col_diff(&self, other: &Color) -> i32 {
        let mut ret = 0;
        ret += {
            let r = self.r as i32 - other.r as i32;
            r * r
        };
        ret += {
            let g = self.g as i32 - other.g as i32;
            g * g
        };
        ret += {
            let b = self.b as i32 - other.b as i32;
            b * b
        };
        ret
    }
    fn from_pixel(pixels: &[u8], pos: &Point) -> Option<Color> {
        let pos = 4 * (pos.y as usize * WIDTH + pos.x as usize);
        if pixels[pos + 3] > 0 {
            Some(Color {
                r: pixels[pos],
                g: pixels[pos + 1],
                b: pixels[pos + 2],
            })
        } else {
            None
        }
    }
    fn fill_point(&self, pixels: &mut [u8], pos: &Point) {
        let pos = 4 * (pos.y as usize * WIDTH + pos.x as usize);
        pixels[pos] = self.r;
        pixels[pos + 1] = self.g;
        pixels[pos + 2] = self.b;
        pixels[pos + 3] = 0xff;
    }
}

fn calc_diff(pixels: &[u8], xy: &Point, c: &Color) -> i32 {
    let mut diffs = vec![];
    diffs.reserve(8);
    for nxy in xy.get_neighbors() {
        let nc = Color::from_pixel(pixels, &nxy);
        if let Some(nc) = nc {
            diffs.push(nc.col_diff(c));
        }
    }

    if AVERAGE {
        let len = diffs.len() as i32;
        if len == 0 {
            0
        } else {
            diffs.into_iter().sum::<i32>() / len
        }
    } else {
        diffs.into_iter().min().unwrap_or(0)
    }
}

struct RainbowSmoke {
    colors: Vec<Color>,
    available: std::collections::HashSet<Point>,
}

impl RainbowSmoke {
    fn new() -> RainbowSmoke {
        let mut colors = vec![];
        colors.reserve(
            COLORS_PER_CHANNEL as usize * COLORS_PER_CHANNEL as usize * COLORS_PER_CHANNEL as usize,
        );
        for r in 0..COLORS_PER_CHANNEL {
            for g in 0..COLORS_PER_CHANNEL {
                for b in 0..COLORS_PER_CHANNEL {
                    colors.push(Color {
                        r: (r as u16 * 0xff as u16 / (COLORS_PER_CHANNEL - 1) as u16) as u8,
                        g: (g as u16 * 0xff as u16 / (COLORS_PER_CHANNEL - 1) as u16) as u8,
                        b: (b as u16 * 0xff as u16 / (COLORS_PER_CHANNEL - 1) as u16) as u8,
                    });
                }
            }
        }
        colors.shuffle(&mut rand::thread_rng());
        let mut available = std::collections::HashSet::new();
        available.insert(Point {
            x: START_X,
            y: START_Y,
        });
        available.insert(Point {
            x: START_X / 2,
            y: START_Y / 2,
        });
        available.insert(Point {
            x: 3 * START_X / 2,
            y: 3 * START_Y / 2,
        });
        available.insert(Point {
            x: 0,
            y: HEIGHT as i32 - 1,
        });
        available.insert(Point {
            x: WIDTH as i32 - 1,
            y: 0,
        });
        RainbowSmoke { colors, available }
    }

    fn next_pixel(&mut self, pixels: &mut [u8]) -> bool {
        if let Some(c) = self.colors.pop() {
            if let Some(best_xy) = self
            .available
            .par_iter()
            .map(|p| (p, calc_diff(pixels, p, &c)))
            .min_by(|(_, a), (_, b)| a.cmp(b))
            .map(|(a, _)| a.clone()) {
                c.fill_point(pixels, &best_xy);
                self.available.remove(&best_xy);
                for nxy in best_xy.get_neighbors() {
                    match Color::from_pixel(pixels, &nxy) {
                        Some(_) => {},
                        None => {
                            self.available.insert(nxy);
                        }
                    }
                }
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

fn main() {
    // main event loop and inpu helper
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    // window that contains the framebuffer
    let window = {
        let size = LogicalSize::new(WIDTH as u32, HEIGHT as u32);
        WindowBuilder::new()
            .with_title("Hello Pixels")
            .with_inner_size(size)
            .with_min_inner_size(size)
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
    // frame timer
    let mut prev_time = Instant::now();
    // rainbow smoke generator
    let mut rainbow_smoke = RainbowSmoke::new();
    pixels.get_frame().fill(0x0);

    event_loop.run(move |event, _, control_flow| {
        // draw a new frame
        if let Event::RedrawRequested(_) = event {
            // draw the content
            let framebuffer = pixels.get_frame();
            print!("Candidates: {}, ", rainbow_smoke.available.len());
            for _ in 0..25 {
                rainbow_smoke.next_pixel(framebuffer);
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
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }
            // input was detected => redraw the window
            window.request_redraw();
        }
    });
}
