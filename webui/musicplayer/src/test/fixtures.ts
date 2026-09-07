/**
 * GraphQL fixtures, generated from a real music-player library.
 *
 * Regenerate with the snippet in `src/test/README.md`. Real rows rather than
 * invented ones on purpose: ids are the 32-char md5 hashes the scanner
 * produces, durations are fractional seconds, and titles carry the
 * parenthetical suffixes and punctuation that actually turn up in tag data —
 * all things a hand-written fixture quietly rounds off, and all things the
 * formatting and elision code has to survive.
 */

export const ALBUM_ID = "3b16b0caed8f12736ae5debf7363fc79";
export const ARTIST_ID = "50fcd83c68216360292343903c891320";

export const albums = [
  {
    "__typename": "Album",
    "id": "1968f33325bcdb8b415dae88bc127839",
    "title": "11:11 (Deluxe)",
    "artist": "Chris Brown",
    "year": 2023,
    "cover": "1968f33325bcdb8b415dae88bc127839.jpg",
    "tracks": []
  },
  {
    "__typename": "Album",
    "id": "3b16b0caed8f12736ae5debf7363fc79",
    "title": "13 (Deluxe Version)",
    "artist": "Black Sabbath",
    "year": 2013,
    "cover": "3b16b0caed8f12736ae5debf7363fc79.jpg",
    "tracks": []
  },
  {
    "__typename": "Album",
    "id": "42963c0ec11b19727bde292af2389dd0",
    "title": "1432",
    "artist": "Katy Perry",
    "year": 2024,
    "cover": "42963c0ec11b19727bde292af2389dd0.jpg",
    "tracks": []
  },
  {
    "__typename": "Album",
    "id": "ecd3fb5214ef4faf77a0eba4eedb2638",
    "title": "2014 Forest Hills Drive",
    "artist": "J. Cole",
    "year": 2014,
    "cover": "ecd3fb5214ef4faf77a0eba4eedb2638.jpg",
    "tracks": []
  },
  {
    "__typename": "Album",
    "id": "1a895d478aa69e3b354e3c7b2515c1b1",
    "title": "21 Reasons (feat. Ella Henderson)",
    "artist": "Nathan Dawe",
    "year": 2019,
    "cover": "1a895d478aa69e3b354e3c7b2515c1b1.jpg",
    "tracks": []
  },
  {
    "__typename": "Album",
    "id": "1b35de9df8fe46560db5fc435f4d38b7",
    "title": "2Kush11",
    "artist": "Rai P, MC Beezy",
    "year": 2011,
    "cover": "1b35de9df8fe46560db5fc435f4d38b7.jpg",
    "tracks": []
  }
] as const;

export const artists = [
  {
    "__typename": "Artist",
    "id": "afd7c81234a0600d94575c57ebd0a8e6",
    "name": "A Perfect Circle",
    "picture": "https://i.scdn.co/image/ab6761610000e5eb41472573351dfea479a4ffba"
  },
  {
    "__typename": "Artist",
    "id": "a9e50f8b7c281a967e6b6f2da681181c",
    "name": "Adema",
    "picture": "https://i.scdn.co/image/ab6761610000e5eb7d99949b3f26c9eb73a5a367"
  },
  {
    "__typename": "Artist",
    "id": "ea3b44d0c1a6f207dc083b427228d83d",
    "name": "Apost",
    "picture": "https://i.scdn.co/image/ab67616d0000b2736ae1a6d886a89885b6d5ebb0"
  },
  {
    "__typename": "Artist",
    "id": "2cdc18d9d49d3536009d79433258209d",
    "name": "Audioslave",
    "picture": "https://i.scdn.co/image/ab6761610000e5eb5a865295befda9e060a72cb0"
  },
  {
    "__typename": "Artist",
    "id": "77ecfbe0b5e206531bae6b5ac3b8b7f0",
    "name": "August Alsina",
    "picture": "https://i.scdn.co/image/ab6761610000e5ebc7b9e9a3adcc3f5cbc92fb0b"
  }
] as const;

