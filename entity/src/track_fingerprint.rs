use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// A track's acoustic fingerprint, and what looking it up turned out to be.
///
/// Two things that belong together because the second is derived from the
/// first: the Chromaprint fingerprint of the audio, and the AcoustID/
/// MusicBrainz identity the fingerprint resolved to. Keeping the lookup result
/// here rather than only folding it into `track` is what stops the pass asking
/// AcoustID the same question every scan — a miss is an answer too, and
/// `looked_up_at` records that it was asked.
///
/// Local files only. A fingerprint is computed from the audio, and the only
/// library whose audio is on this machine is this one.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, Default)]
#[sea_orm(table_name = "track_fingerprint")]
pub struct Model {
    /// The track id — the same md5 of the uri that `track` uses, so a
    /// fingerprint is looked up rather than searched for.
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Chromaprint, compressed, base64url with no padding. What `fpcalc`
    /// prints and what AcoustID's `fingerprint` parameter expects.
    pub fingerprint: String,
    /// The whole track's length in seconds, as decoded. AcoustID matches on
    /// duration as well as audio, so it is stored next to the fingerprint it
    /// has to be sent with.
    pub duration: i32,
    /// The AcoustID uuid, once the fingerprint has been looked up.
    pub acoustid: Option<String>,
    /// The MusicBrainz *recording* id — the identity of a performance, which
    /// is what a fingerprint can actually establish. Which release it was on
    /// is a guess on top of that, and lives in `release_mbid`.
    pub recording_mbid: Option<String>,
    pub release_mbid: Option<String>,
    /// How sure AcoustID was, 0–1.
    pub score: Option<f32>,
    /// When AcoustID was last asked. Null means never — and a row with a time
    /// but no `acoustid` is a miss, which is why the time is stored separately
    /// from the result: without it every scan would re-ask for every track the
    /// database does not know.
    pub looked_up_at: Option<String>,
    pub fingerprinted_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
