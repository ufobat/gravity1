extern crate sdl2;
extern crate rand;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Point;
use std::time::Duration;
use std::cell::RefCell;
use std::error::Error;
use rand::distributions::{Range, Sample};

const WINDOW_WIDTH: u32 = 1200;
const WINDOW_HEIGHT: u32 = 800;
const NUM_OF_MATTER: u32 = 90;

struct MainGame {
    canvas: sdl2::render::WindowCanvas,
    event_pump: sdl2::EventPump
}

struct Matter {
    pos: Vec2D,
    mass: f64,
    velocity: Vec2D
}

#[derive(Clone)]
struct Vec2D {
    x: f64,
    y: f64,
}
impl Vec2D {
    fn add_vec(&mut self, other: &Vec2D) {
        self.x = self.x + other.x;
        self.y = self.y + other.y;
    }

    fn del_vec(&mut self, other: &Vec2D) {
        self.x = self.x - other.x;
        self.y = self.y - other.y;
    }

    fn length(&self) -> f64 {
        return (self.x * self.x + self.y * self.y).sqrt();
    }

    fn scale(&mut self, factor: f64) {
        self.x = self.x * factor;
        self.y = self.y * factor;
    }
}


impl Matter {
    fn new(pos_x: f64, pos_y: f64, mass: f64) -> Matter {
        Matter {
            velocity: Vec2D { x: 0f64 , y: 0f64 },
            pos: Vec2D { x: pos_x , y: pos_y },
            mass: mass,
        }
    }

    fn apply_force(&mut self, mut force: Vec2D) {
        // F = m * a; => a = F / m
        // convert force to "a"
        force.scale( 1.0 / self.mass );
        self.velocity.add_vec(&force);
    }

    fn move_around(&mut self) {
        self.pos.add_vec(&self.velocity);
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
    x_shift: i32,
    y_shift: i32,
}

impl Viewport {
    fn new() -> Self {
        Viewport {
            x_shift: WINDOW_WIDTH as i32 / 2,
            y_shift: WINDOW_HEIGHT as i32 / 2,
        }
    }

    fn to_canvas_pos(&self, pos: &Vec2D) -> (i32, i32) {
        let x = self.x_shift + pos.x as i32;
        let y = self.y_shift + pos.y as i32;
        return (x, y);
    }

    // fn shift_view(&mut self, x: i32, y: i32) {
    //     self.x_shift += x;
    //     self.y_shift += y;
    // }

    fn adjust_to_drift(&mut self, drift: &Vec2D) {
        self.x_shift = WINDOW_WIDTH as i32  / 2 - drift.x as i32;
        self.y_shift = WINDOW_HEIGHT as i32 / 2 - drift.y as i32;
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

    'running: loop {
        // ! calculate next step
        for idx_matter in 0..matter.len() {
            let mut force = Vec2D { x: 0.0, y:0.0 };
            let mut m = matter[idx_matter].borrow_mut();
            for idx_other_matter in 0..matter.len() {
                if idx_matter == idx_other_matter { continue }
                let other = matter[idx_other_matter].borrow();
                let mut from_m_to_other = other.pos.clone();
                from_m_to_other.del_vec(&m.pos);
                let distance = from_m_to_other.length();
                // force = g * m1 * m2 / r*r
                let force_factor = 0.003 * m.mass * other.mass / (distance * distance);
                from_m_to_other.scale(force_factor);
                force.add_vec(&from_m_to_other);
            }
            m.apply_force(force);
            m.move_around();
        }

        // ! draw screen
        // black screen
        game.canvas.set_draw_color(Color::RGB(0, 0, 0));
        game.canvas.clear();
        game.canvas.set_draw_color(Color::RGB(255,255, 0));
        let mut viewdrift = Vec2D { x: 0.0 , y: 0.0 };
        let (x, y) = viewport.to_canvas_pos(&viewdrift);
        game.canvas.draw_point(Point::new(x, y)).unwrap();
        // draw matter
        game.canvas.set_draw_color(Color::RGB(255, 255, 255));
        for matter in matter.iter() {
            let m = matter.borrow();
            let pos = &m.pos;
            viewdrift.add_vec(pos);
            let (x, y) = viewport.to_canvas_pos(pos);
            // println!("drawing white point to {} {}", x, y);
            game.canvas.draw_point(Point::new(x, y)).unwrap();
        }
        game.canvas.set_draw_color(Color::RGB(255,0,0));
        viewdrift.scale(1.0 / matter.len() as f64);
        let (x, y) = viewport.to_canvas_pos(&viewdrift);
        game.canvas.draw_point(Point::new(x, y)).unwrap();
        viewport.adjust_to_drift(&viewdrift);
        println!("viewdrift is {} {}", viewdrift.x, viewdrift.y);
        // update Screen
        game.canvas.present();

        for event in game.event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), ..} => {
                    break 'running
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

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60 ));
        //::std::thread::sleep(Duration::new(1, 0 ));
    }
}
