#![allow(dead_code)]
#![allow(unused)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(unused_import_braces)]

use std::char::MAX;
use std::cmp;
use std::{collections::HashMap, time::Duration};
use bevy::prelude::*;
use bevy::text::FontSmoothing;
use bevy::{input::keyboard::Key, time::Stopwatch};
use rand::Rng;

// =============================================
// GAMESTATE
// =============================================
const MAX_MISTAKES : i32 = 10;

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum GameState
{
    Begin,
    Round,
    GameOver
}

#[derive(Resource)]
struct GameManager
{
    time_since_round_start : Stopwatch,
    curr_state: GameState,
    moles_hit: i32,
    moles_missed: i32,
}

impl GameManager
{
    pub fn new() -> Self
    {
        Self
        {
            time_since_round_start: Stopwatch::new(),
            curr_state: GameState::Begin,
            moles_hit: 0,
            moles_missed: 0
        }
    }

    fn start_round(&mut self)
    {
        self.curr_state = GameState::Round;
        self.time_since_round_start.reset();
        self.moles_hit = 0;
        self.moles_missed = 0;
    }

    fn game_over(&mut self)
    {
        self.curr_state = GameState::GameOver;
    }

    fn get_curr_health(&self) -> i32
    {
        return cmp::max(0, MAX_MISTAKES - self.moles_missed);
    }
}

impl Default for GameManager
{
    fn default() -> Self
    {
        return GameManager::new();
    }
}

fn update_gamemanager(time: Res<Time>, mut manager: ResMut<GameManager>)
{
    match manager.curr_state 
    {
        GameState::Begin =>
        {

        }
        GameState::Round =>
        {
            manager.time_since_round_start.tick(time.delta());
        }
        GameState::GameOver =>
        {

        }
    }
}

#[derive(Component)]
struct Healthbar;

fn update_healthbar(game_manager: Res<GameManager>,
            mut healthbar: Query<(&mut Healthbar, &mut Children)>, 
            mut hb_sprites: Query<&mut Sprite>)
{
    for (mut healthbar, mut children) in &mut healthbar
    {
        for (i, child) in children.iter().enumerate()
        {
            if let Ok(mut sprite) = hb_sprites.get_mut(*child)
            {
                println!("AAA");
                if let Some(atlas) = &mut sprite.texture_atlas
                {
                    let alive : bool = (i as i32) < game_manager.get_curr_health();
                    atlas.index = if alive { 0 } else { 1 };
                }
                else
                {
                    panic!();
                }
            }
        }
    }
}

// =============================================
// ANIMATION
// =============================================
#[derive(Clone, Debug)]
struct SpriteAnimation
{
    anim_frames: Vec<usize>,
    curr_frame_idx: usize,
    timer: Timer
}

impl SpriteAnimation
{
    fn new(anim_frames: &Vec<usize>, frame_len: f32) -> Self
    {
        Self
        {
            anim_frames: anim_frames.clone(),
            curr_frame_idx: 0,
            timer: Timer::from_seconds(frame_len, TimerMode::Repeating),
        }
    }

    fn is_finished(&self) -> bool
    {
        return self.curr_frame_idx >= self.anim_frames.len();
    }
}

#[derive(Component)]
struct SpriteAnimator
{
    animations: HashMap<String, SpriteAnimation>,
    curr_playing_idx: Option<String>
}

impl SpriteAnimator
{
    fn new() -> Self
    {
        Self
        {
            animations: HashMap::new(),
            curr_playing_idx: None
        }
    }

    fn push_anim(&mut self, name: &str, anim: SpriteAnimation)
    {
        self.animations.insert(String::from(name), anim);
    }

    fn play_anim(&mut self, name: &str)
    {
        self.curr_playing_idx = Some(String::from(name));

        let anim = Self::get_curr_anim(self).expect("Invalid animation.");
        anim.curr_frame_idx = 0;
    }

    fn get_curr_anim(&mut self) -> Option<&mut SpriteAnimation>
    {
        if self.curr_playing_idx.is_none()
        {
            return None;
        }

        let string_ref = self.curr_playing_idx.as_ref().unwrap();
        let query = self.animations.get_mut(string_ref);

        return query;
    }
}

