extern crate ggez;

use ggez::{Context, GameResult};
use ggez::conf::Conf;
use ggez::event;
use ggez::graphics;

struct MainState {
}

impl MainState {
    fn new(_ctx: &mut Context) -> GameResult<Self> {
        let s = MainState {
        };
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);
        Ok(())
    }
}

fn main() {
    let mut c = Conf::new();
    c.window_setup.title = "Turn Based Football".to_string();
    let mut ctx = Context::load_from_conf("tbf", "Jamie Nicol", c).unwrap();
    let mut state = MainState::new(&mut ctx).unwrap();
    event::run(&mut ctx, &mut state).unwrap();
}
