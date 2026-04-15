//! Quick smoke tool — runs `dispatch` on a representative set of
//! utterances and prints the resulting envelope JSON. Useful for
//! eyeballing the wire format the Android action handler will see.
//!
//!   cargo run --example shapes

fn main() {
    let inputs = [
        // Untimed reminder (default destination)
        "remind me to buy milk",
        // Timed — relative
        "remind me in 30 minutes to check the oven",
        "remind me in 2 hours to check the oven",
        // Timed — absolute, today
        "remind me to walk the dog at 5pm",
        "remind me to walk the dog at 5:30pm",
        "remind me to take pills at 9am",
        // Timed — absolute, tomorrow
        "remind me at 9am tomorrow to call the dentist",
        "remind me tomorrow at 3pm to pick up the parcel",
        // Date-only (no time)
        "remind me about laundry tomorrow",
        // Named-clock
        "remind me to eat at noon",
        "remind me at midnight to set my alarm",
        // Named-list (always untimed v0.1)
        "add milk to my shopping list",
        "put eggs on the shopping list",
        "add deadline review to my work projects list",
        // Falls through to "use input as title" — defensive default
        "eggs and bacon",
        // Empty
        "",
    ];

    for input in inputs {
        let envelope = ari_reminder_skill::dispatch(input);
        println!("{:<60}  {}", format!("{:?}", input), envelope);
    }
}