export const tracks = [
  {
    "__typename": "Track",
    "id": "b418a0d44e61ef179005ff9a13d5c8bd",
    "title": " Scotty (feat. Babyfxce E )",
    "artist": "Juicy J, DJ Scream, Babyfxce E",
    "trackNumber": 26,
    "duration": 168.66400146484375,
    "artists": [
      {
        "__typename": "Artist",
        "id": "b05145b765fe5784af172944df4f2365",
        "name": "Juicy J, DJ Scream, Babyfxce E"
      }
    ],
    "album": {
      "__typename": "Album",
      "id": "cdbed0e63839fc1d8f0d782a827fc68e",
      "title": "The Trippy Tapes Vol. 1",
      "cover": "cdbed0e63839fc1d8f0d782a827fc68e.jpg"
    }
  },
  {
    "__typename": "Track",
    "id": "07b512bee20deec308eb71ed1b2ae600",
    "title": " Sneaky (feat. rjtheweirdo)",
    "artist": "Juicy J, DJ Scream, RJ the Weirdo",
    "trackNumber": 29,
    "duration": 155.87899780273438,
    "artists": [
      {
        "__typename": "Artist",
        "id": "b05145b765fe5784af172944df4f2365",
        "name": "Juicy J, DJ Scream, RJ the Weirdo"
      }
    ],
    "album": {
      "__typename": "Album",
      "id": "cdbed0e63839fc1d8f0d782a827fc68e",
      "title": "The Trippy Tapes Vol. 1",
      "cover": "cdbed0e63839fc1d8f0d782a827fc68e.jpg"
    }
  },
  {
    "__typename": "Track",
    "id": "0e344bfbef79beed591182e0ef8f24a4",
    "title": " Yeah Dat Thur",
    "artist": "Juicy J, DJ Scream",
    "trackNumber": 21,
    "duration": 139.16299438476562,
    "artists": [
      {
        "__typename": "Artist",
        "id": "b05145b765fe5784af172944df4f2365",
        "name": "Juicy J, DJ Scream"
      }
    ],
    "album": {
      "__typename": "Album",
      "id": "cdbed0e63839fc1d8f0d782a827fc68e",
      "title": "The Trippy Tapes Vol. 1",
      "cover": "cdbed0e63839fc1d8f0d782a827fc68e.jpg"
    }
  },
  {
    "__typename": "Track",
    "id": "4686be348992976e99052b41a54d0727",
    "title": "#BODYGOALS (feat. Tank)",
    "artist": "Chris Brown, Tank",
    "trackNumber": 30,
    "duration": 148.33999633789062,
    "artists": [
      {
        "__typename": "Artist",
        "id": "86517b0d724548d323c8dc0a9397de31",
        "name": "Chris Brown, Tank"
      }
    ],
    "album": {
      "__typename": "Album",
      "id": "e7f7a54331f6c845ac37281027c23a0e",
      "title": "BROWN (The Chocolate Edition)",
      "cover": "e7f7a54331f6c845ac37281027c23a0e.jpg"
    }
  },
  {
    "__typename": "Track",
    "id": "348b20fa58afdcd198b78425385763ec",
    "title": "#IDGAF",
    "artist": "Doe B",
    "trackNumber": 14,
    "duration": 180.3260040283203,
    "artists": [
      {
        "__typename": "Artist",
        "id": "054da8dedab8910108498584f85eeb67",
        "name": "Doe B"
      }
    ],
    "album": {
      "__typename": "Album",
      "id": "132bf6fbab2927bbf076bae992f63765",
      "title": "Baby Jesus",
      "cover": "132bf6fbab2927bbf076bae992f63765.jpg"
    }
  },
  {
    "__typename": "Track",
    "id": "03e8fe5049d3d720d70cafe79ff477d3",
    "title": "'Cause I'm A Man",
    "artist": "Tame Impala",
    "trackNumber": 10,
    "duration": 242.0330047607422,
    "artists": [
      {
        "__typename": "Artist",
        "id": "a9623107ea03b8bf2013e4d5f5658948",
        "name": "Tame Impala"
      }
    ],
    "album": {
      "__typename": "Album",
      "id": "c0a8cb3150e2dfda356902c2bc01a576",
      "title": "Currents",
      "cover": "c0a8cb3150e2dfda356902c2bc01a576.jpg"
    }
  },
  {
    "__typename": "Track",
    "id": "639da99118e4023a10e969d7238edc30",
    "title": "(Anesthesia) Pulling Teeth (Live At The Hollywood Palladium, Los Angeles, CA / March 10th, 1985)",
    "artist": "Metallica",
    "trackNumber": 5,
    "duration": 214.76600646972656,
    "artists": [
      {
        "__typename": "Artist",
        "id": "8b0ee5a501cef4a5699fd3b2d4549e8f",
        "name": "Metallica"
      }
    ],
    "album": {
      "__typename": "Album",
      "id": "5de65ab61eeacec5589c98f003909432",
      "title": "Ride The Lightning (Deluxe / Remastered)",
      "cover": "5de65ab61eeacec5589c98f003909432.jpg"
    }
  },
  {
    "__typename": "Track",
    "id": "892016d5a0c12cfef54a53c810d12065",
    "title": "(Anesthesia) Pulling Teeth (Live At The Kabuki Theatre, San Francisco, CA / March 15th, 1985)",
    "artist": "Metallica",
    "trackNumber": 5,
    "duration": 100.02300262451172,
    "artists": [
      {
        "__typename": "Artist",
        "id": "8b0ee5a501cef4a5699fd3b2d4549e8f",
        "name": "Metallica"
      }
    ],
    "album": {
      "__typename": "Album",
      "id": "5de65ab61eeacec5589c98f003909432",
      "title": "Ride The Lightning (Deluxe / Remastered)",
      "cover": "5de65ab61eeacec5589c98f003909432.jpg"
    }
  }
] as const;

