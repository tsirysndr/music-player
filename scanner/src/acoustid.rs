//! Asking AcoustID what a fingerprint is.
//!
//! This is the half of identification that leaves the machine. A fingerprint
//! goes out, and what comes back is a MusicBrainz recording — a title, the
//! artists who performed it, and the releases it appeared on. It is the same
//! exchange MusicBrainz Picard makes, and the reason a file called
//! `track03.mp3` can end up correctly tagged without anyone typing anything.
//!
//! Two things are worth being careful about here, and both are about being a
//! good guest on someone else's free service. Only tracks whose tags are
//! actually missing are asked about, and every answer — including "I don't
//! know" — is stored, so the same question is never asked twice.

use anyhow::{Error, Result};
use music_player_storage::track_fingerprint::Identity;
use serde::Deserialize;

/// The public AcoustID endpoint.
const DEFAULT_API_URL: &str = "https://api.acoustid.org";

/// How long a lookup may take before it is given up on. Generous: this runs
/// in the background and a retry costs another slot in someone else's rate
/// limit, so waiting is cheaper than asking again.
const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(20);

/// How far a candidate recording's length may be from ours before it is
/// rejected outright.
///
/// AcoustID returns every recording its index associates with a fingerprint,
/// and for a widely sampled song that includes remixes, radio edits and
/// whole live versions. The audio matched; the recording still might not be
/// the one on disk, and length is the cheapest way to tell.
const DURATION_TOLERANCE: i64 = 15;

/// The tags a lookup can fill in.
///
/// Every field optional and separately so: AcoustID knows the recording for
/// certain and the release only by inference, so a lookup routinely yields a
/// confident title and artist alongside no album at all.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Tags {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub year: Option<u32>,
    pub track: Option<u32>,
    pub disc: Option<u32>,
}

impl Tags {
    /// Whether this says anything at all.
    pub fn is_empty(&self) -> bool {
        *self == Tags::default()
    }
}

/// What a fingerprint turned out to be.
#[derive(Clone, Debug, PartialEq)]
pub struct Identified {
    pub identity: Identity,
    pub tags: Tags,
}

/// A client for the AcoustID lookup API.
///
/// Holds one `reqwest::Client` for the whole pass. Building one per request
/// would open a fresh connection pool per track — thousands of them over a
/// library — for a service that is perfectly happy to keep one connection.
pub struct AcoustId {
    http: reqwest::Client,
    api_key: String,
    api_url: String,
}

impl AcoustId {
    /// A client, if there is an api key to use.
    ///
    /// AcoustID requires one and they are free, but they are per-application
    /// and cannot be invented — so with no key configured this returns
    /// `None` and the pass that needs it says so once and stops. Fingerprints
    /// are still computed and stored either way; they are what a duplicate
    /// check compares, and they are what makes turning the key on later a
    /// lookup rather than a rescan.
    pub fn new(api_key: &str) -> Option<Self> {
        let api_key = api_key.trim();
        if api_key.is_empty() {
            return None;
        }
        Some(Self {
            http: reqwest::Client::builder().timeout(TIMEOUT).build().ok()?,
            api_key: api_key.to_string(),
            api_url: std::env::var("ACOUSTID_API_URL")
                .unwrap_or_else(|_| DEFAULT_API_URL.to_string()),
        })
    }

    /// Ask what a fingerprint is. `Ok(None)` means AcoustID answered and does
    /// not know — a real answer, and one worth storing.
    pub async fn lookup(&self, fingerprint: &str, duration: u32) -> Result<Option<Identified>> {
        // POST, not GET: a two-minute fingerprint is a couple of kilobytes of
        // base64 and belongs in a body rather than in a url that a proxy is
        // entitled to truncate.
        let response = self
            .http
            .post(format!("{}/v2/lookup", self.api_url))
            .form(&[
                ("client", self.api_key.as_str()),
                ("duration", &duration.to_string()),
                ("fingerprint", fingerprint),
                // Enough to tag with: the recording names it, the releases say
                // which album it came from, and `tracks` gives the position on
                // that album — which is the one thing a track number can
                // honestly come from.
                ("meta", "recordings+releases+tracks"),
            ])
            .send()
            .await?
            .error_for_status()?;

        let body: LookupResponse = response.json().await?;
        if body.status != "ok" {
            let message = body
                .error
                .map(|e| e.message)
                .unwrap_or_else(|| body.status.clone());
            return Err(Error::msg(format!("acoustid: {message}")));
        }
        Ok(best_match(&body, duration))
    }
}

