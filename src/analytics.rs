//! `music-player analytics …` — importing listening history and asking
//! questions of it.
//!
//! All local: the analytics database sits next to the player's own, so none of
//! this needs a running daemon or a network connection (except the two
//! commands that are explicitly about fetching from Rocksky).

use std::path::PathBuf;
use std::sync::Mutex;

use clap::ArgMatches;
use indicatif::{ProgressBar, ProgressStyle};
use music_player_analytics::progress::Reporter;
use music_player_analytics::query::{self, Scope, TopKind, Window};
use music_player_analytics::{import, Analytics};
use owo_colors::OwoColorize;
use tabled::{builder::Builder, Style};

type CmdResult = Result<(), Box<dyn std::error::Error>>;

/// A [`Reporter`] that draws progress bars.
///
/// Held behind a mutex because the trait is `Sync` — the enrichment stage
/// reports from a task that is draining results while workers run.
struct Bars {
    bar: Mutex<Option<ProgressBar>>,
}

impl Bars {
    fn new() -> Self {
        Self {
            bar: Mutex::new(None),
        }
    }
}

impl Reporter for Bars {
    fn stage(&self, name: &str, total: Option<u64>) {
        let bar = match total {
            Some(total) => {
                let bar = ProgressBar::new(total);
                bar.set_style(
                    ProgressStyle::with_template(
                        "  {spinner:.cyan} {msg:<34} [{bar:28.cyan/blue}] {pos}/{len} {eta:>6}",
                    )
                    .expect("a valid template")
                    .progress_chars("━━╌"),
                );
                bar
            }
            // No total to count towards — a spinner says "working" without
            // implying a finish line it cannot know.
            None => {
                let bar = ProgressBar::new_spinner();
                bar.set_style(
                    ProgressStyle::with_template("  {spinner:.cyan} {msg:<34} {pos} {elapsed:>8}")
                        .expect("a valid template"),
                );
                bar.enable_steady_tick(std::time::Duration::from_millis(120));
                bar
            }
        };
        bar.set_message(name.to_owned());
        *self.bar.lock().unwrap() = Some(bar);
    }

    fn advance(&self, n: u64) {
        if let Some(bar) = self.bar.lock().unwrap().as_ref() {
            bar.inc(n);
        }
    }

    fn finish(&self, summary: &str) {
        if let Some(bar) = self.bar.lock().unwrap().take() {
            // `finish_and_clear` then print, rather than `finish_with_message`,
            // so the completed lines are ordinary output that survives being
            // piped to a file.
            bar.finish_and_clear();
        }
        println!("  {} {}", "✓".bright_green(), summary);
    }

    fn warn(&self, message: &str) {
        if let Some(bar) = self.bar.lock().unwrap().as_ref() {
            bar.suspend(|| println!("  {} {}", "!".bright_yellow(), message.dimmed()));
        } else {
            println!("  {} {}", "!".bright_yellow(), message.dimmed());
        }
    }
}

fn open() -> Result<Analytics, Box<dyn std::error::Error>> {
    Ok(Analytics::open_default()?)
}

/// For the reporting commands. DuckDB permits one writer or many readers, so
/// asking for read-only is what lets a report run while an import is going —
/// and lets two reports run at once.
fn open_ro() -> Result<Analytics, Box<dyn std::error::Error>> {
    Ok(Analytics::open_default_read_only()?)
}

/// `--days N`, absent meaning all of recorded history, plus `--local-only`.
fn scope(matches: &ArgMatches) -> Scope {
    Scope {
        window: match matches
            .get_one::<String>("days")
            .and_then(|d| d.parse().ok())
        {
            Some(days) => Window::last_days(days),
            None => Window::all(),
        },
        // `try_get_one` rather than `get_flag`: the reporting subcommands all
        // take it, but `analytics` with no subcommand falls through to the
        // overview with a bare ArgMatches that has no such argument defined,
        // and `get_flag` panics on an unknown id.
        local_only: matches
            .try_get_one::<bool>("local-only")
            .ok()
            .flatten()
            .copied()
            .unwrap_or(false),
    }
}

