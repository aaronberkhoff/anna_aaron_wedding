use crate::config::SmtpConfig;
use chrono::Local;
use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};
use shared::models::rsvp::RsvpRequest;

pub async fn send_rsvp_notification(
    smtp: &SmtpConfig,
    guest_name: &str,
    guest_email: Option<&str>,
    invite_code: Option<&str>,
    rsvp: &RsvpRequest,
) {
    let first = &rsvp.party[0];
    let attending_label = if first.attending_reception { "YES" } else { "NO" };

    let party_lines = rsvp
        .party
        .iter()
        .map(|g| {
            let reception = if g.attending_reception { "YES" } else { "NO" };
            let rehearsal = match g.attending_rehearsal {
                Some(true) => "YES",
                Some(false) => "NO",
                None => "—",
            };
            format!("  • {} | Reception: {} | Rehearsal: {}", g.name, reception, rehearsal)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let known = if rsvp.known_guests.is_empty() {
        "—".to_string()
    } else {
        rsvp.known_guests.join(", ")
    };

    let submitted_at = Local::now().format("%B %d, %Y at %I:%M %p").to_string();

    let body = format!(
        "New RSVP received!\n\
        \n\
        Name:         {guest_name}\n\
        Email:        {email}\n\
        Invite code:  {code}\n\
        \n\
        Party ({count}):\n\
        {party_lines}\n\
        \n\
        Seated near:  {known}\n\
        Message:      {message}\n\
        \n\
        Submitted: {submitted_at}",
        email = guest_email.unwrap_or("—"),
        code = invite_code.unwrap_or("—"),
        count = rsvp.party.len(),
        message = rsvp.message.as_deref().unwrap_or("—"),
    );

    let mut builder = Message::builder().from(smtp.from.parse().expect("valid from address"));
    for addr in &smtp.to {
        builder = builder.to(addr.parse().expect("valid to address"));
    }
    let notification = match builder
        .subject(format!("RSVP: {guest_name} — {attending_label}"))
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

    match mailer.send(notification).await {
        Ok(_) => tracing::info!("RSVP notification sent to {}", smtp.to.join(", ")),
        Err(e) => tracing::error!("failed to send RSVP email: {e}"),
    }

    // ── Confirmation email to the guest ────────────────────────────────────────
    let Some(guest_addr) = guest_email else {
        return;
    };

    let reception_str = if first.attending_reception { "Yes" } else { "No" };
    let rehearsal_str = match first.attending_rehearsal {
        Some(true) => "Yes",
        Some(false) => "No",
        None => "N/A",
    };
    let others = rsvp.party.len().saturating_sub(1);
    let party_line = if others > 0 {
        format!("Additional party members: {others}\n")
    } else {
        String::new()
    };
    let seated_line = if rsvp.known_guests.is_empty() {
        String::new()
    } else {
        format!("Seated near:  {}\n", rsvp.known_guests.join(", "))
    };
    let message_line = match rsvp.message.as_deref() {
        Some(m) if !m.is_empty() => format!("Your message: {m}\n"),
        _ => String::new(),
    };

    let guest_body = format!(
        "Hi {guest_name},\n\
        \n\
        We have received your RSVP for Anna & Aaron's wedding! Here is a summary:\n\
        \n\
        Reception (November 21, 2026):        {reception_str}\n\
        Rehearsal Dinner (November 19, 2026): {rehearsal_str}\n\
        {party_line}\
        {seated_line}\
        {message_line}\
        \n\
        If anything needs to be corrected, please reach out to us directly.\n\
        \n\
        We cannot wait to celebrate with you!\n\
        \n\
        With love,\n\
        Anna & Aaron",
    );

    let confirmation = match Message::builder()
        .from(smtp.from.parse().expect("valid from address"))
        .to(match guest_addr.parse() {
            Ok(a) => a,
            Err(e) => {
                tracing::error!("invalid guest email address {guest_addr}: {e}");
                return;
            }
        })
        .subject("Your RSVP for Anna & Aaron's Wedding — confirmed!")
        .header(ContentType::TEXT_PLAIN)
        .body(guest_body)
    {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("failed to build guest confirmation email: {e}");
            return;
        }
    };

    let creds = Credentials::new(smtp.username.clone(), smtp.password.clone());
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")
        .expect("valid relay")
        .credentials(creds)
        .build();

    match mailer.send(confirmation).await {
        Ok(_) => tracing::info!("RSVP confirmation sent to {guest_addr}"),
        Err(e) => tracing::error!("failed to send guest confirmation email: {e}"),
    }
}
