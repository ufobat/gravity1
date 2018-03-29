extern crate sdl2;
extern crate rand;
extern crate nalgebra;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Point;
use std::time::Duration;
use std::cell::RefCell;
use std::error::Error;
use rand::distributions::{Range, Sample};
use nalgebra::{Vector2, Point2, Similarity2, norm};

const WINDOW_WIDTH: u32 = 1200;
const WINDOW_HEIGHT: u32 = 800;
const NUM_OF_MATTER: u32 = 90;

struct MainGame {
    canvas: sdl2::render::WindowCanvas,
    event_pump: sdl2::EventPump
}

struct Matter {
    pos: Vector2<f64>,
    mass: f64,
    velocity: Vector2<f64>,
    mass_scaling: Similarity2<f64>,
}

impl Matter {
    fn new(pos_x: f64, pos_y: f64, mass: f64) -> Matter {
        Matter {
            velocity: Vector2::new(0.0, 0.0),
            pos: Vector2::new(pos_x ,pos_y),
            mass: mass,
            mass_scaling: Similarity2::new(nalgebra::zero(), 0.0, 1.0/mass)
        }
    }

    fn apply_force(&mut self, force: Vector2<f64>) {
        // F = m * a; => a = F / m
        // convert force to "a"
        self.velocity = &self.velocity + * &self.mass_scaling * force;
    }

    fn move_around(&mut self) {
        self.pos = self.pos + self.velocity;
    }
}

impl MainGame {
    fn new() -> Result<MainGame, String> {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video()?;
        let window = video_subsystem.window("gravity1", WINDOW_WIDTH, WINDOW_HEIGHT)
            .position_centered()
            .build()
            .unwrap();
        let canvas = match window.into_canvas().build() {
            Ok(canvas) => canvas,
            Err(e) => return Err(e.description().to_string()),
        };
        let event_pump = sdl_context.event_pump()?;
        Ok(MainGame {canvas, event_pump})
    }
}

struct Viewport {
    viewport: Similarity2<f64>,
    default: Vector2<f64>
}
impl Viewport {
    fn new() -> Self {
        let default = Vector2::new(
            WINDOW_WIDTH as f64 / 2.0,
            WINDOW_HEIGHT as f64 / 2.0
        );
        Viewport {
            viewport: Similarity2::new(default.clone(), 0.0, 1.0),
            default: default
        }
    }

    fn to_point(&self, pos: &Vector2<f64>) -> Point {
        let point = self.viewport * Point2::new(pos.x, pos.y);
        let (x, y) = (point.x.round() as i32 , point.y.round() as i32);
        Point::new(x, y)
    }
    fn adjust(&mut self, drift: &Vector2<f64>) {
        let translation = self.default - drift;
        self.viewport = Similarity2::new(translation, 0.0, 1.0);
    }
}

fn main() {

    let mut game = MainGame::new().unwrap();

    game.canvas.set_draw_color(Color::RGB(0, 0, 0));
    game.canvas.clear();
    game.canvas.present();
    let mut space = Range::new(-280f64, 280f64);
    let mut weight = Range::new(0.1f64, 100.0f64);
    let mut rng = rand::thread_rng();

    let mut matter: Vec<RefCell<Matter>> = Vec::new();
    for _ in 0..NUM_OF_MATTER {
        let pos_x = space.sample(&mut rng);
        let pos_y = space.sample(&mut rng);
        let mass  = weight.sample(&mut rng);
        matter.push( RefCell::new(Matter::new(pos_x, pos_y, mass)) );
    }

    let mut viewport = Viewport::new();
    let mut skip_sleep = false;

    'running: loop {
        // ! calculate next step
        for idx_matter in 0..matter.len() {
            let mut force = Vector2::new(0.0, 0.0);
            let mut m = matter[idx_matter].borrow_mut();
            for idx_other_matter in 0..matter.len() {
                if idx_matter == idx_other_matter { continue }
                let other = matter[idx_other_matter].borrow();
                let from_m_to_other = other.pos - m.pos;
                let distance = norm(&from_m_to_other);
                // force = g * m1 * m2 / r*r
                let force_factor = 0.003 * m.mass * other.mass / (distance * distance);
                let scale_operation = Similarity2::new(Vector2::new(0.0, 0.0), 0.0, force_factor);
                force = force + scale_operation * from_m_to_other;
            }
            m.apply_force(force);
            m.move_around();
        }

        // ! draw screen
        // black screen
        game.canvas.set_draw_color(Color::RGB(0, 0, 0));
        game.canvas.clear();

        // draw origin
        game.canvas.set_draw_color(Color::RGB(255,155, 0));
        let mut viewdrift = Vector2::new(0.0, 0.0);
        let origin_point = viewport.to_point(&viewdrift);
        game.canvas.draw_point(origin_point).unwrap();

        // draw matter
        game.canvas.set_draw_color(Color::RGB(255, 255, 255));
        for matter in matter.iter() {
            let m = matter.borrow();
            let pos = &m.pos;
            viewdrift += pos;
            let point = viewport.to_point(pos);

            // println!("drawing white point to {} {}", x, y);
            // println!("!! drawing {:?}", point);
            game.canvas.draw_point(point).unwrap();
        }

        game.canvas.present();

        viewdrift = viewdrift / matter.len() as f64;
        viewport.adjust(&viewdrift);

        for event in game.event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), ..} => {
                    break 'running
                }
                Event::KeyDown { keycode: Some(Keycode::Q), ..} => {
                    skip_sleep = ! skip_sleep;
                }
                // Event::KeyDown { keycode: Some(Keycode::W), ..} => {
                //     viewport.shift_view(0, 50);
                // }
                // Event::KeyDown { keycode: Some(Keycode::A), ..} => {
                //     viewport.shift_view(50, 0);
                // }
                // Event::KeyDown { keycode: Some(Keycode::S), ..} => {
                //     viewport.shift_view(0, -50);
                // }
                // Event::KeyDown { keycode: Some(Keycode::D), ..} => {
                //     viewport.shift_view(-50, 0);
                // }
                _ => {}
            }
        }

        if !skip_sleep {
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60 ));
        }
    }
}
