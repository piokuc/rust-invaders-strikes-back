
use std::collections::HashSet;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;
use bevy::window::PrimaryWindow;
use crate::enemy::EnemyPlugin;
use crate::player::PlayerPlugin;
use crate::components::{
    Enemy, Explosion, ExplosionTimer, ExplosionToSpawn, FromEnemy, FromPlayer, Laser, Movable,
    Player, SpriteSize, Velocity,
};
use crate::state::GameState;
// region:    --- Asset Constants

pub(crate) const PLAYER_SPRITE: &str = "player_a_01.png";
pub(crate) const PLAYER_SIZE: (f32, f32) = (144., 75.);
pub(crate) const PLAYER_LASER_SPRITE: &str = "laser_a_01.png";
pub(crate) const PLAYER_LASER_SIZE: (f32, f32) = (9., 54.);

pub(crate) const ENEMY_SPRITE: &str = "enemy_a_01.png";
pub(crate) const ENEMY_SIZE: (f32, f32) = (144., 75.);
pub(crate) const ENEMY_LASER_SPRITE: &str = "laser_b_01.png";
pub(crate) const ENEMY_LASER_SIZE: (f32, f32) = (17., 55.);

pub(crate) const EXPLOSION_SHEET: &str = "explo_a_sheet.png";
pub(crate) const EXPLOSION_LEN: usize = 16;

pub(crate) const SPRITE_SCALE: f32 = 0.5;

pub(crate) const SCOREBOARD_FONT_SIZE: f32 = 40.0;
pub(crate) const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);
pub(crate) const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
pub(crate) const TEXT_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);

// endregion: --- Asset Constants

// region:    --- Game Constants

pub(crate) const TIME_STEP: f32 = 1. / 60.;
pub(crate) const BASE_SPEED: f32 = 500.;

pub(crate) const PLAYER_RESPAWN_DELAY: f64 = 2.;
pub(crate) const ENEMY_MAX: u32 = 2;
pub(crate) const FORMATION_MEMBERS_MAX: u32 = 2;

// endregion: --- Game Constants

// region:    --- Resources
#[derive(Resource)]
pub struct WinSize {
    pub w: f32,
    pub h: f32,
}

#[derive(Resource)]
pub(crate) struct GameTextures {
    pub player: Handle<Image>,
    pub player_laser: Handle<Image>,
    pub enemy: Handle<Image>,
    pub enemy_laser: Handle<Image>,
    pub explosion: Handle<TextureAtlas>,
}

#[derive(Resource)]
pub(crate) struct EnemyCount(pub u32);

#[derive(Resource)]
pub(crate) struct PlayerState {
    pub on: bool,       // alive
    pub last_shot: f64, // -1 if not shot
}
impl Default for PlayerState {
    fn default() -> Self {
        Self {
            on: false,
            last_shot: -1.,
        }
    }
}

impl PlayerState {
    pub fn shot(&mut self, time: f64) {
        self.on = false;
        self.last_shot = time;
    }
    pub fn spawned(&mut self) {
        self.on = true;
        self.last_shot = -1.;
    }
}

// This resource tracks the game's score
#[derive(Resource)]
struct Scoreboard {
    score: usize,
}
#[derive(Component)]
struct ScoreText;

fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text, With<ScoreText>>) {
    let mut text = query.single_mut();
    text.sections[1].value = scoreboard.score.to_string();
}

// endregion: --- Resources


fn movable_system(
    mut commands: Commands,
    win_size: Res<WinSize>,
    mut query: Query<(Entity, &Velocity, &mut Transform, &Movable)>,
) {
    for (entity, velocity, mut transform, movable) in query.iter_mut() {
        let translation = &mut transform.translation;
        translation.x += velocity.x * TIME_STEP * BASE_SPEED;
        translation.y += velocity.y * TIME_STEP * BASE_SPEED;

        if movable.auto_despawn {
            // despawn when out of screen
            const MARGIN: f32 = 200.;
            if translation.y > win_size.h / 2. + MARGIN
                || translation.y < -win_size.h / 2. - MARGIN
                || translation.x > win_size.w / 2. + MARGIN
                || translation.x < -win_size.w / 2. - MARGIN
            {
                commands.entity(entity).despawn();
            }
        }
    }
}

#[allow(clippy::type_complexity)] // for the Query types.
fn player_laser_hit_enemy_system(
    mut commands: Commands,
    mut enemy_count: ResMut<EnemyCount>,
    mut scoreboard: ResMut<Scoreboard>,
    laser_query: Query<(Entity, &Transform, &SpriteSize), (With<Laser>, With<FromPlayer>)>,
    enemy_query: Query<(Entity, &Transform, &SpriteSize), With<Enemy>>,
) {
    let mut despawned_entities: HashSet<Entity> = HashSet::new();

    // iterate through the lasers
    for (laser_entity, laser_tf, laser_size) in laser_query.iter() {
        if despawned_entities.contains(&laser_entity) {
            continue;
        }

        let laser_scale = laser_tf.scale.xy();

        // iterate through the enemies
        for (enemy_entity, enemy_tf, enemy_size) in enemy_query.iter() {
            if despawned_entities.contains(&enemy_entity)
                || despawned_entities.contains(&laser_entity)
            {
                continue;
            }

            let enemy_scale = enemy_tf.scale.xy();

            // determine if collision
            let collision = collide(
                laser_tf.translation,
                laser_size.0 * laser_scale,
                enemy_tf.translation,
                enemy_size.0 * enemy_scale,
            );

            // perform collision
            if collision.is_some() {
                // remove the enemy
                commands.entity(enemy_entity).despawn();
                despawned_entities.insert(enemy_entity);
                enemy_count.0 -= 1;

                // remove the laser
                commands.entity(laser_entity).despawn();
                despawned_entities.insert(laser_entity);

                // spawn the explosionToSpawn
                commands.spawn(ExplosionToSpawn(enemy_tf.translation));

                scoreboard.score += 1;
            }
        }
    }
}

