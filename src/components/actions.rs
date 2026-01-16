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
