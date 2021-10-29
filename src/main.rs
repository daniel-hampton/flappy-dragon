#![warn(clippy::all, clippy::pedantic)]

use bracket_lib::prelude::*;

enum GameMode {
    Menu,
    Playing,
    End,
}

const SCREEN_WIDTH: i32 = 40;
const SCREEN_HEIGHT: i32 = 25;

/// Game Speed
const FRAME_DURATION: f32 = 50.0;

/// Velocity Parameters
const TERMINAL_VELOCITY: f32 = 1.0;
const DELTA_V: f32 = 0.1;
const FLAP_DELTA_V: f32 = -0.5;

// Graphic Glyphs
const DRAGON_GLYPTH: i32 = 64;
const WALL_GLYPH: i32 = 179;
const GROUND_GLPYH: i32 = 35;

// Wall Gap
const GAP_Y_MIN: i32 = 5;
const GAP_Y_MAX: i32 = 20;

struct Player {
    x: i32,
    y: f32,
    velocity: f32,
}

impl Player {
    fn new(x: i32, y: i32) -> Self {
        Self {
            x,
            y: y as f32,
            velocity: 0.0,
        }
    }

    fn render(&mut self, ctx: &mut BTerm) {
        ctx.set_active_console(1);
        ctx.cls();
        ctx.set_fancy(
            PointF::new(0.0, self.y),
            1,
            Degrees::new(0.0),
            PointF::new(2.0, 2.0),
            YELLOW,
            NAVY,
            DRAGON_GLYPTH,
        );
        ctx.set_active_console(0);
    }

    fn gravity_and_move(&mut self) {
        // increasing velocity from "gravity" & terminal vel.
        if self.velocity < TERMINAL_VELOCITY {
            self.velocity += DELTA_V;
        }

        // Modifying player position.
        self.y += self.velocity;
        self.x += 1;

        // Upper bound for vertical position.
        if self.y < 0.0 {
            self.y = 0.0;
        }
    }

    fn flap(&mut self) {
        self.velocity = FLAP_DELTA_V;
    }
}

struct Obstacle {
    /// World position
    x: i32,
    /// Center of gap in wall
    gap_y: i32,
    /// Width of gap
    size: i32,
}

impl Obstacle {
    fn new(x: i32, score: i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        Self {
            x,
            gap_y: random.range(GAP_Y_MIN, GAP_Y_MAX),
            size: i32::max(2, 20 - score),
        }
    }

    fn render(&mut self, ctx: &mut BTerm, player_x: i32) {
        let screen_x = self.x - player_x;
        let half_size = self.size / 2;

        // Draw the top half of the obstacle
        for y in 0..self.gap_y - half_size {
            ctx.set(screen_x, y, GRAY, NAVY, WALL_GLYPH)
        }

        // Draw the bottom half of the obstacle
        for y in self.gap_y + half_size..SCREEN_HEIGHT {
            ctx.set(screen_x, y, GRAY, NAVY, WALL_GLYPH)
        }
    }

    fn hit_obstacle(&self, player: &Player) -> bool {
        let half_size = self.size / 2;
        let does_x_match = player.x == self.x;
        let player_above_gap = player.y < (self.gap_y - half_size) as f32;
        let player_below_gap = player.y > (self.gap_y + half_size) as f32;
        does_x_match && (player_above_gap || player_below_gap)
    }
}

struct State {
    player: Player,
    frame_time: f32,
    mode: GameMode,
    score: i32,
    obstacle: Obstacle,
}

impl State {
    /// Initialize new game state.
    fn new() -> Self {
        Self {
            player: Player::new(5, SCREEN_HEIGHT / 2),
            frame_time: 0.0,
            mode: GameMode::Menu,
            score: 0,
            obstacle: Obstacle::new(SCREEN_WIDTH, 0),
        }
    }

    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(NAVY);
        self.frame_time += ctx.frame_time_ms;
        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0;

            self.player.gravity_and_move();
        }

        // If the space key has been pressed this frame, flap.
        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.flap();
        }
        self.player.render(ctx);

        self.obstacle.render(ctx, self.player.x);
        if self.player.x > self.obstacle.x {
            self.score += 1;
            self.obstacle = Obstacle::new(self.player.x + SCREEN_WIDTH, self.score);
        }
        // Print controls and score.
        ctx.print_color(0, 0, CYAN, NAVY, "Press SPACE to flap.");
        ctx.print_color(0, 1, MAGENTA, NAVY, &format!("Score: {}", self.score));

        render_land(ctx);

        // SCREEN_HEIGHT - 1 to account for "ground"
        if self.player.y as i32 > (SCREEN_HEIGHT - 1) || self.obstacle.hit_obstacle(&self.player) {
            self.mode = GameMode::End;
        }
    }

    fn restart(&mut self) {
        self.player = Player::new(5, SCREEN_HEIGHT / 2);
        self.frame_time = 0.0;
        self.mode = GameMode::Playing;
        self.score = 0;
        self.obstacle = Obstacle::new(SCREEN_WIDTH, 0);
    }

    fn main_menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_color_centered(5, YELLOW, BLACK, "Welcome to Flappy Dragon");
        ctx.print_color_centered(8, CYAN, BLACK, "(P) Play Game");
        ctx.print_color_centered(9, CYAN, BLACK, "(Q) Quit Game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {} // do nothing
            }
        } // else do nothing
    }

    fn dead(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_color_centered(5, RED, BLACK, "You are dead!");
        ctx.print_centered(6, &format!("You earned {} points", self.score));
        ctx.print_color_centered(8, CYAN, BLACK, "(P) Play Again");
        ctx.print_color_centered(9, CYAN, BLACK, "(Q) Quit Game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {} // do nothing
            }
        } // else do nothing
    }
}

fn render_land(ctx: &mut BTerm) {
    for x in 0..SCREEN_WIDTH {
        ctx.set(x, SCREEN_HEIGHT - 1, WHITE, NAVY, GROUND_GLPYH);
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::End => self.dead(ctx),
            GameMode::Playing => self.play(ctx),
        }
    }
}

fn main() -> BError {
    let context = BTermBuilder::new()
        .with_font("../resources/flappy32.png", 32, 32)
        .with_simple_console(SCREEN_WIDTH, SCREEN_HEIGHT, "../resources/flappy32.png")
        .with_fancy_console(SCREEN_WIDTH, SCREEN_HEIGHT, "../resources/flappy32.png")
        .with_title("Flappy Dragon Enhanced")
        .build()?;

    main_loop(context, State::new())
}
