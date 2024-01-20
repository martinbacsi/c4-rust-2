use ggez::event;
use ggez::graphics::{self, Color, DrawMode, DrawParam, Font, Text};
use ggez::nalgebra as na;
use ggez::{Context, GameResult};
use std::thread;
use std::time::Duration;




use crate::game::Game;

/// White
pub const WHITE: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};

/// Black
pub const BLACK: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};



pub struct Viewer {
    game: Game,
}

impl Viewer {
    fn new(ctx: &mut Context) -> Viewer {
        //let font = Font::default();
        //let text = Text::new(("Hello, Rust!", font, 24.0));

        Viewer {
            game: Game::new(59)
        } 
    }
}


fn get_fish_color_from_value(color_value: i32) -> Color {
    match color_value {
        0 => Color::from_rgb(0xF0, 0x96, 0xC8), // Pink
        1 => Color::from_rgb(0xFF, 0xDC, 0x00), // Yellow
        2 => Color::from_rgb(0x00, 0xFF, 0x00), // Green
        3 => Color::from_rgb(0x00, 0x94, 0xFF), // Blue
        _ => WHITE, // Default to white for unknown values
    }
}

impl event::EventHandler for Viewer {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {



        graphics::clear(ctx, BLACK);
        eprintln!("{} {}",self.game.fishes[0].get_x(), self.game.fishes[0].get_y());
        for f in &self.game.fishes {
                // Draw a circle
            let circle = graphics::Mesh::new_circle(
                ctx,
                DrawMode::fill(),
                na::Point2::new((f.get_x() / 10.0) as f32, (f.get_y() / 10.0 ) as f32),
                15.0,
                1.0,
                get_fish_color_from_value(f.color),
            )?;
            graphics::draw(ctx, &circle, DrawParam::default())?;      
        }
        

        for f in &self.game.uglies {
                // Draw a circle
            let circle = graphics::Mesh::new_circle(
                ctx,
                DrawMode::fill(),
                na::Point2::new((f.get_x() / 10.0) as f32, (f.get_y() / 10.0 ) as f32),
                20.0,
                1.0,
                Color::from_rgb(0xEE, 0x00, 0x00),
            )?;
            graphics::draw(ctx, &circle, DrawParam::default())?;      
        }

        for p in &self.game.players{
            let player_color = if p.index == 0 { Color::from_rgb(0xff, 0x6d, 0x0a) } else {Color::from_rgb(0x95, 0x2E, 0x8F)};
            for d in &p.drones{
                let circle = graphics::Mesh::new_circle(
                    ctx,
                    DrawMode::fill(),
                    na::Point2::new((d.get_x() / 10.0) as f32, (d.get_y() / 10.0 ) as f32),
                    30.0,
                    1.0,
                   player_color
                )?;
                graphics::draw(ctx, &circle, DrawParam::default())?;      
            }
        }        

        // Draw the text
        //graphics::draw(ctx, &self.text, (na::Point2::new(50.0, 50.0), BLACK))?;

        graphics::present(ctx)?;
        self.game.perform_game_update(0);


        // Sleep for 1000 milliseconds (1 second)
        thread::sleep(Duration::from_millis(300));
        Ok(())
    }
}


pub fn run_visu() {
    let cb = ggez::ContextBuilder::new("my_game", "author")
        .window_setup(ggez::conf::WindowSetup::default().title("My Rust Game"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(1000.0, 1000.0));

    let (ctx, event_loop) = &mut cb.build().unwrap();
    let state = &mut Viewer::new(ctx);

    event::run(ctx, event_loop, state);
}