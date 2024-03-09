use criterion::{criterion_group, criterion_main, Criterion, Throughput};

use libgen_scraper::scraper::LibgenClient;

fn blocking_search_five_books() {
    let test_client = LibgenClient::new();
    let titles = vec![
        "The Predator (Animorphs)",
        "Color Atlas of Pharmacology",
        "Physics of life",
        "Physics and Chemistry Basis of Biotechnology",
        "Medical Imaging Physics",
    ];

    for title in titles {
        let result = test_client.search_book_by_title(&title);

        // Block the main thread until the search operation completes
        let blocking_runtime = tokio::runtime::Runtime::new().unwrap();
        blocking_runtime
            .block_on(result)
            .expect("Error occurred during search");
    }
}

fn search_for_five_books() {
    // Initialize a LibgenClient instance
    let libgen_client = LibgenClient::new();

    // Define a list of titles to search
    let titles = vec![
        "The Predator (Animorphs)",
        "Color Atlas of Pharmacology",
        "Physics of life",
        "Physics and Chemistry Basis of Biotechnology",
        "Medical Imaging Physics",
    ];

    // Call the search_books_by_titles method and collect the results
    let results = libgen_client.search_books_by_titles(titles);

    // Calculate the duration
    let blocking_runtime = tokio::runtime::Runtime::new().unwrap();
    blocking_runtime.block_on(results);
}

// Bad
fn bench(c: &mut Criterion) {
    let elements_1 = vec![
        "The Predator (Animorphs)",
        "Color Atlas of Pharmacology",
        "Physics of life",
        "Physics and Chemistry Basis of Biotechnology",
        "Medical Imaging Physics",
    ];
    let libgen_client = LibgenClient::new();

    let mut group = c.benchmark_group("throughput-example");
    group.sample_size(10); // Set the sample size to 10
    for (i, element) in elements_1.iter().enumerate() {
        let throughput = Throughput::Elements(1);
        group.throughput(throughput);

        group.bench_function(format!("Search {}", i), |b| {
            b.iter(|| libgen_client.search_book_by_title(element))
        });
    }

    group.finish();
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("page-scraping");
    group.sample_size(10);

    group.bench_function("search-five-book", |b| {
        b.iter(|| blocking_search_five_books())
    });

    group.bench_function("search-five-book-async", |b| {
        b.iter(|| search_for_five_books())
    });
    group.finish();
}

// TODO: 39 Millions elements per second?
// criterion_group!(benches, bench);
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