#[allow(clippy::type_complexity)] // for the Query types.
fn enemy_laser_hit_player_system(
    mut commands: Commands,
    mut player_state: ResMut<PlayerState>,
    time: Res<Time>,
    mut scoreboard: ResMut<Scoreboard>,
    laser_query: Query<(Entity, &Transform, &SpriteSize), (With<Laser>, With<FromEnemy>)>,
    player_query: Query<(Entity, &Transform, &SpriteSize), With<Player>>,
) {
    if let Ok((player_entity, player_tf, player_size)) = player_query.get_single() {
        let player_scale = player_tf.scale.xy();

        for (laser_entity, laser_tf, laser_size) in laser_query.iter() {
            let laser_scale = laser_tf.scale.xy();

            // determine if collision
            let collision = collide(
                laser_tf.translation,
                laser_size.0 * laser_scale,
                player_tf.translation,
                player_size.0 * player_scale,
            );

            // perform the collision
            if collision.is_some() {
                // remove the player
                commands.entity(player_entity).despawn();
                player_state.shot(time.elapsed_seconds_f64());

                // remove the laser
                commands.entity(laser_entity).despawn();

                // spawn the explosionToSpawn
                commands.spawn(ExplosionToSpawn(player_tf.translation));

                scoreboard.score = 0;

                break;
            }
        }
    }
}

fn explosion_to_spawn_system(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    query: Query<(Entity, &ExplosionToSpawn)>,
) {
    for (explosion_spawn_entity, explosion_to_spawn) in query.iter() {
        // spawn the explosion sprite
        commands
            .spawn(SpriteSheetBundle {
                texture_atlas: game_textures.explosion.clone(),
                transform: Transform {
                    translation: explosion_to_spawn.0,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Explosion)
            .insert(ExplosionTimer::default());

        // despawn the explosionToSpawn
        commands.entity(explosion_spawn_entity).despawn();
    }
}

fn explosion_animation_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ExplosionTimer, &mut TextureAtlasSprite), With<Explosion>>,
) {
    for (entity, mut timer, mut sprite) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            sprite.index += 1; // move to next sprite cell
            if sprite.index >= EXPLOSION_LEN {
                commands.entity(entity).despawn()
            }
        }
    }
}


fn setup_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    query: Query<&Window, With<PrimaryWindow>>,
) {
    // camera
    commands.spawn(Camera2dBundle::default());

    // capture window size
    let Ok(primary) = query.get_single() else {
        return;
    };
    let (win_w, win_h) = (primary.width(), primary.height());

    // position window (for tutorial)
    // window.set_position(IVec2::new(2780, 4900));

    // add WinSize resource
    let win_size = WinSize { w: win_w, h: win_h };
    commands.insert_resource(win_size);

    // create explosion texture atlas
    let texture_handle = asset_server.load(EXPLOSION_SHEET);
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(64., 64.), 4, 4, None, None);
    let explosion = texture_atlases.add(texture_atlas);

    // add GameTextures resource
    let game_textures = GameTextures {
        player: asset_server.load(PLAYER_SPRITE),
        player_laser: asset_server.load(PLAYER_LASER_SPRITE),
        enemy: asset_server.load(ENEMY_SPRITE),
        enemy_laser: asset_server.load(ENEMY_LASER_SPRITE),
        explosion,
    };
    commands.insert_resource(game_textures);
    commands.insert_resource(EnemyCount(0));
    // Scoreboard
    commands.spawn((
        // Create a TextBundle that has a Text with a list of sections.
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
                TextStyle { font: asset_server.load("fonts/FiraSans-Bold.ttf"), font_size: 60.0, color: Color::WHITE, },
            ),
            TextSection::from_style(TextStyle { font: asset_server.load("fonts/FiraMono-Medium.ttf"), font_size: 60.0, color: Color::GOLD, }),
        ]), //.in_set(OnEnter(GameState::Game)),
        ScoreText,
    ));
}


pub(crate) fn setup_game(app: &mut App) {
    app
        .add_plugin(PlayerPlugin)
        .add_plugin(EnemyPlugin)
        .insert_resource(Scoreboard { score: 0 })
        .add_startup_system(setup_system)
        .add_system(movable_system)
        .add_system(player_laser_hit_enemy_system)
        .add_system(enemy_laser_hit_player_system)
        .add_system(explosion_to_spawn_system)
        .add_system(explosion_animation_system)
        .add_system(update_scoreboard);
}
