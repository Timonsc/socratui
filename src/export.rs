//! Anki export. Anki imports this text format through File > Import and reads
//! the deck and tags from the header lines, so no column mapping is needed.

use crate::db::Card;

/// Escapes a card side for Anki's HTML import. ``` fences become `<pre>`, so
/// code and diagrams keep their spacing.
fn field(text: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut in_code = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            in_code = !in_code;
            out.push(if in_code { "<pre>" } else { "</pre>" }.to_string());
            continue;
        }
        out.push(
            line.replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('\t', "    "),
        );
    }
    if in_code {
        out.push("</pre>".to_string());
    }
    out.join("<br>")
        .replace("<pre><br>", "<pre>")
        .replace("<br></pre>", "</pre>")
}

fn tag(text: &str) -> String {
    let cleaned: String = text
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    cleaned
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

pub fn anki_text(cards: &[Card]) -> String {
    let mut out = String::from(
        "#separator:tab\n#html:true\n#notetype:Basic\n#deck column:3\n#tags column:4\n",
    );
    for card in cards {
        // "::" nests decks in Anki, so a title must not contain it.
        let deck = format!("Socratui::{}", card.topic_title.replace("::", " "));
        let mut tags = vec!["socratui".to_string(), tag(&card.topic_title)];
        if let Some(subskill) = &card.subskill {
            tags.push(tag(subskill));
        }
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            field(&card.front),
            field(&card.back),
            deck.replace(['\t', '\n'], " "),
            tags.join(" ")
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::srs::Sched;

    #[test]
    fn writes_one_escaped_row_per_card() {
        let card = Card {
            id: 1,
            topic_title: "Rust: Lifetimes & Borrowing".into(),
            subskill: Some("Elision rules".into()),
            front: "What does Vec<T>\tmean?".into(),
            back: "line one\nline two".into(),
            sched: Sched::default(),
        };
        let text = anki_text(&[card]);
        let rows: Vec<_> = text.lines().collect();
        assert_eq!(rows.len(), 6);
        assert_eq!(rows[3], "#deck column:3");
        assert_eq!(
            rows[5],
            "What does Vec&lt;T&gt;    mean?\tline one<br>line two\tSocratui::Rust: Lifetimes & Borrowing\tsocratui rust-lifetimes-borrowing elision-rules"
        );
    }

    #[test]
    fn fenced_blocks_become_pre() {
        assert_eq!(
            field("Order?\n```text\n[A] --> [B]\n  x < y\n```\ndone"),
            "Order?<br><pre>[A] --&gt; [B]<br>  x &lt; y</pre><br>done"
        );
        assert_eq!(field("```\nopen"), "<pre>open</pre>");
    }
}
