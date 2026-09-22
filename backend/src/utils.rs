
use chrono::NaiveDateTime;

use std::{fs::File, io::Read};
use flate2::read::GzDecoder;

use fit2gpx::Fit;
use std::fs;

pub fn parse_to_bool(bool_str: &str) -> Option<bool> {
    if bool_str.is_empty() {
        return None;
    }

    return Some(bool_str != "0.0");
}

pub fn parse_to_timestamp(date_str: &str) -> Option<i64> {
    if date_str.is_empty() {
        return None;
    }

    // Format: "Oct 30, 2022, 7:46:37 AM"
    let format = "%b %d, %Y, %I:%M:%S %p";

    match NaiveDateTime::parse_from_str(date_str, format) {
        Ok(datetime) => Some(datetime.and_utc().timestamp()),
        Err(e) => {
            eprintln!("Parsing error for '{}': {}", date_str, e);
            None
        }
    }
}

pub fn decompress_gz(dir: &str, filename: &str)  -> std::io::Result<()> { 

    let path = format!("{}/{}", dir, filename);
    let out_path = format!("{}/{}", dir, filename.trim_end_matches(".gz"));

    let gz = File::open(&path)?;
    let mut decoder = GzDecoder::new(gz);
    let mut out = File::create(&out_path)?;

    std::io::copy(&mut decoder, &mut out)?;

    fs::remove_file(path)
}

pub fn get_file_list(dir: &str, ends_with: &str) -> Vec<std::fs::DirEntry> {
    fs::read_dir(dir)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.file_name().to_string_lossy().ends_with(ends_with))
        .collect()
}

pub fn convert_fit_file(file_dir: &str, file_id: &str) -> std::io::Result<()> {
    let in_file = format!("{}/{}.fit", file_dir, file_id);
    let out_file = format!("{}/{}.gpx", file_dir, file_id);
    fit2gpx::Fit::file_to_gpx(&in_file, &out_file).unwrap();
    fs::remove_file(in_file)?;
    Ok(())
}

pub fn get_gpx(dir: &str, file_id: i64) -> std::io::Result<String> {
    let filepath = format!("{}/{}.gpx", dir, file_id);
    fs::read_to_string(filepath)
}