fn limit(matches: &ArgMatches, fallback: u32) -> u32 {
    matches
        .get_one::<String>("limit")
        .and_then(|l| l.parse().ok())
        .unwrap_or(fallback)
}

fn heading(text: &str) {
    println!("\n{}", text.bold());
}

pub async fn analytics(matches: &ArgMatches) -> CmdResult {
    match matches.subcommand() {
        Some(("import", m)) => cmd_import(m).await,
        Some(("rocksky", m)) => cmd_rocksky(m).await,
        Some(("sync", _)) => cmd_sync().await,
        Some(("enrich", m)) => cmd_enrich(m).await,
        Some(("overview", m)) => cmd_overview(m),
        Some(("top", m)) => cmd_top(m),
        Some(("clock", m)) => cmd_clock(m),
        Some(("sessions", m)) => cmd_sessions(m),
        Some(("skips", m)) => cmd_skips(m),
        Some(("drift", m)) => cmd_drift(m),
        Some(("transitions", m)) => cmd_transitions(m),
        Some(("rotation", m)) => cmd_rotation(m),
        Some(("on-this-day", m)) => cmd_on_this_day(m),
        Some(("origins", _)) => cmd_origins(),
        Some(("query", m)) => cmd_query(m),
        // Bare `analytics` shows the summary, which is what it is for.
        _ => cmd_overview(matches),
    }
}

// ── importing ───────────────────────────────────────────────────────────────

async fn cmd_import(matches: &ArgMatches) -> CmdResult {
    let path = PathBuf::from(matches.get_one::<String>("path").expect("path is required"));
    let format = match matches.get_one::<String>("format") {
        Some(forced) => forced.parse()?,
        None => import::detect(&path)?,
    };

    println!(
        "{} {} export from {}",
        "importing".bold(),
        format.to_string().bright_cyan(),
        path.display().to_string().dimmed()
    );

    let analytics = open()?;
    let bars = Bars::new();
    let report = match format {
        import::Format::Spotify => {
            let min_seconds = matches
                .get_one::<String>("min-seconds")
                .and_then(|s| s.parse().ok())
                .unwrap_or(import::spotify::DEFAULT_MIN_SECONDS);
            // Blocking work (DuckDB is synchronous and CPU-bound), but this is
            // a one-shot command with nothing else to run concurrently, so
            // there is no runtime to starve.
            import::spotify::import(&analytics, &path, min_seconds, &bars)?
        }
        import::Format::Lastfm => import::lastfm::import(&analytics, &path, &bars)?,
    };

    print_import(&analytics, &report)?;
    Ok(())
}

async fn cmd_rocksky(matches: &ArgMatches) -> CmdResult {
    let actor = matches
        .get_one::<String>("actor")
        .expect("actor is required");
    println!(
        "{} scrobbles for {}",
        "importing".bold(),
        actor.bright_cyan()
    );

    let analytics = open()?;
    let bars = Bars::new();
    let report =
        import::rocksky::import(&analytics, actor, matches.get_flag("full"), &bars).await?;
    print_import(&analytics, &report)?;
    Ok(())
}

/// The shared tail of every import: what came in, what was skipped, and what
/// the corpus looks like now.
fn print_import(analytics: &Analytics, report: &import::ImportReport) -> CmdResult {
    analytics.refresh_dedup()?;

    println!(
        "  {} read, {} imported",
        report.read.to_string().bright_white(),
        report.inserted.to_string().bright_green()
    );
    // Reported rather than swallowed: "29,224 records, 28,744 imported" begs
    // the question of where the rest went, and the answer is usually
    // "podcasts", not "a bug".
    for (reason, count) in &report.skipped {
        println!(
            "  {} {} {}",
            "·".dimmed(),
            count.to_string().yellow(),
            reason.dimmed()
        );
    }

    let origins = query::origins(analytics)?;
    let dropped: i64 = origins.iter().map(|o| o.raw - o.canonical).sum();
    if dropped > 0 {
        println!(
            "  {} {} cross-source duplicate(s) folded away",
            "·".dimmed(),
            dropped.to_string().yellow()
        );
    }
    print_origins(&origins);

    println!(
        "\n{}",
        "next: `music-player analytics enrich` to fill in albums, genres and years".dimmed()
    );
    Ok(())
}