fn animate_sprite(time: Res<Time>, mut query: Query<(&mut SpriteAnimator, &mut Sprite)>)
{
    for (mut animator, mut sprite) in &mut query
    {
        let mut anim_finished = false;
        if let Some(animation) = animator.get_curr_anim()
        {
            // Update sprite
            if let Some(atlas) = &mut sprite.texture_atlas
            {
                atlas.index = animation.anim_frames[animation.curr_frame_idx];
            }
            else
            {
                panic!();
            }

            // Update anim
            animation.timer.tick(time.delta());

            // Advance frame by 1
            if animation.timer.just_finished()
            {
                animation.curr_frame_idx += 1;

                if animation.is_finished()
                {
                    animation.curr_frame_idx = 0;
                    anim_finished = true;
                }
            }
        }

        if anim_finished
        {
            animator.curr_playing_idx = None;
        }
    }
}


// =============================================
// MOLE
// =============================================
const MOLE_HIDE_ANIM: &str = "MoleHide";
const MOLE_RISE_ANIM: &str = "MoleUp";
const MOLE_BONK_ANIM: &str = "MoleBonk";

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum MoleState
{
    Hidden,
    HeadUp,
    Bonked
}

#[derive(Component)]
struct Mole
{
    kill_key: KeyCode,
    status: MoleState,
    timer: Timer
}

impl Mole
{
    fn new(key: KeyCode) -> Self
    {
        let mut new_mole = Self
        {
            kill_key: key,
            status: MoleState::Hidden,
            timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating)
        };

        new_mole.reset_mole_time();

        return new_mole;
    }

    fn reset_mole_time(&mut self)
    {
        let mut rng = rand::thread_rng();
        let duration = match self.status
        {
            MoleState::Hidden => rng.gen_range(4.0 .. 14.0),
            MoleState::HeadUp => rng.gen_range(2.0 .. 3.0),
            MoleState::Bonked => rng.gen_range(1.0 .. 2.0),
        };

        self.timer.set_duration(Duration::from_secs_f32(duration));
    }
}

fn update_moles(time: Res<Time>, 
    keys: Res<ButtonInput<KeyCode>>,
    mut manager: ResMut<GameManager>,
    mut query: Query<(&mut SpriteAnimator, &mut Mole)>)
{
    if manager.curr_state != GameState::Round
    {
        return;
    }

    let elapsed_sec = manager.time_since_round_start.elapsed_secs();
    let max_mole_up = (1.0 - (1.0 / (1.0 + elapsed_sec * 0.01))) * 18.0;
    let total_mole_up = query.iter().filter(|(a, m)| m.status == MoleState::HeadUp).count() as f32;

    for (mut animator, mut mole) in &mut query
    {
        mole.timer.tick(time.delta());
        let prev_state = mole.status;

        if keys.just_pressed(mole.kill_key) 
        {
            if mole.status == MoleState::HeadUp
            {
                animator.play_anim(MOLE_BONK_ANIM);
                mole.status = MoleState::Bonked;
                manager.moles_hit += 1;
            }
            else if mole.status == MoleState::Hidden
            {
                manager.moles_missed += 1;
            }
        }
        else if mole.timer.just_finished()
        {
            mole.status = match mole.status 
            {
                MoleState::Hidden => if total_mole_up < max_mole_up { MoleState::HeadUp } else { MoleState::Hidden },
                MoleState::HeadUp => MoleState::Hidden,
                MoleState::Bonked => MoleState::Hidden,
            };

            // Do random something here.
            mole.reset_mole_time();

            // Adjust time to make game increasingly difficult.
            let new_dur = mole.timer.duration();
            let diff_factor = 1.0 / (elapsed_sec * 0.02 + 2.0) + 0.5;
            mole.timer.set_duration(Duration::from_secs_f32(new_dur.as_secs_f32() * diff_factor));
        }

        if prev_state == MoleState::HeadUp && mole.status == MoleState::Hidden
        { 
            manager.moles_missed += 1;
        }

        if prev_state != mole.status && prev_state != MoleState::Bonked
        {
            match mole.status 
            {
                MoleState::HeadUp => animator.play_anim(MOLE_RISE_ANIM),
                MoleState::Hidden => animator.play_anim(MOLE_HIDE_ANIM),
                MoleState::Bonked => animator.play_anim(MOLE_BONK_ANIM),
            }
        }
    }

    println!("Score: {} | HP: {}", manager.moles_hit, manager.get_curr_health());

    if manager.get_curr_health() == 0
    {
        manager.game_over();
        for (mut animator, mut mole) in &mut query
        {
            if mole.status == MoleState::HeadUp
            {
                mole.status = MoleState::Hidden;
                animator.play_anim(MOLE_HIDE_ANIM);
            }
        }
    }
}

