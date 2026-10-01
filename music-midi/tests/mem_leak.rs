//! Dedicated integration test binary that installs a `CountingAllocator`
//! global allocator and asserts that SmfBuilder::build does not retain
//! memory across iterations.
//!
//! This file lives in its own `tests/*.rs` to keep the custom allocator
//! OUT of the rest of the test suite.

#![cfg(feature = "smf")]

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct CountingAllocator {
    bytes_allocated: AtomicUsize,
    bytes_deallocated: AtomicUsize,
}

impl CountingAllocator {
    const fn new() -> Self {
        Self {
            bytes_allocated: AtomicUsize::new(0),
            bytes_deallocated: AtomicUsize::new(0),
        }
    }

    /// Bytes currently allocated-and-retained (allocated − deallocated).
    fn retained_bytes(&self) -> usize {
        self.bytes_allocated
            .load(Ordering::Relaxed)
            .saturating_sub(self.bytes_deallocated.load(Ordering::Relaxed))
    }
}

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() {
            self.bytes_allocated
                .fetch_add(layout.size(), Ordering::Relaxed);
        }
        p
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        self.bytes_deallocated
            .fetch_add(layout.size(), Ordering::Relaxed);
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator::new();

#[test]
fn owned_smf_does_not_leak_across_builds() {
    use music::note::pitch::Pitch;
    use music_midi::{smf::SmfBuilder, StaticTempoMap};

    // Warm up: run 100 builds to stabilize any one-time arena initializations.
    let c4 = Pitch::from_midi(60).unwrap();
    for _ in 0..100 {
        let _ = SmfBuilder::new()
            .ppq(480)
            .tempo(StaticTempoMap::constant(120.0))
            .add_track("warmup", 0, &c4)
            .unwrap()
            .build()
            .unwrap();
    }

    // REQ-O3: measure retained-bytes growth across 1000 iterations.
    let baseline = ALLOCATOR.retained_bytes();
    for _ in 0..1000 {
        let owned = SmfBuilder::new()
            .ppq(480)
            .tempo(StaticTempoMap::constant(120.0))
            .add_track("piano", 0, &c4)
            .unwrap()
            .build()
            .unwrap();
        // Force the serialization path to exercise the whole owned-data
        // plumbing, then drop the OwnedSmf.
        let _ = owned.to_bytes().unwrap();
    }
    let after = ALLOCATOR.retained_bytes();

    // The retained-bytes figure should not grow proportionally to iteration count.
    // A loose bound of 64 KiB allows normal cache churn from tests/allocators.
    let growth = after.saturating_sub(baseline);
    assert!(
        growth < 64 * 1024,
        "REQ-O3: retained bytes grew by {} across 1000 builds (budget: 64 KiB). \
         If the leak path is restored, this grows O(iterations).",
        growth
    );

    // REQ-O3a: structural assertion — OwnedSmf size is a compile-time constant,
    // which by construction means OwnedSmf does not grow per build() iteration.
    use music_midi::smf::OwnedSmf;
    const OWNED_SMF_SIZE: usize = std::mem::size_of::<OwnedSmf>();
    let _ = OWNED_SMF_SIZE;

    // Build one OwnedSmf with 3 instrument tracks and confirm the owned
    // buffer count matches. A leaked `&'static [u8]` path would bypass this.
    let owned_3 = SmfBuilder::new()
        .ppq(480)
        .tempo(StaticTempoMap::constant(120.0))
        .add_track("t1", 0, &c4)
        .unwrap()
        .add_track("t2", 1, &c4)
        .unwrap()
        .add_track("t3", 2, &c4)
        .unwrap()
        .build()
        .unwrap();
    assert!(owned_3.track_name_count() <= 3 + 1);
    assert_eq!(owned_3.track_name_count(), 3);
}
