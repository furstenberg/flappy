#![warn(clippy::pedantic)]

use bracket_lib::prelude::*;

enum GameMode {
    Menu,
    Playing,
    End,
}

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const FRAME_DURATION: f32 = 75.0;

struct Player {
    x: f32,
    y: f32,
    velocity: f32,
}

impl Player {
    fn new(x: f32, y: f32) -> Self {
        Player {
            x,
            y,
            velocity: 0.0,
        }
    }

    fn render(&mut self, ctx: &mut BTerm) {
        ctx.set(0, self.y as i32, YELLOW, BLACK, to_cp437('@'));
    }

    fn gravity_and_move(&mut self) {
        if self.velocity < 2.0 {
            self.velocity += 0.35;
        }

        self.y += self.velocity;
        self.x += 1.0;

        if self.y < 0.0 {
            self.y = 0.0;
        }
    }

    fn flap(&mut self) {
        self.velocity = -2.0;
    }
}

struct Obstacle {
    x: f32,
    gap_y: f32,
    size: i32,
}

impl Obstacle {
    fn new(x: f32, score: i32) -> Self {
        let mut random = RandomNumberGenerator::new();

        Obstacle {
            x,
            gap_y: random.range(10.0, 40.0),
            size: i32::max(2, 20 - score),
        }
    }

    fn render(&mut self, ctx: &mut BTerm, player_x: f32) {
        let screen_x = self.x - player_x;
        let half_size = self.size / 2;

        // Draw the top half of the obstacle
        for y in 0..self.gap_y as i32 - half_size {
            ctx.set(screen_x as i32, y as i32, RED, BLACK, to_cp437('|'));
        }

        // Draw the bottom half of the obstacle
        for y in self.gap_y as i32 + half_size..SCREEN_HEIGHT {
            ctx.set(screen_x as i32, y as i32, RED, BLACK, to_cp437('|'));
        }
    }

    fn hit_obstacle(&self, player: &Player) -> bool {
        let half_size = self.size as f32 / 2.0;
        let does_x_match = player.x == self.x;
        let player_above_gap = player.y < self.gap_y - half_size;
        let player_below_gap = player.y > self.gap_y + half_size;

        does_x_match && (player_above_gap || player_below_gap)
    }
}

struct State {
    player: Player,
    frame_time: f32,
    obstacle1: Obstacle,
    obstacle2: Obstacle,
    mode: GameMode,
    score: i32,
}

impl State {
    fn new() -> Self {
        State {
            player: Player::new(5.0, 25.0),
            frame_time: 0.0,
            obstacle1: Obstacle::new(SCREEN_WIDTH as f32 / 2.0, 0),
            obstacle2: Obstacle::new(SCREEN_WIDTH as f32, 0),
            mode: GameMode::Menu,
            score: 0,
        }
    }

    fn play(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(NAVY);
        self.frame_time += ctx.frame_time_ms;

        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0;
            self.player.gravity_and_move();
        }

        if let Some(VirtualKeyCode::Space) = ctx.key {
            self.player.flap();
        }

        self.player.render(ctx);

        ctx.print(0, 0, "Press SPACE to flap.");
        ctx.print(0, 1, &format!("Score: {}", self.score));

        self.obstacle1.render(ctx, self.player.x);
        self.obstacle2.render(ctx, self.player.x);

        if self.player.x > self.obstacle1.x {
            self.score += 1;
            self.obstacle1 = Obstacle::new(self.player.x + SCREEN_WIDTH as f32, self.score);
        }

        if self.player.x > self.obstacle2.x {
            self.score += 1;
            self.obstacle2 = Obstacle::new(self.player.x + SCREEN_WIDTH as f32, self.score);
        }

        if self.player.y > SCREEN_HEIGHT as f32
            || self.obstacle1.hit_obstacle(&self.player)
            || self.obstacle2.hit_obstacle(&self.player)
        {
            self.mode = GameMode::End;
        }
    }

    fn restart(&mut self) {
        self.player = Player::new(5.0, 25.0);
        self.frame_time = 0.0;
        self.obstacle1 = Obstacle::new(SCREEN_WIDTH as f32 / 2.0, 0);
        self.obstacle2 = Obstacle::new(SCREEN_WIDTH as f32, 0);
        self.mode = GameMode::Playing;
        self.score = 0;
    }

    fn main_menu(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(
            3,
            "********************************************************************************",
        );
        ctx.print_centered(5, "Welcome to Flappy Rust");
        ctx.print_centered(
            7,
            "********************************************************************************",
        );

        ctx.print_centered(9, "(P) Play Game");
        ctx.print_centered(10, "(Q) Quit Game");

        ctx.print_centered(
            44,
            "********************************************************************************",
        );
        ctx.print_centered(46, "A game by Michael F\u{fc}rstenberg");
        ctx.print_centered(
            48,
            "********************************************************************************",
        );

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn dead(&mut self, ctx: &mut BTerm) {
        ctx.cls();
        ctx.print_centered(5, "You are dead!");
        ctx.print_centered(6, &format!("You earned {} points", self.score));
        ctx.print_centered(8, "(P) Play Again");
        ctx.print_centered(9, "(Q) Quit Game");

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::P => self.restart(),
                VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
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
    let context = BTermBuilder::simple80x50()
        .with_title("Flappy Rust")
        .build()?;

    main_loop(context, State::new())
}
