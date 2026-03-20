use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::{
    fs,
    io::{self, stdin},
    path::Path,
};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct TelegramExport {
    messages: Vec<ChatMessage>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ChatMessage {
    id: i64,
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    date: Option<String>,
    #[serde(default)]
    date_unixtime: Option<String>,
    #[serde(default)]
    edited: Option<String>,
    #[serde(default)]
    edited_unixtime: Option<String>,
    #[serde(default)]
    from: Option<String>,
    #[serde(default)]
    from_id: Option<String>,
    #[serde(default)]
    actor: Option<String>,
    #[serde(default)]
    actor_id: Option<String>,
    #[serde(default)]
    action: Option<String>,
    #[serde(default)]
    forwarded_from: Option<String>,
    #[serde(default)]
    saved_from: Option<String>,
    #[serde(default)]
    via_bot: Option<String>,
    #[serde(default)]
    reply_to_message_id: Option<i64>,
    #[serde(default)]
    text: MessageText,
    #[serde(default)]
    text_entities: Vec<TextEntity>,
    #[serde(default)]
    reactions: Vec<Reaction>,
    #[serde(default)]
    file: Option<String>,
    #[serde(default)]
    file_name: Option<String>,
    #[serde(default)]
    file_size: Option<u64>,
    #[serde(default)]
    thumbnail: Option<String>,
    #[serde(default)]
    thumbnail_file_size: Option<u64>,
    #[serde(default)]
    media_type: Option<String>,
    #[serde(default)]
    performer: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    mime_type: Option<String>,
    #[serde(default)]
    duration_seconds: Option<u32>,
    #[serde(default)]
    width: Option<u32>,
    #[serde(default)]
    height: Option<u32>,
    #[serde(default)]
    sticker_emoji: Option<String>,
    #[serde(default)]
    poll: Option<Value>,
    #[serde(default)]
    contact_information: Option<Value>,
    #[serde(default)]
    location_information: Option<Value>,
    #[serde(default)]
    game_title: Option<String>,
    #[serde(default)]
    game_description: Option<String>,
    #[serde(default)]
    invoice: Option<Value>,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Default)]
