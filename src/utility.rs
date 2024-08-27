use indicatif::{ProgressBar, ProgressStyle};

pub fn get_progressbar_long_jobs(job_name: &str, len: u64) -> ProgressBar {
    let bar = ProgressBar::new(len);
    bar.set_message(job_name.to_string());
    bar.set_style(
        ProgressStyle::with_template(" {msg} {wide_bar} estimated remaining: {eta_precise}")
            .unwrap(),
    );
    bar
}