/// Pick the recording a fingerprint most likely is.
///
/// Separated from the request so the judgement can be tested against real
/// responses without a network — which matters, because this is where a
/// lookup goes subtly wrong: picking a remix over the album version leaves a
/// file *worse* tagged than the blank it started with.
fn best_match(body: &LookupResponse, duration: u32) -> Option<Identified> {
    let wanted = duration as i64;

    // Every (result, recording) pair, ranked. AcoustID sorts results by score
    // but a result holds several recordings, and the best of those is not
    // necessarily under the best-scoring result.
    let mut candidates: Vec<(&AcoustIdResult, Option<&Recording>)> = Vec::new();
    for result in &body.results {
        if result.recordings.is_empty() {
            // A fingerprint the index knows but nothing has been attached to.
            // Still an identity — it is what a later lookup will return — but
            // there are no tags in it.
            candidates.push((result, None));
            continue;
        }
        for recording in &result.recordings {
            candidates.push((result, Some(recording)));
        }
    }

    candidates.retain(|(_, recording)| match recording.and_then(|r| r.duration) {
        // A length that disagrees this much is a different recording, however
        // well the two minutes we fingerprinted happened to line up.
        Some(length) => (length as i64 - wanted).abs() <= DURATION_TOLERANCE,
        // Plenty of recordings have no length stored. Not knowing is not a
        // reason to reject — it is a reason to prefer one that does.
        None => true,
    });

    candidates.sort_by(|(left_result, left), (right_result, right)| {
        let closeness = |recording: &Option<&Recording>| {
            recording
                .and_then(|r| r.duration)
                .map(|length| (length as i64 - wanted).abs())
                // Sorts behind every candidate whose length is known and fits.
                .unwrap_or(DURATION_TOLERANCE + 1)
        };
        // Score first — it is AcoustID's own confidence that the audio matched
        // at all — then length, to choose among the recordings it matched to.
        right_result
            .score
            .partial_cmp(&left_result.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(closeness(left).cmp(&closeness(right)))
            .then(has_tags(right).cmp(&has_tags(left)))
    });

    let (result, recording) = candidates.into_iter().next()?;
    let release = recording.and_then(|r| pick_release(&r.releases));

    Some(Identified {
        identity: Identity {
            acoustid: result.id.clone(),
            recording_mbid: recording.map(|r| r.id.clone()),
            release_mbid: release.and_then(|r| r.id.clone()),
            score: result.score as f32,
        },
        tags: Tags {
            title: recording.and_then(|r| non_empty(r.title.as_deref())),
            artist: recording.and_then(|r| credit(&r.artists)),
            album: release.and_then(|r| non_empty(r.title.as_deref())),
            album_artist: release
                .and_then(|r| credit(&r.artists))
                // A release with no credit of its own is a single-artist
                // release, and the performer is the album artist.
                .or_else(|| recording.and_then(|r| credit(&r.artists))),
            // Negative years exist in MusicBrainz's schema and not in a
            // music library; a nonsensical one is no year at all.
            year: release
                .and_then(|r| r.date.as_ref())
                .and_then(|d| d.year)
                .and_then(|year| u32::try_from(year).ok()),
            track: release.and_then(|r| position(r).map(|(_, track)| track)),
            // Only when the release really has more than one disc: a
            // single-disc album must not grow a "DISC 1" header because
            // MusicBrainz numbers its one medium.
            disc: release.and_then(|r| match position(r) {
                Some((disc, _)) if r.mediums.len() > 1 => Some(disc),
                _ => None,
            }),
        },
    })
}

/// Whether a candidate carries anything worth tagging with. The last
/// tie-break: between two equally good matches, take the one that says
/// something.
fn has_tags(recording: &Option<&Recording>) -> bool {
    recording.is_some_and(|r| r.title.is_some() || !r.artists.is_empty())
}

/// Which release of a recording to tag from.
///
/// The earliest dated one, which is the original album rather than the
/// compilation or the anniversary reissue that a popular track accumulates.
/// It is a heuristic and it is sometimes wrong, but it is wrong in the
/// direction a listener expects — and it only ever fills in a field the file
/// left blank.
fn pick_release(releases: &[Release]) -> Option<&Release> {
    releases.iter().min_by_key(|release| {
        (
            // Undated releases sort last rather than first: a missing date
            // is not the year zero.
            release
                .date
                .as_ref()
                .and_then(|d| d.year)
                .unwrap_or(i32::MAX),
            // Stable among releases sharing a year, so the same lookup
            // twice tags the same way.
            release.id.clone().unwrap_or_default(),
        )
    })
}

/// Where this recording sits on a release, as (disc, track).
fn position(release: &Release) -> Option<(u32, u32)> {
    // With `meta=tracks` a medium lists only the tracks that *are* this
    // recording, so the first one found is the answer rather than a search.
    release
        .mediums
        .iter()
        .enumerate()
        .find_map(|(index, medium)| {
            let track = medium.tracks.first()?.position?;
            Some((medium.position.unwrap_or(index as u32 + 1), track))
        })
}

/// The artist credit as it should be displayed: the names in order, joined
/// the way MusicBrainz says to join them, so "Jay-Z feat. Alicia Keys" comes
/// out as one string rather than two rows.
fn credit(artists: &[ArtistCredit]) -> Option<String> {
    if artists.is_empty() {
        return None;
    }
    let mut credited = String::new();
    for artist in artists {
        credited.push_str(artist.name.as_deref().unwrap_or_default());
        credited.push_str(artist.joinphrase.as_deref().unwrap_or_default());
    }
    non_empty(Some(credited.trim()))
}

fn non_empty(value: Option<&str>) -> Option<String> {
    let value = value?.trim();
    (!value.is_empty()).then(|| value.to_string())
}

// ── The wire format ─────────────────────────────────────────────────────────
//
// Deliberately lenient: every field optional, every list defaulted. AcoustID
// returns what MusicBrainz happens to hold, which varies per recording, and a
// strict struct would turn a partially known release into a parse failure and
// lose the title along with it.

#[derive(Debug, Deserialize)]
struct LookupResponse {
    status: String,
    error: Option<ApiError>,
    #[serde(default)]
    results: Vec<AcoustIdResult>,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct AcoustIdResult {
    id: String,
    #[serde(default)]
    score: f64,
    #[serde(default)]
    recordings: Vec<Recording>,
}

#[derive(Debug, Deserialize)]
struct Recording {
    id: String,
    title: Option<String>,
    /// Seconds.
    duration: Option<u32>,
    #[serde(default)]
    artists: Vec<ArtistCredit>,
    #[serde(default)]
    releases: Vec<Release>,
}

#[derive(Debug, Deserialize)]
struct ArtistCredit {
    name: Option<String>,
    joinphrase: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Release {
    id: Option<String>,
    title: Option<String>,
    date: Option<PartialDate>,
    #[serde(default)]
    artists: Vec<ArtistCredit>,
    #[serde(default)]
    mediums: Vec<Medium>,
}

#[derive(Debug, Deserialize)]
struct PartialDate {
    year: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct Medium {
    position: Option<u32>,
    #[serde(default)]
    tracks: Vec<TrackPosition>,
}

#[derive(Debug, Deserialize)]
struct TrackPosition {
    position: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(json: &str) -> LookupResponse {
        serde_json::from_str(json).unwrap()
    }

    /// The shape of a real answer, trimmed to the fields that are read.
    const HIT: &str = r#"{
        "status": "ok",
        "results": [{
            "id": "9ff43b6a-4f16-427c-93c2-92307ca505e0",
            "score": 0.96,
            "recordings": [{
                "id": "b7d9f4ab-1111-2222-3333-444455556666",
                "title": "Wet Dreamz",
                "duration": 239,
                "artists": [{"id": "a1", "name": "J. Cole"}],
                "releases": [{
                    "id": "r-later",
                    "title": "2014 Forest Hills Drive (Deluxe)",
                    "date": {"year": 2018},
                    "mediums": [{"position": 1, "tracks": [{"position": 9}]}]
                }, {
                    "id": "r-first",
                    "title": "2014 Forest Hills Drive",
                    "date": {"year": 2014},
                    "mediums": [{"position": 1, "tracks": [{"position": 3}]}]
                }]
            }]
        }]
    }"#;

    #[test]
    fn a_hit_yields_tags_from_the_earliest_release() {
        let found = best_match(&parse(HIT), 239).unwrap();
        assert_eq!(
            found.identity.acoustid,
            "9ff43b6a-4f16-427c-93c2-92307ca505e0"
        );
        assert_eq!(
            found.identity.recording_mbid.as_deref(),
            Some("b7d9f4ab-1111-2222-3333-444455556666")
        );
        assert_eq!(found.identity.release_mbid.as_deref(), Some("r-first"));
        assert_eq!(found.tags.title.as_deref(), Some("Wet Dreamz"));
        assert_eq!(found.tags.artist.as_deref(), Some("J. Cole"));
        assert_eq!(found.tags.album.as_deref(), Some("2014 Forest Hills Drive"));
        assert_eq!(found.tags.year, Some(2014));
        assert_eq!(found.tags.track, Some(3));
        // One medium: not a disc worth numbering.
        assert_eq!(found.tags.disc, None);
    }

    /// The case that makes a fingerprint worth having *and* dangerous: the
    /// audio matches a remix too, and tagging the album version as the remix
    /// is worse than leaving it blank.
    #[test]
    fn a_recording_of_the_wrong_length_is_not_this_track() {
        let json = r#"{
            "status": "ok",
            "results": [{
                "id": "acoust-1", "score": 0.9,
                "recordings": [
                    {"id": "remix", "title": "Song (Extended Mix)", "duration": 480,
                     "artists": [{"name": "Someone"}]},
                    {"id": "album", "title": "Song", "duration": 212,
                     "artists": [{"name": "Someone"}]}
                ]
            }]
        }"#;
        let found = best_match(&parse(json), 210).unwrap();
        assert_eq!(found.identity.recording_mbid.as_deref(), Some("album"));
        assert_eq!(found.tags.title.as_deref(), Some("Song"));
    }

    /// A featured artist is one credit, spelled the way MusicBrainz spells it.
    #[test]
    fn artist_credits_are_joined_as_credited() {
        let json = r#"{
            "status": "ok",
            "results": [{
                "id": "acoust-1", "score": 0.9,
                "recordings": [{"id": "rec", "title": "Empire State of Mind", "duration": 276,
                    "artists": [
                        {"name": "JAY-Z", "joinphrase": " feat. "},
                        {"name": "Alicia Keys"}
                    ]}]
            }]
        }"#;
        let found = best_match(&parse(json), 276).unwrap();
        assert_eq!(
            found.tags.artist.as_deref(),
            Some("JAY-Z feat. Alicia Keys")
        );
    }

    /// A multi-disc release numbers its discs; the medium a track is on is
    /// part of where it sits.
    #[test]
    fn a_boxed_set_keeps_its_disc_number() {
        let json = r#"{
            "status": "ok",
            "results": [{
                "id": "acoust-1", "score": 0.9,
                "recordings": [{"id": "rec", "title": "Track", "duration": 200,
                    "releases": [{"id": "r", "title": "Anthology",
                        "mediums": [{"position": 2, "tracks": [{"position": 5}]}, {"position": 3, "tracks": []}]}]}]
            }]
        }"#;
        let found = best_match(&parse(json), 200).unwrap();
        assert_eq!(found.tags.disc, Some(2));
        assert_eq!(found.tags.track, Some(5));
    }

    /// A fingerprint the index knows, attached to nothing. Worth storing as
    /// an identity — it is stable, and a later lookup returns the same — but
    /// there is nothing here to tag with.
    #[test]
    fn a_known_fingerprint_with_no_recording_is_still_an_identity() {
        let json = r#"{"status": "ok", "results": [{"id": "acoust-1", "score": 0.8}]}"#;
        let found = best_match(&parse(json), 200).unwrap();
        assert_eq!(found.identity.acoustid, "acoust-1");
        assert!(found.identity.recording_mbid.is_none());
        assert!(found.tags.is_empty());
    }

    /// Nothing known at all.
    #[test]
    fn no_results_is_no_match() {
        let json = r#"{"status": "ok", "results": []}"#;
        assert!(best_match(&parse(json), 200).is_none());
    }

    /// A response holding fields we do not model must still parse — losing a
    /// title because MusicBrainz grew a key is the failure mode a strict
    /// struct has.
    #[test]
    fn unknown_fields_do_not_break_a_lookup() {
        let json = r#"{
            "status": "ok",
            "results": [{"id": "acoust-1", "score": 0.9, "sources": 42,
                "recordings": [{"id": "rec", "title": "Track", "duration": 200,
                    "sources": 12, "releasegroups": [{"id": "rg", "type": "Album"}]}]}]
        }"#;
        let found = best_match(&parse(json), 200).unwrap();
        assert_eq!(found.tags.title.as_deref(), Some("Track"));
    }

    /// No key, no client — and the pass that wanted one has to say so rather
    /// than send requests that will be refused.
    #[test]
    fn a_blank_api_key_is_no_client() {
        assert!(AcoustId::new("").is_none());
        assert!(AcoustId::new("   ").is_none());
        assert!(AcoustId::new("abc123").is_some());
    }
}