async fn cmd_sync() -> CmdResult {
    let analytics = open()?;
    let db = music_player_storage::shared().await;
    println!("{}", "syncing from the player's listen log".bold());
    let report = music_player_analytics::sync::sync(&analytics, db.get_connection()).await?;
    println!(
        "  {} {} listens, {} library tracks",
        "✓".bright_green(),
        report.listens.to_string().bright_green(),
        report.tracks.to_string().bright_green()
    );
    Ok(())
}

async fn cmd_enrich(matches: &ArgMatches) -> CmdResult {
    let analytics = open()?;
    println!(
        "{}",
        "resolving metadata for imported tracks via Rocksky".bold()
    );
    let bars = Bars::new();
    let report =
        music_player_analytics::enrich::enrich(&analytics, limit(matches, 2_000), &bars).await?;
    if report.attempted == 0 {
        return Ok(());
    }
    println!(
        "  {} of {} tracks resolved",
        report.resolved.to_string().bright_green(),
        report.attempted.to_string().bright_white()
    );
    Ok(())
}

// ── reporting ───────────────────────────────────────────────────────────────

fn cmd_overview(matches: &ArgMatches) -> CmdResult {
    let analytics = open_ro()?;
    let scope = scope(matches);
    let o = query::overview(&analytics, scope)?;

    if o.listens == 0 {
        println!(
            "{}",
            "No listening history yet. Import one with `music-player analytics import <path>`, \
             or `analytics sync` to pull in what the player has recorded."
                .dimmed()
        );
        return Ok(());
    }

    heading("LISTENING");
    let mut builder = Builder::default();
    builder.set_columns(vec!["", ""]);
    builder.add_record(vec!["listens", &group(o.listens)]);
    builder.add_record(vec!["distinct tracks", &group(o.distinct_tracks)]);
    builder.add_record(vec!["distinct artists", &group(o.distinct_artists)]);
    builder.add_record(vec!["distinct albums", &group(o.distinct_albums)]);
    builder.add_record(vec!["hours played", &format!("{:.1}", o.hours_played)]);
    builder.add_record(vec!["sessions", &group(o.sessions)]);
    builder.add_record(vec!["active days", &group(o.active_days)]);
    builder.add_record(vec![
        "longest streak",
        &format!("{} days", o.longest_streak),
    ]);
    if let (Some(first), Some(last)) = (&o.first_listen, &o.last_listen) {
        builder.add_record(vec!["from", &day(first)]);
        builder.add_record(vec!["to", &day(last)]);
    }
    println!("{}", builder.build().with(Style::rounded()));

    let top = query::top(&analytics, TopKind::Artists, scope, 5)?;
    if !top.is_empty() {
        heading("TOP ARTISTS");
        print_ranks(&top, false);
    }
    Ok(())
}

fn cmd_top(matches: &ArgMatches) -> CmdResult {
    let kind = match matches
        .get_one::<String>("what")
        .map(String::as_str)
        .unwrap_or("artists")
    {
        "artists" | "artist" => TopKind::Artists,
        "tracks" | "track" | "songs" => TopKind::Tracks,
        "albums" | "album" => TopKind::Albums,
        "genres" | "genre" => TopKind::Genres,
        other => return Err(format!("don't know how to rank '{other}'").into()),
    };
    let analytics = open_ro()?;
    let rows = query::top(&analytics, kind, scope(matches), limit(matches, 20))?;
    if rows.is_empty() {
        println!("{}", "Nothing in this window.".dimmed());
        return Ok(());
    }
    print_ranks(&rows, !matches!(kind, TopKind::Artists | TopKind::Genres));
    Ok(())
}

