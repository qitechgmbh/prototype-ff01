use anyhow::{Result, anyhow};
use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::{Read, Write, Cursor, BufRead, BufReader},
    path::{Path, PathBuf},
};

use zip::{ZipArchive, write::FileOptions, ZipWriter};

static REQUIRED_FILES: &[&str] = &[
    "logs.csv",
    "bounds.csv",
    "orders.csv",
    "states.csv",
    "plates.parqet",
    "weights.parqet",
];

pub fn create_archive_entry(input_dir: PathBuf, output_dir: PathBuf) {
    let time_entries = extract_entries(&input_dir);

    // batch_ + unix timestamp   
}

fn extract_entries(dir: &PathBuf) -> Result<Vec<String>, String> {
    let mut folders: Vec<String> = Vec::new();

    let entries = match fs::read_dir(&dir) {
        Ok(e)  => e,
        Err(e) => return Err(format!("Failed to read directory {:?}: {}", dir, e))
    };

    // iterate each day 
    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {

                // check: exactly 4 digits, should match XXXX like 1430
                if name.len() != 4 || !name.chars().all(|c| c.is_ascii_digit()) {
                    return Err(format!("Dir didn't match predicate {:?}", dir));
                }



                folders.push(name.to_string());
            }
        } else {
            return Err(format!("Only paths are allowed {:?}", path));
        }
    }

    // sort numerically
    folders.sort_by_key(|s| s.parse::<u32>().expect("folder must be numeric"));
    return Ok(folders);
}

pub fn deconstruct_and_save(
    data: &[u8],
    output_dir: &Path,
    name: &str,
) -> Result<()> {
    fs::create_dir_all(output_dir)?;

    let mut groups: HashMap<String, Vec<Vec<u8>>> = HashMap::new();

    let reader = Cursor::new(data);
    let mut zip = ZipArchive::new(reader)?;

    let mut names = Vec::new();
    for i in 0..zip.len() {
        names.push(zip.by_index(i)?.name().to_string());
    }

    let (folders, hours) = validate_and_collect(&names)?;

    println!("Processing hours: {:?}", hours);

    // READ FILES
    for hour in folders.keys() {
        for file in &folders[hour] {
            let path = format!("{}/{}", hour, file);
            println!("Reading {}", path);

            let mut f = zip.by_name(&path)?;
            let mut content = Vec::new();
            f.read_to_end(&mut content)?;

            while content.ends_with(&[b'\n']) || content.ends_with(&[b'\r']) {
                content.pop();
            }

            if content.is_empty() {
                println!("Skipping empty file: {}", path);
                continue;
            }

            groups.entry(file.clone()).or_default().push(content);
        }
    }

    let tmp_dir = output_dir.join("tmp");
    fs::create_dir_all(&tmp_dir)?;

    // ENSURE FILES EXIST
    for file in REQUIRED_FILES {
        let path = tmp_dir.join(file);
        if !path.exists() {
            println!("Creating empty file: {}", file);
            File::create(path)?;
        }
    }

    // MERGE + SAVE
    for (file, chunks) in &groups {
        let out_path = tmp_dir.join(file);

        println!("Writing {} ({} parts)", file, chunks.len());

        let mut out = File::create(&out_path)?;

        for (i, chunk) in chunks.iter().enumerate() {
            if i > 0 {
                out.write_all(b"\n")?;
            }
            out.write_all(chunk)?;
        }

        println!("Saved {:?}", out_path);
    }

    let orders = extract_orders(&tmp_dir)?;

    // INSTALL ORDERS
    let order_dir = output_dir.join("orders");
    let order_tmp = order_dir.join("tmp");
    fs::create_dir_all(&order_tmp)?;

    for (id, start, end) in &orders {
        for file in REQUIRED_FILES {
            let src = tmp_dir.join(file);
            let dst = order_tmp.join(file);

            let src_file = File::open(&src)?;
            let reader = BufReader::new(src_file);
            let mut dest = File::create(&dst)?;

            let mut state = "BEFORE";

            for line in reader.lines() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }

                let timestamp = line.split(',').next().unwrap_or("").trim();

                if state == "BEFORE" {
                    if timestamp >= start {
                        state = "INSIDE";
                    } else {
                        continue;
                    }
                }

                if state == "INSIDE" {
                    if timestamp > end {
                        break;
                    }
                    writeln!(dest, "{}", line)?;
                }
            }
        }

        let zip_path = order_dir.join(format!("{}.zip", id));
        create_archive(&order_tmp, &zip_path)?;
    }

    println!("Removing {:?}", order_tmp);
    let _ = fs::remove_dir_all(&order_tmp);

    let final_zip = output_dir.join("days").join(format!("{}.zip", name));
    create_archive(&tmp_dir, &final_zip)?;

    println!("Removing {:?}", tmp_dir);
    let _ = fs::remove_dir_all(&tmp_dir);

    Ok(())
}

