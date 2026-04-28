use std::{fs, io, path::PathBuf};

fn replay_and_process(logs_dir: &PathBuf) -> io::Result<()> {
    let mut valid_wals: Vec<(u64, PathBuf)> = Vec::new();
    let mut to_delete: Vec<PathBuf> = Vec::new();

    let entries = fs::read_dir(&logs_dir)?;

    for entry in entries.flatten() {
        let path = entry.path();

        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        if is_valid_wal(name) {
            if let Some(date) = parse_date(name) {
                valid_wals.push((date, path));
            } else {
                to_delete.push(path);
            }
        } else {
            to_delete.push(path);
        }
    }

    // sort WALs by date ascending
    valid_wals.sort_by_key(|(date, _)| *date);

    println!("valid WALs (sorted):");
    for (d, p) in &valid_wals {
        println!("  {} -> {:?}", d, p);
    }

    println!("files to delete:");
    for p in &to_delete {
        println!("  {:?}", p);
    }

    Ok(())

    // next step (NOT here):
    // - compare with archive/
    // - spawn conversion jobs
    // - delete invalid files
}

fn is_valid_wal(name: &str) -> bool {
    name.len() == 13
        && name.ends_with(".wal")
        && name[..8].chars().all(|c| c.is_ascii_digit())
}

fn parse_date(name: &str) -> Option<u64> {
    name[..8].parse::<u64>().ok()
}

#[cfg(test)]
mod test {
    use std::{fs, path::PathBuf};

    #[test]
    pub fn replay_and_process() {
        let logs_dir: PathBuf = "/home/entity/sandbox/telemetry/logs".into();
        fs::create_dir_all(&logs_dir).unwrap();
        super::replay_and_process(&logs_dir).unwrap();
    }
}