export const album = {
  "__typename": "Album",
  "id": "3b16b0caed8f12736ae5debf7363fc79",
  "title": "13 (Deluxe Version)",
  "artist": "Black Sabbath",
  "year": 2013,
  "cover": "3b16b0caed8f12736ae5debf7363fc79.jpg",
  "tracks": [
    {
      "__typename": "Track",
      "id": "e5f9895f3560dce3181eff25ca349f43",
      "trackNumber": 1,
      "discNumber": 0,
      "uri": "/Users/tsirysandratraina/Music/Black Sabbath - 13 (Deluxe Version)/01. Black Sabbath - End Of The Beginning.m4a",
      "title": "End Of The Beginning",
      "artist": "Black Sabbath",
      "duration": 486.2950134277344,
      "artists": [
        {
          "__typename": "Artist",
          "id": "50fcd83c68216360292343903c891320",
          "name": "Black Sabbath"
        }
      ]
    },
    {
      "__typename": "Track",
      "id": "f7a13b7b4d461c2682b0ed34c7157392",
      "trackNumber": 2,
      "discNumber": 0,
      "uri": "/Users/tsirysandratraina/Music/Black Sabbath - 13 (Deluxe Version)/02. Black Sabbath - God Is Dead.m4a",
      "title": "God Is Dead?",
      "artist": "Black Sabbath",
      "duration": 532.3170166015625,
      "artists": [
        {
          "__typename": "Artist",
          "id": "50fcd83c68216360292343903c891320",
          "name": "Black Sabbath"
        }
      ]
    },
    {
      "__typename": "Track",
      "id": "22575ccf427e10a66a2be70c7a43e521",
      "trackNumber": 3,
      "discNumber": 0,
      "uri": "/Users/tsirysandratraina/Music/Black Sabbath - 13 (Deluxe Version)/03. Black Sabbath - Loner.m4a",
      "title": "Loner",
      "artist": "Black Sabbath",
      "duration": 300.0010070800781,
      "artists": [
        {
          "__typename": "Artist",
          "id": "50fcd83c68216360292343903c891320",
          "name": "Black Sabbath"
        }
      ]
    },
    {
      "__typename": "Track",
      "id": "896944391bd11369075020297a76e493",
      "trackNumber": 4,
      "discNumber": 0,
      "uri": "/Users/tsirysandratraina/Music/Black Sabbath - 13 (Deluxe Version)/04. Black Sabbath - Zeitgeist.m4a",
      "title": "Zeitgeist",
      "artist": "Black Sabbath",
      "duration": 277.8489990234375,
      "artists": [
        {
          "__typename": "Artist",
          "id": "50fcd83c68216360292343903c891320",
          "name": "Black Sabbath"
        }
      ]
    },
    {
      "__typename": "Track",
      "id": "1d25bef6e4af48edcddc55abc72d7e92",
      "trackNumber": 5,
      "discNumber": 0,
      "uri": "/Users/tsirysandratraina/Music/Black Sabbath - 13 (Deluxe Version)/05. Black Sabbath - Age Of Reason.m4a",
      "title": "Age Of Reason",
      "artist": "Black Sabbath",
      "duration": 421.11700439453125,
      "artists": [
        {
          "__typename": "Artist",
          "id": "50fcd83c68216360292343903c891320",
          "name": "Black Sabbath"
        }
      ]
    },
    {
      "__typename": "Track",
      "id": "e04ed0e31083c75afbcc6890feffad77",
      "trackNumber": 6,
      "discNumber": 0,
      "uri": "/Users/tsirysandratraina/Music/Black Sabbath - 13 (Deluxe Version)/06. Black Sabbath - Live Forever.m4a",
      "title": "Live Forever",
      "artist": "Black Sabbath",
      "duration": 286.5570068359375,
      "artists": [
        {
          "__typename": "Artist",
          "id": "50fcd83c68216360292343903c891320",
          "name": "Black Sabbath"
        }
      ]
    },
    {
      "__typename": "Track",
      "id": "673ce0c92722afe67b5120cf0d0e2acb",
      "trackNumber": 7,
      "discNumber": 0,
      "uri": "/Users/tsirysandratraina/Music/Black Sabbath - 13 (Deluxe Version)/07. Black Sabbath - Damaged Soul.m4a",
      "title": "Damaged Soul",
      "artist": "Black Sabbath",
      "duration": 471.7130126953125,
      "artists": [
        {
          "__typename": "Artist",
          "id": "50fcd83c68216360292343903c891320",
          "name": "Black Sabbath"
        }
      ]
    },
    {
      "__typename": "Track",
      "id": "6c1d7eda1093dc4c63fbf13a1f998f94",
      "trackNumber": 8,
      "discNumber": 0,
      "uri": "/Users/tsirysandratraina/Music/Black Sabbath - 13 (Deluxe Version)/08. Black Sabbath - Dear Father.m4a",
      "title": "Dear Father",
      "artist": "Black Sabbath",
      "duration": 440.22698974609375,
      "artists": [
        {
          "__typename": "Artist",
          "id": "50fcd83c68216360292343903c891320",
          "name": "Black Sabbath"
        }
      ]
    },
    {
      "__typename": "Track",
      "id": "e2c731d3153bc98582974dc8f285f8a0",
      "trackNumber": 9,
      "discNumber": 0,
      "uri": "/Users/tsirysandratraina/Music/Black Sabbath - 13 (Deluxe Version)/09. Black Sabbath - Methademic.m4a",
      "title": "Methademic",
      "artist": "Black Sabbath",
      "duration": 358.260009765625,
      "artists": [
        {
          "__typename": "Artist",
          "id": "50fcd83c68216360292343903c891320",
          "name": "Black Sabbath"
        }
      ]
    },
    {
      "__typename": "Track",
      "id": "84c0114a1e5ca907be997de446aad686",
      "trackNumber": 10,
      "discNumber": 0,
      "uri": "/Users/tsirysandratraina/Music/Black Sabbath - 13 (Deluxe Version)/10. Black Sabbath - Peace Of Mind.m4a",
      "title": "Peace Of Mind",
      "artist": "Black Sabbath",
      "duration": 220.8209991455078,
      "artists": [
        {
          "__typename": "Artist",
          "id": "50fcd83c68216360292343903c891320",
          "name": "Black Sabbath"
        }
      ]
    },
    {
      "__typename": "Track",
      "id": "241aa168c37a284ea815a2350aecf97b",
      "trackNumber": 11,
      "discNumber": 0,
      "uri": "/Users/tsirysandratraina/Music/Black Sabbath - 13 (Deluxe Version)/11. Black Sabbath - Pariah.m4a",
      "title": "Pariah",
      "artist": "Black Sabbath",
      "duration": 334.66900634765625,
      "artists": [
        {
          "__typename": "Artist",
          "id": "50fcd83c68216360292343903c891320",
          "name": "Black Sabbath"
        }
      ]
    }
  ]
} as const;