#[serde(untagged)]
enum MessageText {
    #[default]
    Empty,
    Plain(String),
    Rich(Vec<TextFragment>),
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TextFragment {
    Plain(String),
    Rich(TextEntity),
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct TextEntity {
    #[serde(rename = "type")]
    kind: String,
    text: String,
    #[serde(default)]
    href: Option<String>,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct Reaction {
    #[serde(rename = "type")]
    kind: String,
    count: u32,
    emoji: String,
    #[serde(default)]
    recent: Vec<Value>,
    #[serde(flatten)]
    extra: HashMap<String, Value>,
}

struct PersonReactionStats {
    name: String,
    total: u64,
    reactions: Vec<(String, u64)>,
}

fn message_text_len(text: &MessageText) -> usize {
    match text {
        MessageText::Empty => 0,
        MessageText::Plain(text) => text.chars().count(),
        MessageText::Rich(fragments) => fragments
            .iter()
            .map(|fragment| match fragment {
                TextFragment::Plain(text) => text.chars().count(),
                TextFragment::Rich(entity) => entity.text.chars().count(),
            })
            .sum(),
    }
}

fn author_name(message: &ChatMessage) -> String {
    message
        .from
        .clone()
        .or_else(|| message.actor.clone())
        .unwrap_or_else(|| "Unknown".to_string())
}

fn top_reactions(messages: &[ChatMessage]) -> Vec<(String, u64)> {
    let mut reaction_totals = HashMap::new();

    for message in messages {
        for reaction in &message.reactions {
            *reaction_totals
                .entry(reaction.emoji.clone())
                .or_insert(0_u64) += reaction.count as u64;
        }
    }

    let mut reaction_totals: Vec<_> = reaction_totals.into_iter().collect();
    reaction_totals.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    reaction_totals
}

fn top_people_by_reactions(messages: &[ChatMessage]) -> Vec<PersonReactionStats> {
    let mut totals: HashMap<String, (u64, HashMap<String, u64>)> = HashMap::new();

    for message in messages {
        if !message.reactions.is_empty() {
            let entry = totals
                .entry(author_name(message))
                .or_insert_with(|| (0_u64, HashMap::new()));

            for reaction in &message.reactions {
                let count = reaction.count as u64;
                entry.0 += count;
                *entry.1.entry(reaction.emoji.clone()).or_insert(0_u64) += count;
            }
        }
    }

    let mut totals: Vec<_> = totals
        .into_iter()
        .map(|(name, (total, reactions))| {
            let mut reactions: Vec<_> = reactions.into_iter().collect();
            reactions.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

            PersonReactionStats {
                name,
                total,
                reactions,
            }
        })
        .collect();
    totals.sort_by(|a, b| b.total.cmp(&a.total).then_with(|| a.name.cmp(&b.name)));
    totals
}

fn top_people_by_symbols(messages: &[ChatMessage]) -> Vec<(String, usize)> {
    let mut totals = HashMap::new();

    for message in messages {
        let text_len = message_text_len(&message.text);
        if text_len > 0 {
            *totals.entry(author_name(message)).or_insert(0_usize) += text_len;
        }
    }

    let mut totals: Vec<_> = totals.into_iter().collect();
    totals.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    totals
}

fn top_people_by_messages(messages: &[ChatMessage]) -> Vec<(String, u64)> {
    let mut totals = HashMap::new();

    for message in messages {
        if message.kind == "message" {
            *totals.entry(author_name(message)).or_insert(0_u64) += 1;
        }
    }

    let mut totals: Vec<_> = totals.into_iter().collect();
    totals.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    totals
}

fn write_stats_file<T: std::fmt::Display>(
    file_path: &str,
    header: &str,
    rows: &[(String, T)],
) -> io::Result<()> {
    let mut output = String::new();
    output.push_str(header);
    output.push('\n');

    for (index, (name, value)) in rows.iter().enumerate() {
        output.push_str(&format!("{}. {}: {}\n", index + 1, name, value));
    }

    fs::write(file_path, output)
}

fn write_people_reactions_file(
    file_path: &str,
    header: &str,
    rows: &[PersonReactionStats],
) -> io::Result<()> {
    let mut output = String::new();
    output.push_str(header);
    output.push('\n');

    for (index, row) in rows.iter().enumerate() {
        let reactions = row
            .reactions
            .iter()
            .map(|(emoji, count)| format!("{}: {}", emoji, count))
            .collect::<Vec<_>>()
            .join(", ");

        output.push_str(&format!(
            "{}. {}: {} [{}]\n",
            index + 1,
            row.name,
            row.total,
            reactions
        ));
    }

    fs::write(file_path, output)
}

fn write_all_stats(messages: &[ChatMessage], input_path: &str) -> io::Result<()> {
    let base_dir = Path::new(input_path)
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));

    write_stats_file(
        &base_dir.join("top_reactions.txt").to_string_lossy(),
        "Top reactions",
        &top_reactions(messages),
    )?;
    write_people_reactions_file(
        &base_dir
            .join("top_people_by_reactions.txt")
            .to_string_lossy(),
        "Top people by reactions",
        &top_people_by_reactions(messages),
    )?;
    write_stats_file(
        &base_dir.join("top_people_by_symbols.txt").to_string_lossy(),
        "Top people by symbols",
        &top_people_by_symbols(messages),
    )?;
    write_stats_file(
        &base_dir
            .join("top_people_by_messages.txt")
            .to_string_lossy(),
        "Top people by messages",
        &top_people_by_messages(messages),
    )?;

    Ok(())
}

fn read_json_file(file_path: &str) -> io::Result<Vec<ChatMessage>> {
    let file_content = fs::read_to_string(file_path)?;
    let export: TelegramExport = serde_json::from_str(&file_content).map_err(io::Error::other)?;
    Ok(export.messages)
}

fn main() {
    let mut input_string = String::new();
    let mut path = String::new();

    for file in fs::read_dir(".").unwrap() {
        let file = file.unwrap();
        let file_name = file.file_name().into_string().unwrap();

        if file_name.ends_with(".json") {
            println!("Found JSON file: {}", file_name);
            path = file_name;
            break;
        }
    }

    let path = if path.is_empty() {
        match stdin().read_line(&mut input_string) {
            Ok(_) => input_string.trim().to_string(),
            Err(e) => {
                eprintln!("Failed to read input: {}", e);
                return;
            }
        }
    } else {
        path
    };

    match read_json_file(&path) {
        Ok(chat_messages) => {
            println!("Successfully read {} chat messages.", chat_messages.len());
            match write_all_stats(&chat_messages, &path) {
                Ok(()) => println!("Statistics saved to separate files."),
                Err(e) => eprintln!("Failed to save statistics: {}", e),
            }
        }
        Err(e) => {
            eprintln!("Failed to read file '{}': {}", path, e);
        }
    }
}
