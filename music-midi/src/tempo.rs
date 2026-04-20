// REQ-O11, O15: TempoSource trait, StaticTempoMap, BoxedTempoSource, empty-rejection contract

use crate::event::AbsoluteTicks;

/// A source of tempo information for MIDI conversion and playback.
///
/// Implementors provide the BPM at any tick position and optionally
/// expose the list of change points for offline SMF construction.
///
/// # Implementing `TempoSource`
///
/// For a fixed tempo, use [`StaticTempoMap::constant`].
/// For a dynamic tempo (e.g. live-coded), implement this trait and supply
/// `change_points()` returning an ordered `(tick, bpm)` slice.
pub trait TempoSource: Send + Sync {
    /// Return the tempo in beats-per-minute at the given absolute tick.
    fn bpm_at(&self, tick: AbsoluteTicks) -> f32;

    /// Return all (tick, bpm) change points in ascending tick order.
    ///
    /// `SmfBuilder::build` calls this to populate the conductor track's
    /// `Tempo` meta events. Returns an empty slice by default (override
    /// for offline export).
    fn change_points(&self) -> &[(AbsoluteTicks, f32)] { &[] }
}

/// A heap-allocated, type-erased [`TempoSource`].
pub type BoxedTempoSource = Box<dyn TempoSource>;

/// A piecewise-constant tempo map backed by a sorted list of `(tick, bpm)` pairs.
///
/// The simplest way to construct one is [`StaticTempoMap::constant`].
/// For gradual tempo changes, call [`StaticTempoMap::push`] in tick order.
///
/// # Example
///
/// ```
/// use music_midi::{StaticTempoMap, TempoSource};
///
/// // Constant 120 BPM from the start
/// let map = StaticTempoMap::constant(120.0);
/// assert_eq!(map.bpm_at(0), 120.0);
/// assert_eq!(map.bpm_at(9999), 120.0);
/// ```
#[derive(Clone, Debug)]
pub struct StaticTempoMap {
    pub(crate) entries: Vec<(AbsoluteTicks, f32)>,
    pub(crate) ppq: u16,
}

impl StaticTempoMap {
    /// Create a tempo map with a single constant tempo starting at tick 0.
    ///
    /// Panics if `bpm` is not finite or is ≤ 0.
    pub fn constant(bpm: f32) -> Self {
        assert!(bpm.is_finite() && bpm > 0.0, "bpm must be finite and positive");
        Self { entries: vec![(0, bpm)], ppq: crate::DEFAULT_PPQ }  // REQ-O10
    }

    /// Append (or replace) a tempo change at `tick`.
    ///
    /// If an entry already exists at or after `tick`, all such entries are
    /// removed before the new entry is added, so the map always stays sorted.
    ///
    /// Panics if `bpm` is not finite or is ≤ 0.
    pub fn push(&mut self, tick: AbsoluteTicks, bpm: f32) {
        assert!(bpm.is_finite() && bpm > 0.0);
        match self.entries.last() {
            Some((last, _)) if *last >= tick => {
                self.entries.retain(|(t, _)| *t < tick);
            }
            _ => {}
        }
        self.entries.push((tick, bpm));
    }

    /// Convert an absolute tick position to wall-clock seconds by integrating
    /// the piecewise-constant tempo map.
    ///
    /// This is used by `MidiPlayer` to compute event deadlines during playback.
    pub fn ticks_to_seconds(&self, tick: AbsoluteTicks) -> f64 {
        // Integrate: between change_points[i] and change_points[i+1], bpm is constant.
        // seconds += (tick_delta / ppq) * 60 / bpm
        let ppq = self.ppq as f64;
        let mut seconds = 0.0;
        for window in self.entries.windows(2) {
            let (t0, bpm0) = window[0];
            let (t1, _) = window[1];
            if tick <= t0 { return seconds; }
            let end = tick.min(t1);
            seconds += ((end - t0) as f64 / ppq) * 60.0 / bpm0 as f64;
            if tick <= t1 { return seconds; }
        }
        if let Some(&(t, bpm)) = self.entries.last() {
            if tick > t {
                seconds += ((tick - t) as f64 / ppq) * 60.0 / bpm as f64;
            }
        }
        seconds
    }
}

impl TempoSource for StaticTempoMap {
    fn bpm_at(&self, tick: AbsoluteTicks) -> f32 {
        let idx = self.entries.partition_point(|(t, _)| *t <= tick);
        let i = idx.saturating_sub(1);
        self.entries[i].1
    }
    fn change_points(&self) -> &[(AbsoluteTicks, f32)] { &self.entries }
}
