#[cfg(feature = "progress")]
use indicatif::ProgressBar;

#[cfg(feature = "progress")]
pub struct MaybeProgressBar {
    bar: ProgressBar,
}

#[cfg(feature = "progress")]
impl MaybeProgressBar {
    pub fn new(len: u64) -> Self {
        Self {
            bar: ProgressBar::new(len),
        }
    }

    pub fn inc(&self, n: u64) {
        self.bar.inc(n);
    }

    pub fn finish_and_clear(&self) {
        self.bar.finish_and_clear();
    }
}

#[cfg(not(feature = "progress"))]
pub struct MaybeProgressBar;

#[cfg(not(feature = "progress"))]
impl MaybeProgressBar {
    pub fn new(_: u64) -> Self {
        Self
    }

    pub fn inc(&self, _: u64) {}

    pub fn finish_and_clear(&self) {}
}
