use super::*;
use std::collections::HashSet;

fn local() -> LocalSourceRef {
    LocalSourceRef::for_extension(
        "com.example.Source_1",
        "saved-a".into(),
        1,
        ResourceKind::Item,
        "55508".into(),
    )
    .unwrap()
}

fn remote(resolver: &str) -> SourceRef {
    SourceRef {
        resolver: resolver.into(),
        account_id: "saved-a".into(),
        remote: RemoteIdentity {
            server_id: "server-1".into(),
            user_id: "user-1".into(),
        },
        kind: ResourceKind::Item,
        item_id: "55508".into(),
    }
}

#[test]
fn remote_wire_bytes_and_identity_survive_the_unified_boundary() {
    for (resolver, expected) in [
        ("emby", "mp-source:v1?resolver=emby&account=saved-a&server=server-1&user=user-1&kind=item&id=55508"),
        ("extension:com.example.Source_1", "mp-source:v1?resolver=extension%3Acom.example.Source_1&account=saved-a&server=server-1&user=user-1&kind=item&id=55508"),
    ] {
        let source = remote(resolver);
        let handle = SourceHandle::from(source.clone());
        assert_eq!(source.to_handle(), expected);
        assert_eq!(handle.to_handle(), expected);
        assert_eq!(SourceHandle::parse(expected), Ok(handle));
        assert_eq!(SourceRef::parse(expected), Ok(source));
        assert_eq!(LocalSourceRef::parse(expected), Err(SourceError::UnsupportedVersion));
    }
    let mut source = remote("emby");
    source.account_id = "old id&+/%雪".into();
    source.item_id = "item?x=1#two".into();
    let expected = "mp-source:v1?resolver=emby&account=old+id%26%2B%2F%25%E9%9B%AA&server=server-1&user=user-1&kind=item&id=item%3Fx%3D1%23two";
    let handle = SourceHandle::from(source);
    assert_eq!(handle.to_handle(), expected);
    assert_eq!(SourceHandle::parse(expected), Ok(handle));
}

#[test]
fn local_wire_roundtrips_opaque_ids_kinds_and_exact_revisions() {
    let mut source = local();
    source.account_id = "old id&+/%雪".into();
    source.item_id = "item?x=1#two".into();
    assert_eq!(source.to_handle(), "mp-source:v2?resolver=extension%3Acom.example.Source_1&account=old+id%26%2B%2F%25%E9%9B%AA&scope=local&revision=1&kind=item&id=item%3Fx%3D1%23two");
    for kind in [
        ResourceKind::Item,
        ResourceKind::Album,
        ResourceKind::Artist,
        ResourceKind::Playlist,
        ResourceKind::Container,
    ] {
        for revision in [1, 9_007_199_254_740_993, i64::MAX as u64] {
            source.kind = kind;
            source.config_revision = revision;
            let wire = source.to_handle();
            assert_eq!(LocalSourceRef::parse(&wire), Ok(source.clone()));
            assert_eq!(
                SourceHandle::parse(&wire),
                Ok(SourceHandle::Local(source.clone()))
            );
        }
    }
}

#[test]
fn unified_accessors_preserve_variant_metadata() {
    let local = local();
    assert_eq!(local.extension_id(), Some("com.example.Source_1"));
    for (source, extension_id) in [
        (SourceHandle::from(remote("emby")), None),
        (
            SourceHandle::from(remote("extension:com.example.Source_1")),
            Some("com.example.Source_1"),
        ),
        (SourceHandle::from(local), Some("com.example.Source_1")),
    ] {
        assert_eq!(source.account_id(), "saved-a");
        assert_eq!(source.kind(), ResourceKind::Item);
        assert_eq!(source.item_id(), "55508");
        assert_eq!(source.extension_id(), extension_id);
        assert_eq!(
            source.resolver(),
            if extension_id.is_some() {
                "extension:com.example.Source_1"
            } else {
                "emby"
            }
        );
    }
}

#[test]
fn each_local_identity_component_and_remote_variant_are_distinct() {
    let original = local();
    let mut account = original.clone();
    account.account_id = "saved-b".into();
    let mut revision = original.clone();
    revision.config_revision = 2;
    let mut resolver = original.clone();
    resolver.resolver = "extension:another.plugin".into();
    let mut kind = original.clone();
    kind.kind = ResourceKind::Container;
    let mut item = original.clone();
    item.item_id = "other-item".into();
    let mut values: Vec<_> = [original, account, revision, resolver, kind, item]
        .into_iter()
        .map(SourceHandle::from)
        .collect();
    values.push(SourceHandle::from(remote("extension:com.example.Source_1")));
    assert_eq!(values.iter().cloned().collect::<HashSet<_>>().len(), 7);
    assert_eq!(
        values
            .iter()
            .map(SourceHandle::to_handle)
            .collect::<HashSet<_>>()
            .len(),
        7
    );
    // A matching item/account name never authorizes promotion between scopes.
    for left in &values {
        for right in &values {
            if left == right {
                continue;
            }
            let mut id = left.to_handle();
            let mut uri = right.to_handle();
            let before = (id.clone(), uri.clone());
            assert_eq!(
                normalize_source_track(&mut id, &mut uri),
                Err(SourceError::ConflictingTrack)
            );
            assert_eq!((id, uri), before);
        }
    }
}

