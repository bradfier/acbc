use acbc::IncomingMessage;
use criterion::{criterion_group, criterion_main, Criterion, Throughput};

fn decode_incoming_update(c: &mut Criterion) {
    let input = include_bytes!("../docs/pcap/realtime_update.bin");
    let mut bench = c.benchmark_group("decoding");
    bench.throughput(Throughput::Elements(1));

    bench.bench_function("decode_realtime_update", |b| {
        b.iter(|| IncomingMessage::parse(input).unwrap());
    });

    let input = include_bytes!("../docs/pcap/realtime_car_update.bin");
    bench.bench_function("decode_realtime_car_update", |b| {
        b.iter(|| IncomingMessage::parse(input).unwrap());
    });
}

criterion_group!(decode, decode_incoming_update);
criterion_main!(decode);
