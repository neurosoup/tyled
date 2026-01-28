use crate::prelude::*;
use bevy::prelude::*;
use bevy_spritesheet_animation::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.add_plugins(SpritesheetAnimationPlugin);
    app.add_systems(PreUpdate, attach_player_animations);
    app.add_systems(Update, update_player_animation);
}

#[derive(Resource, Clone)]
struct PlayerOneAnimations {
    idle_x: Handle<Animation>,
    idle_up: Handle<Animation>,
    idle_down: Handle<Animation>,
}

#[derive(Resource, Clone)]
struct PlayerTwoAnimations {
    idle_x: Handle<Animation>,
    idle_up: Handle<Animation>,
    idle_down: Handle<Animation>,
}

fn update_player_animation(
    players: Query<(
        &Player,
        &LookDirection,
        &mut Sprite,
        &mut SpritesheetAnimation,
    )>,
    player_one_animations: If<Res<PlayerOneAnimations>>,
    player_two_animations: If<Res<PlayerTwoAnimations>>,
) {
    for (player, look_direction, mut sprite, mut animation) in players {
        match player.player_id {
            0 => match look_direction.direction {
                Some(Direction::Up) => {
                    animation.switch(player_one_animations.idle_up.clone());
                }
                Some(Direction::Down) => {
                    animation.switch(player_one_animations.idle_down.clone());
                }
                Some(Direction::Left) => {
                    animation.switch(player_one_animations.idle_x.clone());
                    sprite.flip_x = true;
                }
                Some(Direction::Right) => {
                    animation.switch(player_one_animations.idle_x.clone());
                    sprite.flip_x = false;
                }
                None => {}
            },
            1 => match look_direction.direction {
                Some(Direction::Up) => {
                    animation.switch(player_two_animations.idle_up.clone());
                }
                Some(Direction::Down) => {
                    animation.switch(player_two_animations.idle_down.clone());
                }
                Some(Direction::Left) => {
                    animation.switch(player_two_animations.idle_x.clone());
                    sprite.flip_x = true;
                }
                Some(Direction::Right) => {
                    animation.switch(player_two_animations.idle_x.clone());
                    sprite.flip_x = false;
                }
                None => {}
            },
            _ => panic!("Invalid player ID"),
        }
    }
}

fn attach_player_animations(
    mut commands: Commands,
    mut animations: ResMut<Assets<Animation>>,
    players: Query<(Entity, &Player, &Sprite), Added<Player>>,
) {
    for (entity, player, sprite) in &players {
        let spritesheet = Spritesheet::new(&sprite.image, 8, 1);
        let animation_builder = spritesheet.create_animation();

        let idle_x_animation_builder = animation_builder.clone();
        let idle_x_animation_handle = match player.player_id {
            0 => animations.add(idle_x_animation_builder.add_cell(3, 0).build()),
            1 => animations.add(idle_x_animation_builder.add_cell(7, 0).build()),
            _ => panic!("Invalid player ID"),
        };

        let idle_down_animation_builder = animation_builder.clone();
        let idle_down_animation_handle = match player.player_id {
            0 => animations.add(idle_down_animation_builder.add_cell(0, 0).build()),
            1 => animations.add(idle_down_animation_builder.add_cell(4, 0).build()),
            _ => panic!("Invalid player ID"),
        };

        let idle_up_animation_builder = animation_builder.clone();
        let idle_up_animation_handle = match player.player_id {
            0 => animations.add(idle_up_animation_builder.add_cell(2, 0).build()),
            1 => animations.add(idle_up_animation_builder.add_cell(6, 0).build()),
            _ => panic!("Invalid player ID"),
        };

        if player.player_id == 0 {
            println!("Player 1 animations initialized");
            commands.insert_resource(PlayerOneAnimations {
                idle_x: idle_x_animation_handle.clone(),
                idle_down: idle_down_animation_handle,
                idle_up: idle_up_animation_handle,
            });
        } else if player.player_id == 1 {
            println!("Player 2 animations initialized");
            commands.insert_resource(PlayerTwoAnimations {
                idle_x: idle_x_animation_handle.clone(),
                idle_down: idle_down_animation_handle,
                idle_up: idle_up_animation_handle,
            });
        } else {
            panic!("Invalid player ID");
        }

        commands
            .entity(entity)
            .insert(SpritesheetAnimation::new(idle_x_animation_handle));
    }
}