// =============================================
// MAIN
// =============================================
fn main()
{
    App::new()
        // Plugin
        .add_plugins((DefaultPlugins.set(ImagePlugin::default_nearest()), bevy_framepace::FramepacePlugin))

        // Startup
        .init_resource::<GameManager>()
        .add_systems(Startup, 
            setup)
        
        // Update
        .add_systems(Update, 
            (update_gamemanager,
                    animate_sprite,
                    update_healthbar,
                    update_moles))

        .run();
}

/// ===========================================
/// SETUP
/// ===========================================
fn setup(mut commands: Commands, 
    mut settings: ResMut<bevy_framepace::FramepaceSettings>,
    mut window: Single<&mut Window>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
    mut game_manager: ResMut<GameManager>)
{
    // Setup window
    use bevy_framepace::Limiter;
    settings.limiter = Limiter::from_framerate(60.0);

    window.resolution.set(1280.0, 720.0);
    window.resizable = false;
    window.name = Some(String::from("Whack-a-key"));
    window.title = String::from("Whack-a-key");

    commands.spawn(Camera2d::default());
    
    // Game BG
    commands.spawn(Sprite::from_image(asset_server.load("GameBG.png")));
    
    // Create moles
    create_all_moles(&mut commands, &mut texture_atlas_layouts, &asset_server);

    // Create healthbar
    let hearts_tex = asset_server.load("Hearts.png");
    let hearts_layout = TextureAtlasLayout::from_grid(UVec2::new(22, 16), 2, 1, Some(UVec2::new(1, 1)), None);
    let hearts_atlas_layout = texture_atlas_layouts.add(hearts_layout);

    let hb = commands.spawn((Healthbar, Transform::from_xyz(0.0, 260.0, 1.0))).id();

    for i in 0..MAX_MISTAKES
    {
        let mut transform = Transform::from_xyz(i as f32 * 50.0 - 225.0, 0.0, 1.0);
        transform.scale = Vec3::splat(2.0);
        let child = commands.spawn((
            transform,
            Sprite::from_atlas_image(
                hearts_tex.clone(),
            TextureAtlas {
                layout: hearts_atlas_layout.clone(),
                index: 0,
            },
        ))).id();

        commands.entity(hb).add_child(child);
    }

    game_manager.start_round();
}

