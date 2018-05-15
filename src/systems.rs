use std::collections::HashMap;

use conrod::{self, Labelable, Positionable, Sizeable, Widget};
use nalgebra::{Point2, Vector2};
use specs::{Entities, Fetch, FetchMut, Join, ReadStorage, System, WriteStorage};

use components::{Ball, BallState, CanMove, Cursor, CursorState, Direction, Player, PlayerState,
                 SubTilePosition, TilePosition};
use game::WidgetIds;
use resources::{DeltaTime, Input, Map, Turn, TurnState};

const CURSOR_SPEED: f32 = 320.0;
const PLAYER_SPEED: f32 = 640.0;
const PASS_SPEED: f32 = 960.0;
const TILE_SIZE: u32 = 64;
const PLAYER_MOVE_DISTANCE: u32 = 4;
const BALL_PASS_DISTANCE: u32 = 8;

fn tile_to_subtile(tile_pos: &Point2<u32>) -> Point2<f32> {
    Point2::new(
        (tile_pos.x * TILE_SIZE) as f32,
        (tile_pos.y * TILE_SIZE) as f32,
    )
}

fn get_input(input: &Input) -> Option<Direction> {
    if input.left && !input.right {
        Some(Direction::Left)
    } else if input.up && !input.down {
        Some(Direction::Up)
    } else if input.right && !input.left {
        Some(Direction::Right)
    } else if input.down && !input.up {
        Some(Direction::Down)
    } else {
        None
    }
}

fn vector_from_direction_i32(direction: &Direction) -> Vector2<i32> {
    match *direction {
        Direction::Left => Vector2::new(-1, 0),
        Direction::Up => Vector2::new(0, -1),
        Direction::Right => Vector2::new(1, 0),
        Direction::Down => Vector2::new(0, 1),
    }
}

fn vector_from_direction_f32(direction: &Direction) -> Vector2<f32> {
    match *direction {
        Direction::Left => Vector2::new(-1.0, 0.0),
        Direction::Up => Vector2::new(0.0, -1.0),
        Direction::Right => Vector2::new(1.0, 0.0),
        Direction::Down => Vector2::new(0.0, 1.0),
    }
}

fn required_time(displacement: f32, velocity: f32) -> f32 {
    if velocity != 0.0 {
        displacement / velocity
    } else {
        0.0
    }
}

pub struct CursorMovementSystem;

impl<'a> System<'a> for CursorMovementSystem {
    type SystemData = (
        Fetch<'a, DeltaTime>,
        Fetch<'a, Input>,
        WriteStorage<'a, Cursor>,
        WriteStorage<'a, TilePosition>,
        WriteStorage<'a, SubTilePosition>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (dt, input, mut cursors, mut tile_positions, mut sub_tile_positions) = data;

        for (cursor, tile_position, sub_tile_position) in
            (&mut cursors, &mut tile_positions, &mut sub_tile_positions).join()
        {
            let mut remaining_dt = dt.dt;

            while remaining_dt > 0.0 {
                if cursor.state == CursorState::Still {
                    // If we're still then the subtile position must be equal to the tile position
                    assert!(tile_to_subtile(&tile_position.pos) == sub_tile_position.pos);

                    if let Some(direction) = get_input(&input) {
                        let velocity =
                            vector_from_direction_f32(&direction).normalize() * CURSOR_SPEED;
                        let offset = vector_from_direction_i32(&direction);
                        // Can't move left or above 0, 0
                        if tile_position.pos.x as i32 + offset.x >= 0
                            && tile_position.pos.y as i32 + offset.y >= 0
                        {
                            let target = Point2::new(
                                (tile_position.pos.x as i32 + offset.x) as u32,
                                (tile_position.pos.y as i32 + offset.y) as u32,
                            );

                            cursor.state = CursorState::Moving { velocity, target };
                        }
                    }
                }

                if cursor.state == CursorState::Still {
                    // Exit the loop as we will not use the remaining time to move.
                    break;
                }

                if let CursorState::Moving { velocity, target } = cursor.state {
                    let disp = tile_to_subtile(&target) - sub_tile_position.pos;
                    let required_dt_x = required_time(disp.x, velocity.x);
                    let required_dt_y = required_time(disp.y, velocity.y);

                    let remaining_dt_x = remaining_dt.min(required_dt_x);
                    let remaining_dt_y = remaining_dt.min(required_dt_y);

                    sub_tile_position.pos +=
                        Vector2::new(velocity.x * remaining_dt_x, velocity.y * remaining_dt_y);
                    remaining_dt -= remaining_dt_x.max(remaining_dt_y);

                    if sub_tile_position.pos == tile_to_subtile(&target) {
                        tile_position.pos = target;
                        cursor.state = CursorState::Still;
                    }
                }
            }
        }
    }
}

