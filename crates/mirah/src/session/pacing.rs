use std::time::{Duration, Instant};

pub(super) const FRAME_INTERVAL: Duration = Duration::from_millis(8);
pub(super) const SYNC_LIMIT: Duration = Duration::from_millis(150);

#[derive(Default)]
pub(super) struct Pacer {
    last_frame: Option<Instant>,
    sync_since: Option<Instant>,
}

impl Pacer {
    pub(super) fn ready_at(&mut self, now: Instant, synchronizing: bool) -> Instant {
        let paced = self.last_frame.map_or(now, |t| t + FRAME_INTERVAL);
        if !synchronizing {
            self.sync_since = None;
            return paced;
        }
        let since = *self.sync_since.get_or_insert(now);
        paced.max(since + SYNC_LIMIT)
    }

    pub(super) fn drew(&mut self, now: Instant, synchronizing: bool) {
        self.last_frame = Some(now);
        self.sync_since = synchronizing.then_some(now);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_frame_is_drawn_at_once() {
        let now = Instant::now();
        assert_eq!(Pacer::default().ready_at(now, false), now);
    }

    #[test]
    fn frames_are_spaced_by_the_interval() {
        let now = Instant::now();
        let mut pacer = Pacer::default();
        pacer.drew(now, false);
        assert_eq!(pacer.ready_at(now, false), now + FRAME_INTERVAL);
        let later = now + FRAME_INTERVAL * 3;
        assert_eq!(pacer.ready_at(later, false), now + FRAME_INTERVAL);
    }

    #[test]
    fn a_synchronized_update_waits_for_its_end_up_to_a_limit() {
        let now = Instant::now();
        let mut pacer = Pacer::default();
        assert_eq!(pacer.ready_at(now, true), now + SYNC_LIMIT);
        let soon = now + Duration::from_millis(5);
        assert_eq!(pacer.ready_at(soon, true), now + SYNC_LIMIT);
        assert_eq!(pacer.ready_at(soon, false), soon);
    }

    #[test]
    fn a_long_synchronized_update_is_drawn_once_per_limit() {
        let now = Instant::now();
        let mut pacer = Pacer::default();
        pacer.ready_at(now, true);
        let forced = now + SYNC_LIMIT;
        pacer.drew(forced, true);
        assert_eq!(pacer.ready_at(forced, true), forced + SYNC_LIMIT);
    }
}
