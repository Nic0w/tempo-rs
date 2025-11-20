use tempo_rs::{TempoCalendars, TempoColor};

#[test]
fn test_deserialize_calendars() {
    let json = r#"
    {
        "tempo_like_calendars": {
            "start_date": "2025-11-15T00:00:00+01:00",
            "end_date": "2025-11-20T00:00:00+01:00",
            "values": [
                {
                    "start_date": "2025-11-19T00:00:00+01:00",
                    "end_date": "2025-11-20T00:00:00+01:00",
                    "value": "BLUE",
                    "updated_date": "2025-11-18T10:20:00+01:00"
                },
                {
                    "start_date": "2025-11-18T00:00:00+01:00",
                    "end_date": "2025-11-19T00:00:00+01:00",
                    "value": "BLUE",
                    "updated_date": "2025-11-17T10:20:00+01:00"
                }
            ]
        }
    }
    "#;

    let calendars: TempoCalendars = serde_json::from_str(json).expect("Failed to deserialize JSON");

    assert_eq!(calendars.tempo_like_calendars.len(), 1);

    let calendar = &calendars.tempo_like_calendars[0];
    assert_eq!(calendar.values.len(), 2);

    let first_value = &calendar.values[0];
    assert_eq!(first_value.value, TempoColor::Blue);
}

#[test]
fn test_unwrap_first_day_value() {
    let json = r#"
    {
        "tempo_like_calendars": {
            "start_date": "2025-11-19T00:00:00+01:00",
            "end_date": "2025-11-20T00:00:00+01:00",
            "values": [
                {
                    "start_date": "2025-11-19T00:00:00+01:00",
                    "end_date": "2025-11-20T00:00:00+01:00",
                    "value": "RED",
                    "updated_date": "2025-11-18T10:20:00+01:00"
                }
            ]
        }
    }
    "#;
    let calendars: TempoCalendars = serde_json::from_str(json).unwrap();
    let first_day = calendars.unwrap_first_day_value().unwrap();
    assert_eq!(first_day.value, TempoColor::Red);
}

#[test]
fn test_unwrap_days_values() {
    let json = r#"
    {
        "tempo_like_calendars": {
            "start_date": "2025-11-15T00:00:00+01:00",
            "end_date": "2025-11-20T00:00:00+01:00",
            "values": [
                {
                    "start_date": "2025-11-19T00:00:00+01:00",
                    "end_date": "2025-11-20T00:00:00+01:00",
                    "value": "BLUE",
                    "updated_date": "2025-11-18T10:20:00+01:00"
                },
                {
                    "start_date": "2025-11-18T00:00:00+01:00",
                    "end_date": "2025-11-19T00:00:00+01:00",
                    "value": "WHITE",
                    "updated_date": "2025-11-17T10:20:00+01:00"
                }
            ]
        }
    }
    "#;
    let calendars: TempoCalendars = serde_json::from_str(json).unwrap();
    let values: Vec<&tempo_rs::CalendarValue> = calendars.unwrap_days_values().collect();
    assert_eq!(values.len(), 2);
    assert_eq!(values[0].value, TempoColor::Blue);
    assert_eq!(values[1].value, TempoColor::White);
}

#[test]
fn test_tempo_color_display() {
    assert_eq!(format!("{}", TempoColor::Blue), "blue");
    assert_eq!(format!("{}", TempoColor::White), "white");
    assert_eq!(format!("{}", TempoColor::Red), "red");
}

#[test]
fn test_vec_or_struct_deserialization() {
    // Test single object (map)
    let json_single = r#"
    {
        "tempo_like_calendars": {
            "start_date": "2025-11-19T00:00:00+01:00",
            "end_date": "2025-11-20T00:00:00+01:00",
            "values": []
        }
    }
    "#;
    let calendars_single: TempoCalendars = serde_json::from_str(json_single).unwrap();
    assert_eq!(calendars_single.tempo_like_calendars.len(), 1);

    // Test array
    let json_array = r#"
    {
        "tempo_like_calendars": [
            {
                "start_date": "2025-11-19T00:00:00+01:00",
                "end_date": "2025-11-20T00:00:00+01:00",
                "values": []
            },
            {
                "start_date": "2025-11-18T00:00:00+01:00",
                "end_date": "2025-11-19T00:00:00+01:00",
                "values": []
            }
        ]
    }
    "#;
    let calendars_array: TempoCalendars = serde_json::from_str(json_array).unwrap();
    assert_eq!(calendars_array.tempo_like_calendars.len(), 2);
}
