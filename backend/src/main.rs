#[macro_use] extern crate rocket;
use rocket::http::Header;
use rocket::{Request, Response};
use rocket::fairing::{Fairing, Info, Kind};

pub struct Cors;

mod database;
use rocket::serde::json::Json;
use database::{Activity, get_activities_by_year, save_strava_activities};

use std::error::Error;
use csv::ReaderBuilder;
use chrono::NaiveDateTime;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}


#[get("/activities/<year>")]
fn get_year(year: i32) -> Result<Json<Vec<Activity>>, String> {
    // Call the function we wrote earlier in db.rs
    match get_activities_by_year(year) {
        Ok(activities) => Ok(Json(activities)),
        Err(e) => Err(format!("Database error: {}", e)),
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Cors) // Add this line!
        .mount("/", routes![get_year])
}

// 0 - Activity ID
// 1 - Activity Date
// 2 - Activity Name
// 3 - Activity Type
// 4 - Activity Description
// 5 - Elapsed Time
// 6 - Distance
// 7 - Max Heart Rate
// 8 - Relative Effort
// 9 - Commute
// 10 - Activity Private Note
// 11 - Activity Gear
// 12 - Filename
// 13 - Athlete Weight
// 14 - Bike Weight
// 15 - Elapsed Time
// 16 - Moving Time
// 17 - Distance
// 18 - Max Speed
// 19 - Average Speed
// 20 - Elevation Gain
// 21 - Elevation Loss
// 22 - Elevation Low
// 23 - Elevation High
// 24 - Max Grade
// 25 - Average Grade
// 26 - Average Positive Grade
// 27 - Average Negative Grade
// 28 - Max Cadence
// 29 - Average Cadence
// 30 - Max Heart Rate
// 31 - Average Heart Rate
// 32 - Max Watts
// 33 - Average Watts
// 34 - Calories
// 35 - Max Temperature
// 36 - Average Temperature
// 37 - Relative Effort
// 38 - Total Work
// 39 - Number of Runs
// 40 - Uphill Time
// 41 - Downhill Time
// 42 - Other Time
// 43 - Perceived Exertion
// 44 - Type
// 45 - Start Time
// 46 - Weighted Average Power
// 47 - Power Count
// 48 - Prefer Perceived Exertion
// 49 - Perceived Relative Effort
// 50 - Commute
// 51 - Total Weight Lifted
// 52 - From Upload
// 53 - Grade Adjusted Distance
// 54 - Weather Observation Time
// 55 - Weather Condition
// 56 - Weather Temperature
// 57 - Apparent Temperature
// 58 - Dewpoint
// 59 - Humidity
// 60 - Weather Pressure
// 61 - Wind Speed
// 62 - Wind Gust
// 63 - Wind Bearing
// 64 - Precipitation Intensity
// 65 - Sunrise Time
// 66 - Sunset Time
// 67 - Moon Phase
// 68 - Bike
// 69 - Gear
// 70 - Precipitation Probability
// 71 - Precipitation Type
// 72 - Cloud Cover
// 73 - Weather Visibility
// 74 - UV Index
// 75 - Weather Ozone
// 76 - Jump Count
// 77 - Total Grit
// 78 - Average Flow
// 79 - Flagged
// 80 - Average Elapsed Speed
// 81 - Dirt Distance
// 82 - Newly Explored Distance
// 83 - Newly Explored Dirt Distance
// 84 - Activity Count
// 85 - Total Steps
// 86 - Carbon Saved
// 87 - Pool Length
// 88 - Training Load
// 89 - Intensity
// 90 - Average Grade Adjusted Pace
// 91 - Timer Time
// 92 - Total Cycles


fn parse_to_timestamp(date_str: &str) -> Option<i64> {
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

// fn run() -> Result<(), Box<dyn Error>> {
//     let mut reader = ReaderBuilder::new()
//         .has_headers(true)
//         .from_path("activities.csv")?;
//     let mut activities: Vec<Activity> = Vec::new();

//     for result in reader.records() {
//         let record = result?;
        
//         let activity = Activity {
//             id: record.get(0).unwrap_or("0").parse()?,
//             activity_timestamp: parse_to_timestamp(record.get(1).unwrap_or("")).unwrap_or(0),
//             activity_name: record.get(2).unwrap_or("").to_string(),
//             activity_type: record.get(3).unwrap_or("").to_string(),
//             description: record.get(4).filter(|s| !s.is_empty()).map(|s| s.to_string()),
            
//             // Note: Indices 5 and 15 are both Elapsed Time; using index 5 here
//             elapsed_time: record.get(5).and_then(|s| s.parse().ok()).unwrap_or(0.0),
//             moving_time: record.get(16).and_then(|s| s.parse().ok()).unwrap_or(0.0),
            
//             elevation_gain: record.get(20).and_then(|s| s.parse().ok()).unwrap_or(0.0),
//             elevation_loss: record.get(21).and_then(|s| s.parse().ok()).unwrap_or(0.0),
            
//             avg_speed: record.get(19).and_then(|s| s.parse().ok()).unwrap_or(0.0),
//             max_speed: record.get(18).and_then(|s| s.parse().ok()).unwrap_or(0.0),
            
//             // Commute at index 9 (there is also one at 50)
//             commute: record.get(9).map(|s| s == "true" || s == "1").unwrap_or(false),
            
//             bike: record.get(11).filter(|s| !s.is_empty()).map(|s| s.to_string()),
            
//             // Distance at index 6 (duplicate at 17)
//             distance: record.get(6).and_then(|s| s.parse().ok()).unwrap_or(0.0),
            
//             // Heart rate indices: Max is 7 (or 30), Avg is 31
//             max_hr: record.get(7).and_then(|s| s.parse().ok()),
//             avg_hr: record.get(31).and_then(|s| s.parse().ok()),
//         };

//         activities.push(activity);

//         // break;
//     }

//     save_strava_activities(activities)?;

//     Ok(())
// }

// fn main() {
//     if let Err(err) = run() {
//         eprintln!("{}", err);
//     }
// }