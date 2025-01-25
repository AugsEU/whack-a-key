use std::collections::HashMap;

#[allow(dead_code)]
#[allow(unused)]
#[allow(unused_variables)]
#[allow(unused_mut)]

use bevy::prelude::*;

#[derive(Component)]
enum Direction
{
    Up,
    Down,
    Left,
    Right,
    None
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



fn main()
{
    App::new()
        // Plugin
        .add_plugins((DefaultPlugins.set(ImagePlugin::default_nearest()), bevy_framepace::FramepacePlugin))

        // Startup
        .add_systems(Startup, 
            setup)
        
        // Update
        .add_systems(Update, 
            (sprite_movement,
                    animate_sprite))

        .run();
}

/// ===========================================
/// SETUP
/// ===========================================
fn setup(mut commands: Commands, 
    mut settings: ResMut<bevy_framepace::FramepaceSettings>,
    mut window: Single<&mut Window>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>)
{
    commands.spawn(Camera2d::default());
    
    // Game BG
    commands.spawn(Sprite::from_image(asset_server.load("GameBG.png")));

    // Setup moles
    let texture = asset_server.load("Mole.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::new(46, 37), 2, 2, Some(UVec2::new(1, 1)), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    let mut mole_start = Transform::from_scale(Vec3::splat(2.0));
    mole_start.translation = Vec3::new(100.0, 0.0, 1.0);

    let mut anim_controller = SpriteAnimator::new();
    anim_controller.push_anim("MoleUp", SpriteAnimation::new( &vec![3,2,1,0], 0.8));
    anim_controller.play_anim("MoleUp");

    commands.spawn((
        Sprite::from_atlas_image(
            texture,
            TextureAtlas {
                layout: texture_atlas_layout,
                index: 0,
            },
        ),
        mole_start,
        anim_controller
    ));

    // commands.spawn(
    //     (Sprite::from_image(asset_server.load("Arnold.png")),
    //     Transform::from_xyz(100., 0., 1.),
    //     Direction::Up)
    // );

    use bevy_framepace::Limiter;
    settings.limiter = Limiter::from_framerate(60.0);

    window.resolution.set(1280.0, 720.0);
    window.resizable = false;
    window.name = Some(String::from("Thor's Power"));
    window.title = String::from("Thor's Power");
}

/// The sprite is animated by changing its translation depending on the time that has passed since
/// the last frame.
fn sprite_movement(time: Res<Time>, 
    keys: Res<ButtonInput<KeyCode>>,
    mut sprite_position: Query<(&mut Direction, &mut Transform)>)
{
    for (mut logo, mut transform) in &mut sprite_position
    {
        match *logo
        {
            Direction::Up => transform.translation.y += 150. * time.delta_secs(),
            Direction::Down => transform.translation.y -= 150. * time.delta_secs(),
            Direction::Left => transform.translation.x -= 150. * time.delta_secs(),
            Direction::Right => transform.translation.x += 150. * time.delta_secs(),
            Direction::None => {},
        }

        *logo = Direction::None;

        if keys.pressed(KeyCode::ArrowUp) 
        {
            *logo = Direction::Up;
        } 
        else if keys.pressed(KeyCode::ArrowDown) 
        {
            *logo = Direction::Down;
        }
        else if keys.pressed(KeyCode::ArrowLeft) 
        {
            *logo = Direction::Left;
        }
        else if keys.pressed(KeyCode::ArrowRight) 
        {
            *logo = Direction::Right;
        }
    }
}