#[test]
fn local_rejects_native_mixed_duplicate_unknown_missing_and_empty_fields() {
    let valid = local().to_handle();
    let mut invalid = vec![
        valid.replace("resolver=extension%3Acom.example.Source_1", "resolver=emby"),
        valid.replace(
            "resolver=extension%3Acom.example.Source_1",
            "resolver=extension%3A",
        ),
        valid.replace(
            "resolver=extension%3Acom.example.Source_1",
            "resolver=extension%3Aa%2Fb",
        ),
        valid.replace("scope=local", "scope=remote"),
        valid.replace("kind=item", "kind=movie"),
        format!("{valid}&server=server-1"),
        format!("{valid}&user=user-1"),
        format!("{valid}&token=synthetic"),
        format!("{valid}&%61ccount=saved-b"),
        valid.replace("v2?", "v1?"),
        format!("{}&scope=local&revision=1", remote("emby").to_handle()),
    ];
    for field in [
        "resolver=extension%3Acom.example.Source_1",
        "account=saved-a",
        "scope=local",
        "revision=1",
        "kind=item",
        "id=55508",
    ] {
        invalid.push(format!("{valid}&{field}"));
        invalid.push(valid.replace(field, ""));
        let (key, _) = field.split_once('=').unwrap();
        invalid.push(valid.replace(field, &format!("{key}=")));
        invalid.push(valid.replace(field, &format!("{key}=+")));
    }
    for value in invalid {
        assert!(SourceHandle::is_handle(&value));
        assert!(SourceHandle::parse(&value).is_err(), "accepted {value}");
        assert!(LocalSourceRef::parse(&value).is_err());
    }
}

#[test]
fn local_revision_rejects_overflow_zero_and_noncanonical_spellings() {
    let valid = local().to_handle();
    for revision in [
        "0",
        "-1",
        "9223372036854775808",
        "18446744073709551615",
        "18446744073709551616",
    ] {
        let invalid = valid.replace("revision=1", &format!("revision={revision}"));
        assert_eq!(
            SourceHandle::parse(&invalid),
            Err(SourceError::InvalidRevision)
        );
    }
    for revision in ["01", "%2B1", "+1", "1+", "1.0", "1e0", "%31"] {
        let invalid = valid.replace("revision=1", &format!("revision={revision}"));
        assert!(
            SourceHandle::parse(&invalid).is_err(),
            "accepted {revision}"
        );
    }
}

#[test]
fn local_rejects_noncanonical_and_lossy_encodings() {
    let valid = local().to_handle();
    for invalid in [
        valid.replace("account=saved-a&scope=local", "scope=local&account=saved-a"),
        valid.replace("extension%3A", "extension%3a"),
        valid.replace("extension%3A", "extension:"),
        valid.replace("id=55508", "id=%FF"),
        valid.replace("id=55508", "id=%zz"),
        valid.replace("id=55508", "id=%35%35%35%30%38"),
        format!("{valid}&"),
        format!("{valid}#fragment"),
    ] {
        assert_eq!(
            SourceHandle::parse(&invalid),
            Err(SourceError::NonCanonical)
        );
    }
}

