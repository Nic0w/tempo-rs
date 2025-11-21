use std::path::Path;

const CREDENTIALS_FILE: &str = "tempo_api_credentials.secret";

#[tokio::test]
async fn test_real_authentication() {
    if !Path::new(CREDENTIALS_FILE).exists() {
        eprintln!(
            "Skipping test_real_authentication: {} not found",
            CREDENTIALS_FILE
        );
        return;
    }

    let result = tempo_rs::authorize_with_file(CREDENTIALS_FILE).await;
    assert!(result.is_ok(), "Authentication failed: {:?}", result.err());
}

#[tokio::test]
async fn test_real_api_call() {
    if !Path::new(CREDENTIALS_FILE).exists() {
        eprintln!(
            "Skipping test_real_api_call: {} not found",
            CREDENTIALS_FILE
        );
        return;
    }

    let tempo = tempo_rs::authorize_with_file(CREDENTIALS_FILE)
        .await
        .expect("Authentication failed");

    // Test next_day() as it's a simple call
    let next_day = tempo.next_day().await;
    assert!(
        next_day.is_ok(),
        "Failed to get next day: {:?}",
        next_day.err()
    );
    println!("Next day color: {:?}", next_day.unwrap());

    // Test calendars() with a safe range
    // The API requires a minimum period and rejects future dates.
    // Let's try a 1-week range in the past.
    let now = chrono::Utc::now();
    let end_date = now - chrono::Duration::days(1);
    let start_date = end_date - chrono::Duration::days(7);

    let calendars = tempo
        .calendars(Some(start_date), Some(end_date), None)
        .await;
    assert!(
        calendars.is_ok(),
        "Failed to get calendars: {:?}",
        calendars.err()
    );

    let calendars = calendars.unwrap();
    assert!(
        !calendars.tempo_like_calendars.is_empty(),
        "Calendars list should not be empty"
    );
}
