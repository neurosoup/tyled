use crate::prelude::*;
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

#[derive(Debug, PartialEq, Clone, Reflect, Default)]
#[reflect(Default)]
pub enum Direction {
    #[default]
    Down,
    Up,
    Left,
    Right,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
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
            println!("LookDirection {:?}", vec);
            let new_direction = match vec {
                Vec2::Y => Some(Direction::Up),
                Vec2::NEG_Y => Some(Direction::Down),
                Vec2::X => Some(Direction::Right),
                Vec2::NEG_X => Some(Direction::Left),
                Vec2 { x: 1.0, y: 1.0 } => match self.direction {
                    Some(Direction::Down) => Some(Direction::Up),
                    Some(Direction::Left) => Some(Direction::Right),
                    _ => None,
                },
                Vec2 { x: -1.0, y: -1.0 } => match self.direction {
                    Some(Direction::Up) => Some(Direction::Down),
                    Some(Direction::Right) => Some(Direction::Left),
                    _ => None,
                },
                Vec2 { x: 1.0, y: -1.0 } => match self.direction {
                    Some(Direction::Up) => Some(Direction::Down),
                    Some(Direction::Left) => Some(Direction::Right),
                    _ => None,
                },
                Vec2 { x: -1.0, y: 1.0 } => match self.direction {
                    Some(Direction::Down) => Some(Direction::Up),
                    Some(Direction::Right) => Some(Direction::Left),
                    _ => None,
                },
                _ => None,
            };

            // If we do not have a new direction then do not change the direction
            if new_direction.is_none() {
                return;
            }

            if self.direction != new_direction {
                self.direction = new_direction;
                println!("LookDirection changed to {:?}", self.direction);
            }
        }
    }
}
