use crate::config::SmtpConfig;
use chrono::Local;
use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use shared::models::rsvp::RsvpRequest;

pub async fn send_rsvp_notification(smtp: &SmtpConfig, rsvp: &RsvpRequest) {
    let attending = if rsvp.attending { "YES" } else { "NO" };
    let plus_one = if rsvp.plus_one {
        format!(
            "\nPlus-one: Yes — {}",
            rsvp.plus_one_name.as_deref().unwrap_or("name not given")
        )
    } else {
        "\nPlus-one: No".to_string()
    };
    let submitted_at = Local::now().format("%B %d, %Y at %I:%M %p").to_string();

    let body = format!(
        "New RSVP received!\n\
        \n\
        Name:      {} {}\n\
        Email:     {}\n\
        Attending: {}{}\n\
        Song:      {}\n\
        Message:   {}\n\
        \n\
        Submitted: {}",
        rsvp.first_name,
        rsvp.last_name,
        rsvp.email,
        attending,
        plus_one,
        rsvp.song_request.as_deref().unwrap_or("—"),
        rsvp.message.as_deref().unwrap_or("—"),
        submitted_at,
    );

    let mut builder = Message::builder()
        .from(smtp.from.parse().expect("valid from address"));
    for addr in &smtp.to {
        builder = builder.to(addr.parse().expect("valid to address"));
    }
    let email = match builder.subject(format!(
            "RSVP: {} {} — {}",
            rsvp.first_name, rsvp.last_name, attending
        ))
        .header(ContentType::TEXT_PLAIN)
        .body(body)
    {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("failed to build RSVP email: {e}");
            return;
        }
    };

    let creds = Credentials::new(smtp.username.clone(), smtp.password.clone());
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")
        .expect("valid relay")
        .credentials(creds)
        .build();

    match mailer.send(email).await {
        Ok(_) => tracing::info!("RSVP notification sent to {}", smtp.to.join(", ")),
        Err(e) => tracing::error!("failed to send RSVP email: {e}"),
    }
}
