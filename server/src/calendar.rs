use std::fmt::Write;

use time::{Date, OffsetDateTime, UtcOffset};

use crate::gatherings::{Gathering, Revision};

/// The namespace `81aa7074-4fe3-41aa-8af7-5d34430bdd9d` of the UIDs of gatherings. Changing it duplicates every entry in the calendars of members.
const NAMESPACE: [u8; 16] = [
    0x81, 0xaa, 0x70, 0x74, 0x4f, 0xe3, 0x41, 0xaa, 0x8a, 0xf7, 0x5d, 0x34, 0x43, 0x0b, 0xdd, 0x9d,
];
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
    let mut input = NAMESPACE.to_vec();
    input.extend_from_slice(name.as_bytes());
    let mut bytes = sha1(&input);
    bytes[6] = (bytes[6] & 0x0f) | 0x50;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let hex = hex(&bytes[..16]);
    format!(
        "{}-{}-{}-{}-{}",
        &hex[..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..]
    )
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut text, byte| {
        let _ = write!(text, "{byte:02x}");
        text
    })
}

/// SHA-1 as FIPS 180-4 describes it. It only derives names here, so its weakness against collisions does not matter.
fn sha1(message: &[u8]) -> [u8; 20] {
    let mut state: [u32; 5] = [
        0x6745_2301,
        0xefcd_ab89,
        0x98ba_dcfe,
        0x1032_5476,
        0xc3d2_e1f0,
    ];
    let bit_length = u64::try_from(message.len()).expect("a message fits in u64") * 8;
    let mut padded = message.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_length.to_be_bytes());
    for block in padded.chunks_exact(64) {
        let mut words = [0u32; 80];
        for (word, bytes) in words.iter_mut().zip(block.chunks_exact(4)) {
            *word = u32::from_be_bytes(bytes.try_into().expect("chunks of four bytes"));
        }
        for index in 16..80 {
            words[index] =
                (words[index - 3] ^ words[index - 8] ^ words[index - 14] ^ words[index - 16])
                    .rotate_left(1);
        }
        let mut working = state;
        for (round, word) in words.iter().enumerate() {
            let [first, second, third, fourth, fifth] = working;
            let (mixed, constant) = match round {
                0..20 => ((second & third) | (!second & fourth), 0x5a82_7999),
                20..40 => (second ^ third ^ fourth, 0x6ed9_eba1),
                40..60 => (
                    (second & third) | (second & fourth) | (third & fourth),
                    0x8f1b_bcdc,
                ),
                _ => (second ^ third ^ fourth, 0xca62_c1d6),
            };
            let next = first
                .rotate_left(5)
                .wrapping_add(mixed)
                .wrapping_add(fifth)
                .wrapping_add(constant)
                .wrapping_add(*word);
            working = [next, first, second.rotate_left(30), third, fourth];
        }
        for (value, added) in state.iter_mut().zip(working) {
            *value = value.wrapping_add(added);
        }
    }
    let mut digest = [0u8; 20];
    for (bytes, value) in digest.chunks_exact_mut(4).zip(state) {
        bytes.copy_from_slice(&value.to_be_bytes());
    }
    digest
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
            cancelled: false,
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
    fn sha1_of_fips_vectors() {
        assert_eq!(
            hex(&sha1(b"abc")),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );
        assert_eq!(hex(&sha1(b"")), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
        assert_eq!(
            hex(&sha1(
                b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
            )),
            "84983e441c3bd26ebaae4aa1f95129e5e54670f1"
        );
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
