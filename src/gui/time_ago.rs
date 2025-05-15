use chrono::TimeDelta;

// There's gotta be an existing library for this that does things much nicer.
/// Formats "X days ago" and et cetera.
pub fn relative_time_ago(time: TimeDelta) -> String {
    match time.num_days() {
        0 => match time.num_hours() {
            0 => {
                format!("{} minutes ago", time.num_minutes())
            }
            h => format!("{h} hours ago"),
        },
        1..=6 => format!("{} days ago", time.num_days()),
        7..=13 => format!("1 week ago"),
        14..=27 => format!("{} weeks ago", time.num_days() / 7),
        28..=45 => format!("1 month ago"),
        46..=364 => format!("{} months ago", time.num_days() / 30),
        d => format!("{} years ago", d / 365),
    }
}
