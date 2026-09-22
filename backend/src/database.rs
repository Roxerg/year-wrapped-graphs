use rusqlite::{params, Connection};
use serde::Serialize;
use std::error::Error;
use chrono::{TimeZone, Utc};
use csv::ReaderBuilder;
use csv::StringRecord;

use crate::utils::parse_to_timestamp;
use crate::utils::parse_to_bool;
use crate::utils::decompress_gz;
use crate::utils::get_file_list;
use crate::utils::convert_fit_file;


#[derive(Debug, Serialize)]
pub struct Activity {
    pub id: i64,
    pub file_id: i64,
    pub activity_timestamp: i64,
    pub activity_name: String,
    pub activity_type: String,
    pub description: Option<String>,
    pub elapsed_time: f32,
    pub moving_time: f32,
    pub elevation_gain: f32,
    pub elevation_loss: f32,
    pub avg_speed: f32,
    pub max_speed: f32,
    pub commute: bool,
    pub bike: Option<String>,
    pub distance: f32,
    pub max_hr: Option<f32>,
    pub avg_hr: Option<f32>,
}

pub fn load_strava_main_file(strava_dir: &str) -> Result<(), Box<dyn Error>> {
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path(format!("{}/{}", strava_dir, "activities.csv"))?;

    let mut activities: Vec<Activity> = Vec::new();
    for res  in reader.records() {
        // println!("{:?}", res);

        let row = res.unwrap();

        // println!("{:?}", row.get(1));

        fn get_col_impl<T: std::str::FromStr>(
            row: &StringRecord, 
            field_name: &str, 
            idx: usize, 
            maybe_parse_fn: Option<&dyn Fn(&str) -> Option<T>>
        ) -> T {
            let raw = row.get(idx)
                .unwrap_or_else(|| {panic!("failed to get {:?} from row {:?}", field_name, row)});
            
            let parsed = match maybe_parse_fn {
                Some(f) => f(raw),
                None => raw.parse().ok(),
            };

            return parsed.unwrap_or_else(|| {panic!("failed to parse {:?}={:?} as {}", field_name, raw, std::any::type_name::<T>())});
        }

        macro_rules! get_col {
            ($row:expr, $field:expr, $idx:expr) => {
                get_col_impl($row, $field, $idx, None)
            };
            ($row:expr, $field:expr, $idx:expr, $pfn:expr) => {
                get_col_impl($row, $field, $idx, Some($pfn))
            };
        }

        fn get_col_opt<T: std::str::FromStr>(
            row: &StringRecord,
            idx: usize,
        ) -> Option<T> {
            row.get(idx)
                .filter(|s| !s.is_empty())
                .and_then(|raw| raw.parse().ok())
        }

        fn trim_file(f: &str) -> Option<i64> {
            return f.trim_end_matches(".gz")
            .trim_end_matches(".fit")
            .trim_end_matches(".gpx")
            .trim_end_matches(".tcx")
            .trim_start_matches("activities/")
            .parse()
            .ok();
        }

        if row.get(3).unwrap() != "Ride" {
            dbg!(row.get(3).unwrap());
            continue;
        }

        let activity: Activity = Activity {
            id: get_col!(&row, "id", 0),
            file_id: get_col!(&row, "file_id", 12, &trim_file),
            activity_timestamp: get_col!(&row, "activity_timestamp", 1, &parse_to_timestamp),
            activity_name: get_col!(&row, "activity_name",2),
            activity_type: get_col!(&row, "activity_type",3),
            description: get_col_opt(&row,4),
            elapsed_time: get_col!(&row, "elapsed_time",15),
            moving_time: get_col!(&row, "moving_time",16),
            elevation_gain: get_col!(&row, "elevation_gain",20),
            elevation_loss: get_col!(&row, "elevation_loss",21),
            avg_speed: get_col!(&row, "avg_speed",19),
            max_speed: get_col!(&row, "max_speed",18),
            commute: get_col!(&row, "commute", 50, &parse_to_bool),
            bike: get_col_opt(&row,68),
            distance: get_col!(&row, "distance",6),
            max_hr: get_col_opt(&row,14),
            avg_hr: get_col_opt(&row,15)
        };

        activities.push(activity);
    }

    let act_dir = format!("{strava_dir}/activities");
    for fit_f in get_file_list(&act_dir.to_string(), ".gz") {
        let filename = fit_f.file_name();
        decompress_gz(
            &act_dir.to_string(), 
            filename.to_str().unwrap()
        ).ok();
    }

    for fit_f in get_file_list(&act_dir.to_string(), ".fit") {
        let f_path = fit_f.path();
        let f_name = fit_f.file_name();
        let f_str = f_name.to_string_lossy();

        let source_dir = f_path.parent().unwrap().to_str().unwrap();
        let file_id = f_str.trim_end_matches(".fit");
        let _ = convert_fit_file(source_dir, file_id);
    }


    // dbg!(&activities);
    let _ = save_strava_activities(activities);
    Ok(())
}

