static SAMPLE_TXT: &[u8] = include_bytes!("../samples/lorem.txt");
static SAMPLE_JPG: &[u8] = include_bytes!("../samples/obama.jpg");
static SAMPLE_JSON: &[u8] = include_bytes!("../samples/schema.json");
static SAMPLE_HTML: &[u8] = include_bytes!("../samples/steganography.html");

static SAMPLES: &[(&str, &[u8])] = &[
    ("txt", SAMPLE_TXT),
    ("jpg", SAMPLE_JPG),
    ("json", SAMPLE_JSON),
    ("html", SAMPLE_HTML),
];

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;

fn roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("raw");
    group
        .sample_size(50)
        .measurement_time(Duration::from_secs(10));

    for (name, sample) in SAMPLES.iter().copied() {
        group.throughput(Throughput::Bytes(sample.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), sample, |b, s| {
            b.iter(|| {
                let encoded = zwc::encode(s.iter().copied());
                let decoded = zwc::decode(encoded);
                decoded.count();
            });
        });
    }
    group.finish();
}

fn roundtrip_compression(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression");
    group
        .sample_size(40)
        .measurement_time(Duration::from_secs(10));

    for (name, sample) in SAMPLES.iter().copied() {
        group.throughput(Throughput::Bytes(sample.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), sample, |b, s| {
            let (compression, ..) = zwc::Compression::optimal(s);
            b.iter(|| {
                let encoded = zwc::encode_compress(s.iter().copied(), compression);
                let decoded = zwc::decode_decompress(encoded, compression);
                decoded.count();
            });
        });
    }
    group.finish();
}

fn roundtrip_camouflage(c: &mut Criterion) {
    let mut group = c.benchmark_group("camouflage+compression");
    group
        .sample_size(20)
        .measurement_time(Duration::from_secs(45));
    for (name, sample) in SAMPLES.iter().copied() {
        group.throughput(Throughput::Bytes(sample.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), sample, |b, s| {
            b.iter(|| {
                let camouflaged =
                    zwc::camouflage(s.to_vec(), "Hello, World", None, Some(10)).unwrap();
                zwc::decamouflage(&camouflaged, None).unwrap();
            });
        });
    }
    group.finish();

    let mut group = c.benchmark_group("camouflage+encryption");
    group
        .sample_size(20)
        .measurement_time(Duration::from_secs(30));
    for (name, sample) in SAMPLES.iter().copied() {
        group.throughput(Throughput::Bytes(sample.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), sample, |b, s| {
            b.iter(|| {
                let camouflaged =
                    zwc::camouflage(s.to_vec(), "Hello, World", Some("secret"), Some(0)).unwrap();
                zwc::decamouflage(&camouflaged, Some("secret")).unwrap();
            });
        });
    }
    group.finish();

    let mut group = c.benchmark_group("camouflage+compression+encryption");
    group
        .sample_size(20)
        .measurement_time(Duration::from_secs(60));
    for (name, sample) in SAMPLES.iter().copied() {
        group.throughput(Throughput::Bytes(sample.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), sample, |b, s| {
            b.iter(|| {
                let camouflaged =
                    zwc::camouflage(s.to_vec(), "Hello, World", Some("secret"), Some(10)).unwrap();
                zwc::decamouflage(&camouflaged, Some("secret")).unwrap();
            });
        });
    }
    group.finish();
}

criterion_group!(
    roundtrips,
    roundtrip,
    roundtrip_compression,
    roundtrip_camouflage,
);
criterion_main!(roundtrips);
