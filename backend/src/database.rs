use rusqlite::{params, Connection};
use serde::Serialize;
use std::error::Error;
use chrono::{TimeZone, Utc};

#[derive(Debug, Serialize)]
pub struct Activity {
    pub id: i64,
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

pub fn save_strava_activities(activities: Vec<Activity>) -> Result<(), Box<dyn Error>> {
    // 1. Open connection
    let mut conn = Connection::open("wrapped.db")?;

    // 2. Create Table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS strava_activities (
            id                 INTEGER PRIMARY KEY,
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
                id, activity_timestamp, activity_name, activity_type, description,
                elapsed_time, moving_time, elevation_gain, elevation_loss,
                avg_speed, max_speed, commute, bike, distance, max_hr, avg_hr
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)"
        )?;

        for act in activities {
            stmt.execute(params![
                act.id,
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




pub fn get_activities_by_year(year: i32) -> Result<Vec<Activity>, Box<dyn Error>> {
    let conn = Connection::open("wrapped.db")?;

    // 1. Calculate the Unix timestamps for the start and end of the year
    let start_of_year = Utc.with_ymd_and_hms(year, 1, 1, 0, 0, 0).unwrap().timestamp();
    let end_of_year = Utc.with_ymd_and_hms(year, 12, 31, 23, 59, 59).unwrap().timestamp();

    // 2. Prepare the query
    let mut stmt = conn.prepare(
        "SELECT id, activity_timestamp, activity_name, activity_type, description, 
                elapsed_time, moving_time, elevation_gain, elevation_loss, 
                avg_speed, max_speed, commute, bike, distance, max_hr, avg_hr 
         FROM activities 
         WHERE activity_timestamp BETWEEN ?1 AND ?2
         ORDER BY activity_timestamp DESC"
    )?;

    // 3. Map the database rows back into Activity structs
    let activity_iter = stmt.query_map(params![start_of_year, end_of_year], |row| {
        Ok(Activity {
            id: row.get(0)?,
            activity_timestamp: row.get(1)?,
            activity_name: row.get(2)?,
            activity_type: row.get(3)?,
            description: row.get(4)?,
            elapsed_time: row.get(5)?,
            moving_time: row.get(6)?,
            elevation_gain: row.get(7)?,
            elevation_loss: row.get(8)?,
            avg_speed: row.get(9)?,
            max_speed: row.get(10)?,
            commute: row.get(11)?,
            bike: row.get(12)?,
            distance: row.get(13)?,
            max_hr: row.get(14)?,
            avg_hr: row.get(15)?,
        })
    })?;

    let mut results = Vec::new();
    for activity in activity_iter {
        results.push(activity?);
    }

    Ok(results)
}