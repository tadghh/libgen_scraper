use std::{fs, time::Duration};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use libgen_scraper::{processor::Processor, scraper::LibgenClient};
use scraper::Html;
use tokio::runtime::Runtime;

pub fn processor_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("html-processing");
    group.sample_size(100);

    let libgen_client = Processor::new();
    let search_string = "benchmark".to_string();
    let search_string_worst = "Benchmarking Security and Trust in Europe and the Us".to_string();
    let search_string_none = "Elephant".to_string();

    let html_content = fs::read_to_string("benches/benchmark_page.htm").unwrap();

    let document = Html::parse_document(&html_content);

    group.bench_with_input(
        BenchmarkId::new("search_title_in_document", "benchmark_page"),
        &document,
        |b, i| {
            b.iter(|| {
                let result = libgen_client.search_title_in_document(i, &search_string);
            })
        },
    );
    group.bench_with_input(
        BenchmarkId::new("search_title_in_document_end", "benchmark_page"),
        &document,
        |b, i| {
            b.iter(|| {
                let result = libgen_client.search_title_in_document(i, &search_string_worst);
            })
        },
    );

    // TODO: Makes tests
    // group.bench_with_input(
    //     BenchmarkId::new("search_title_not_in_document", "benchmark_page"),
    //     &document,
    //     |b, i| {
    //         b.iter(|| {
    //             let result = libgen_client
    //                 .search_title_in_document(i, &search_string_none)
    //                 .unwrap();

    //             assert!(result.is_some(), "Response is not none");
    //         })
    //     },
    // );
    group.finish();
}

pub fn html_scraper_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("page-scraping");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(61));

    let libgen_client = LibgenClient::new();

    // Define a list of titles to search
    let titles = vec![
        "The Predator (Animorphs)",
        "Color Atlas of Pharmacology",
        "Physics of life",
        "Physics and Chemistry Basis of Biotechnology",
        "Medical Imaging Physics",
    ];

    group.bench_function("search-group-books", |b| {
        b.iter(|| {
            for title in &titles {
                let result = libgen_client.search_book_by_title(title);
                // Block the main thread until the search operation completes
                let blocking_runtime = tokio::runtime::Runtime::new().unwrap();
                blocking_runtime
                    .block_on(result)
                    .expect("Error occurred during search");
            }
        })
    });

    group.finish();
}

pub fn async_scraper_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("page-scraping");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(31));

    // Async bench
    let libgen_client = LibgenClient::new();

    // Define a list of titles to search
    let titles = vec![
        "The Predator (Animorphs)",
        "Color Atlas of Pharmacology",
        "Physics of life",
        "Physics and Chemistry Basis of Biotechnology",
        "Medical Imaging Physics",
    ];

    group.bench_function("search-group-books-async", |b| {
        b.iter(|| {
            // Create a vector of async tasks for each title search
            let mut search_futures = Vec::new();
            for title in &titles {
                let future = libgen_client.search_book_by_title(title);
                search_futures.push(future);
            }

            // Block the main thread until all search futures complete
            Runtime::new().unwrap().block_on(async {
                for future in search_futures {
                    let _ = future.await;
                }
            });
        })
    });

    group.finish();
}

// TODO: 39 Million elements per second?
// criterion_group!(benches, bench);
criterion_group!(
    benches,
    processor_benchmark,
    // html_scraper_benchmark,
    // async_scraper_benchmark
);
criterion_main!(benches);
