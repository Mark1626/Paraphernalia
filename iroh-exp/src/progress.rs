use indicatif::{ProgressBar, ProgressStyle};

/// Create a progress bar for tracking download bytes.
pub fn make_download_bar(total: Option<u64>) -> ProgressBar {
    let pb = match total {
        Some(total) => ProgressBar::new(total),
        None => ProgressBar::new_spinner(),
    };
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec})")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb
}

/// Create a spinner for connecting/setup phases.
pub fn make_spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    pb
}

/// Create a spinner for per-file export progress.
pub fn make_export_bar(name: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} Exporting: {msg}")
            .unwrap(),
    );
    pb.set_message(name.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    pb
}
