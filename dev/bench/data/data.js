window.BENCHMARK_DATA = {
  "lastUpdate": 1745680722221,
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
      }
    ]
  }
}