pub struct PlayerSelectSystem;

impl<'a> System<'a> for PlayerSelectSystem {
    type SystemData = (
        Entities<'a>,
        Fetch<'a, Input>,
        FetchMut<'a, Turn>,
        ReadStorage<'a, Cursor>,
        ReadStorage<'a, TilePosition>,
        ReadStorage<'a, Player>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, input, mut turn, cursors, tile_positions, players) = data;

        for (_, cursor_pos) in (&cursors, &tile_positions).join() {
            if input.select {
                for (player_id, _, player_pos) in (&*entities, &players, &tile_positions).join() {
                    if player_pos.pos == cursor_pos.pos {
                        turn.state = TurnState::ActionMenu { player_id };
                        break;
                    }
                }
            }
        }
    }
}

fn get_adjacent_tiles(tile_pos: &Point2<u32>, map_size: &Vector2<u32>) -> Vec<Point2<u32>> {
    let mut tiles = Vec::new();
    if tile_pos.x >= 1 {
        tiles.push(Point2::new(tile_pos.x - 1, tile_pos.y));
    }
    if tile_pos.x < map_size.x - 1 {
        tiles.push(Point2::new(tile_pos.x + 1, tile_pos.y));
    }
    if tile_pos.y >= 1 {
        tiles.push(Point2::new(tile_pos.x, tile_pos.y - 1));
    }
    if tile_pos.y < map_size.y - 1 {
        tiles.push(Point2::new(tile_pos.x, tile_pos.y + 1));
    }

    tiles
}

fn calculate_run_targets<'a>(
    start_pos: &Point2<u32>,
    map: &Map,
    max_distance: u32,
    players: &ReadStorage<'a, Player>,
    tile_positions: &ReadStorage<'a, TilePosition>,
) -> Vec<Point2<u32>> {
    // targets that we can run to
    let mut targets: Vec<Point2<u32>> = Vec::new();
    targets.push(start_pos.clone());

    // tiles that still need to be searched. start with those adjacent to the start.
    let mut to_search: Vec<Point2<u32>> =
        get_adjacent_tiles(start_pos, &Vector2::new(map.map.width, map.map.height));

    // tiles that are just about to be or have already been searched,
    // and their cost for the path to the tile (but not including itself).
    // if the same tile is encountered with a lower cost then it should be searched again.
    let mut searched: HashMap<Point2<u32>, u32> = HashMap::new();

    // set the cost for the first tiles to 0
    searched.insert(*start_pos, 0);
    for tile in &to_search {
        searched.insert(*tile, 0);
    }

    while let Some(next) = to_search.pop() {
        let new_distance = searched[&next] + 1; // TODO change 1 to tile cost

        // if the distance is too far stop searching.
        // but don't rule it out if we find this tile through a shorter path.
        if new_distance > max_distance {
            continue;
        }

        // if the tile is already occupied then rule it out
        let occupied = (players, tile_positions)
            .join()
            .find(|&(_, pos)| pos.pos == next)
            .is_some();
        if occupied && next != *start_pos {
            continue;
        }

        // tile looks good
        if !targets.contains(&next) {
            targets.push(next);
        }

        // queue adjacent tiles to be searched
        for tile in get_adjacent_tiles(&next, &Vector2::new(map.map.width, map.map.height)) {
            let should_search = !searched.contains_key(&tile)
                || (searched.contains_key(&tile) && new_distance < searched[&tile]);
            if should_search {
                searched.insert(tile, new_distance);
                to_search.push(tile);
            }
        }
    }

    targets
}

