//! Choosing what plays next.
//!
//! A set is built as a **chain**, not as a ranking. Each pick is measured
//! against the track before it rather than against the original seed, so an
//! hour of music can travel a long way from where it started while every
//! individual transition is close. Ranking everything against one seed gives
//! the opposite: a set that never leaves the first track's neighbourhood and
//! then falls off a cliff when it runs out.
//!
//! Nothing here touches the database or the player. It takes candidates and
//! returns ids, which is what makes the rules testable — the interesting
//! behaviour is in what gets picked, not in how it is fetched.

use std::collections::HashSet;

use music_player_analysis::Analysis;

/// A track that could be played, with what is known about how it sounds.
#[derive(Clone, Debug)]
pub struct Candidate {
    pub track_id: String,
    /// Used only to avoid playing the same artist twice in a row. Empty is
    /// treated as "unknown", which never blocks anything.
    pub artist: String,
    pub analysis: Analysis,
}

/// Where the set should be heading.
///
/// Every field is optional and absent means "wherever the music goes". A
/// target of all-`None` still produces a coherent set, because the chain keeps
/// each transition close to the last track; the target only says which
/// direction to lean while doing it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Target {
    pub bpm: Option<f32>,
    /// -1 dark to 1 bright.
    pub valence: Option<f32>,
    /// 0 calm to 1 driving.
    pub arousal: Option<f32>,
}

impl Target {
    pub fn is_empty(&self) -> bool {
        self.bpm.is_none() && self.valence.is_none() && self.arousal.is_none()
    }
}

/// How much a target pulls against the previous track.
///
/// Set so a target can overcome roughly one step of chain distance, and no
/// more. That is the whole balance: lower and a stated target changes nothing —
/// the next track is always the nearest one and "make it more energetic" is
/// ignored — while higher and every pick jumps straight to the most extreme
/// track that matches, which is a filter rather than a set.
///
/// At this weight, asking for energy from a calm seed moves up through the
/// track next to it, and does not land on the most frantic thing in the
/// library.
const TARGET_PULL: f32 = 0.8;

/// The widest tempo gap that still counts as a match, in BPM.
///
/// Chosen so the term saturates rather than dominating: past this, one track is
/// simply "a different tempo" and mood should decide between the candidates.
const TEMPO_SPAN: f32 = 40.0;

/// Plan the next `count` tracks.
///
/// `seed` is what is playing, or what the last set ended on. `exclude` is
/// everything already played or queued — the caller owns that, because only it
/// knows what is in the queue.
pub fn plan(
    candidates: &[Candidate],
    seed: Option<&Analysis>,
    target: &Target,
    exclude: &HashSet<String>,
    count: usize,
) -> Vec<String> {
    let mut chosen = Vec::with_capacity(count);
    let mut taken: HashSet<String> = exclude.clone();
    // Not an Option<&Analysis> because each step re-centres on what it picked.
    let mut current = seed.cloned();
    let mut last_artist = String::new();

    for _ in 0..count {
        let Some(next) = nearest(candidates, current.as_ref(), target, &taken, &last_artist) else {
            break;
        };
        taken.insert(next.track_id.clone());
        chosen.push(next.track_id.clone());
        last_artist = next.artist.clone();
        current = Some(next.analysis.clone());
    }

    chosen
}

/// The best candidate to follow `current`.
fn nearest<'a>(
    candidates: &'a [Candidate],
    current: Option<&Analysis>,
    target: &Target,
    taken: &HashSet<String>,
    last_artist: &str,
) -> Option<&'a Candidate> {
    let mut best: Option<(&Candidate, f32)> = None;

    for candidate in candidates {
        if taken.contains(&candidate.track_id) {
            continue;
        }

        let mut cost = match current {
            Some(current) => distance(current, &candidate.analysis),
            // Nothing to follow: rank purely by the target, or take anything.
            None => 0.0,
        };
        if !target.is_empty() {
            cost += TARGET_PULL * target_distance(target, &candidate.analysis);
        }
        // Two tracks by the same artist back to back is the most obvious way an
        // automatic set stops sounding chosen. A penalty rather than a ban:
        // in a library of one artist, a set of that artist is still correct.
        if !last_artist.is_empty() && candidate.artist == last_artist {
            cost += 0.5;
        }

        // Strictly better, so an exact tie keeps the earlier candidate and the
        // result does not depend on iteration order.
        if best.is_none_or(|(_, best_cost)| cost < best_cost) {
            best = Some((candidate, cost));
        }
    }

    best.map(|(candidate, _)| candidate)
}

