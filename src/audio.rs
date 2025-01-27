use crate::loading::AudioAssets;
use crate::GameState;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

pub struct InternalAudioPlugin;

// This plugin is responsible to control the game audio
impl Plugin for InternalAudioPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(AudioPlugin)
            .add_systems(OnEnter(GameState::Playing), start_audio);
    }
}

fn start_audio(audio_assets: Res<AudioAssets>, audio: Res<Audio>)
{
    let _ = audio.play(audio_assets.monty_moles.clone())
                        .looped()
                        .with_volume(0.2);
}
