use std::fs::File;
use std::path::Path;

use owo_colors::OwoColorize;
use symphonia::core::formats::{Cue, FormatOptions, Track};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::{ColorMode, MetadataOptions, MetadataRevision, Tag, Value, Visual};
use symphonia::core::probe::Hint;
use symphonia::core::units::TimeBase;
use url::Url;

use log::info;

pub fn print_format(path: &str) {
    if Url::parse(path).is_ok() {
        println!("+ {}", path.magenta());
        return;
    }
    let mut hint = Hint::new();

    let source = Box::new(File::open(Path::new(path)).unwrap());

    // Provide the file extension as a hint.
    if let Some(extension) = Path::new(path).extension() {
        if let Some(extension_str) = extension.to_str() {
            hint.with_extension(extension_str);
        }
    }
    let mss = MediaSourceStream::new(source, Default::default());

    let format_opts = FormatOptions {
        enable_gapless: false,
        ..Default::default()
    };

    let metadata_opts: MetadataOptions = Default::default();

    let mut probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .unwrap();

    println!("+ {}", path);
    print_tracks(probed.format.tracks());

    // Prefer metadata that's provided in the container format, over other tags found during the
    // probe operation.
    if let Some(metadata_rev) = probed.format.metadata().current() {
        print_tags(metadata_rev.tags());
        print_visuals(metadata_rev.visuals());

        // Warn that certain tags are preferred.
        if probed.metadata.get().as_ref().is_some() {
            info!("tags that are part of the container format are preferentially printed.");
            info!("not printing additional tags that were found while probing.");
        }
    } else if let Some(metadata_rev) = probed.metadata.get().as_ref().and_then(|m| m.current()) {
        print_tags(metadata_rev.tags());
        print_visuals(metadata_rev.visuals());
    }

    print_cues(probed.format.cues());
    println!(":");
    println!();
}

fn print_update(rev: &MetadataRevision) {
    print_tags(rev.tags());
    print_visuals(rev.visuals());
    println!(":");
    println!();
}

fn print_tracks(tracks: &[Track]) {
    if !tracks.is_empty() {
        println!("|");
        println!("| // Tracks //");

        for (idx, track) in tracks.iter().enumerate() {
            let params = &track.codec_params;

            print!("|     [{:0>2}] Codec:           ", idx + 1);

            if let Some(codec) = symphonia::default::get_codecs().get_codec(params.codec) {
                println!("{} ({})", codec.long_name, codec.short_name);
            } else {
                println!("Unknown (#{})", params.codec);
            }

            if let Some(sample_rate) = params.sample_rate {
                println!("|          Sample Rate:     {}", sample_rate);
            }
            if params.start_ts > 0 {
                if let Some(tb) = params.time_base {
                    println!(
                        "|          Start Time:      {} ({})",
                        fmt_time(params.start_ts, tb),
                        params.start_ts
                    );
                } else {
                    println!("|          Start Time:      {}", params.start_ts);
                }
            }
            if let Some(n_frames) = params.n_frames {
                if let Some(tb) = params.time_base {
                    println!(
                        "|          Duration:        {} ({})",
                        fmt_time(n_frames, tb),
                        n_frames
                    );
                } else {
                    println!("|          Frames:          {}", n_frames);
                }
            }
            if let Some(tb) = params.time_base {
                println!("|          Time Base:       {}", tb);
            }
            if let Some(padding) = params.delay {
                println!("|          Encoder Delay:   {}", padding);
            }
            if let Some(padding) = params.padding {
                println!("|          Encoder Padding: {}", padding);
            }
            if let Some(sample_format) = params.sample_format {
                println!("|          Sample Format:   {:?}", sample_format);
            }
            if let Some(bits_per_sample) = params.bits_per_sample {
                println!("|          Bits per Sample: {}", bits_per_sample);
            }
            if let Some(channels) = params.channels {
                println!("|          Channel(s):      {}", channels.count());
                println!("|          Channel Map:     {}", channels);
            }
            if let Some(channel_layout) = params.channel_layout {
                println!("|          Channel Layout:  {:?}", channel_layout);
            }
            if let Some(language) = &track.language {
                println!("|          Language:        {}", language);
            }
        }
    }
}

