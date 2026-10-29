use crate::AlbumItem;
use slint::{FilterModel, ModelRc, SharedString};
use std::collections::VecDeque;

const ART_SIZE: u32 = 384;
const CANVAS_HEIGHT: u32 = ART_SIZE * 3 / 2;

fn same_art(left: &slint::Image, right: &slint::Image) -> bool {
    left == right || (left.size().width == 0 && right.size().width == 0)
}

/// A cover and its reflection share one projection and one GPU texture.
/// Bounded caches avoid hundreds of clipped image nodes per animation frame,
/// while retaining artwork updates (the Image handle is part of the key).
#[derive(Default)]
pub struct ProjectionCache {
    sources: VecDeque<(slint::Image, image::RgbaImage)>,
    frames: VecDeque<(slint::Image, i16, slint::Image)>,
}

impl ProjectionCache {
    pub fn project(&mut self, art: slint::Image, turn: f32) -> slint::Image {
        let angle = (turn.clamp(-1.0, 1.0) * 128.0).round() as i16;
        if let Some(index) = self
            .frames
            .iter()
            .position(|(source, t, _)| same_art(source, &art) && *t == angle)
        {
            let frame = self.frames.remove(index).unwrap();
            let result = frame.2.clone();
            self.frames.push_front(frame);
            return result;
        }
        let source_index = self
            .sources
            .iter()
            .position(|(source, _)| same_art(source, &art));
        let source = if let Some(index) = source_index {
            self.sources.remove(index).unwrap()
        } else {
            let pixels = art
                .to_rgba8()
                .filter(|pixels| pixels.width() > 0 && pixels.height() > 0);
            let pixels = pixels
                .map(|pixels| {
                    let input = image::RgbaImage::from_raw(
                        pixels.width(),
                        pixels.height(),
                        pixels.as_bytes().to_vec(),
                    )
                    .unwrap();
                    image::imageops::resize(
                        &input,
                        ART_SIZE,
                        ART_SIZE,
                        image::imageops::FilterType::Triangle,
                    )
                })
                .unwrap_or_else(|| {
                    image::RgbaImage::from_fn(ART_SIZE, ART_SIZE, |x, y| {
                        let radius =
                            ((x as f32 - 192.0).powi(2) + (y as f32 - 192.0).powi(2)).sqrt();
                        let shade =
                            if (112.0..126.0).contains(&radius) || (27.0..39.0).contains(&radius) {
                                155
                            } else {
                                45
                            };
                        image::Rgba([shade, shade, shade, 255])
                    })
                });
            (art.clone(), pixels)
        };
        let projected = project_pixels(&source.1, angle as f32 / 128.0);
        let result = slint::Image::from_rgba8(projected);
        self.sources.push_front(source);
        self.sources.truncate(12);
        self.frames.push_front((art, angle, result.clone()));
        self.frames.truncate(64); // ~54 MiB maximum, independent of library size.
        result
    }
}

fn project_pixels(
    source: &image::RgbaImage,
    turn: f32,
) -> slint::SharedPixelBuffer<slint::Rgba8Pixel> {
    let mut result = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::new(ART_SIZE, CANVAS_HEIGHT);
    let pixels = result.make_mut_slice();
    for x in 0..ART_SIZE {
        let u = (x as f32 + 0.5) / ART_SIZE as f32;
        let edge = if turn > 0.0 { u } else { 1.0 - u };
        let top = ART_SIZE as f32 * turn.abs() * (1.0 - edge) * 0.14;
        let bottom = ART_SIZE as f32 - top;
        let height = bottom - top;
        let shade = 1.0 - turn.abs() * (0.18 + 0.3 * (1.0 - edge));
        let reflection_top = bottom + 8.0;
        let reflection_height = height * 0.44;
        for y in top.floor() as u32
            ..(reflection_top + reflection_height)
                .ceil()
                .min(CANVAS_HEIGHT as f32) as u32
        {
            let (v, opacity, coverage) = if (y as f32) < bottom {
                (
                    ((y as f32 + 0.5 - top) / height).clamp(0.0, 1.0),
                    1.0,
                    (bottom.min(y as f32 + 1.0) - top.max(y as f32)).clamp(0.0, 1.0),
                )
            } else if (y as f32 + 1.0) > reflection_top {
                let depth = ((y as f32 + 0.5 - reflection_top) / reflection_height).clamp(0.0, 1.0);
                (
                    1.0 - ((y as f32 + 0.5 - reflection_top) / height).clamp(0.0, 1.0),
                    0.22 * (1.0 - depth).powi(2),
                    (y as f32 + 1.0 - reflection_top).clamp(0.0, 1.0),
                )
            } else {
                continue;
            };
            let source_y = v * (ART_SIZE - 1) as f32;
            let y0 = source_y.floor() as u32;
            let y1 = (y0 + 1).min(ART_SIZE - 1);
            let mix = source_y - y0 as f32;
            let a = source.get_pixel(x, y0).0;
            let b = source.get_pixel(x, y1).0;
            let channel = |i: usize| a[i] as f32 * (1.0 - mix) + b[i] as f32 * mix;
            pixels[(y * ART_SIZE + x) as usize] = slint::Rgba8Pixel::new(
                (channel(0) * shade) as u8,
                (channel(1) * shade) as u8,
                (channel(2) * shade) as u8,
                (channel(3) * opacity * coverage) as u8,
            );
        }
    }
    result
}

