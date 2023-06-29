use std::time::Instant;
use crate::puzzle::{Mode, Puzzle};
use dotenv::dotenv;

mod puzzle;
mod telegram;
mod speed_checker;

fn telegram_notify(solution: &String) -> anyhow::Result<()> {
    let token = std::env::var("TELEGRAM_BOT_TOKEN")?;
    let chat_id = std::env::var("TELEGRAM_BOT_CHAT_ID")?.parse()?;

    telegram::send_message(
        format!("Found solution: {:?}", solution), &token, &chat_id
    )
}

#[inline]
fn main() -> anyhow::Result<()> {
    dotenv()?;

    let number = std::env::var("PUZZLE_NUMBER")?.parse()?;
    let mut puzzle = Puzzle::number(number);
    let start = Instant::now();

    match puzzle.start(Mode::Linear) {
        Err(error) => println!("{:?}", error),
        Ok(solution) => {
            println!("Found solution {:?} in: {:?}", solution, start.elapsed());

            if let Ok(enabled) = std::env::var("TELEGRAM_BOT_ENABLED") {
                if enabled.parse()? {
                    telegram_notify(&solution)?;
                }
            }
        }
    }

    Ok(())
}
