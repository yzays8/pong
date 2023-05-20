use rand::Rng;
use std::collections::VecDeque;
use std::process;
use std::time::Instant;

use sdl2::event::Event;
use sdl2::keyboard::{Keycode, Scancode};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

struct Vector2 {
    x: f32,
    y: f32,
}

struct Ball {
    pos: Vector2,
    vel: Vector2,
}

pub struct Game {
    sdl_context: sdl2::Sdl,
    canvas: Canvas<Window>,
    is_running: bool,
    ticks_count: Instant,
    balls: VecDeque<Ball>,
    paddle_pos: Vector2,
    paddle_dir: i32,
}

impl Game {
    const THICKNESS: f32 = 15.0;
    const WINDOW_WIDTH: f32 = 1024.0;
    const WINDOW_HEIGHT: f32 = 768.0;
    const PADDLE_WIDTH: f32 = 6.0 * Game::THICKNESS;
    const PADDLE_VEL: f32 = 800.0;

    pub fn build() -> Result<Game, String> {
        let sdl_context = match sdl2::init() {
            Ok(sdl_context) => sdl_context,
            Err(err) => return Err(format!("Failed to initialize SDL2: {err}")),
        };

        let video_subsystem = match sdl_context.video() {
            Ok(video_subsystem) => video_subsystem,
            Err(err) => return Err(format!("Failed to initialize SDL2 video subsystem: {err}")),
        };

        let window = video_subsystem
            .window(
                "Pong",
                Self::WINDOW_WIDTH as u32,
                Self::WINDOW_HEIGHT as u32,
            )
            .position_centered()
            .build();
        let window = match window {
            Ok(window) => window,
            Err(err) => return Err(format!("Failed to create window: {err}")),
        };

        let canvas = match window.into_canvas().build() {
            Ok(canvas) => canvas,
            Err(err) => return Err(format!("Failed to create canvas: {err}")),
        };

        let mut balls: VecDeque<Ball> = VecDeque::new();
        balls.push_front(Ball {
            pos: Vector2 {
                x: Self::WINDOW_WIDTH * 3.0 / 4.0,
                y: Self::WINDOW_HEIGHT / 2.0,
            },
            vel: Self::get_random_velocity(),
        });
        balls.push_front(Ball {
            pos: Vector2 {
                x: Self::WINDOW_WIDTH / 4.0,
                y: Self::WINDOW_HEIGHT / 2.0,
            },
            vel: Self::get_random_velocity(),
        });

        Ok(Game {
            sdl_context,
            canvas,
            is_running: true,
            ticks_count: Instant::now(),
            balls,
            paddle_pos: Vector2 {
                x: Self::WINDOW_WIDTH / 2.0,
                y: Self::WINDOW_HEIGHT - Self::THICKNESS,
            },
            paddle_dir: 0,
        })
    }

    pub fn run(&mut self) {
        while self.is_running {
            self.process_input();
            self.update();
            self.render();
        }
    }