fn create_archive(src_dir: &Path, out_path: &Path) -> Result<()> {
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let file = File::create(out_path)?;
    let mut zip = ZipWriter::new(file);

    let options = FileOptions::default();

    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let name = path.file_name().unwrap().to_string_lossy();
            zip.start_file(name, options)?;

            let mut f = File::open(path)?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
        }
    }

    zip.finish()?;
    Ok(())
}

fn validate_and_collect(
    namelist: &[String],
) -> Result<(HashMap<String, HashSet<String>>, Vec<u32>)> {
    let mut folders: HashMap<String, HashSet<String>> = HashMap::new();

    for name in namelist {
        let parts: Vec<&str> = name.split('/').collect();

        if parts.len() != 2 {
            return Err(anyhow!("Invalid format: {}", name));
        }

        let hour = parts[0];
        let file = parts[1];

        if hour.len() != 2 || !hour.chars().all(|c| c.is_ascii_digit()) {
            return Err(anyhow!("Invalid hour format: {}", name));
        }

        let h: u32 = hour.parse()?;
        if h > 23 {
            return Err(anyhow!("Hour out of range: {}", hour));
        }

        folders
            .entry(hour.to_string())
            .or_default()
            .insert(file.to_string());
    }

    if folders.is_empty() {
        return Err(anyhow!("No valid folders"));
    }

    let mut hours: Vec<u32> = folders.keys()
        .map(|h| h.parse().unwrap())
        .collect();

    hours.sort();

    let expected: Vec<u32> = (hours[0]..=hours[hours.len() - 1]).collect();

    if hours != expected {
        return Err(anyhow!("Non-continuous hours"));
    }

    for (hour, files) in &folders {
        let required: HashSet<String> =
            REQUIRED_FILES.iter().map(|s| s.to_string()).collect();

        if files != &required {
            return Err(anyhow!("Invalid files in hour {}", hour));
        }
    }

    Ok((folders, hours))
}

fn extract_orders(tmp_dir: &Path) -> Result<Vec<(String, String, String)>> {
    let states_file = tmp_dir.join("states.csv");

    if !states_file.exists() {
        return Err(anyhow!("Missing states.csv"));
    }

    let file = File::open(states_file)?;
    let reader = BufReader::new(file);

    let mut orders: HashMap<String, (Option<String>, Option<String>)> = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() != 3 {
            continue;
        }

        let (ts, state, id) = (parts[0], parts[1], parts[2]);

        let entry = orders.entry(id.to_string()).or_insert((None, None));

        if state == "1" {
            entry.0 = Some(ts.to_string());
        } else if state == "2" {
            entry.1 = Some(ts.to_string());
        }
    }

    let mut result = Vec::new();

    for (id, (start, end)) in orders {
        if let (Some(s), Some(e)) = (start, end) {
            result.push((id, s, e));
        }
    }

    Ok(result)
}