fn cmd_clock(matches: &ArgMatches) -> CmdResult {
    let analytics = open_ro()?;
    let cells = query::clock(&analytics, scope(matches))?;
    if cells.is_empty() {
        println!("{}", "Nothing in this window.".dimmed());
        return Ok(());
    }

    // A heatmap rather than a table: the shape of a week is the answer here,
    // and 168 numbers in a grid is not readable.
    let peak = cells.iter().map(|c| c.listens).max().unwrap_or(1).max(1);
    let blocks = [' ', '░', '▒', '▓', '█'];

    heading("LISTENING CLOCK");
    print!("      ");
    for hour in 0..24 {
        print!(
            "{}",
            if hour % 6 == 0 {
                format!("{hour:<2}")
            } else {
                "  ".into()
            }
        );
    }
    println!();

    for (index, name) in ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]
        .iter()
        .enumerate()
    {
        print!("  {name}  ");
        for hour in 0..24 {
            let listens = cells
                .iter()
                .find(|c| c.weekday == index as i32 && c.hour == hour)
                .map(|c| c.listens)
                .unwrap_or(0);
            // Scaled against the busiest hour, so the picture is about this
            // library rather than about absolute counts.
            let level = if listens == 0 {
                0
            } else {
                (((listens as f64 / peak as f64) * 4.0).ceil() as usize).clamp(1, 4)
            };
            let cell = format!("{}{}", blocks[level], blocks[level]);
            print!("{}", cell.cyan());
        }
        println!();
    }
    println!("  {}", format!("busiest hour: {peak} listens").dimmed());
    Ok(())
}

fn cmd_sessions(matches: &ArgMatches) -> CmdResult {
    let analytics = open_ro()?;
    let stats = query::session_stats(&analytics, scope(matches), limit(matches, 10))?;
    if stats.sessions == 0 {
        println!("{}", "Nothing in this window.".dimmed());
        return Ok(());
    }

    heading("SESSIONS");
    let mut builder = Builder::default();
    builder.set_columns(vec!["", ""]);
    builder.add_record(vec!["sessions", &group(stats.sessions)]);
    builder.add_record(vec!["avg tracks", &format!("{:.1}", stats.avg_tracks)]);
    builder.add_record(vec![
        "median length",
        &format!("{:.0} min", stats.median_minutes),
    ]);
    builder.add_record(vec![
        "longest",
        &format!("{:.0} min", stats.longest_minutes),
    ]);
    println!("{}", builder.build().with(Style::rounded()));

    if !stats.openers.is_empty() {
        heading("WHAT YOU PUT ON FIRST");
        print_ranks(&stats.openers, true);
    }
    Ok(())
}

fn cmd_skips(matches: &ArgMatches) -> CmdResult {
    let analytics = open_ro()?;
    let min_listens = matches
        .get_one::<String>("min-listens")
        .and_then(|m| m.parse().ok())
        .unwrap_or(3);
    let rows = query::skips(&analytics, scope(matches), limit(matches, 20), min_listens)?;
    if rows.is_empty() {
        println!(
            "{}",
            "No skip data — only this player and a Spotify import record how much was played."
                .dimmed()
        );
        return Ok(());
    }

    heading("MOST SKIPPED");
    let mut builder = Builder::default();
    builder.set_columns(vec!["#", "track", "artist", "skips", "rate", "heard"]);
    for (i, row) in rows.iter().enumerate() {
        builder.add_record(vec![
            (i + 1).to_string(),
            truncate(&row.name, 34),
            truncate(&row.secondary, 24),
            format!("{}/{}", row.skips, row.listens),
            format!("{:.0}%", row.skip_rate * 100.0),
            row.median_completion
                .map(|c| format!("{:.0}%", c * 100.0))
                .unwrap_or_else(|| "—".into()),
        ]);
    }
    println!("{}", builder.build().with(Style::rounded()));
    Ok(())
}

