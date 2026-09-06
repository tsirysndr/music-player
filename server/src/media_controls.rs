//! Desktop-environment media controls for the daemon.
//!
//! On Linux this registers an `org.mpris.MediaPlayer2.music_player` D-Bus
//! service (via souvlaki), so GNOME/KDE media widgets, playerctl and
//! keyboard media keys control the daemon and show what's playing. Other
//! platforms are a no-op — the Slint desktop app integrates with the macOS
//! Now Playing center itself.

#[cfg(target_os = "linux")]
pub fn spawn(
    tracklist: std::sync::Arc<std::sync::Mutex<music_player_tracklist::Tracklist>>,
    cmd_tx: std::sync::Arc<
        std::sync::Mutex<
            tokio::sync::mpsc::UnboundedSender<music_player_playback::player::PlayerCommand>,
        >,
    >,
) {
    std::thread::Builder::new()
        .name("mpris".into())
        .spawn(move || {
            if let Err(e) = linux::run(tracklist, cmd_tx) {
                tracing::warn!("MPRIS disabled: {e}");
            }
        })
        .expect("spawn mpris thread");
}

#[cfg(not(target_os = "linux"))]
pub fn spawn(
    _tracklist: std::sync::Arc<std::sync::Mutex<music_player_tracklist::Tracklist>>,
    _cmd_tx: std::sync::Arc<
        std::sync::Mutex<
            tokio::sync::mpsc::UnboundedSender<music_player_playback::player::PlayerCommand>,
        >,
    >,
) {
}

#[cfg(target_os = "linux")]
mod linux {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use music_player_playback::player::PlayerCommand;
    use music_player_tracklist::Tracklist;
    use souvlaki::{
        MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition,
        PlatformConfig, SeekDirection,
    };
    use tokio::sync::mpsc::UnboundedSender;

    pub fn run(
        tracklist: Arc<Mutex<Tracklist>>,
        cmd_tx: Arc<Mutex<UnboundedSender<PlayerCommand>>>,
    ) -> Result<(), String> {
        let mut controls = MediaControls::new(PlatformConfig {
            dbus_name: "music_player",
            display_name: "Music Player",
            hwnd: None,
        })
        .map_err(|e| format!("{e:?}"))?;

        let position = Arc::new(Mutex::new(0u32));
        {
            let cmd_tx = cmd_tx.clone();
            let position = position.clone();
            let tracklist = tracklist.clone();
            controls
                .attach(move |event| {
                    let send = |cmd: PlayerCommand| {
                        let _ = cmd_tx.lock().unwrap().send(cmd);
                    };
                    match event {
                        MediaControlEvent::Play => send(PlayerCommand::Play),
                        MediaControlEvent::Pause => send(PlayerCommand::Pause),
                        MediaControlEvent::Toggle => {
                            let playing = tracklist.lock().unwrap().playback_state().is_playing;
                            send(if playing {
                                PlayerCommand::Pause
                            } else {
                                PlayerCommand::Play
                            })
                        }
                        MediaControlEvent::Next => send(PlayerCommand::Next),
                        MediaControlEvent::Previous => send(PlayerCommand::Previous),
                        MediaControlEvent::Stop => send(PlayerCommand::Stop),
                        MediaControlEvent::SetPosition(MediaPosition(pos)) => {
                            send(PlayerCommand::Seek(pos.as_millis() as u32))
                        }
                        MediaControlEvent::Seek(direction) => {
                            let cur = *position.lock().unwrap();
                            let target = match direction {
                                SeekDirection::Forward => cur.saturating_add(5_000),
                                SeekDirection::Backward => cur.saturating_sub(5_000),
                            };
                            send(PlayerCommand::Seek(target))
                        }
                        MediaControlEvent::SeekBy(direction, by) => {
                            let cur = *position.lock().unwrap();
                            let by = by.as_millis() as u32;
                            let target = match direction {
                                SeekDirection::Forward => cur.saturating_add(by),
                                SeekDirection::Backward => cur.saturating_sub(by),
                            };
                            send(PlayerCommand::Seek(target))
                        }
                        _ => {}
                    }
                })
                .map_err(|e| format!("{e:?}"))?;
        }

        tracing::info!("MPRIS media controls registered (org.mpris.MediaPlayer2.music_player)");

        // Push now-playing state once a second.
        loop {
            let (track, state) = {
                let tracklist = tracklist.lock().unwrap();
                let (track, _) = tracklist.current_track();
                (track, tracklist.playback_state())
            };
            *position.lock().unwrap() = state.position_ms;

            match &track {
                Some(track) => {
                    let duration = track
                        .duration
                        .map(|s| Duration::from_secs_f32(s))
                        .unwrap_or_default();
                    let _ = controls.set_metadata(MediaMetadata {
                        title: Some(&track.title),
                        artist: Some(&track.artist),
                        album: Some(&track.album.title),
                        duration: Some(duration),
                        cover_url: None,
                    });
                    let progress = Some(MediaPosition(Duration::from_millis(
                        state.position_ms as u64,
                    )));
                    let _ = controls.set_playback(if state.is_playing {
                        MediaPlayback::Playing { progress }
                    } else {
                        MediaPlayback::Paused { progress }
                    });
                }
                None => {
                    let _ = controls.set_playback(MediaPlayback::Stopped);
                }
            }
            std::thread::sleep(Duration::from_secs(1));
        }
    }
}