#[test]
fn checked_local_construction_rejects_invalid_fields_and_encoded_size() {
    let make = |plugin, account: &str, revision, item: &str| {
        LocalSourceRef::for_extension(
            plugin,
            account.into(),
            revision,
            ResourceKind::Item,
            item.into(),
        )
    };
    for plugin in ["", "extension:example", "with space", "a/b", "雪"] {
        assert_eq!(
            make(plugin, "a", 1, "i"),
            Err(SourceError::UnsupportedResolver)
        );
    }
    for revision in [0, i64::MAX as u64 + 1, u64::MAX] {
        assert_eq!(
            make("example", "a", revision, "i"),
            Err(SourceError::InvalidRevision)
        );
    }
    for (account, item) in [("", "i"), (" ", "i"), ("a", ""), ("a", " ")] {
        assert_eq!(
            make("example", account, 1, item),
            Err(SourceError::EmptyIdentity)
        );
    }
    let mut source = local();
    source.item_id.clear();
    let overhead = source.to_handle().len();
    // One non-ASCII character occupies nine percent-encoded bytes.
    source.item_id = format!("{}雪", "x".repeat(MAX_HANDLE_BYTES - overhead - 9));
    let wire = source.to_handle();
    assert_eq!(wire.len(), MAX_HANDLE_BYTES);
    assert_eq!(
        SourceHandle::parse(&wire),
        Ok(SourceHandle::Local(source.clone()))
    );
    assert_eq!(
        make("com.example.Source_1", "saved-a", 1, &source.item_id),
        Ok(source.clone())
    );
    source.item_id.push('x');
    assert_eq!(
        SourceHandle::parse(&source.to_handle()),
        Err(SourceError::TooLong)
    );
    assert_eq!(
        make("com.example.Source_1", "saved-a", 1, &source.item_id),
        Err(SourceError::TooLong)
    );
}

#[test]
fn remote_only_entry_points_reject_local_handles_without_mutation() {
    let wire = local().to_handle();
    assert!(SourceRef::is_handle(&wire));
    assert_eq!(
        SourceRef::parse(&wire),
        Err(SourceError::UnsupportedVersion)
    );
    for (mut id, mut uri) in [
        (wire.clone(), String::new()),
        (String::new(), wire.clone()),
        (wire.clone(), wire),
    ] {
        let before = (id.clone(), uri.clone());
        assert_eq!(
            normalize_track(&mut id, &mut uri),
            Err(SourceError::UnsupportedVersion)
        );
        assert_eq!((id, uri), before);
    }
}

#[test]
fn reserved_invalid_handles_cannot_fall_through_to_legacy_resolution() {
    for wire in [
        "mp-source:",
        "mp-source:v2?",
        "mp-source:v3?",
        "mp-source:v02?",
        "MP-SOURCE:v2?",
        "Mp-Source:v1?",
    ] {
        assert!(SourceHandle::is_handle(wire));
        assert!(SourceHandle::parse(wire).is_err());
        for (mut id, mut uri) in [
            (wire.to_owned(), String::new()),
            (String::new(), wire.to_owned()),
        ] {
            let before = (id.clone(), uri.clone());
            assert!(normalize_source_track(&mut id, &mut uri).is_err());
            assert_eq!((id, uri), before);
        }
    }
    let uppercase = local().to_handle().replacen("mp-source:", "MP-SOURCE:", 1);
    assert!(SourceHandle::is_handle(&uppercase));
    assert_eq!(
        SourceHandle::parse(&uppercase),
        Err(SourceError::NonCanonical)
    );
    assert!(!SourceHandle::is_handle(
        "https://example.invalid/mp-source:v2"
    ));
    assert_eq!(
        SourceHandle::parse("legacy-id"),
        Err(SourceError::NotHandle)
    );
}

#[test]
fn unified_normalization_fills_only_empty_mirrors_and_preserves_legacy() {
    for source in [
        SourceHandle::from(remote("emby")),
        SourceHandle::from(remote("extension:com.example.Source_1")),
        SourceHandle::from(local()),
    ] {
        let wire = source.to_handle();
        for (mut id, mut uri) in [
            (wire.clone(), String::new()),
            (String::new(), wire.clone()),
            (wire.clone(), wire.clone()),
        ] {
            assert_eq!(
                normalize_source_track(&mut id, &mut uri),
                Ok(Some(source.clone()))
            );
            assert_eq!(id, wire);
            assert_eq!(uri, wire);
        }
    }
    for (mut id, mut uri) in [
        ("legacy-id".into(), "/music/track.mp3".into()),
        (String::new(), String::new()),
        (String::new(), "https://example.invalid/stream".into()),
    ] {
        let before = (id.clone(), uri.clone());
        assert_eq!(normalize_source_track(&mut id, &mut uri), Ok(None));
        assert_eq!((id, uri), before);
    }
}

#[test]
fn unified_normalization_failures_are_atomic_for_either_mirror() {
    let wire = local().to_handle();
    for other in [
        "55508".to_owned(),
        "https://example.invalid/stream".to_owned(),
        "mp-source:v3?".to_owned(),
        wire.replace("revision=1", "revision=0"),
        format!("{wire}&account=saved-b"),
    ] {
        for (mut id, mut uri) in [(wire.clone(), other.clone()), (other, wire.clone())] {
            let before = (id.clone(), uri.clone());
            assert!(normalize_source_track(&mut id, &mut uri).is_err());
            assert_eq!((id, uri), before);
        }
    }
}
