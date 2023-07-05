#![allow(unused)] // silence unused warnings while exploring (to comment out)

use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;
use bevy::window::PrimaryWindow;
use game_menu::splash::*;
use game_menu::menu::*;
use game_menu::game::*;
use crate::game::*;
use crate::game_menu::*;

mod components;
mod enemy;
mod player;
mod game_menu;
mod game;
mod state;


pub(crate) fn setup_window(app: &mut App) {
    app
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rust Invaders!".into(),
                resolution: (598., 676.).into(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .insert_resource(game_menu::DisplayQuality::Medium);
}


fn main() {
    let mut app = App::new();
	setup_window(&mut app);
	setup_menu(&mut app);
	setup_game(&mut app);
	app.run();
}