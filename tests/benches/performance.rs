use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use tokio::runtime::Runtime;
use brea_core::{Database, Property, PropertyType, PropertyStatus};
use brea_scrapers::{argenprop::ArgenPropScraper, Scraper, ScrapeQuery};
use fake::{Fake, Faker};
use rand::Rng;
use std::time::Duration;
use url::Url;
use chrono::Utc;

// Helper function to create a test database
async fn setup_test_db() -> Database {
    let db = Database::new(":memory:").await.unwrap();
    db.migrate().await.unwrap();
    db
}

// Helper function to generate fake properties
fn generate_fake_properties(count: usize) -> Vec<Property> {
    (0..count)
        .map(|_| {
            let property = Property {
                id: None,
                external_id: Faker.fake(),
                source: "benchmark".to_string(),
                property_type: Some(PropertyType::Apartment),
                district: Faker.fake(),
                title: Faker.fake(),
                description: Some(Faker.fake()),
                price_usd: rand::thread_rng().gen_range(50000.0..1000000.0),
                address: Faker.fake(),
                covered_size: Some(rand::thread_rng().gen_range(30.0..500.0)),
                rooms: Some(rand::thread_rng().gen_range(1..6)),
                antiquity: Some(rand::thread_rng().gen_range(0..50)),
                url: Url::parse("https://example.com").unwrap(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                status: PropertyStatus::Active,
            };
            property
        })
        .collect()
}

fn bench_database_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("database");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(10));
    group.json_output(true);

    // Benchmark property insertion
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("insert", size), size, |b, &size| {
            let properties = generate_fake_properties(size);
            b.to_async(&rt).iter(|| async {
                let db = setup_test_db().await;
                for mut property in properties.clone() {
                    black_box(db.save_property(&mut property).await.unwrap());
                }
            });
        });
    }

    // Benchmark property querying
    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("query", size), size, |b, &size| {
            b.to_async(&rt).iter(|| async {
                let db = setup_test_db().await;
                let properties = generate_fake_properties(size);
                for mut property in properties.clone() {
                    db.save_property(&mut property).await.unwrap();
                }
                black_box(db.list_properties(
                    Some("benchmark"),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    false
                ).await.unwrap());
            });
        });
    }

    group.finish();
}

fn bench_scraper_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("scraper");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));
    group.json_output(true);

    // Benchmark scraping operations
    group.bench_function("scrape_query", |b| {
        let scraper = ArgenPropScraper::new();
        let query = ScrapeQuery {
            district: "Palermo".to_string(),
            property_type: PropertyType::Apartment,
            page: 1,
            min_price: None,
            max_price: None,
            min_size: None,
            max_size: None,
            db: None,
        };
        b.to_async(&rt).iter(|| async {
            black_box(scraper.scrape_page(&query).await.unwrap());
        });
    });

    // Benchmark HTML parsing with real sample data
    group.bench_function("html_parsing", |b| {
        let _html = include_str!("../debug/argenprop_House_boca_1.html");
        let scraper = ArgenPropScraper::new();
        let query = ScrapeQuery {
            district: "Boca".to_string(),
            property_type: PropertyType::House,
            page: 1,
            min_price: None,
            max_price: None,
            min_size: None,
            max_size: None,
            db: None,
        };
        b.to_async(&rt).iter(|| async {
            black_box(scraper.scrape_page(&query).await.unwrap());
        });
    });

    group.finish();
}

fn bench_concurrent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("concurrent");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));
    group.json_output(true);

    // Benchmark concurrent property insertions
    for size in [10, 100].iter() {
        group.bench_with_input(BenchmarkId::new("concurrent_insert", size), size, |b, &size| {
            let properties = generate_fake_properties(size);
            b.to_async(&rt).iter(|| async {
                let futures: Vec<_> = properties
                    .clone()
                    .into_iter()
                    .map(|p| {
                        let mut p = p;
                        async move {
                            let db = setup_test_db().await;
                            db.save_property(&mut p).await.unwrap()
                        }
                    })
                    .collect();
                black_box(futures::future::join_all(futures).await);
            });
        });
    }

    // Benchmark concurrent property queries
    for size in [10, 100].iter() {
        group.bench_with_input(BenchmarkId::new("concurrent_query", size), size, |b, &size| {
            b.to_async(&rt).iter(|| async {
                let db = setup_test_db().await;
                let properties = generate_fake_properties(size);
                for mut property in properties.clone() {
                    db.save_property(&mut property).await.unwrap();
                }
                let futures: Vec<_> = (0..size)
                    .map(|_| db.list_properties(
                        Some("benchmark"),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        false
                    ))
                    .collect();
                black_box(futures::future::join_all(futures).await);
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_database_operations,
    bench_scraper_operations,
    bench_concurrent_operations
);

criterion_main!(benches); 