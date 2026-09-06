//! Skin loading. A skin is a .toml file of design tokens (colors, radii,
//! fonts). Five skins ship embedded in the binary; users can drop extra
//! .toml files in the `skins/` folder of the music-player config directory
//! and they show up in the sidebar switcher. The selected skin name
//! persists across launches.

use std::path::PathBuf;

use serde::Deserialize;
use slint::{Color, ComponentHandle};

use crate::{AppWindow, Theme};

#[derive(Deserialize, Clone, Debug)]
pub struct Skin {
    pub name: String,
    pub colors: Colors,
    pub metrics: Metrics,
    pub fonts: Fonts,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Colors {
    pub window_bg: String,
    pub panel_bg: String,
    pub panel_raised: String,
    pub sidebar_bg: String,
    pub display_bg: String,
    pub display_glow: String,
    pub display_text: String,
    pub display_dim: String,
    pub accent: String,
    pub accent_hover: String,
    pub on_accent: String,
    pub text: String,
    pub text_dim: String,
    pub text_muted: String,
    pub border: String,
    pub hover_bg: String,
    pub selected_bg: String,
    pub slider_track: String,
    pub meter_low: String,
    pub meter_mid: String,
    pub meter_high: String,
    pub meter_off: String,
    /// Content-loader shimmer. Optional so skins written before these tokens
    /// existed keep loading; they fall back to the panel/slider surfaces.
    #[serde(default)]
    pub skeleton_bg: Option<String>,
    #[serde(default)]
    pub skeleton_fg: Option<String>,
    pub art_placeholder: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Metrics {
    pub radius: f32,
    pub control_radius: f32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Fonts {
    pub ui: String,
    pub mono: String,
}

fn parse_hex(s: &str) -> Color {
    let h = s.trim_start_matches('#');
    let (r, g, b, a) = match h.len() {
        6 => (
            u8::from_str_radix(&h[0..2], 16).unwrap_or(0),
            u8::from_str_radix(&h[2..4], 16).unwrap_or(0),
            u8::from_str_radix(&h[4..6], 16).unwrap_or(0),
            255,
        ),
        8 => (
            u8::from_str_radix(&h[0..2], 16).unwrap_or(0),
            u8::from_str_radix(&h[2..4], 16).unwrap_or(0),
            u8::from_str_radix(&h[4..6], 16).unwrap_or(0),
            u8::from_str_radix(&h[6..8], 16).unwrap_or(255),
        ),
        _ => (255, 0, 255, 255), // loud magenta = malformed token
    };
    Color::from_argb_u8(a, r, g, b)
}

pub fn apply(skin: &Skin, app: &AppWindow) {
    let c = &skin.colors;
    let th = app.global::<Theme>();
    th.set_window_bg(parse_hex(&c.window_bg));
    th.set_panel_bg(parse_hex(&c.panel_bg));
    th.set_panel_raised(parse_hex(&c.panel_raised));
    th.set_sidebar_bg(parse_hex(&c.sidebar_bg));
    th.set_display_bg(parse_hex(&c.display_bg));
    th.set_display_glow(parse_hex(&c.display_glow));
    th.set_display_text(parse_hex(&c.display_text));
    th.set_display_dim(parse_hex(&c.display_dim));
    th.set_accent(parse_hex(&c.accent));
    th.set_accent_hover(parse_hex(&c.accent_hover));
    th.set_on_accent(parse_hex(&c.on_accent));
    th.set_text(parse_hex(&c.text));
    th.set_text_dim(parse_hex(&c.text_dim));
    th.set_text_muted(parse_hex(&c.text_muted));
    th.set_border(parse_hex(&c.border));
    th.set_hover_bg(parse_hex(&c.hover_bg));
    th.set_selected_bg(parse_hex(&c.selected_bg));
    th.set_slider_track(parse_hex(&c.slider_track));
    th.set_meter_low(parse_hex(&c.meter_low));
    th.set_meter_mid(parse_hex(&c.meter_mid));
    th.set_meter_high(parse_hex(&c.meter_high));
    th.set_meter_off(parse_hex(&c.meter_off));
    th.set_skeleton_bg(parse_hex(
        c.skeleton_bg.as_deref().unwrap_or(&c.panel_raised),
    ));
    th.set_skeleton_fg(parse_hex(
        c.skeleton_fg.as_deref().unwrap_or(&c.slider_track),
    ));
    th.set_art_placeholder(parse_hex(&c.art_placeholder));
    th.set_radius(skin.metrics.radius);
    th.set_control_radius(skin.metrics.control_radius);
    th.set_font_ui(skin.fonts.ui.clone().into());
    th.set_font_mono(skin.fonts.mono.clone().into());
    app.set_skin_name(skin.name.clone().into());
}

fn config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("music-player"))
}

pub fn load_all() -> Vec<Skin> {
    let mut skins: Vec<Skin> = [
        include_str!("../skins/synthwave.toml"),
        include_str!("../skins/late-night.toml"),
        include_str!("../skins/neutron.toml"),
        include_str!("../skins/lunar.toml"),
        include_str!("../skins/porcelain.toml"),
    ]
    .iter()
    .filter_map(|s| toml::from_str(s).ok())
    .collect();

    if let Some(dir) = config_dir().map(|d| d.join("skins")) {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("toml") {
                    continue;
                }
                match std::fs::read_to_string(&path)
                    .map_err(|e| e.to_string())
                    .and_then(|s| toml::from_str::<Skin>(&s).map_err(|e| e.to_string()))
                {
                    Ok(skin) => skins.push(skin),
                    Err(e) => tracing::warn!("skipping skin {}: {e}", path.display()),
                }
            }
        }
    }
    skins
}

fn selection_file() -> Option<PathBuf> {
    config_dir().map(|d| d.join("desktop-skin"))
}

pub fn load_selection(skins: &[Skin]) -> usize {
    let Some(file) = selection_file() else {
        return 0;
    };
    let Ok(name) = std::fs::read_to_string(file) else {
        return 0;
    };
    let name = name.trim();
    skins.iter().position(|s| s.name == name).unwrap_or(0)
}

pub fn save_selection(name: &str) {
    if let Some(file) = selection_file() {
        if let Some(parent) = file.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(file, name);
    }
}