/// How badly two tracks go together, 0 (identical) upwards.
fn distance(from: &Analysis, to: &Analysis) -> f32 {
    let mut cost = 0.0;
    let mut weight = 0.0;

    if let (Some(a), Some(b)) = (from.bpm, to.bpm) {
        // Weighted by how much the tempos are worth believing. A confident
        // 128 next to a confident 130 is a real match; two guesses that happen
        // to agree are not, and should not outrank a good mood match.
        let confidence = from.bpm_confidence.unwrap_or(0.5).clamp(0.0, 1.0)
            * to.bpm_confidence.unwrap_or(0.5).clamp(0.0, 1.0);
        let gap = (tempo_gap(a, b) / TEMPO_SPAN).min(1.0);
        cost += 1.6 * confidence * gap;
        weight += 1.6 * confidence;
    }

    // Energy carries a transition more than brightness does: a bright track and
    // a dark one at the same energy still flow, the reverse rarely does.
    if let (Some(a), Some(b)) = (from.arousal, to.arousal) {
        cost += 1.2 * (a - b).abs();
        weight += 1.2;
    }
    if let (Some(a), Some(b)) = (from.valence, to.valence) {
        // Valence spans -1..1, so halved to sit on the same 0..1 scale.
        cost += 0.7 * ((a - b).abs() / 2.0);
        weight += 0.7;
    }

    if weight == 0.0 {
        // Nothing comparable. Mid-distance rather than zero, so a track with no
        // analysis at all does not look like a perfect match for everything.
        return 0.5;
    }
    cost / weight
}

/// The gap between two tempos, allowing for half and double time.
///
/// 140 and 70 are the same tempo counted differently, and a set that moves
/// between them is doing what a DJ does. Without this the two look 70 BPM apart
/// — further than anything else in the library — and never follow each other.
fn tempo_gap(a: f32, b: f32) -> f32 {
    [(a - b).abs(), (a - b * 2.0).abs(), (a * 2.0 - b).abs()]
        .into_iter()
        .fold(f32::INFINITY, f32::min)
}