    fn process_input(&mut self) {
        let mut event_pump = self.sdl_context.event_pump().unwrap_or_else(|err| {
            eprintln!("Failed to get SDL2 event pump: {err}");
            process::exit(1);
        });

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => self.is_running = false,
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    ..
                } => {
                    if self.balls.len() == 5 {
                        self.balls.pop_back();
                    }
                    self.balls.push_front({
                        Ball {
                            pos: Vector2 {
                                x: Self::WINDOW_WIDTH / 2.0,
                                y: Self::WINDOW_HEIGHT / 2.0,
                            },
                            vel: Self::get_random_velocity(),
                        }
                    });
                }
                _ => {}
            }
        }

        self.paddle_dir = 0;
        for key in event_pump.keyboard_state().pressed_scancodes() {
            match key {
                Scancode::A => self.paddle_dir = -1,
                Scancode::D => self.paddle_dir = 1,
                _ => {}
            }
        }
    }

    fn update(&mut self) {
        // wait until 16ms has elapsed since last frame
        while !(self.ticks_count.elapsed().as_millis() > 16) {}

        let mut delta_time = self.ticks_count.elapsed().as_secs_f32();
        // cap delta time to 50ms
        if delta_time >= 0.05 {
            delta_time = 0.05;
        }

        // move paddle
        if self.paddle_dir != 0 {
            self.paddle_pos.x += self.paddle_dir as f32 * Self::PADDLE_VEL * delta_time;

            // make sure the paddle doesn't go off the screen
            if self.paddle_pos.x > (Self::WINDOW_WIDTH - Self::PADDLE_WIDTH / 2.0 - Self::THICKNESS)
            {
                self.paddle_pos.x = Self::WINDOW_WIDTH - Self::PADDLE_WIDTH / 2.0 - Self::THICKNESS;
            } else if self.paddle_pos.x < (Self::THICKNESS + Self::PADDLE_WIDTH / 2.0) {
                self.paddle_pos.x = Self::THICKNESS + Self::PADDLE_WIDTH / 2.0;
            }
        }

        // move balls
        for ball in &mut self.balls {
            ball.pos.x += ball.vel.x * delta_time;
            ball.pos.y += ball.vel.y * delta_time;

            // collision detection with right and left walls
            if (ball.pos.x <= Self::THICKNESS && ball.vel.x < 0.0)
                || ((ball.pos.x >= Self::WINDOW_WIDTH - Self::THICKNESS) && ball.vel.x > 0.0)
            {
                ball.vel.x = -ball.vel.x;
            }

            // collision detection with top wall
            if (ball.pos.y <= Self::THICKNESS) && (ball.vel.y < 0.0) {
                ball.vel.y = -ball.vel.y;
            }

            // collision detection with paddle
            if (self.paddle_pos.x - ball.pos.x).abs() <= (Self::PADDLE_WIDTH / 2.0)
                && (ball.pos.y >= Self::WINDOW_HEIGHT - Self::THICKNESS)
                && (ball.pos.y <= Self::WINDOW_HEIGHT)
                && (ball.vel.y > 0.0)
            {
                ball.vel.y = -ball.vel.y;
            }
        }

        self.ticks_count = Instant::now();
    }

    fn render(&mut self) {
        // draw background
        self.canvas.set_draw_color(Color::RGB(124, 199, 232));
        self.canvas.clear();

        self.canvas.set_draw_color(Color::RGB(255, 255, 255));

        // draw top wall
        let mut wall = Rect::new(0, 0, Self::WINDOW_WIDTH as u32, Self::THICKNESS as u32);
        self.canvas.fill_rect(wall).unwrap();

        // draw left wall
        wall.w = Self::THICKNESS as i32;
        wall.h = (Self::WINDOW_HEIGHT - Self::THICKNESS) as i32;

        self.canvas.fill_rect(wall).unwrap();

        // draw right wall
        wall.x = (Self::WINDOW_WIDTH - Self::THICKNESS) as i32;
        wall.w = Self::THICKNESS as i32;
        self.canvas.fill_rect(wall).unwrap();

        // draw paddle
        let paddle = Rect::new(
            (self.paddle_pos.x - Self::PADDLE_WIDTH / 2.0) as i32,
            self.paddle_pos.y as i32,
            Self::PADDLE_WIDTH as u32,
            Self::THICKNESS as u32,
        );
        self.canvas.fill_rect(paddle).unwrap();

        // draw ball
        for ball in &self.balls {
            self.canvas
                .fill_rect(Rect::new(
                    (ball.pos.x - Self::THICKNESS / 2.0) as i32,
                    (ball.pos.y - Self::THICKNESS / 2.0) as i32,
                    Self::THICKNESS as u32,
                    Self::THICKNESS as u32,
                ))
                .unwrap();
        }

        self.canvas.present();
    }

    // get appropriate random velocity for the ball
    fn get_random_velocity() -> Vector2 {
        let mut rng = rand::thread_rng();
        let mut sp_x = rng.gen_range(0..400) as f32;
        let sp_y = rng.gen_range(-400..-200) as f32;

        if sp_x < 200.0 {
            sp_x = -(sp_x + 200.0);
        }

        Vector2 { x: sp_x, y: sp_y }
    }
}
