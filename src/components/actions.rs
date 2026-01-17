use crate::prelude::player::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum Action {
    #[actionlike(DualAxis)]
    Move,
    LockLeft,
    LockRight,
    LockUp,
    LockDown,
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

        input_map.insert(Action::LockLeft, KeyCode::KeyA);
        input_map.insert(Action::LockRight, KeyCode::KeyD);
        input_map.insert(Action::LockUp, KeyCode::KeyW);
        input_map.insert(Action::LockDown, KeyCode::KeyS);

        input_map
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockedDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Component)]
pub struct DirectionLockState {
    // Define how fast double-tapping the key can activate the lock
    pub activation_window: Timer,
    // Prevent normal movement just after locking (because we use the same key for direction locking and movement)
    pub cooldown: Timer,
    // True on the first tap of the key
    pub is_armed: bool,
    pub locked_direction: LockedDirection,
}

impl Default for DirectionLockState {
    fn default() -> Self {
        Self {
            activation_window: Timer::from_seconds(0.2, TimerMode::Once),
            cooldown: Timer::from_seconds(0.1, TimerMode::Once),
            is_armed: false,
            locked_direction: LockedDirection::Down,
        }
    }
}

impl DirectionLockState {
    pub fn direction(&mut self, direction: LockedDirection) {
        self.locked_direction = direction;
    }
}
