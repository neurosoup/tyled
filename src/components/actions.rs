use crate::prelude::player::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum Action {
    #[actionlike(DualAxis)]
    Move,
    Lock,
    Shoot,
    Parry,
}
impl Action {
    pub fn default_input_map(player: &Player) -> InputMap<Action> {
        let mut input_map = InputMap::default();

        let default_wasd = VirtualDPad::wasd();

        // Move action
        input_map.insert_dual_axis(
            Action::Move,
            match player.player_id {
                0 => default_wasd,
                1 => VirtualDPad::arrow_keys(),
                _ => VirtualDPad::wasd(),
            },
        );

        // Lock action
        input_map.insert(
            Action::Lock,
            match player.player_id {
                0 => KeyCode::KeyQ,
                1 => KeyCode::ShiftRight,
                _ => KeyCode::KeyQ,
            },
        );

        input_map
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Component)]
pub struct LookDirection {
    pub direction: Option<Direction>,
    pub locked: bool,
}

impl Default for LookDirection {
    fn default() -> Self {
        Self {
            direction: None,
            locked: false,
        }
    }
}

impl LookDirection {
    pub fn new(direction: Direction) -> Self {
        Self {
            direction: Some(direction),
            locked: false,
        }
    }

    pub fn toggle_lock(&mut self) {
        self.locked = !self.locked;
    }

    pub fn look_at(&mut self, vec: Vec2) {
        if !self.locked {
            let new_direction = match vec {
                Vec2::Y => Direction::Up,
                Vec2::NEG_Y => Direction::Down,
                Vec2::X => Direction::Right,
                Vec2::NEG_X => Direction::Left,
                _ => Direction::Up,
            };

            let should_update = match &self.direction {
                Some(current_direction) => new_direction != *current_direction,
                None => true,
            };
            if should_update {
                self.direction = Some(new_direction);
                println!("LookDirection changed to {:?}", self.direction);
            }
        }
    }
}