fn cmd_drift(matches: &ArgMatches) -> CmdResult {
    let analytics = open_ro()?;
    let bucket = matches
        .get_one::<String>("bucket")
        .map(String::as_str)
        .unwrap_or("month");
    let points = query::drift(&analytics, scope(matches), bucket)?;
    if points.is_empty() {
        println!("{}", "Nothing in this window.".dimmed());
        return Ok(());
    }

    heading("HOW YOUR LISTENING MOVED");
    let mut builder = Builder::default();
    builder.set_columns(vec![
        "period",
        "listens",
        "bpm",
        "valence",
        "energy",
        "median year",
        "new",
    ]);
    for point in &points {
        builder.add_record(vec![
            point.bucket.clone(),
            group(point.listens),
            optional(point.avg_bpm, 0),
            optional(point.avg_valence, 2),
            optional(point.avg_arousal, 2),
            point
                .median_year
                .map(|y| format!("{y:.0}"))
                .unwrap_or_else(|| "—".into()),
            format!("{:.0}%", point.discovery_rate * 100.0),
        ]);
    }
    println!("{}", builder.build().with(Style::rounded()));
    // A column of dashes otherwise looks like a broken query rather than
    // missing input.
    if points.iter().all(|p| p.avg_bpm.is_none()) {
        println!(
            "  {}",
            "bpm/valence/energy are blank because the tracks played in those periods \
             have not been analysed — they come from the local library's audio analysis"
                .dimmed()
        );
    }
    Ok(())
}

fn cmd_transitions(matches: &ArgMatches) -> CmdResult {
    let analytics = open_ro()?;
    let rows = query::transitions(&analytics, scope(matches), limit(matches, 20))?;
    if rows.is_empty() {
        println!("{}", "Nothing in this window.".dimmed());
        return Ok(());
    }

    heading("WHAT FOLLOWS WHAT");
    let mut builder = Builder::default();
    builder.set_columns(vec!["from", "to", "times"]);
    for row in &rows {
        builder.add_record(vec![
            format!(
                "{} — {}",
                truncate(&row.from_title, 26),
                truncate(&row.from_artist, 18)
            ),
            format!(
                "{} — {}",
                truncate(&row.to_title, 26),
                truncate(&row.to_artist, 18)
            ),
            row.times.to_string(),
        ]);
    }
    println!("{}", builder.build().with(Style::rounded()));
    Ok(())
}

fn cmd_rotation(matches: &ArgMatches) -> CmdResult {
    let analytics = open_ro()?;
    let r = query::rotation(&analytics, scope(matches), limit(matches, 10))?;

    heading("ROTATION");
    let mut builder = Builder::default();
    builder.set_columns(vec!["", ""]);
    builder.add_record(vec![
        "top 1% of tracks",
        &format!("{:.0}% of listens", r.top_1_percent_share * 100.0),
    ]);
    builder.add_record(vec![
        "top 10% of tracks",
        &format!("{:.0}% of listens", r.top_10_percent_share * 100.0),
    ]);
    builder.add_record(vec!["concentration (gini)", &format!("{:.2}", r.gini)]);
    if r.library_tracks > 0 {
        // Spelled out because the bare figure reads as "you have barely
        // touched your library" when the truth is usually that most listening
        // went through a streaming server, whose tracks are not library files.
        builder.add_record(vec![
            "local files played",
            &format!(
                "{} of {} ({:.0}%)",
                group(r.library_played),
                group(r.library_tracks),
                r.library_played as f64 / r.library_tracks as f64 * 100.0
            ),
        ]);
    }
    println!("{}", builder.build().with(Style::rounded()));

    if !r.rediscover.is_empty() {
        heading("LOVED ONCE, NOT PLAYED IN A WHILE");
        print_ranks(&r.rediscover, true);
    }
    Ok(())
}

fn cmd_on_this_day(matches: &ArgMatches) -> CmdResult {
    let analytics = open_ro()?;
    let rows = query::on_this_day(&analytics, limit(matches, 20))?;
    if rows.is_empty() {
        println!("{}", "Nothing on this date in previous years.".dimmed());
        return Ok(());
    }
    heading("ON THIS DAY");
    print_ranks(&rows, true);
    Ok(())
}

fn cmd_origins() -> CmdResult {
    let analytics = open_ro()?;
    print_origins(&query::origins(&analytics)?);
    Ok(())
}

