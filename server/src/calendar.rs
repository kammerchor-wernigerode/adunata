use time::{Date, OffsetDateTime, UtcOffset};
use uuid::Uuid;

use crate::gatherings::{Gathering, Revision};

/// The namespace of the UIDs of gatherings. Changing it duplicates every entry in the calendars of members.
const NAMESPACE: Uuid = Uuid::from_u128(0x81aa_7074_4fe3_41aa_8af7_5d34_430b_dd9d);
const MAX_LINE_OCTETS: usize = 75;

/// The calendar file of a gathering as RFC 5545 describes it, with whole days and a UID that stays the same across every download.
pub fn file(gathering: &Gathering, revision: &Revision) -> String {
    let changed = timestamp(revision.changed_at);
    let mut lines = vec![
        "BEGIN:VCALENDAR".to_owned(),
        "VERSION:2.0".to_owned(),
        "PRODID:-//Adunata//Adunata//EN".to_owned(),
        "CALSCALE:GREGORIAN".to_owned(),
        "BEGIN:VEVENT".to_owned(),
        format!("UID:{}", uid(gathering.id, revision.created_at)),
        format!("DTSTAMP:{changed}"),
        format!("DTSTART;VALUE=DATE:{}", day(gathering.first_day)),
        format!(
            "DTEND;VALUE=DATE:{}",
            day(gathering
                .last_day
                .next_day()
                .expect("validation keeps a day after the last day"))
        ),
        format!("SUMMARY:{}", escaped(&gathering.title)),
        format!("LOCATION:{}", escaped(&gathering.location)),
    ];
    if let Some(description) = &gathering.description {
        lines.push(format!("DESCRIPTION:{}", escaped(description)));
    }
    let status = if gathering.cancelled {
        "CANCELLED"
    } else {
        "CONFIRMED"
    };
    lines.extend([
        format!("STATUS:{status}"),
        "TRANSP:OPAQUE".to_owned(),
        format!("CREATED:{}", timestamp(revision.created_at)),
        format!("LAST-MODIFIED:{changed}"),
        format!("SEQUENCE:{}", revision.version),
        "END:VEVENT".to_owned(),
        "END:VCALENDAR".to_owned(),
    ]);
    lines.iter().map(|line| folded(line)).collect()
}

/// The UUID of version 5 for the name `gathering:<id>:<created>`, with the creation time in microseconds since the Unix epoch.
fn uid(id: i64, created_at: OffsetDateTime) -> String {
    let microseconds = created_at.unix_timestamp_nanos() / 1000;
    uuid_v5(&format!("gathering:{id}:{microseconds}"))
}

fn uuid_v5(name: &str) -> String {
    Uuid::new_v5(&NAMESPACE, name.as_bytes()).to_string()
}

fn day(date: Date) -> String {
    format!(
        "{:04}{:02}{:02}",
        date.year(),
        u8::from(date.month()),
        date.day()
    )
}

fn timestamp(instant: OffsetDateTime) -> String {
    let utc = instant.to_offset(UtcOffset::UTC);
    format!(
        "{}T{:02}{:02}{:02}Z",
        day(utc.date()),
        utc.hour(),
        utc.minute(),
        utc.second()
    )
}

/// Text as RFC 5545 section 3.3.11 escapes it.
fn escaped(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace("\r\n", "\\n")
        .replace(['\r', '\n'], "\\n")
}

/// The line with CRLF, folded after at most 75 octets without splitting a character.
fn folded(line: &str) -> String {
    let mut result = String::with_capacity(line.len() + 2);
    let mut width = 0;
    for character in line.chars() {
        if width + character.len_utf8() > MAX_LINE_OCTETS {
            result.push_str("\r\n ");
            width = 1;
        }
        result.push(character);
        width += character.len_utf8();
    }
    result.push_str("\r\n");
    result
}

#[cfg(test)]
mod tests {
    use time::macros::{date, datetime};

    use super::*;

    fn rehearsal_weekend() -> Gathering {
        Gathering {
            id: 7,
            title: "Rehearsal weekend".to_owned(),
            location: "Music academy, Springfield".to_owned(),
            description: Some("Please bring your scores.\nWe start with dinner.".to_owned()),
            first_day: date!(2027 - 03 - 12),
            last_day: date!(2027 - 03 - 14),
            response_deadline_days: None,
            last_day_to_answer: None,
            deadline_passed: false,
            cancelled: false,
            begun: false,
            ended: false,
        }
    }

