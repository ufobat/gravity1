extern crate sdl2;
extern crate rand;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Point;
use std::time::Duration;
use std::cell::RefCell;
use rand::distributions::{Range, Sample};

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
    fn new(pos_x: f64, pos_y: f64) -> Matter {
        Matter {
            velocity: Vec2D { x: 0f64 , y: 0f64 },
            pos: Vec2D { x: pos_x , y: pos_y },
            mass: 1.0,
        }
    }

    fn apply_force(&mut self, force: &Vec2D) {
        self.velocity.add_vec(force);
    }

    fn move_around(&mut self) {
        self.pos.add_vec(&self.velocity);
    }
}

impl MainGame {
    fn new() -> MainGame {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem.window("gravity1", 1401, 1401)
            .position_centered()
            .build()
            .unwrap();
        let canvas = window.into_canvas().build().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();
        MainGame {canvas, event_pump}
    }
}

fn to_canvas_pos(pos_x: f64, pos_y: f64) -> (i32, i32) {
    let x = 801 + pos_x as i32;
    let y = 801 + pos_y as i32;
    return (x, y);
}

fn main() {

    let mut game = MainGame::new();
    
    game.canvas.set_draw_color(Color::RGB(0, 0, 0));
    game.canvas.clear();
    game.canvas.present();
    let mut space = Range::new(-280f64, 280f64);
    let mut rng = rand::thread_rng();
    
    let mut matter: Vec<RefCell<Matter>> = Vec::new();
    for _ in 0..15 {
        let pos_x = space.sample(&mut rng);
        let pos_y = space.sample(&mut rng);
        matter.push( RefCell::new(Matter::new(pos_x, pos_y)) );
    }

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
                let force_factor = 0.2 * m.mass * other.mass / (distance * distance);
                from_m_to_other.scale(force_factor);
                force.add_vec(&from_m_to_other);
            }
            m.apply_force(&force);
            m.move_around();
        }
        
        // ! draw screen
        // black screen
        game.canvas.set_draw_color(Color::RGB(0, 0, 0));
        game.canvas.clear();
        // draw matter
        game.canvas.set_draw_color(Color::RGB(255, 255, 255));
        for matter in matter.iter() {
            let m = matter.borrow();
            let (x, y) = to_canvas_pos(m.pos.x, m.pos.y);
            // println!("drawing white point to {} {}", x, y);
            game.canvas.draw_point(Point::new(x, y)).unwrap();
        }
        // update Screen
        game.canvas.present();

        for event in game.event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), ..} => {
                    break 'running
                }
                _ => {}
            }
        }

        game.canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60 ));
    }
}
