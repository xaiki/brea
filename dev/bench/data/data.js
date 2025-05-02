window.BENCHMARK_DATA = {
  "lastUpdate": 1746221708622,
  "repoUrl": "https://github.com/xaiki/brea",
  "entries": {
    "Rust Benchmarks": [
      {
        "commit": {
          "author": {
            "email": "xaiki@evilgiggle.com",
            "name": "Niv Sardi",
            "username": "xaiki"
          },
          "committer": {
            "email": "xaiki@evilgiggle.com",
            "name": "Niv Sardi",
            "username": "xaiki"
          },
          "distinct": true,
          "id": "50e456a7918f3933ae4af4f3f2499d92b38a32de",
          "message": "fix: use standard output format for benchmarks",
          "timestamp": "2025-04-26T12:10:52-03:00",
          "tree_id": "b79c0624076caf0fa40093287dd1eb05f06ca8f4",
          "url": "https://github.com/xaiki/brea/commit/50e456a7918f3933ae4af4f3f2499d92b38a32de"
        },
        "date": 1745680721820,
        "tool": "cargo",
        "benches": [
          {
            "name": "database/insert/10",
            "value": 4601878,
            "range": "± 53337",
            "unit": "ns/iter"
          },
          {
            "name": "database/insert/100",
            "value": 23891033,
            "range": "± 42851",
            "unit": "ns/iter"
          },
          {
            "name": "database/insert/1000",
            "value": 220005210,
            "range": "± 796031",
            "unit": "ns/iter"
          },
          {
            "name": "database/query/10",
            "value": 4735632,
            "range": "± 15959",
            "unit": "ns/iter"
          },
          {
            "name": "database/query/100",
            "value": 25149497,
            "range": "± 69301",
            "unit": "ns/iter"
          },
          {
            "name": "database/query/1000",
            "value": 232370068,
            "range": "± 553505",
            "unit": "ns/iter"
          },
          {
            "name": "scraper/scrape_query",
            "value": 25624509,
            "range": "± 800165",
            "unit": "ns/iter"
          },
          {
            "name": "scraper/html_parsing",
            "value": 24368484,
            "range": "± 369993",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent/concurrent_insert/10",
            "value": 16132272,
            "range": "± 55294",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent/concurrent_insert/100",
            "value": 177020460,
            "range": "± 793696",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent/concurrent_query/10",
            "value": 6785539,
            "range": "± 21084",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent/concurrent_query/100",
            "value": 129396710,
            "range": "± 639247",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "xaiki@evilgiggle.com",
            "name": "Niv Sardi",
            "username": "xaiki"
          },
          "committer": {
            "email": "xaiki@evilgiggle.com",
            "name": "Niv Sardi",
            "username": "xaiki"
          },
          "distinct": true,
          "id": "d2a57cd7897691fd70d21e5d4d0d0a1ccb95588f",
          "message": "chore: remove benchmark.txt from tracking to allow gh-pages management",
          "timestamp": "2025-05-02T18:27:08-03:00",
          "tree_id": "af8eb68468ff849b4b4dee14f7fb8c9addd7e39d",
          "url": "https://github.com/xaiki/brea/commit/d2a57cd7897691fd70d21e5d4d0d0a1ccb95588f"
        },
        "date": 1746221633799,
        "tool": "cargo",
        "benches": [
          {
            "name": "database/insert/10",
            "value": 5480078,
            "range": "± 76367",
            "unit": "ns/iter"
          },
          {
            "name": "database/insert/100",
            "value": 24834513,
            "range": "± 173704",
            "unit": "ns/iter"
          },
          {
            "name": "database/insert/1000",
            "value": 223588493,
            "range": "± 2631737",
            "unit": "ns/iter"
          },
          {
            "name": "database/query/10",
            "value": 5660789,
            "range": "± 61188",
            "unit": "ns/iter"
          },
          {
            "name": "database/query/100",
            "value": 26165503,
            "range": "± 254561",
            "unit": "ns/iter"
          },
          {
            "name": "database/query/1000",
            "value": 231783883,
            "range": "± 840409",
            "unit": "ns/iter"
          },
          {
            "name": "scraper/scrape_query",
            "value": 198032828,
            "range": "± 771001",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent/concurrent_queries/10",
            "value": 7578500,
            "range": "± 40824",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent/concurrent_queries/100",
            "value": 113432759,
            "range": "± 805849",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "xaiki@evilgiggle.com",
            "name": "Niv Sardi",
            "username": "xaiki"
          },
          "committer": {
            "email": "xaiki@evilgiggle.com",
            "name": "Niv Sardi",
            "username": "xaiki"
          },
          "distinct": true,
          "id": "372a25285dce9267d55ed32815cc5f22341e26fc",
          "message": "fix: improve benchmark workflow to handle git operations gracefully",
          "timestamp": "2025-05-02T18:28:04-03:00",
          "tree_id": "a70c57e6a0a5b9b7d90187f087271f00cd8e15d2",
          "url": "https://github.com/xaiki/brea/commit/372a25285dce9267d55ed32815cc5f22341e26fc"
        },
        "date": 1746221708332,
        "tool": "cargo",
        "benches": [
          {
            "name": "database/insert/10",
            "value": 5345845,
            "range": "± 37685",
            "unit": "ns/iter"
          },
          {
            "name": "database/insert/100",
            "value": 24444918,
            "range": "± 116062",
            "unit": "ns/iter"
          },
          {
            "name": "database/insert/1000",
            "value": 217831200,
            "range": "± 1143747",
            "unit": "ns/iter"
          },
          {
            "name": "database/query/10",
            "value": 5497909,
            "range": "± 41738",
            "unit": "ns/iter"
          },
          {
            "name": "database/query/100",
            "value": 25471440,
            "range": "± 83791",
            "unit": "ns/iter"
          },
          {
            "name": "database/query/1000",
            "value": 227408977,
            "range": "± 526502",
            "unit": "ns/iter"
          },
          {
            "name": "scraper/scrape_query",
            "value": 193024860,
            "range": "± 760444",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent/concurrent_queries/10",
            "value": 7433229,
            "range": "± 21854",
            "unit": "ns/iter"
          },
          {
            "name": "concurrent/concurrent_queries/100",
            "value": 111817806,
            "range": "± 247701",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}