    fn changed_twice() -> Revision {
        Revision {
            created_at: datetime!(2027-01-10 09:00:00.123456 UTC),
            changed_at: datetime!(2027-02-20 19:00:00 +01:00),
            version: 2,
        }
    }

    fn lines(gathering: &Gathering) -> Vec<String> {
        file(gathering, &changed_twice())
            .split("\r\n")
            .map(str::to_owned)
            .collect()
    }

    #[test]
    fn stable_uid() {
        assert_eq!(
            uuid_v5("gathering:7:1799571600123456"),
            "32d095c3-d573-5219-bd27-a2702c08aa6a"
        );
        assert_eq!(
            uid(7, datetime!(2027-01-10 09:00:00.123456 UTC)),
            uuid_v5(&format!(
                "gathering:7:{}",
                datetime!(2027-01-10 09:00:00 UTC).unix_timestamp() * 1_000_000 + 123_456
            ))
        );
    }

    #[test]
    fn gathering_over_three_days() {
        let text = file(&rehearsal_weekend(), &changed_twice());
        let expected = [
            "BEGIN:VCALENDAR",
            "VERSION:2.0",
            "PRODID:-//Adunata//Adunata//EN",
            "CALSCALE:GREGORIAN",
            "BEGIN:VEVENT",
            &format!("UID:{}", uid(7, datetime!(2027-01-10 09:00:00.123456 UTC))),
            "DTSTAMP:20270220T180000Z",
            "DTSTART;VALUE=DATE:20270312",
            "DTEND;VALUE=DATE:20270315",
            "SUMMARY:Rehearsal weekend",
            "LOCATION:Music academy\\, Springfield",
            "DESCRIPTION:Please bring your scores.\\nWe start with dinner.",
            "STATUS:CONFIRMED",
            "TRANSP:OPAQUE",
            "CREATED:20270110T090000Z",
            "LAST-MODIFIED:20270220T180000Z",
            "SEQUENCE:2",
            "END:VEVENT",
            "END:VCALENDAR",
            "",
        ]
        .join("\r\n");
        assert_eq!(text, expected);
        for absent in [
            "METHOD",
            "URL",
            "ATTENDEE",
            "ORGANIZER",
            "VALARM",
            "VTIMEZONE",
        ] {
            assert!(!text.contains(absent), "{absent}");
        }
    }

    #[test]
    fn gathering_of_one_day() {
        let mut gathering = rehearsal_weekend();
        gathering.last_day = gathering.first_day;
        let lines = lines(&gathering);
        assert!(lines.contains(&"DTSTART;VALUE=DATE:20270312".to_owned()));
        assert!(lines.contains(&"DTEND;VALUE=DATE:20270313".to_owned()));
    }

    #[test]
    fn gathering_without_description() {
        let mut gathering = rehearsal_weekend();
        gathering.description = None;
        assert!(
            !lines(&gathering)
                .iter()
                .any(|line| line.starts_with("DESCRIPTION"))
        );
    }

    #[test]
    fn cancelled_gathering() {
        let mut gathering = rehearsal_weekend();
        gathering.cancelled = true;
        assert!(lines(&gathering).contains(&"STATUS:CANCELLED".to_owned()));
    }

    #[test]
    fn escapes_text() {
        assert_eq!(escaped("a\\b;c,d\r\ne\nf\rg"), "a\\\\b\\;c\\,d\\ne\\nf\\ng");
    }

    #[test]
    fn long_title_with_umlauts() {
        let mut gathering = rehearsal_weekend();
        gathering.title = "Übungstag für Bässe, Größe; "
            .repeat(8)
            .chars()
            .take(200)
            .collect();
        let text = file(&gathering, &changed_twice());
        assert!(text.ends_with("\r\n"));
        for line in text.trim_end_matches("\r\n").split("\r\n") {
            assert!(line.len() <= MAX_LINE_OCTETS, "{line}");
        }
        let unfolded = text.replace("\r\n ", "");
        assert!(unfolded.contains(&format!("\r\nSUMMARY:{}\r\n", escaped(&gathering.title))));
        assert!(
            text.split("\r\n")
                .filter(|line| line.starts_with(' '))
                .count()
                >= 3
        );
    }
}