pub fn save_strava_activities(activities: Vec<Activity>) -> Result<(), Box<dyn Error>> {
    // 1. Open connection
    let mut conn = Connection::open("wrapped.db")?;

    // 2. Create Table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS strava_activities (
            id                 INTEGER PRIMARY KEY,
            file_id            INTEGER NOT NULL,
            activity_timestamp INTEGER NOT NULL,
            activity_name      TEXT NOT NULL,
            activity_type      TEXT NOT NULL,
            description        TEXT,
            elapsed_time       REAL,
            moving_time        REAL,
            elevation_gain     REAL,
            elevation_loss     REAL,
            avg_speed          REAL,
            max_speed          REAL,
            commute            INTEGER,
            bike               TEXT,
            distance           REAL,
            max_hr             REAL,
            avg_hr             REAL
        )",
        [],
    )?;

    // 3. Use a Transaction for speed
    // This is critical for performance when saving many rows
    let tx = conn.transaction()?;

    {
        let mut stmt = tx.prepare(
            "INSERT OR REPLACE INTO strava_activities (
                id, file_id, activity_timestamp, activity_name, activity_type, description,
                elapsed_time, moving_time, elevation_gain, elevation_loss,
                avg_speed, max_speed, commute, bike, distance, max_hr, avg_hr
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)"
        )?;

        for act in activities {
            stmt.execute(params![
                act.id,
                act.file_id,
                act.activity_timestamp,
                act.activity_name,
                act.activity_type,
                act.description,
                act.elapsed_time,
                act.moving_time,
                act.elevation_gain,
                act.elevation_loss,
                act.avg_speed,
                act.max_speed,
                act.commute,
                act.bike,
                act.distance,
                act.max_hr,
                act.avg_hr,
            ])?;
        }
    } // stmt goes out of scope here so we can commit

    tx.commit()?;
    Ok(())
}

pub fn db_res_to_struct(row: &rusqlite::Row) -> Result<Activity, rusqlite::Error> {
    Ok(Activity {
        id: row.get(0)?,
        file_id: row.get(1)?,
        activity_timestamp: row.get(2)?,
        activity_name: row.get(3)?,
        activity_type: row.get(4)?,
        description: row.get(5)?,
        elapsed_time: row.get(6)?,
        moving_time: row.get(7)?,
        elevation_gain: row.get(8)?,
        elevation_loss: row.get(9)?,
        avg_speed: row.get(10)?,
        max_speed: row.get(11)?,
        commute: row.get(12)?,
        bike: row.get(13)?,
        distance: row.get(14)?,
        max_hr: row.get(15)?,
        avg_hr: row.get(16)?,
    })
}

pub fn get_activity_by_id(act_id: i64) -> Result<Activity, Box<dyn Error>> {
    let conn = Connection::open("wrapped.db")?;
    dbg!(act_id);

    let mut stmt = conn.prepare(
        "SELECT id, file_id, activity_timestamp, activity_name, activity_type, description, 
                elapsed_time, moving_time, elevation_gain, elevation_loss, 
                avg_speed, max_speed, commute, bike, distance, max_hr, avg_hr 
         FROM strava_activities 
         WHERE id = ?1"
    )?;

    let activity = stmt.query_row(params![act_id], db_res_to_struct)?;
    Ok(activity)
}

pub fn get_activities_by_year(year: i32) -> Result<Vec<Activity>, Box<dyn Error>> {
    let conn = Connection::open("wrapped.db")?;

    // 1. Calculate the Unix timestamps for the start and end of the year
    let start_of_year = Utc.with_ymd_and_hms(year, 1, 1, 0, 0, 0).unwrap().timestamp();
    let end_of_year = Utc.with_ymd_and_hms(year, 12, 31, 23, 59, 59).unwrap().timestamp();

    // 2. Prepare the query
    let mut stmt = conn.prepare(
        "SELECT id, file_id, activity_timestamp, activity_name, activity_type, description, 
                elapsed_time, moving_time, elevation_gain, elevation_loss, 
                avg_speed, max_speed, commute, bike, distance, max_hr, avg_hr 
         FROM strava_activities 
         WHERE activity_timestamp BETWEEN ?1 AND ?2
         ORDER BY activity_timestamp DESC"
    )?;

    // 3. Map the database rows back into Activity structs
    let activity_iter = stmt.query_map(params![start_of_year, end_of_year], |row| {
        db_res_to_struct(row)
    })?;

    let mut results = Vec::new();
    for activity in activity_iter {
        results.push(activity?);
    }

    Ok(results)
}