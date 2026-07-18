use std::collections::VecDeque;
use std::time::Duration;

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
                0 => KeyCode::Tab,
                1 => KeyCode::ShiftRight,
                _ => KeyCode::KeyQ,
            },
        );

        // Shoot action
        input_map.insert(
            Action::Shoot,
            match player.player_id {
                0 => KeyCode::KeyQ,
                1 => KeyCode::Slash,
                _ => KeyCode::Tab,
            },
        );

        input_map
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Reflect, Default)]
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

    pub fn lock(&mut self) {
        self.locked = true;
    }

    pub fn unlock(&mut self) {
        self.locked = false;
    }

    pub fn to_grid_coords(&self) -> GridCoords {
        match self.direction {
            Some(Direction::Up) => GridCoords::new(0, 1),
            Some(Direction::Down) => GridCoords::new(0, -1),
            Some(Direction::Left) => GridCoords::new(-1, 0),
            Some(Direction::Right) => GridCoords::new(1, 0),
            None => GridCoords::new(0, 0),
        }
    }

    pub fn would_look_at(&self, vec: Vec2) -> Option<Direction> {
        if self.locked {
            return None;
        }
        match vec {
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
        }
    }

    pub fn look_at(&mut self, vec: Vec2) {
        if let Some(new_direction) = self.would_look_at(vec) {
            self.direction = Some(new_direction);
        }
    }
}

pub const TURN_STEP_MS: u64 = 100;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TurnPose {
    Ne,
    Se,
    Sw,
    Nw,
}

#[derive(Component)]
pub struct IsTurning {
    pub timer: Timer,
    pub from: Direction,
    pub target: Direction,
    pub remaining: VecDeque<Direction>,
}

impl IsTurning {
    pub fn new(from: Direction, target: Direction) -> Self {
        Self {
            timer: Timer::new(Duration::from_millis(TURN_STEP_MS), TimerMode::Once),
            from,
            target,
            remaining: Self::path(from, target),
        }
    }

    fn path(from: Direction, target: Direction) -> VecDeque<Direction> {
        use Direction::*;
        let mut queue = VecDeque::new();
        if from == target {
            return queue;
        }
        // Clockwise cardinal indices: Up=0, Right=1, Down=2, Left=3.
        let index = |direction: Direction| match direction {
            Up => 0i32,
            Right => 1,
            Down => 2,
            Left => 3,
        };
        let steps = (index(target) - index(from)).rem_euclid(4);
        if steps == 2 {
            // 180°: route through a fixed middle cardinal (N<->S via East, W<->E via South).
            let middle = match (from, target) {
                (Up, Down) | (Down, Up) => Right,
                (Left, Right) | (Right, Left) => Down,
                _ => target,
            };
            queue.push_back(middle);
        }
        queue.push_back(target);
        queue
    }

    pub fn pose(&self) -> TurnPose {
        let next = self.remaining.front().copied().unwrap_or(self.target);
        Self::diagonal(self.from, next)
    }

    fn diagonal(a: Direction, b: Direction) -> TurnPose {
        use Direction::*;
        match (a, b) {
            (Up, Right) | (Right, Up) => TurnPose::Ne,
            (Right, Down) | (Down, Right) => TurnPose::Se,
            (Down, Left) | (Left, Down) => TurnPose::Sw,
            (Left, Up) | (Up, Left) => TurnPose::Nw,
            _ => TurnPose::Se,
        }
    }
}
