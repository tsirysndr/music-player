//! Reuse the native client's bounded loopback fixture; no detached server task.

use super::*;
use crate::MediaEntry;

const VIEWS_PATH: &str = "/Users/fixture-user/Views";
const LARGE_SEASON: u64 = 4_294_967_301;
const LARGE_EPISODE: u64 = 9_007_199_254_740_993;

fn entry_source(entry: &MediaEntry, item_id: &str, container: bool) -> SourceRef {
    let source = SourceRef::parse(&entry.id).unwrap();
    assert_eq!(source.account_id, ACCOUNT);
    assert_eq!(source.remote.server_id, SERVER);
    assert_eq!(source.remote.user_id, USER);
    assert_eq!(source.item_id, item_id);
    assert_eq!(
        source.kind,
        if container {
            ResourceKind::Container
        } else {
            ResourceKind::Item
        }
    );
    assert_eq!(entry.is_container, container);
    assert!(!entry.id.contains(TOKEN));
    if container {
        assert!(entry.track.is_none());
    }
    source
}

fn assert_listing(request: &Request, parent: &str, recursive: bool, start: usize, limit: usize) {
    request.authenticated();
    let mut expected = BTreeMap::from([
        ("ParentId".into(), parent.into()),
        ("Recursive".into(), recursive.to_string()),
        ("StartIndex".into(), start.to_string()),
        ("Limit".into(), limit.to_string()),
        ("EnableTotalRecordCount".into(), "true".into()),
        ("SortBy".into(), "SortName,Id".into()),
    ]);
    if recursive {
        expected.insert("MediaTypes".into(), "Audio,Video".into());
        expected.insert("IsFolder".into(), "false".into());
    }
    assert_eq!(request.query(), expected);
}

#[tokio::test]
async fn views_series_seasons_and_leaves_preserve_hierarchy_and_original_metadata() {
    bounded(async {
        let fixture = Fixture::new().await;
        let client = authenticate(&fixture, false, true).await;
        assert!(client.capabilities().media_browse);
        let root = json!({
            "Items":[
                {"Id":"films","Name":"Films","Type":"CollectionFolder","IsFolder":true},
                {"Id":"library","Name":"节目库 / Original","Type":"CollectionFolder","IsFolder":true},
                {"Id":"music","Name":"Music","Type":"UserView","IsFolder":true}
            ],
            "TotalRecordCount":3
        });
        // Views has no remote pagination API. Preserve root order and page it locally.
        let (result, request) = tokio::join!(
            client.browse(None, Page::new(1, 1)),
            fixture.json("GET", VIEWS_PATH, root),
        );
        request.authenticated();
        assert!(request.query().is_empty());
        let roots = result.unwrap();
        assert_eq!(roots.len(), 1);
        entry_source(&roots[0], "library", true);
        assert_eq!(roots[0].title, "节目库 / Original");

        let (result, request) = tokio::join!(
            client.browse(Some(&roots[0].id), Page::all()),
            fixture.json("GET", LIST_PATH, json!({
                "Items":[
                    {"Id":"series","Name":"First Ever Minecraft Playthrough","Type":"Series"},
                    {"Id":"set","Name":"Collection","Type":"BoxSet","IsFolder":true},
                    {"Id":"custom","Name":"Other folder","Type":"FutureContainer","IsFolder":true}
                ],
                "StartIndex":0,"TotalRecordCount":3
            })),
        );
        assert_listing(&request, "library", false, 0, 500);
        let children = result.unwrap();
        assert_eq!(children.len(), 3);
        for (entry, id) in children.iter().zip(["series", "set", "custom"]) {
            entry_source(entry, id, true);
            // Container handles cannot enter the Item-only playback metadata path.
            assert!(client.track(&entry.id).await.is_err());
        }
        assert_eq!(children[2].item_type.as_deref(), Some("FutureContainer"));

        let (result, request) = tokio::join!(
            client.browse(Some(&children[0].id), Page::all()),
            fixture.json("GET", LIST_PATH, json!({
                "Items":[{"Id":"season","Name":"Season title — Ep.40 remains a title",
                    "Type":"Season","IndexNumber":LARGE_SEASON}],
                "TotalRecordCount":1
            })),
        );
        assert_listing(&request, "series", false, 0, 500);
        let seasons = result.unwrap();
        entry_source(&seasons[0], "season", true);
        assert_eq!(seasons[0].season_number, Some(LARGE_SEASON));
        assert_eq!(seasons[0].episode_number, None);
        assert_eq!(seasons[0].title, "Season title — Ep.40 remains a title");

        let mut target = episode();
        target["ParentIndexNumber"] = json!(LARGE_SEASON);
        target["IndexNumber"] = json!(LARGE_EPISODE);
        let title = target["Name"].as_str().unwrap().to_owned();
        let (result, request) = tokio::join!(
            client.browse(Some(&seasons[0].id), Page::all()),
            fixture.json("GET", LIST_PATH, json!({
                "Items":[target,
                    {"Id":"photo","Name":"Poster","Type":"Photo","MediaType":"Photo"},
                    {"Id":"movie","Name":"Original movie","Type":"Movie","MediaType":"Video"},
                    {"Id":"audio","Name":"Recording","Type":"Audio","MediaType":"Audio"}
                ],
                "TotalRecordCount":4
            })),
        );
        assert_listing(&request, "season", false, 0, 500);
        let leaves = result.unwrap();
        assert_eq!(leaves.len(), 4);
        entry_source(&leaves[0], "55508", false);
        assert_eq!(leaves[0].season_number, Some(LARGE_SEASON));
        assert_eq!(leaves[0].episode_number, Some(LARGE_EPISODE));
        assert_eq!(leaves[0].title, title);
        assert_eq!(leaves[0].item_type.as_deref(), Some("Episode"));
        assert_eq!(leaves[0].media_type.as_deref(), Some("Video"));
        let track = leaves[0].track.as_ref().unwrap();
        assert_eq!(track.title, title);
        assert_eq!(track.id, leaves[0].id);
        assert_eq!(track.uri, track.id);
        assert_eq!(track.disc_number, 0);
        assert_eq!(track.track_number, None);
        entry_source(&leaves[1], "photo", false);
        assert!(leaves[1].track.is_none());
        for index in [0, 2, 3] {
            let track = leaves[index].track.as_ref().unwrap();
            assert!(track.album.is_none());
        }
        // Exact request count plus the scripted paths exclude PlaybackInfo/detail fan-out.
        fixture.idle(5).await;
    }).await;
}