/// How far a track is from where the set is meant to be heading.
fn target_distance(target: &Target, analysis: &Analysis) -> f32 {
    let mut cost = 0.0;
    let mut terms = 0.0;

    if let (Some(wanted), Some(actual)) = (target.bpm, analysis.bpm) {
        cost += (tempo_gap(wanted, actual) / TEMPO_SPAN).min(1.0);
        terms += 1.0;
    }
    if let (Some(wanted), Some(actual)) = (target.arousal, analysis.arousal) {
        cost += (wanted - actual).abs();
        terms += 1.0;
    }
    if let (Some(wanted), Some(actual)) = (target.valence, analysis.valence) {
        cost += (wanted - actual).abs() / 2.0;
        terms += 1.0;
    }

    if terms == 0.0 {
        // A target was asked for and this track cannot answer it. Costly, so
        // an analysed track that *can* answer wins — but not disqualifying,
        // since otherwise a specific target could empty the pool entirely.
        return 1.0;
    }
    cost / terms
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(id: &str, artist: &str, bpm: f32, valence: f32, arousal: f32) -> Candidate {
        Candidate {
            track_id: id.into(),
            artist: artist.into(),
            analysis: Analysis {
                bpm: Some(bpm),
                bpm_confidence: Some(0.9),
                valence: Some(valence),
                arousal: Some(arousal),
                ..Default::default()
            },
        }
    }

    fn seed(bpm: f32, valence: f32, arousal: f32) -> Analysis {
        candidate("seed", "seed", bpm, valence, arousal).analysis
    }

    /// The nearest track comes first. The floor everything else builds on.
    #[test]
    fn the_closest_track_is_picked_first() {
        let pool = vec![
            candidate("far", "a", 75.0, -0.8, 0.1),
            candidate("near", "b", 126.0, 0.5, 0.75),
            candidate("middling", "c", 105.0, 0.0, 0.5),
        ];
        let picks = plan(
            &pool,
            Some(&seed(128.0, 0.5, 0.8)),
            &Target::default(),
            &HashSet::new(),
            1,
        );
        assert_eq!(picks, ["near"]);
    }

    /// Half and double time are the same tempo. A set that cannot move between
    /// them is refusing the most ordinary transition there is.
    #[test]
    fn half_time_counts_as_the_same_tempo() {
        assert_eq!(tempo_gap(140.0, 70.0), 0.0);
        assert_eq!(tempo_gap(70.0, 140.0), 0.0);
        assert_eq!(tempo_gap(128.0, 130.0), 2.0);
        // Unrelated tempos are still far apart.
        assert!(tempo_gap(128.0, 95.0) > 30.0);
    }

    /// Each pick is measured against the one before it, so a set travels.
    /// Ranking against the seed would return the same neighbourhood forever.
    #[test]
    fn the_set_travels_away_from_where_it_started() {
        let pool: Vec<Candidate> = (0..10)
            .map(|i| {
                candidate(
                    &format!("t{i}"),
                    &format!("artist{i}"),
                    100.0 + i as f32 * 5.0,
                    0.0,
                    0.5,
                )
            })
            .collect();

        let picks = plan(
            &pool,
            Some(&seed(100.0, 0.0, 0.5)),
            &Target::default(),
            &HashSet::new(),
            5,
        );
        // Walks up the tempo ladder one rung at a time rather than returning
        // the same nearest track's neighbours over and over.
        assert_eq!(picks, ["t0", "t1", "t2", "t3", "t4"]);
    }

    /// Nothing is played twice, and nothing already queued is queued again.
    #[test]
    fn the_excluded_are_never_picked() {
        let pool = vec![
            candidate("played", "a", 128.0, 0.5, 0.8),
            candidate("queued", "b", 127.0, 0.5, 0.8),
            candidate("fresh", "c", 110.0, 0.2, 0.6),
        ];
        let exclude: HashSet<String> = ["played".to_string(), "queued".to_string()].into();
        let picks = plan(
            &pool,
            Some(&seed(128.0, 0.5, 0.8)),
            &Target::default(),
            &exclude,
            5,
        );
        assert_eq!(picks, ["fresh"]);
    }

    /// A plan asks for more than the library can give: it returns what there
    /// is rather than repeating tracks to make up the number.
    #[test]
    fn a_short_library_gives_a_short_set() {
        let pool = vec![candidate("only", "a", 120.0, 0.0, 0.5)];
        let picks = plan(&pool, None, &Target::default(), &HashSet::new(), 10);
        assert_eq!(picks, ["only"]);
    }

    /// Two tracks by one artist should not sit next to each other when there is
    /// an alternative — the clearest sign a set was assembled by a machine.
    #[test]
    fn the_same_artist_does_not_play_twice_in_a_row() {
        let pool = vec![
            candidate("a1", "Artist A", 120.0, 0.0, 0.5),
            // Nearer than the alternative, but the same artist as the pick
            // before it.
            candidate("a2", "Artist A", 120.0, 0.0, 0.5),
            candidate("b1", "Artist B", 124.0, 0.05, 0.55),
        ];
        let picks = plan(
            &pool,
            Some(&seed(120.0, 0.0, 0.5)),
            &Target::default(),
            &HashSet::new(),
            3,
        );
        assert_eq!(picks[0], "a1");
        assert_eq!(picks[1], "b1", "the same artist followed itself");
        // The third has nowhere else to go, and that is fine.
        assert_eq!(picks[2], "a2");
    }

    /// An artist with no name recorded must not be treated as one artist that
    /// everything belongs to — that would penalise every consecutive pick.
    #[test]
    fn an_unknown_artist_never_blocks_anything() {
        let pool = vec![
            candidate("t1", "", 120.0, 0.0, 0.5),
            candidate("t2", "", 121.0, 0.0, 0.5),
        ];
        let picks = plan(
            &pool,
            Some(&seed(120.0, 0.0, 0.5)),
            &Target::default(),
            &HashSet::new(),
            2,
        );
        assert_eq!(picks, ["t1", "t2"]);
    }

    /// A target leans the set in its direction rather than jumping to it.
    ///
    /// Stated as a comparison — the same pool picks differently with and
    /// without the target — because that is the property. Asserting a single
    /// pick would only be asserting the current weights.
    #[test]
    fn a_target_steers_without_teleporting() {
        let pool = vec![
            candidate("calm", "a", 90.0, 0.0, 0.15),
            candidate("stepping-up", "b", 105.0, 0.0, 0.45),
            // Far in energy. Note its tempo is *not* far: 175 is double-time of
            // 90, and the octave folding is right to call that a close match.
            candidate("wild", "c", 175.0, 0.0, 0.98),
        ];
        let from = seed(90.0, 0.0, 0.2);

        // Left alone, the set stays where it is.
        let drifting = plan(&pool, Some(&from), &Target::default(), &HashSet::new(), 1);
        assert_eq!(drifting, ["calm"]);

        // Asked for more energy, it moves — to the track next to where it is,
        // not to the most energetic thing in the library.
        let lift = Target {
            arousal: Some(0.5),
            ..Default::default()
        };
        let steered = plan(&pool, Some(&from), &lift, &HashSet::new(), 1);
        assert_eq!(steered, ["stepping-up"]);
    }

    /// With nothing playing, a target is the only thing to go on and should be
    /// obeyed directly.
    #[test]
    fn with_no_seed_the_target_decides() {
        let pool = vec![
            candidate("calm", "a", 80.0, 0.0, 0.1),
            candidate("driving", "b", 130.0, 0.0, 0.9),
        ];
        let picks = plan(
            &pool,
            None,
            &Target {
                arousal: Some(0.95),
                ..Default::default()
            },
            &HashSet::new(),
            1,
        );
        assert_eq!(picks, ["driving"]);
    }

    #[test]
    fn an_empty_library_plans_nothing() {
        assert!(plan(&[], None, &Target::default(), &HashSet::new(), 5).is_empty());
    }

    /// A track nothing is known about is not a perfect match for everything.
    /// Zero distance would make it win every comparison.
    #[test]
    fn an_unanalysed_track_is_not_a_perfect_match() {
        let known = seed(128.0, 0.5, 0.8);
        let unknown = Analysis::default();
        assert!(distance(&known, &unknown) > 0.0);
        assert!(distance(&known, &known) < distance(&known, &unknown));
    }

    #[test]
    fn an_empty_target_is_recognised_as_empty() {
        assert!(Target::default().is_empty());
        assert!(!Target {
            bpm: Some(120.0),
            ..Default::default()
        }
        .is_empty());
    }
}
