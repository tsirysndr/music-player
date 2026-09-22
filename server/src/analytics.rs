//! Listening analytics over gRPC.
//!
//! A thin translation layer: every figure comes from
//! [`music_player_analytics::query`], so the daemon, the CLI and the MCP tools
//! cannot disagree about what "most played" means.
//!
//! Read-only by design. Importing an export is a command-line operation
//! because it reads arbitrary paths on the daemon's filesystem and can run for
//! minutes; neither belongs behind an RPC a client can call.

use music_player_analytics::query::{
    self, Scope as QueryScope, TopKind as QueryKind, Window as QueryWindow,
};
use music_player_analytics::Analytics as Engine;
use music_player_storage::Database;
use tonic::{Request, Response, Status};

use crate::api::music::v1alpha1::{
    analytics_service_server::AnalyticsService, ClockCell, DriftPoint, GetClockRequest,
    GetClockResponse, GetDriftRequest, GetDriftResponse, GetOverviewRequest, GetOverviewResponse,
    GetRotationRequest, GetRotationResponse, GetSessionsRequest, GetSessionsResponse,
    GetSkipsRequest, GetSkipsResponse, GetTopRequest, GetTopResponse, GetTransitionsRequest,
    GetTransitionsResponse, Rank, Scope, SkipRow, Source, SyncRequest, SyncResponse, TopKind,
    Transition,
};

pub struct Analytics {
    db: Database,
}

impl Analytics {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

/// Run a query on a blocking thread.
///
/// DuckDB is synchronous and its connection is not `Send` across an await, so
/// the handle is opened *inside* the closure and dropped with it. Opening
/// per-request rather than holding one open matters for a second reason: a
/// long-lived read-write handle in the daemon would lock the file and make
/// `music-player analytics import` fail for as long as the daemon runs.
async fn blocking<T, F>(work: F) -> Result<T, Status>
where
    F: FnOnce(&Engine) -> anyhow::Result<T> + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(move || {
        let engine = Engine::open_default_read_only().map_err(|cause| {
            Status::failed_precondition(format!(
                "no listening analytics yet ({cause}). Run `music-player analytics sync`."
            ))
        })?;
        work(&engine).map_err(|cause| Status::internal(cause.to_string()))
    })
    .await
    .map_err(|cause| Status::internal(format!("analytics task failed: {cause}")))?
}

fn scope(scope: Option<Scope>) -> QueryScope {
    let scope = scope.unwrap_or_default();
    QueryScope {
        window: QueryWindow {
            since: scope.since,
            until: scope.until,
        },
        local_only: scope.local_only,
    }
}

/// Zero means "unspecified" over the wire, not "return nothing".
fn limit(limit: u32, fallback: u32) -> u32 {
    if limit == 0 {
        fallback
    } else {
        limit.min(500)
    }
}

fn rank(r: query::Rank) -> Rank {
    Rank {
        name: r.name,
        secondary: r.secondary,
        listens: r.listens,
        hours: r.hours,
        last_played: r.last_played.unwrap_or_default(),
    }
}

#[tonic::async_trait]
impl AnalyticsService for Analytics {
    async fn get_overview(
        &self,
        request: Request<GetOverviewRequest>,
    ) -> Result<Response<GetOverviewResponse>, Status> {
        let scope = scope(request.into_inner().scope);
        let response = blocking(move |engine| {
            let overview = query::overview(engine, scope)?;
            let sources = query::origins(engine)?
                .into_iter()
                .map(|o| Source {
                    origin: o.origin,
                    raw: o.raw,
                    canonical: o.canonical,
                    first: o.first.unwrap_or_default(),
                    last: o.last.unwrap_or_default(),
                })
                .collect();
            Ok(GetOverviewResponse {
                listens: overview.listens,
                distinct_tracks: overview.distinct_tracks,
                distinct_artists: overview.distinct_artists,
                distinct_albums: overview.distinct_albums,
                hours_played: overview.hours_played,
                sessions: overview.sessions,
                first_listen: overview.first_listen.unwrap_or_default(),
                last_listen: overview.last_listen.unwrap_or_default(),
                active_days: overview.active_days,
                longest_streak: overview.longest_streak,
                sources,
            })
        })
        .await?;
        Ok(Response::new(response))
    }

    async fn get_top(
        &self,
        request: Request<GetTopRequest>,
    ) -> Result<Response<GetTopResponse>, Status> {
        let request = request.into_inner();
        let kind = match request.kind() {
            TopKind::Tracks => QueryKind::Tracks,
            TopKind::Albums => QueryKind::Albums,
            TopKind::Genres => QueryKind::Genres,
            // Unspecified included: artists is the useful default.
            _ => QueryKind::Artists,
        };
        let scope = scope(request.scope);
        let count = limit(request.limit, 20);
        let ranks = blocking(move |engine| Ok(query::top(engine, kind, scope, count)?)).await?;
        Ok(Response::new(GetTopResponse {
            ranks: ranks.into_iter().map(rank).collect(),
        }))
    }

    async fn get_clock(
        &self,
        request: Request<GetClockRequest>,
    ) -> Result<Response<GetClockResponse>, Status> {
        let scope = scope(request.into_inner().scope);
        let cells = blocking(move |engine| Ok(query::clock(engine, scope)?)).await?;
        Ok(Response::new(GetClockResponse {
            cells: cells
                .into_iter()
                .map(|c| ClockCell {
                    weekday: c.weekday,
                    hour: c.hour,
                    listens: c.listens,
                    avg_bpm: c.avg_bpm,
                    avg_valence: c.avg_valence,
                })
                .collect(),
        }))
    }

