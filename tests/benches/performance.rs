use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use tokio::runtime::Runtime;
use brea_core::{Database, Property, PropertyType, db::types::{DbTimestamp, DbPropertyStatus, STATUS_ACTIVE}, db::migrations::apply_migrations};
use brea_scrapers::{argenprop::ArgenPropScraper, ScrapeQuery, Scraper};
use fake::{Fake, Faker};
use rand::Rng;
use std::time::Duration;
use url::Url;

// Helper function to create a test database
async fn setup_test_db() -> Database {
    let db = Database::new(":memory:").await.unwrap();
    apply_migrations(db.pool()).await.unwrap();
    db
}

// Helper function to generate fake properties
fn generate_fake_properties(count: usize) -> Vec<Property> {
    (0..count)
        .map(|_| {
            let property = Property {
                id: 0,
                external_id: Faker.fake(),
                source: "benchmark".to_string(),
                property_type: Some(PropertyType::Apartment.to_string()),
                district: Faker.fake(),
                title: Faker.fake(),
                description: Some(Faker.fake()),
                price_usd: rand::thread_rng().gen_range(50000.0..1000000.0),
                address: Faker.fake(),
                covered_size: Some(rand::thread_rng().gen_range(30.0..500.0)),
                rooms: Some(rand::thread_rng().gen_range(1..6)),
                antiquity: Some(rand::thread_rng().gen_range(0..50)),
                url: Url::parse("https://example.com").unwrap().to_string(),
                created_at: DbTimestamp::now(),
                updated_at: DbTimestamp::now(),
                status: DbPropertyStatus::new(STATUS_ACTIVE),
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
                black_box(db.get_properties().await.unwrap());
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

    // Benchmark scraping operations
    group.bench_function("scrape_query", |b| {
        b.to_async(&rt).iter(|| async {
            let scraper = ArgenPropScraper::new();
            let query = ScrapeQuery::new(
                "palermo".to_string(),
                PropertyType::Apartment,
                Some(100000.0),
                Some(500000.0),
                Some(50.0),
                Some(200.0),
                None,
            );
            black_box(scraper.scrape_page(&query).await.unwrap());
        });
    });

    group.finish();
}

fn bench_concurrent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("concurrent");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));

    // Benchmark concurrent database operations
    for size in [10, 100].iter() {
        group.bench_with_input(BenchmarkId::new("concurrent_queries", size), size, |b, &size| {
            b.to_async(&rt).iter(|| async {
                let db = setup_test_db().await;
                let properties = generate_fake_properties(size);
                for mut property in properties.clone() {
                    db.save_property(&mut property).await.unwrap();
                }
                let futures: Vec<_> = (0..size)
                    .map(|_| db.get_properties())
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