/// Keep the source model attached so asynchronously loaded artwork also
/// updates covers while a search is active.
pub fn filter_albums(albums: ModelRc<AlbumItem>, query: SharedString) -> ModelRc<AlbumItem> {
    let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    if words.is_empty() {
        return albums;
    }
    ModelRc::new(FilterModel::new(albums, move |album: &AlbumItem| {
        let haystack = format!("{} {}", album.title, album.artist).to_lowercase();
        words.iter().all(|word| haystack.contains(word))
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use slint::{Model, VecModel};
    use std::rc::Rc;

    #[test]
    fn reflected_edges_remain_parallel_and_the_cache_is_bounded() {
        let white =
            image::RgbaImage::from_pixel(ART_SIZE, ART_SIZE, image::Rgba([255, 255, 255, 255]));
        for turn in [-1.0, -0.5, 0.0, 0.5, 1.0] {
            let projected = project_pixels(&white, turn);
            for x in [0, 96, 192, 288, 383] {
                let alpha = |y: u32| projected.as_slice()[(y * ART_SIZE + x) as usize].a;
                let bottom = (0..ART_SIZE).rev().find(|&y| alpha(y) > 128).unwrap();
                let reflection = (bottom + 2..CANVAS_HEIGHT).find(|&y| alpha(y) > 0).unwrap();
                assert!(
                    (8..=10).contains(&(reflection - bottom)),
                    "turn={turn}, x={x}"
                );
            }
        }
        let mut cache = ProjectionCache::default();
        let art = slint::Image::default();
        let first = cache.project(art.clone(), 0.0);
        assert!(
            first == cache.project(art.clone(), 0.0),
            "identical requests reuse the texture"
        );
        for step in 0..80 {
            cache.project(art.clone(), step as f32 / 128.0);
        }
        assert_eq!(cache.frames.len(), 64);
        assert_eq!(cache.sources.len(), 1);
    }

    #[test]
    fn searches_title_and_artist_case_insensitively_and_tracks_source_changes() {
        let source = Rc::new(VecModel::from(vec![
            AlbumItem {
                title: "Début".into(),
                artist: "Björk".into(),
                ..Default::default()
            },
            AlbumItem {
                title: "Discovery".into(),
                artist: "Daft Punk".into(),
                ..Default::default()
            },
        ]));
        let albums = ModelRc::from(source.clone());
        assert_eq!(filter_albums(albums.clone(), "  ".into()).row_count(), 2);
        assert_eq!(
            filter_albums(albums.clone(), "DÉB BJÖRK".into()).row_count(),
            1
        );
        assert_eq!(
            filter_albums(albums.clone(), "missing".into()).row_count(),
            0
        );
        let filtered = filter_albums(albums, "disco punk".into());
        assert_eq!(filtered.row_count(), 1);
        let mut updated = source.row_data(1).unwrap();
        updated.has_art = true;
        source.set_row_data(1, updated);
        assert!(filtered.row_data(0).unwrap().has_art);
        source.remove(1);
        assert_eq!(filtered.row_count(), 0);
    }
}