#[tokio::test]
async fn child_browse_fills_a_large_offset_page_despite_small_server_pages() {
    bounded(async {
        let fixture = Fixture::new().await;
        let client = authenticate(&fixture, false, true).await;
        let parent = client
            .reference(ResourceKind::Container, "library")
            .unwrap()
            .to_handle();
        const START: usize = 31;
        const WANTED: usize = 601;
        const CAP: usize = 73;
        let (result, requests) =
            tokio::join!(
                client.browse(Some(&parent), Page::new(START as i32, WANTED as i32)),
                async {
                    let mut cursor = START;
                    let mut requests = 0;
                    while cursor < START + WANTED {
                        let limit = (START + WANTED - cursor).min(500);
                        let end = cursor + CAP.min(limit);
                        let items: Vec<_> = (cursor..end).map(|index| json!({
                        "Id":format!("series-{index}"), "Name":format!("Series {index}"),
                        "Type":"Series", "IsFolder":true
                    })).collect();
                        let request = fixture
                            .json(
                                "GET",
                                LIST_PATH,
                                json!({
                                    "Items":items, "StartIndex":cursor, "TotalRecordCount":711
                                }),
                            )
                            .await;
                        assert_listing(&request, "library", false, cursor, limit);
                        cursor = end;
                        requests += 1;
                    }
                    requests
                }
            );
        let entries = result.unwrap();
        assert_eq!(entries.len(), WANTED);
        for (index, entry) in entries.iter().enumerate() {
            entry_source(entry, &format!("series-{}", START + index), true);
        }
        fixture.idle(1 + requests).await;
    })
    .await;
}