export const artistDetail = {
  "__typename": "Artist",
  "id": "50fcd83c68216360292343903c891320",
  "name": "Black Sabbath",
  "picture": "https://cdn.bsky.app/img/avatar/plain/did:plc:7vdlgi2bflelz7mmuxoqjfcr/bafkreidmheeskq7spkbcfp37khy4afkjdnxwgzmztoy4gsokl2zae2wppm@jpeg",
  "songs": [
    {
      "__typename": "Track",
      "id": "1d25bef6e4af48edcddc55abc72d7e92",
      "title": "Age Of Reason",
      "artist": "Black Sabbath",
      "trackNumber": null,
      "duration": 421.11700439453125,
      "artists": [
        {
          "__typename": "Artist",
          "id": "50fcd83c68216360292343903c891320",
          "name": "Black Sabbath"
        }
      ],
      "album": {
        "__typename": "Album",
        "id": "3b16b0caed8f12736ae5debf7363fc79",
        "title": "13 (Deluxe Version)",
        "cover": "3b16b0caed8f12736ae5debf7363fc79.jpg"
      }
    },
    {
      "__typename": "Track",
      "id": "4007ce73c25759d54cb1fc6acf82302f",
      "title": "Children of the Sea",
      "artist": "Black Sabbath",
      "trackNumber": null,
      "duration": 354.9259948730469,
      "artists": [
        {
          "__typename": "Artist",
          "id": "50fcd83c68216360292343903c891320",
          "name": "Black Sabbath"
        }
      ],
      "album": {
        "__typename": "Album",
        "id": "1203bbbb08d027f63a75641f15a4b8be",
        "title": "Heaven and Hell (Remastered and Expanded Edition)",
        "cover": "1203bbbb08d027f63a75641f15a4b8be.jpg"
      }
    },
    {
      "__typename": "Track",
      "id": "f5971054d0d53f3c3a691252a04305d8",
      "title": "Children of the Sea",
      "artist": "Black Sabbath",
      "trackNumber": null,
      "duration": 360.72601318359375,
      "artists": [
        {
          "__typename": "Artist",
          "id": "50fcd83c68216360292343903c891320",
          "name": "Black Sabbath"
        }
      ],
      "album": {
        "__typename": "Album",
        "id": "1203bbbb08d027f63a75641f15a4b8be",
        "title": "Heaven and Hell (Remastered and Expanded Edition)",
        "cover": "1203bbbb08d027f63a75641f15a4b8be.jpg"
      }
    },
    {
      "__typename": "Track",
      "id": "673ce0c92722afe67b5120cf0d0e2acb",
      "title": "Damaged Soul",
      "artist": "Black Sabbath",
      "trackNumber": null,
      "duration": 471.7130126953125,
      "artists": [
        {
          "__typename": "Artist",
          "id": "50fcd83c68216360292343903c891320",
          "name": "Black Sabbath"
        }
      ],
      "album": {
        "__typename": "Album",
        "id": "3b16b0caed8f12736ae5debf7363fc79",
        "title": "13 (Deluxe Version)",
        "cover": "3b16b0caed8f12736ae5debf7363fc79.jpg"
      }
    },
    {
      "__typename": "Track",
      "id": "6c1d7eda1093dc4c63fbf13a1f998f94",
      "title": "Dear Father",
      "artist": "Black Sabbath",
      "trackNumber": null,
      "duration": 440.22698974609375,
      "artists": [
        {
          "__typename": "Artist",
          "id": "50fcd83c68216360292343903c891320",
          "name": "Black Sabbath"
        }
      ],
      "album": {
        "__typename": "Album",
        "id": "3b16b0caed8f12736ae5debf7363fc79",
        "title": "13 (Deluxe Version)",
        "cover": "3b16b0caed8f12736ae5debf7363fc79.jpg"
      }
    },
    {
      "__typename": "Track",
      "id": "9f5e28e58d3ec3dc85247eee67364a99",
      "title": "Die Young",
      "artist": "Black Sabbath",
      "trackNumber": null,
      "duration": 276.4729919433594,
      "artists": [
        {
          "__typename": "Artist",
          "id": "50fcd83c68216360292343903c891320",
          "name": "Black Sabbath"
        }
      ],
      "album": {
        "__typename": "Album",
        "id": "1203bbbb08d027f63a75641f15a4b8be",
        "title": "Heaven and Hell (Remastered and Expanded Edition)",
        "cover": "1203bbbb08d027f63a75641f15a4b8be.jpg"
      }
    }
  ],
  "albums": [
    {
      "__typename": "Album",
      "id": "3b16b0caed8f12736ae5debf7363fc79",
      "title": "13 (Deluxe Version)",
      "artist": "Black Sabbath",
      "year": 2013,
      "cover": "3b16b0caed8f12736ae5debf7363fc79.jpg"
    },
    {
      "__typename": "Album",
      "id": "1203bbbb08d027f63a75641f15a4b8be",
      "title": "Heaven and Hell (Remastered and Expanded Edition)",
      "artist": "Black Sabbath",
      "year": 1980,
      "cover": "1203bbbb08d027f63a75641f15a4b8be.jpg"
    }
  ]
} as const;

