use std::env;

use chrono::{Datelike, Days, SubsecRound, Timelike, Utc};

#[tokio::main]
async fn main() {

    let path = env::args().nth(1).expect("Missing argument: credential file");

    let tempo = tempo_rs::authorize_with_file(path)
        .await
        .unwrap();

    let now = Utc::now();

    let today = now.weekday();
    let today = today.num_days_from_monday();
    let to_last_monday = Days::new(today as u64);

    let last_monday = now.checked_sub_days(to_last_monday).unwrap();

    let last_monday = last_monday.with_hour(0).unwrap();
    let last_monday = last_monday.with_minute(0).unwrap();
    let last_monday = last_monday.with_second(0).unwrap();
    let last_monday = last_monday.round_subsecs(0);

    let tommorow = now.checked_add_days(Days::new(1)).unwrap();

    println!("last monday: {}", last_monday);

    let this_week = tempo
        .calendars(Some(last_monday), Some(tommorow), None)
        .await
        .unwrap();

    for calendar in this_week.tempo_like_calendars {
        for day in calendar.values.iter().rev() {
            let weekday = day.start_date.weekday();

            println!(
                "{} ({}) was: {}",
                weekday,
                day.start_date.format("%d/%m/%Y"),
                day.value
            );
        }
    }

    let next_day = tempo.next_day().await.unwrap();

    let value = &next_day.tempo_like_calendars[0].values[0];

    println!(
        "Tomorrow ({}) is: {}",
        value.start_date.format("%d/%m/%Y"),
        value.value
    );
}
