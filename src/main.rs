use lockless_datastructures::{AtomicRingBufferMpmc, AtomicRingBufferSpsc, MutexRingBuffer};
use std::{
    hint::black_box,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};

const CAPACITY: usize = 1024;
const OPERATIONS: usize = 1_000_000; 

fn main() {
    println!("Running benchmarks with {} operations per test...\n", OPERATIONS);

    println!("--- SPSC ---");
    
    let start = Instant::now();
    mutex_ring_buffer_spsc_benchmark();
    let duration = start.elapsed();
    println!("MutexRingBufferSpsc:  {:?}", duration);

    let start = Instant::now();
    atomic_ring_buffer_spsc_benchmark();
    let duration = start.elapsed();
    println!("AtomicRingBufferSpsc: {:?}\n", duration);


    println!("--- MPMC ---");

    let start = Instant::now();
    mutex_ring_buffer_mpmc_benchmark();
    let duration = start.elapsed();
    println!("MutexRingBufferMpmc:  {:?}", duration);

    let start = Instant::now();
    atomic_ring_buffer_mpmc_benchmark();
    let duration = start.elapsed();
    println!("AtomicRingBufferMpmc: {:?}", duration);
}

fn mutex_ring_buffer_spsc_benchmark() {
    let buffer: MutexRingBuffer<i32, CAPACITY> = MutexRingBuffer::new();

    // spawns two threads, one producer ,other consumer
    let producer_buffer = buffer.clone();
    let consumer_buffer = buffer.clone();

    let producer = std::thread::spawn(move || {
        for i in 0..OPERATIONS {
            while producer_buffer.push(black_box(i as i32)).is_err() {
                std::hint::spin_loop();
            }
        }
    });

    let consumer = std::thread::spawn(move || {
        let mut count = 0;
        while count < OPERATIONS {
            if let Some(value) = consumer_buffer.pop() {
                black_box(value);
                count += 1;
            } else {
                std::hint::spin_loop();
            }
        }
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}

fn atomic_ring_buffer_spsc_benchmark() {
    let buffer = AtomicRingBufferSpsc::<i32, CAPACITY>::new();

    // spawns two threads, one producer ,other consumer
    let producer_buffer = buffer.clone();
    let consumer_buffer = buffer.clone();

    let producer = std::thread::spawn(move || {
        for i in 0..OPERATIONS {
            while producer_buffer.push(black_box(i as i32)).is_err() {
                std::hint::spin_loop();
            }
        }
    });

    let consumer = std::thread::spawn(move || {
        let mut count = 0;
        while count < OPERATIONS {
            if let Some(value) = consumer_buffer.pop() {
                black_box(value);
                count += 1;
            } else {
                 std::hint::spin_loop();
            }
        }
    });

    producer.join().unwrap();
    consumer.join().unwrap();
}

fn mutex_ring_buffer_mpmc_benchmark() {
    let buffer: MutexRingBuffer<i32, CAPACITY> = MutexRingBuffer::new();

    let consumed_count = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::with_capacity(4);

    let ops_per_producer = OPERATIONS / 2;


    // spawns 4 threads, two producer ,two consumer
    for _ in 0..2 {
        let buf = buffer.clone();
        handles.push(std::thread::spawn(move || {
            for i in 0..ops_per_producer {
                while buf.push(black_box(i as i32)).is_err() {
                    std::hint::spin_loop();
                }
            }
        }));
    }

    for _ in 0..2 {
        let buf = buffer.clone();
        let counter = consumed_count.clone();
        handles.push(std::thread::spawn(move || {
            loop {
                if counter.load(Ordering::Relaxed) >= OPERATIONS {
                    break;
                }

                if let Some(val) = buf.pop() {
                    black_box(val);
                    counter.fetch_add(1, Ordering::Relaxed);
                } else {
                    std::hint::spin_loop();
                }
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
}

fn atomic_ring_buffer_mpmc_benchmark() {
    let buffer = AtomicRingBufferMpmc::<i32, CAPACITY>::new();

    let consumed_count = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::with_capacity(4);

    let ops_per_producer = OPERATIONS / 2;


    // spawns 4 threads, two producer ,two consumer
    for _ in 0..2 {
        let buf = buffer.clone();
        handles.push(std::thread::spawn(move || {
            for i in 0..ops_per_producer {
                while buf.push(black_box(i as i32)).is_err() {
                    std::hint::spin_loop();
                }
            }
        }));
    }

    for _ in 0..2 {
        let buf = buffer.clone();
        let counter = consumed_count.clone();
        handles.push(std::thread::spawn(move || {
            loop {
                if counter.load(Ordering::Relaxed) >= OPERATIONS {
                    break;
                }

                if let Some(val) = buf.pop() {
                    black_box(val);
                    counter.fetch_add(1, Ordering::Relaxed);
                } else {
                    std::hint::spin_loop();
                }
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
}