fn calculate_pass_targets(
    start_pos: &Point2<u32>,
    map: &Map,
    max_distance: u32,
) -> Vec<Point2<u32>> {
    let mut targets: Vec<Point2<u32>> = Vec::new();

    for i in 1..max_distance {
        if start_pos.x >= i {
            targets.push(Point2::new(start_pos.x - i, start_pos.y));
        }
        if start_pos.x + i < map.map.width {
            targets.push(Point2::new(start_pos.x + i, start_pos.y));
        }
        if start_pos.y >= i {
            targets.push(Point2::new(start_pos.x, start_pos.y - i));
        }
        if start_pos.y + i < map.map.height {
            targets.push(Point2::new(start_pos.x, start_pos.y + i));
        }
    }

    targets
}

pub struct ActionMenuSystem<'a, 'b>
where
    'b: 'a,
{
    ui: &'a mut conrod::UiCell<'b>,
    widget_ids: &'a WidgetIds,
}

impl<'a, 'b> ActionMenuSystem<'a, 'b> {
    pub fn new(ui: &'a mut conrod::UiCell<'b>, widget_ids: &'a WidgetIds) -> Self {
        ActionMenuSystem { ui, widget_ids }
    }
}

impl<'a, 'b, 'c> System<'c> for ActionMenuSystem<'a, 'b> {
    type SystemData = (
        Entities<'c>,
        FetchMut<'c, Turn>,
        Fetch<'c, Input>,
        Fetch<'c, Map>,
        ReadStorage<'c, Player>,
        ReadStorage<'c, Ball>,
        ReadStorage<'c, TilePosition>,
        WriteStorage<'c, CanMove>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut turn, input, map, players, balls, tile_positions, mut can_moves) = data;

        if let TurnState::ActionMenu { player_id } = turn.state {
            if conrod::widget::Button::new()
                .label("Run")
                .top_right_with_margin_on(self.ui.window, 16.0)
                .w_h(160.0, 32.0)
                .set(self.widget_ids.action_menu_run, self.ui)
                .was_clicked() || input.select
            {
                let player_pos = tile_positions.get(player_id).unwrap().clone();
                let dests = calculate_run_targets(
                    &player_pos.pos,
                    &map,
                    PLAYER_MOVE_DISTANCE,
                    &players,
                    &tile_positions,
                );
                let can_move = CanMove {
                    start: player_pos.pos,
                    distance: PLAYER_MOVE_DISTANCE,
                    dests: dests,
                    path: Vec::new(),
                };
                can_moves.insert(player_id, can_move);
                turn.state = TurnState::SelectRun { player_id };
            }

            if conrod::widget::Button::new()
                .label("Pass")
                .down_from(self.widget_ids.action_menu_run, 16.0)
                .w_h(160.0, 32.0)
                .set(self.widget_ids.action_menu_pass, self.ui)
                .was_clicked()
            {
                for (ball_id, ball) in (&*entities, &balls).join() {
                    // Why can't I do this?
                    // if ball.state == BallState::Possessed { player_id }
                    if let BallState::Possessed {
                        player_id: possessed_by,
                    } = ball.state
                    {
                        if player_id == possessed_by {
                            let ball_pos = tile_positions.get(ball_id).unwrap().clone();
                            let dests =
                                calculate_pass_targets(&ball_pos.pos, &map, BALL_PASS_DISTANCE);
                            let can_move = CanMove {
                                start: ball_pos.pos,
                                distance: BALL_PASS_DISTANCE,
                                dests: dests,
                                path: Vec::new(),
                            };
                            can_moves.insert(ball_id, can_move);
                            turn.state = TurnState::SelectPass { player_id, ball_id };

                            // Can't pass more than one ball
                            break;
                        }
                    }
                }
            }

            if conrod::widget::Button::new()
                .label("Cancel")
                .down_from(self.widget_ids.action_menu_pass, 16.0)
                .w_h(160.0, 32.0)
                .set(self.widget_ids.action_menu_cancel, self.ui)
                .was_clicked() || input.cancel
            {
                turn.state = TurnState::SelectPlayer;
            }
        }
    }
}

pub struct RunSelectSystem;