fn print_cues(cues: &[Cue]) {
    if !cues.is_empty() {
        println!("|");
        println!("| // Cues //");

        for (idx, cue) in cues.iter().enumerate() {
            println!("|     [{:0>2}] Track:      {}", idx + 1, cue.index);
            println!("|          Timestamp:  {}", cue.start_ts);

            // Print tags associated with the Cue.
            if !cue.tags.is_empty() {
                println!("|          Tags:");

                for (tidx, tag) in cue.tags.iter().enumerate() {
                    if let Some(std_key) = tag.std_key {
                        println!(
                            "{}",
                            print_tag_item(tidx + 1, &format!("{:?}", std_key), &tag.value, 21)
                        );
                    } else {
                        println!("{}", print_tag_item(tidx + 1, &tag.key, &tag.value, 21));
                    }
                }
            }

            // Print any sub-cues.
            if !cue.points.is_empty() {
                println!("|          Sub-Cues:");

                for (ptidx, pt) in cue.points.iter().enumerate() {
                    println!(
                        "|                      [{:0>2}] Offset:    {:?}",
                        ptidx + 1,
                        pt.start_offset_ts
                    );

                    // Start the number of sub-cue tags, but don't print them.
                    if !pt.tags.is_empty() {
                        println!(
                            "|                           Sub-Tags:  {} (not listed)",
                            pt.tags.len()
                        );
                    }
                }
            }
        }
    }
}

fn print_tags(tags: &[Tag]) {
    if !tags.is_empty() {
        println!("|");
        println!("| // Tags //");

        let mut idx = 1;

        // Print tags with a standard tag key first, these are the most common tags.
        for tag in tags.iter().filter(|tag| tag.is_known()) {
            if let Some(std_key) = tag.std_key {
                println!(
                    "{}",
                    print_tag_item(idx, &format!("{:?}", std_key), &tag.value, 4)
                );
            }
            idx += 1;
        }

        // Print the remaining tags with keys truncated to 26 characters.
        for tag in tags.iter().filter(|tag| !tag.is_known()) {
            println!("{}", print_tag_item(idx, &tag.key, &tag.value, 4));
            idx += 1;
        }
    }
}

fn print_visuals(visuals: &[Visual]) {
    if !visuals.is_empty() {
        println!("|");
        println!("| // Visuals //");

        for (idx, visual) in visuals.iter().enumerate() {
            if let Some(usage) = visual.usage {
                println!("|     [{:0>2}] Usage:      {:?}", idx + 1, usage);
                println!("|          Media Type: {}", visual.media_type);
            } else {
                println!("|     [{:0>2}] Media Type: {}", idx + 1, visual.media_type);
            }
            if let Some(dimensions) = visual.dimensions {
                println!(
                    "|          Dimensions: {} px x {} px",
                    dimensions.width, dimensions.height
                );
            }
            if let Some(bpp) = visual.bits_per_pixel {
                println!("|          Bits/Pixel: {}", bpp);
            }
            if let Some(ColorMode::Indexed(colors)) = visual.color_mode {
                println!("|          Palette:    {} colors", colors);
            }
            println!("|          Size:       {} bytes", visual.data.len());

            // Print out tags similar to how regular tags are printed.
            if !visual.tags.is_empty() {
                println!("|          Tags:");
            }

            for (tidx, tag) in visual.tags.iter().enumerate() {
                if let Some(std_key) = tag.std_key {
                    println!(
                        "{}",
                        print_tag_item(tidx + 1, &format!("{:?}", std_key), &tag.value, 21)
                    );
                } else {
                    println!("{}", print_tag_item(tidx + 1, &tag.key, &tag.value, 21));
                }
            }
        }
    }
}

fn print_tag_item(idx: usize, key: &str, value: &Value, indent: usize) -> String {
    let key_str = match key.len() {
        0..=28 => format!("| {:w$}[{:0>2}] {:<28} : ", "", idx, key, w = indent),
        _ => format!(
            "| {:w$}[{:0>2}] {:.<28} : ",
            "",
            idx,
            key.split_at(26).0,
            w = indent
        ),
    };

    let line_prefix = format!("\n| {:w$} : ", "", w = indent + 4 + 28 + 1);
    let line_wrap_prefix = format!("\n| {:w$}   ", "", w = indent + 4 + 28 + 1);

    let mut out = String::new();

    out.push_str(&key_str);

    for (wrapped, line) in value.to_string().lines().enumerate() {
        if wrapped > 0 {
            out.push_str(&line_prefix);
        }

        let mut chars = line.chars();
        let split = (0..)
            .map(|_| chars.by_ref().take(72).collect::<String>())
            .take_while(|s| !s.is_empty())
            .collect::<Vec<_>>();

        out.push_str(&split.join(&line_wrap_prefix).magenta().to_string());
    }

    out
}

fn fmt_time(ts: u64, tb: TimeBase) -> String {
    let time = tb.calc_time(ts);

    let hours = time.seconds / (60 * 60);
    let mins = (time.seconds % (60 * 60)) / 60;
    let secs = f64::from((time.seconds % 60) as u32) + time.frac;

    format!("{}:{:0>2}:{:0>6.3}", hours, mins, secs)
}