fn create_all_moles(commands: &mut Commands, texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>, asset_server: &Res<AssetServer>)
{
    // Setup moles
    let texture = asset_server.load("Mole.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::new(46, 37), 3, 2, Some(UVec2::new(1, 1)), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    let font = asset_server.load("Pixica-Bold.ttf");
    let text_font = TextFont {
        font: font.clone(),
        font_size: 50.0,
        ..default()
    };

    // Top row
    create_mole_at(commands, Vec2::new(-460.0, -38.0) ,  KeyCode::KeyQ, &texture, &texture_atlas_layout, &font, &text_font);
    create_mole_at(commands, Vec2::new(-364.0, -38.0) ,  KeyCode::KeyW, &texture, &texture_atlas_layout, &font, &text_font);
    create_mole_at(commands, Vec2::new(-268.0, -38.0) ,  KeyCode::KeyE, &texture, &texture_atlas_layout, &font, &text_font);
    create_mole_at(commands, Vec2::new(-172.0, -38.0) ,  KeyCode::KeyR, &texture, &texture_atlas_layout, &font, &text_font);
    create_mole_at(commands, Vec2::new(-76.0, -38.0) ,  KeyCode::KeyT, &texture, &texture_atlas_layout, &font, &text_font);
    create_mole_at(commands, Vec2::new(20.0, -38.0) ,  KeyCode::KeyY, &texture, &texture_atlas_layout, &font, &text_font);
    create_mole_at(commands, Vec2::new(116.0, -38.0) ,  KeyCode::KeyU, &texture, &texture_atlas_layout, &font, &text_font);
    create_mole_at(commands, Vec2::new(212.0, -38.0) ,  KeyCode::KeyI, &texture, &texture_atlas_layout, &font, &text_font);
    create_mole_at(commands, Vec2::new(308.0, -38.0) ,  KeyCode::KeyO, &texture, &texture_atlas_layout, &font, &text_font);
    create_mole_at(commands, Vec2::new(404.0, -38.0), KeyCode::KeyP, &texture, &texture_atlas_layout, &font, &text_font);

    // Middle row
    create_mole_at(commands, Vec2::new(-424.0, -112.0),  KeyCode::KeyA, &texture, &texture_atlas_layout, &font, &text_font);
    create_mole_at(commands, Vec2::new(-328.0, -112.0),  KeyCode::KeyS, &texture, &texture_atlas_layout, &font, &text_font);
    create_mole_at(commands, Vec2::new(-232.0, -112.0),  KeyCode::KeyD, &texture, &texture_atlas_layout, &font, &text_font);
    create_mole_at(commands, Vec2::new(-136.0, -112.0),  KeyCode::KeyF, &texture, &texture_atlas_layout, &font, &text_font);
    create_mole_at(commands, Vec2::new(-40.0, -112.0),  KeyCode::KeyG, &texture, &texture_atlas_layout, &font, &text_font);
    create_mole_at(commands, Vec2::new(56.0, -112.0),  KeyCode::KeyH, &texture, &texture_atlas_layout, &font, &text_font);
    create_mole_at(commands, Vec2::new(152.0, -112.0),  KeyCode::KeyJ, &texture, &texture_atlas_layout, &font, &text_font);
    create_mole_at(commands, Vec2::new(248.0, -112.0),  KeyCode::KeyK, &texture, &texture_atlas_layout, &font, &text_font);
    create_mole_at(commands, Vec2::new(344.0, -112.0),  KeyCode::KeyL, &texture, &texture_atlas_layout, &font, &text_font);

    // Bottom row
    create_mole_at(commands, Vec2::new(-358.0, -186.0),  KeyCode::KeyZ, &texture, &texture_atlas_layout, &font, &text_font);
    create_mole_at(commands, Vec2::new(-262.0, -186.0),  KeyCode::KeyX, &texture, &texture_atlas_layout, &font, &text_font);
    create_mole_at(commands, Vec2::new(-166.0, -186.0),  KeyCode::KeyC, &texture, &texture_atlas_layout, &font, &text_font);
    create_mole_at(commands, Vec2::new(-70.0, -186.0),  KeyCode::KeyV, &texture, &texture_atlas_layout, &font, &text_font);
    create_mole_at(commands, Vec2::new(26.0, -186.0),  KeyCode::KeyB, &texture, &texture_atlas_layout, &font, &text_font);
    create_mole_at(commands, Vec2::new(122.0, -186.0),  KeyCode::KeyN, &texture, &texture_atlas_layout, &font, &text_font);
    create_mole_at(commands, Vec2::new(218.0, -186.0),  KeyCode::KeyM, &texture, &texture_atlas_layout, &font, &text_font);
}

fn create_mole_at(commands: &mut Commands, mut pos: Vec2, key: KeyCode, texture: &Handle<Image>,  texture_atlas_layout: &Handle<TextureAtlasLayout>, font: &Handle<Font>, text_font: &TextFont)
{
    pos += Vec2::new(46.0, -35.0);
    let mut mole_start = Transform::from_scale(Vec3::splat(2.0));
    mole_start.translation = Vec3::new(pos.x, pos.y, 1.0);

    let mut anim_controller = SpriteAnimator::new();

    anim_controller.push_anim(MOLE_RISE_ANIM, SpriteAnimation::new( &vec![4,3,1,0], 0.08));
    anim_controller.push_anim(MOLE_HIDE_ANIM, SpriteAnimation::new( &vec![0,1,3,4], 0.04));
    anim_controller.push_anim(MOLE_BONK_ANIM, SpriteAnimation::new( &vec![2,4,2,4,2,4,2,4], 0.05));

    anim_controller.play_anim(MOLE_HIDE_ANIM);

    commands.spawn((
        Sprite::from_atlas_image(
            texture.clone(),
            TextureAtlas {
                layout: texture_atlas_layout.clone(),
                index: 0,
            },
        ),
        mole_start,
        anim_controller,
        Mole::new(key)
    ));    

    let font_pos = Vec3::new(pos.x - 22.0, pos.y + 18.0, 2.0);
    let key_string = key_code_to_string(key);
    commands.spawn((
        Text2d::new(key_string),
        text_font
            .clone()
            .with_font_smoothing(FontSmoothing::None),
        Transform::from_translation(font_pos),
        TextColor(Color::linear_rgb(0.1, 0.1, 0.1))
    ));
}

/// The sprite is animated by changing its translation depending on the time that has passed since
/// the last frame.
// fn sprite_movement(time: Res<Time>, 
//     keys: Res<ButtonInput<KeyCode>>,
//     mut sprite_position: Query<(&mut Direction, &mut Transform)>)
// {
//     for (mut logo, mut transform) in &mut sprite_position
//     {
//         match *logo
//         {
//             Direction::Up => transform.translation.y += 150. * time.delta_secs(),
//             Direction::Down => transform.translation.y -= 150. * time.delta_secs(),
//             Direction::Left => transform.translation.x -= 150. * time.delta_secs(),
//             Direction::Right => transform.translation.x += 150. * time.delta_secs(),
//             Direction::None => {},
//         }

//         *logo = Direction::None;

//         if keys.pressed(KeyCode::ArrowUp) 
//         {
//             *logo = Direction::Up;
//         } 
//         else if keys.pressed(KeyCode::ArrowDown) 
//         {
//             *logo = Direction::Down;
//         }
//         else if keys.pressed(KeyCode::ArrowLeft) 
//         {
//             *logo = Direction::Left;
//         }
//         else if keys.pressed(KeyCode::ArrowRight) 
//         {
//             *logo = Direction::Right;
//         }
//     }
// }

fn key_code_to_string(key_code: KeyCode) -> String
{
    match key_code
    {
        KeyCode::KeyA => String::from("A"),
        KeyCode::KeyB => String::from("B"),
        KeyCode::KeyC => String::from("C"),
        KeyCode::KeyD => String::from("D"),
        KeyCode::KeyE => String::from("E"),
        KeyCode::KeyF => String::from("F"),
        KeyCode::KeyG => String::from("G"),
        KeyCode::KeyH => String::from("H"),
        KeyCode::KeyI => String::from("I"),
        KeyCode::KeyJ => String::from("J"),
        KeyCode::KeyK => String::from("K"),
        KeyCode::KeyL => String::from("L"),
        KeyCode::KeyM => String::from("M"),
        KeyCode::KeyN => String::from("N"),
        KeyCode::KeyO => String::from("O"),
        KeyCode::KeyP => String::from("P"),
        KeyCode::KeyQ => String::from("Q"),
        KeyCode::KeyR => String::from("R"),
        KeyCode::KeyS => String::from("S"),
        KeyCode::KeyT => String::from("T"),
        KeyCode::KeyU => String::from("U"),
        KeyCode::KeyV => String::from("V"),
        KeyCode::KeyW => String::from("W"),
        KeyCode::KeyX => String::from("X"),
        KeyCode::KeyY => String::from("Y"),
        KeyCode::KeyZ => String::from("Z"),
        _ => panic!()
    }
}