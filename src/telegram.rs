/// Create a bot with BotFather
/// Send a message to your bot
/// Then call: `curl https://api.telegram.org/bot<your-bot-token>/getUpdates` and copy the chat_id
pub fn send_message(message: String, token: &String, chat_id: &u64) -> anyhow::Result<()> {
    let request = ureq::post(&format!(
        "https://api.telegram.org/bot{token}/sendMessage",
        token = &token
    ));

    println!("--- Sending telegram message: {:?}", message);

    let payload = ureq::json!({
        "parse_mode": "MarkdownV2",
        "chat_id": chat_id,
        "text": message
    });

    match request.send_json(payload) {
        Ok(response) => println!("--- Message sent: {}", response.status_text()),
        Err(error) => println!("--- Message failed: {}", error),
    }

    Ok(())
}