#[tokio::test]
async fn recursive_queue_lists_beyond_500_without_playback_info_or_container_tracks() {
    bounded(async {
        let fixture = Fixture::new().await;
        let client = authenticate(&fixture, false, true).await;
        let parent = client.reference(ResourceKind::Container, "season").unwrap().to_handle();
        const TOTAL: usize = 617;
        const CAP: usize = 137;
        let (result, ()) = tokio::join!(client.container_tracks(&parent, Page::all()), async {
            for cursor in (0..TOTAL).step_by(CAP) {
                let items: Vec<_> = (cursor..(cursor + CAP).min(TOTAL)).map(|index| {
                    match index {
                        // Badly filtered server rows must not become playable,
                        // nor affect the number of remote rows consumed.
                        4 => json!({"Id":"folder","Name":"Season","Type":"Season","MediaType":"Video"}),
                        5 => json!({"Id":"photo","Name":"Photo","MediaType":"Photo"}),
                        _ => json!({
                            "Id":format!("leaf-{index}"), "Name":format!("Original {index}"),
                            "MediaType":if index % 2 == 0 { "Audio" } else { "Video" },
                            "IsFolder":false
                        })
                    }
                }).collect();
                let request = fixture.json("GET", LIST_PATH, json!({
                    "Items":items, "StartIndex":cursor, "TotalRecordCount":TOTAL
                })).await;
                assert_listing(&request, "season", true, cursor, 500);
            }
        });
        let tracks = result.unwrap();
        assert_eq!(tracks.len(), TOTAL - 2);
        for (track, index) in tracks.iter().zip((0..TOTAL).filter(|index| ![4, 5].contains(index))) {
            let source = SourceRef::parse(&track.id).unwrap();
            assert_eq!(source.item_id, format!("leaf-{index}"));
            assert_eq!(source.kind, ResourceKind::Item);
            assert_eq!(source.account_id, ACCOUNT);
            assert_eq!(source.remote, *client.identity());
            assert_eq!(track.id, track.uri);
        }
        fixture.idle(6).await;
    }).await;
}

#[tokio::test]
async fn foreign_malformed_and_non_container_parents_are_rejected_before_network() {
    bounded(async {
        let fixture = Fixture::new().await;
        let client = authenticate(&fixture, false, true).await;
        let source = client
            .reference(ResourceKind::Container, "library")
            .unwrap();
        let mut wrong_account = source.clone();
        wrong_account.account_id = "other-saved-account".into();
        let mut wrong_server = source.clone();
        wrong_server.remote.server_id = "other-server".into();
        let mut wrong_user = source.clone();
        wrong_user.remote.user_id = "other-user".into();
        let mut old_provider = source.clone();
        old_provider.resolver = "jellyfin".into();
        let mut parents = vec![
            wrong_account.to_handle(),
            wrong_server.to_handle(),
            wrong_user.to_handle(),
            old_provider.to_handle(),
            "library".into(),
            "mp-source:v2?broken".into(),
        ];
        for kind in [
            ResourceKind::Item,
            ResourceKind::Album,
            ResourceKind::Artist,
            ResourceKind::Playlist,
        ] {
            parents.push(client.reference(kind, "library").unwrap().to_handle());
        }
        for parent in parents {
            assert!(client.browse(Some(&parent), Page::all()).await.is_err());
            assert!(client.container_tracks(&parent, Page::all()).await.is_err());
        }
        fixture.idle(1).await;
    })
    .await;
}

#[tokio::test]
async fn incomplete_roots_and_repeated_child_pages_fail_instead_of_returning_partial_lists() {
    bounded(async {
        let fixture = Fixture::new().await;
        let client = authenticate(&fixture, false, true).await;
        let (result, _) = tokio::join!(
            client.browse(None, Page::all()),
            fixture.json(
                "GET",
                VIEWS_PATH,
                json!({
                    "Items":[{"Id":"library","Name":"Library","IsFolder":true}],
                    "TotalRecordCount":2
                })
            ),
        );
        assert!(matches!(result, Err(ProviderError::Other(message))
            if message == "Emby returned an incomplete root listing"));
        let parent = client
            .reference(ResourceKind::Container, "library")
            .unwrap()
            .to_handle();
        for recursive in [false, true] {
            let (result, ()) = tokio::join!(
                async {
                    if recursive {
                        client
                            .container_tracks(&parent, Page::all())
                            .await
                            .map(|items| items.len())
                    } else {
                        client
                            .browse(Some(&parent), Page::all())
                            .await
                            .map(|items| items.len())
                    }
                },
                async {
                    for cursor in [0, 1] {
                        let request = fixture.json("GET", LIST_PATH, json!({
                        "Items":[{"Id":"duplicate","Name":"Same row","MediaType":"Audio"}],
                        "StartIndex":cursor,"TotalRecordCount":3
                    })).await;
                        assert_listing(&request, "library", recursive, cursor, 500);
                    }
                }
            );
            assert!(matches!(result, Err(ProviderError::Other(message))
                if message == "Emby listing repeated an item; retry the listing"));
        }
        fixture.idle(6).await;
    })
    .await;
}