/// An escape hatch: the whole point of keeping this in DuckDB is that the data
/// is queryable, and no fixed set of commands will cover every question.
fn cmd_query(matches: &ArgMatches) -> CmdResult {
    let sql = matches.get_one::<String>("sql").expect("sql is required");
    let analytics = open_ro()?;
    let mut statement = analytics.conn().prepare(sql)?;
    let mut rows = statement.query([])?;

    let mut builder = Builder::default();
    let mut columns: Option<Vec<String>> = None;
    while let Some(row) = rows.next()? {
        let statement = row.as_ref();
        if columns.is_none() {
            let names: Vec<String> = statement
                .column_names()
                .iter()
                .map(|c| c.to_string())
                .collect();
            builder.set_columns(names.clone());
            columns = Some(names);
        }
        let width = columns.as_ref().map(Vec::len).unwrap_or(0);
        let mut record = Vec::with_capacity(width);
        for i in 0..width {
            record.push(cell(row, i));
        }
        builder.add_record(record);
    }
    match columns {
        Some(_) => println!("{}", builder.build().with(Style::rounded())),
        None => println!("{}", "(no rows)".dimmed()),
    }
    Ok(())
}

/// Render a value without needing to know its type up front.
fn cell(row: &music_player_analytics::duckdb::Row<'_>, index: usize) -> String {
    use music_player_analytics::duckdb::types::ValueRef;

    match row.get_ref(index) {
        Ok(ValueRef::Null) => "—".to_string(),
        Ok(ValueRef::Text(bytes)) => String::from_utf8_lossy(bytes).into_owned(),
        // `Int(42)` -> `42`, `Boolean(true)` -> `true`. Stripping leading
        // alphabetic characters instead ate the *value* of anything spelled
        // with letters, so every boolean in a query result rendered blank.
        Ok(other) => {
            let text = format!("{other:?}");
            match (text.find('('), text.rfind(')')) {
                (Some(open), Some(close)) if close > open => text[open + 1..close].to_string(),
                _ => text,
            }
        }
        Err(_) => String::new(),
    }
}

// ── shared rendering ────────────────────────────────────────────────────────

fn print_ranks(rows: &[query::Rank], with_artist: bool) {
    let mut builder = Builder::default();
    if with_artist {
        builder.set_columns(vec!["#", "title", "artist", "plays", "last"]);
    } else {
        builder.set_columns(vec!["#", "name", "plays", "last"]);
    }
    for (i, row) in rows.iter().enumerate() {
        let mut record = vec![(i + 1).to_string(), truncate(&row.name, 38)];
        if with_artist {
            record.push(truncate(&row.secondary, 26));
        }
        record.push(group(row.listens));
        record.push(
            row.last_played
                .as_deref()
                .map(day)
                .unwrap_or_else(|| "—".into()),
        );
        builder.add_record(record);
    }
    println!("{}", builder.build().with(Style::rounded()));
}

fn print_origins(origins: &[query::OriginRow]) {
    if origins.is_empty() {
        return;
    }
    heading("SOURCES");
    let mut builder = Builder::default();
    builder.set_columns(vec!["source", "listens", "after dedup", "from", "to"]);
    for origin in origins {
        builder.add_record(vec![
            origin.origin.clone(),
            group(origin.raw),
            group(origin.canonical),
            origin
                .first
                .as_deref()
                .map(day)
                .unwrap_or_else(|| "—".into()),
            origin
                .last
                .as_deref()
                .map(day)
                .unwrap_or_else(|| "—".into()),
        ]);
    }
    println!("{}", builder.build().with(Style::rounded()));
}

/// `2025-06-02 07:25:07+03` -> `2025-06-02`. The time is noise in a table of
/// dates spanning years.
fn day(timestamp: &str) -> String {
    timestamp.split(' ').next().unwrap_or(timestamp).to_string()
}

/// Thousands separators. Nine years of history runs to five figures, and
/// `28744` is harder to read at a glance than `28,744`.
fn group(n: i64) -> String {
    let digits = n.abs().to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    if n < 0 {
        format!("-{out}")
    } else {
        out
    }
}

fn optional(value: Option<f64>, precision: usize) -> String {
    value
        .map(|v| format!("{v:.precision$}"))
        .unwrap_or_else(|| "—".into())
}

fn truncate(text: &str, width: usize) -> String {
    // By characters, not bytes: a title can be in any script, and slicing a
    // multi-byte character in half panics.
    if text.chars().count() <= width {
        return text.to_string();
    }
    let kept: String = text.chars().take(width.saturating_sub(1)).collect();
    format!("{kept}…")
}
