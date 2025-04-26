# BREA Performance Benchmarks

This directory contains performance benchmarks and regression tracking for the BREA project.

## Overview

We use [Criterion.rs](https://bheisler.github.io/criterion.rs/book/) for benchmarking and [github-action-benchmark](https://github.com/benchmark-action/github-action-benchmark) for tracking performance over time.

## Benchmark Categories

1. Database Operations
   - Single property insertion (1.5ms/10, 6ms/100, 49ms/1000)
   - Property querying (1.6ms/10, 6.8ms/100, 58ms/1000)

2. Scraper Operations
   - Scrape query construction (~56ms)
   - HTML parsing (~54ms)

3. Concurrent Operations
   - Concurrent insertions (19ms/10, 208ms/100)
   - Concurrent queries (3.4ms/10, 87ms/100)

## Running Benchmarks Locally

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark group
cargo bench database
cargo bench scraper
cargo bench concurrent

# Run with custom configuration
cargo bench -- --warm-up-time 5 --measurement-time 30
```

## Regression Detection

Performance regressions are automatically detected in CI:

1. Benchmarks run on every push to main and on pull requests
2. Results are compared against the baseline
3. Alerts are triggered if performance degrades by more than 10%
4. Results are stored in the gh-pages branch
5. GitHub issues are created for significant regressions

## Investigating Regressions

When a regression is detected:

1. Check the GitHub Actions output for detailed benchmark results
2. Compare against historical data in the gh-pages branch
3. Look for recent changes that might affect performance
4. Consider hardware/environment differences
5. Update baseline if regression is expected/acceptable

## Maintenance

- Keep baseline measurements updated
- Review and clean up historical data periodically
- Update alert thresholds as needed
- Document known performance characteristics
- Track and investigate outliers

## Directory Structure

```
dev/bench/
├── README.md           # This file
├── data/              # Historical benchmark data
└── baselines/         # Baseline measurements
``` 