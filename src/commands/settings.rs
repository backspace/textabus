use crate::models::Number;
use sqlx::PgPool;

pub async fn handle_settings_clock_request(
    db: &PgPool,
    number: &Option<Number>,
) -> Result<String, Box<dyn std::error::Error>> {
    let response_text = if let Some(number) = number {
        if number.twelve_hour {
            sqlx::query(
                "UPDATE numbers
                SET twelve_hour = false
                WHERE number = $1",
            )
            .bind(&number.number)
            .execute(db)
            .await?;
            "times will now be in 24h format".to_string()
        } else {
            sqlx::query(
                "UPDATE numbers
                SET twelve_hour = true
                WHERE number = $1",
            )
            .bind(&number.number)
            .execute(db)
            .await?;
            "times will now be in 12h format".to_string()
        }
    } else {
        "Cannot change settings with this interface".to_string()
    };

    Ok(response_text)
}