impl<'a> System<'a> for RunSelectSystem {
    type SystemData = (
        Fetch<'a, Input>,
        FetchMut<'a, Turn>,
        WriteStorage<'a, CanMove>,
        ReadStorage<'a, Cursor>,
        ReadStorage<'a, TilePosition>,
        WriteStorage<'a, Player>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (input, mut turn, mut can_moves, cursors, tile_positions, mut players) = data;

        if let TurnState::SelectRun { player_id } = turn.state {
            // Find the player
            let player = players.get_mut(player_id).unwrap();

            for (cursor, cursor_pos) in (&cursors, &tile_positions).join() {
                if input.select {
                    if cursor.state == CursorState::Still {
                        let at_end_of_path = {
                            can_moves.get(player_id).unwrap().path.last() == Some(&cursor_pos.pos)
                        };
                        if at_end_of_path {
                            turn.state = TurnState::Running { player_id };
                            player.state = PlayerState::Running {
                                path: can_moves.get(player_id).unwrap().path.clone(),
                            };
                            can_moves.remove(player_id).unwrap();
                        }
                    }
                } else if input.cancel {
                    can_moves.remove(player_id).unwrap();
                    turn.state = TurnState::SelectPlayer;
                }
            }
        }
    }
}

pub struct PathSelectSystem;