    async fn get_sessions(
        &self,
        request: Request<GetSessionsRequest>,
    ) -> Result<Response<GetSessionsResponse>, Status> {
        let request = request.into_inner();
        let scope = scope(request.scope);
        let count = limit(request.limit, 10);
        let stats = blocking(move |engine| Ok(query::session_stats(engine, scope, count)?)).await?;
        Ok(Response::new(GetSessionsResponse {
            sessions: stats.sessions,
            avg_tracks: stats.avg_tracks,
            median_minutes: stats.median_minutes,
            longest_minutes: stats.longest_minutes,
            openers: stats.openers.into_iter().map(rank).collect(),
        }))
    }

    async fn get_skips(
        &self,
        request: Request<GetSkipsRequest>,
    ) -> Result<Response<GetSkipsResponse>, Status> {
        let request = request.into_inner();
        let scope = scope(request.scope);
        let count = limit(request.limit, 20);
        let min_listens = if request.min_listens <= 0 {
            3
        } else {
            request.min_listens
        };
        let rows =
            blocking(move |engine| Ok(query::skips(engine, scope, count, min_listens)?)).await?;
        Ok(Response::new(GetSkipsResponse {
            rows: rows
                .into_iter()
                .map(|r| SkipRow {
                    name: r.name,
                    secondary: r.secondary,
                    listens: r.listens,
                    skips: r.skips,
                    skip_rate: r.skip_rate,
                    median_completion: r.median_completion,
                })
                .collect(),
        }))
    }

    async fn get_drift(
        &self,
        request: Request<GetDriftRequest>,
    ) -> Result<Response<GetDriftResponse>, Status> {
        let request = request.into_inner();
        let scope = scope(request.scope);
        let bucket = if request.bucket.trim().is_empty() {
            "month".to_string()
        } else {
            request.bucket
        };
        let points = blocking(move |engine| Ok(query::drift(engine, scope, &bucket)?))
            .await
            // An unsupported bucket is the caller's mistake, not a server fault.
            .map_err(|status| match status.code() {
                tonic::Code::Internal if status.message().contains("unsupported bucket") => {
                    Status::invalid_argument(status.message().to_owned())
                }
                _ => status,
            })?;
        Ok(Response::new(GetDriftResponse {
            points: points
                .into_iter()
                .map(|p| DriftPoint {
                    bucket: p.bucket,
                    listens: p.listens,
                    avg_bpm: p.avg_bpm,
                    avg_valence: p.avg_valence,
                    avg_arousal: p.avg_arousal,
                    median_year: p.median_year,
                    discovery_rate: p.discovery_rate,
                })
                .collect(),
        }))
    }

    async fn get_transitions(
        &self,
        request: Request<GetTransitionsRequest>,
    ) -> Result<Response<GetTransitionsResponse>, Status> {
        let request = request.into_inner();
        let scope = scope(request.scope);
        let count = limit(request.limit, 20);
        let transitions =
            blocking(move |engine| Ok(query::transitions(engine, scope, count)?)).await?;
        Ok(Response::new(GetTransitionsResponse {
            transitions: transitions
                .into_iter()
                .map(|t| Transition {
                    from_title: t.from_title,
                    from_artist: t.from_artist,
                    to_title: t.to_title,
                    to_artist: t.to_artist,
                    times: t.times,
                })
                .collect(),
        }))
    }

    async fn get_rotation(
        &self,
        request: Request<GetRotationRequest>,
    ) -> Result<Response<GetRotationResponse>, Status> {
        let request = request.into_inner();
        let scope = scope(request.scope);
        let count = limit(request.limit, 10);
        let rotation = blocking(move |engine| Ok(query::rotation(engine, scope, count)?)).await?;
        Ok(Response::new(GetRotationResponse {
            top_1_percent_share: rotation.top_1_percent_share,
            top_10_percent_share: rotation.top_10_percent_share,
            gini: rotation.gini,
            library_tracks: rotation.library_tracks,
            library_played: rotation.library_played,
            rediscover: rotation.rediscover.into_iter().map(rank).collect(),
        }))
    }

    async fn sync(&self, _: Request<SyncRequest>) -> Result<Response<SyncResponse>, Status> {
        // The only writing rpc, and it opens its own read-write handle for the
        // duration rather than holding one: see `blocking` above for why the
        // daemon must not keep the file locked.
        let connection = self.db.get_connection().clone();
        let handle = tokio::runtime::Handle::current();

        // The whole sync runs on one blocking thread, engine and all. A DuckDB
        // connection is not `Sync`, so it cannot be held across an `.await`
        // that the executor might resume elsewhere — and the sync is
        // unavoidably async, because reading SQLite goes through sea-orm.
        // Blocking on that future *inside* the thread that owns the engine is
        // what keeps the two compatible.
        let report = tokio::task::spawn_blocking(move || {
            let engine = Engine::open_default()?;
            handle.block_on(music_player_analytics::sync::sync(&engine, &connection))
        })
        .await
        .map_err(|cause| Status::internal(format!("analytics task failed: {cause}")))?
        .map_err(|cause: anyhow::Error| Status::internal(cause.to_string()))?;

        Ok(Response::new(SyncResponse {
            listens: report.listens,
            tracks: report.tracks,
        }))
    }
}
