//! Write the GraphQL schema as an introspection JSON document.
//!
//! The web UI's typed hooks are generated from that file
//! (`bun run graphql:generate`), so it has to be refreshed whenever the schema
//! changes. Doing it here rather than by introspecting a running daemon means
//! no server has to be started, and the output always matches the source in
//! the working tree rather than whatever binary happens to be listening.
//!
//! ```sh
//! cargo run --release -p music-player-graphql --example dump_schema \
//!     -- webui/musicplayer/graphql.schema.json
//! bun run graphql:generate   # in webui/musicplayer
//! ```

use async_graphql::Schema;
use music_player_graphql::schema::{Mutation, Query, Subscription};

/// The introspection query, in the shape graphql-codegen expects.
const INTROSPECTION: &str = r#"
query IntrospectionQuery {
  __schema {
    queryType { name }
    mutationType { name }
    subscriptionType { name }
    types { ...FullType }
    directives { name description locations args { ...InputValue } }
  }
}
fragment FullType on __Type {
  kind name description
  fields(includeDeprecated: true) {
    name description
    args { ...InputValue }
    type { ...TypeRef }
    isDeprecated deprecationReason
  }
  inputFields { ...InputValue }
  interfaces { ...TypeRef }
  enumValues(includeDeprecated: true) { name description isDeprecated deprecationReason }
  possibleTypes { ...TypeRef }
}
fragment InputValue on __InputValue {
  name description type { ...TypeRef } defaultValue
}
fragment TypeRef on __Type {
  kind name
  ofType { kind name
    ofType { kind name
      ofType { kind name
        ofType { kind name
          ofType { kind name
            ofType { kind name
              ofType { kind name }
            }
          }
        }
      }
    }
  }
}
"#;

#[tokio::main]
async fn main() {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "graphql.schema.json".to_string());

    let schema = Schema::build(
        Query::default(),
        Mutation::default(),
        Subscription::default(),
    )
    .finish();

    let response = schema.execute(INTROSPECTION).await;
    if !response.errors.is_empty() {
        eprintln!("introspection failed: {:?}", response.errors);
        std::process::exit(1);
    }
    let json =
        serde_json::to_string_pretty(&response.data).expect("introspection result is serializable");
    std::fs::write(&out, json + "\n").unwrap_or_else(|e| panic!("writing {out}: {e}"));
    println!("wrote {out}");
}
