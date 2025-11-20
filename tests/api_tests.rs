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
    // Check date parsing if needed, but just checking structure for now
}