impl<'a> System<'a> for PathSelectSystem {
    type SystemData = (
        Fetch<'a, Map>,
        ReadStorage<'a, Cursor>,
        ReadStorage<'a, TilePosition>,
        WriteStorage<'a, CanMove>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (map, cursors, tile_positions, mut can_moves) = data;

        for (_cursor, cursor_pos) in (&cursors, &tile_positions).join() {
            for (can_move,) in (&mut can_moves,).join() {
                if Some(&cursor_pos.pos) != can_move.path.last() {
                    // If we're back on the start position clear the path
                    if cursor_pos.pos == can_move.start {
                        can_move.path.clear();
                    }

                    // If this is a valid move location
                    if can_move.dests.contains(&cursor_pos.pos) {
                        // If we cross our existing path then shrink back
                        if let Some((i, _)) = can_move
                            .path
                            .iter()
                            .enumerate()
                            .find(|&(_, &step)| step == cursor_pos.pos)
                        {
                            can_move.path.truncate(i + 1);
                        } else {
                            // Check we've only moved by 1 tile
                            if get_adjacent_tiles(
                                can_move.path.last().unwrap_or(&can_move.start),
                                &Vector2::new(map.map.width, map.map.height),
                            ).contains(&cursor_pos.pos)
                            {
                                // Check our path isn't too long
                                if can_move.path.len() <= can_move.distance as usize {
                                    can_move.path.push(cursor_pos.pos.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub struct PlayerMovementSystem;

impl<'a> System<'a> for PlayerMovementSystem {
    type SystemData = (
        Fetch<'a, DeltaTime>,
        FetchMut<'a, Turn>,
        WriteStorage<'a, Player>,
        WriteStorage<'a, TilePosition>,
        WriteStorage<'a, SubTilePosition>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (dt, mut turn, mut players, mut tile_positions, mut sub_tile_positions) = data;

        for (player, tile_position, sub_tile_position) in
            (&mut players, &mut tile_positions, &mut sub_tile_positions).join()
        {
            let mut finished_run = false;
            if let PlayerState::Running { ref mut path } = player.state {
                let mut remaining_dt = dt.dt;

                while !finished_run && remaining_dt > 0.0 {
                    let target = match path.first() {
                        None => break,
                        Some(target) => target.clone(),
                    };
                    let disp = tile_to_subtile(&target) - sub_tile_position.pos;

                    if disp != Vector2::new(0.0, 0.0) {
                        let velocity = disp.normalize() * PLAYER_SPEED;

                        let required_dt_x = required_time(disp.x, velocity.x);
                        let required_dt_y = required_time(disp.y, velocity.y);

                        let remaining_dt_x = remaining_dt.min(required_dt_x);
                        let remaining_dt_y = remaining_dt.min(required_dt_y);

                        sub_tile_position.pos +=
                            Vector2::new(velocity.x * remaining_dt_x, velocity.y * remaining_dt_y);
                        remaining_dt -= remaining_dt_x.max(remaining_dt_y);
                    }

                    if sub_tile_position.pos == tile_to_subtile(&target) {
                        tile_position.pos = target.clone();
                        path.remove(0);
                    }

                    if path.is_empty() {
                        finished_run = true;
                    }
                }
            }

            if finished_run {
                player.state = PlayerState::Still;
                turn.state = TurnState::SelectPlayer;
            }
        }
    }
}

pub struct BallDribbleSystem;

impl<'a> System<'a> for BallDribbleSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Ball>,
        ReadStorage<'a, Player>,
        WriteStorage<'a, TilePosition>,
        WriteStorage<'a, SubTilePosition>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut balls, players, mut tile_positions, mut sub_tile_positions) = data;

        for (ball_id, ball) in (&*entities, &mut balls).join() {
            match ball.state {
                BallState::Free => {
                    let ball_pos = tile_positions.get(ball_id).unwrap();

                    for (player_id, _) in (&*entities, &players).join() {
                        let player_pos = tile_positions.get(player_id).unwrap();
                        if ball_pos.pos == player_pos.pos {
                            ball.state = BallState::Possessed { player_id };
                        }
                    }
                }
                BallState::Possessed { player_id } => {
                    let player_tile_pos = tile_positions.get(player_id).unwrap().pos.clone();
                    let player_subtile_pos = sub_tile_positions.get(player_id).unwrap().pos.clone();

                    tile_positions.get_mut(ball_id).unwrap().pos = player_tile_pos;
                    sub_tile_positions.get_mut(ball_id).unwrap().pos = player_subtile_pos;
                }
                _ => {}
            }
        }
    }
}

pub struct PassSelectSystem;

impl<'a> System<'a> for PassSelectSystem {
    type SystemData = (
        Fetch<'a, Input>,
        FetchMut<'a, Turn>,
        WriteStorage<'a, CanMove>,
        ReadStorage<'a, Cursor>,
        ReadStorage<'a, TilePosition>,
        WriteStorage<'a, Ball>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (input, mut turn, mut can_moves, cursors, tile_positions, mut balls) = data;

        if let TurnState::SelectPass { player_id, ball_id } = turn.state {
            // Find the ball
            let ball = balls.get_mut(ball_id).unwrap();

            for (cursor, cursor_pos) in (&cursors, &tile_positions).join() {
                if input.select {
                    if cursor.state == CursorState::Still {
                        let at_end_of_path = {
                            can_moves.get(ball_id).unwrap().path.last() == Some(&cursor_pos.pos)
                        };
                        if at_end_of_path {
                            turn.state = TurnState::Passing { player_id, ball_id };
                            ball.state = BallState::Moving {
                                player_id,
                                path: can_moves.get(ball_id).unwrap().path.clone(),
                            };
                            can_moves.remove(ball_id).unwrap();
                        }
                    }
                } else if input.cancel {
                    can_moves.remove(ball_id).unwrap();
                    turn.state = TurnState::SelectPlayer;
                }
            }
        }
    }
}

// TODO: this should probably be combined with PlayerMovementSystem
pub struct BallMovementSystem;

impl<'a> System<'a> for BallMovementSystem {
    type SystemData = (
        Fetch<'a, DeltaTime>,
        FetchMut<'a, Turn>,
        WriteStorage<'a, Ball>,
        WriteStorage<'a, TilePosition>,
        WriteStorage<'a, SubTilePosition>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (dt, mut turn, mut balls, mut tile_positions, mut sub_tile_positions) = data;

        for (ball, tile_position, sub_tile_position) in
            (&mut balls, &mut tile_positions, &mut sub_tile_positions).join()
        {
            let mut finished_movement = false;
            if let BallState::Moving { ref mut path, .. } = ball.state {
                let mut remaining_dt = dt.dt;

                while !finished_movement && remaining_dt > 0.0 {
                    let target = match path.first() {
                        None => break,
                        Some(target) => target.clone(),
                    };
                    let disp = tile_to_subtile(&target) - sub_tile_position.pos;

                    if disp != Vector2::new(0.0, 0.0) {
                        let velocity = disp.normalize() * PASS_SPEED;

                        let required_dt_x = required_time(disp.x, velocity.x);
                        let required_dt_y = required_time(disp.y, velocity.y);

                        let remaining_dt_x = remaining_dt.min(required_dt_x);
                        let remaining_dt_y = remaining_dt.min(required_dt_y);

                        sub_tile_position.pos +=
                            Vector2::new(velocity.x * remaining_dt_x, velocity.y * remaining_dt_y);
                        remaining_dt -= remaining_dt_x.max(remaining_dt_y);
                    }

                    if sub_tile_position.pos == tile_to_subtile(&target) {
                        tile_position.pos = target.clone();
                        path.remove(0);
                    }

                    if path.is_empty() {
                        finished_movement = true;
                    }
                }
            }

            if finished_movement {
                ball.state = BallState::Free;
                turn.state = TurnState::SelectPlayer;
            }
        }
    }
}
