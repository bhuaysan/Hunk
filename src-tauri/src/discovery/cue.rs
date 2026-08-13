use std::collections::BTreeSet;

use crate::domain::{TrackKind, ValidationProblem, ValidationProblemKind};

use super::{ParsedDescriptor, ParsedReference, ParsedTrack};

pub(super) fn parse(contents: &str) -> ParsedDescriptor {
    let mut parsed = ParsedDescriptor::default();
    let mut current_reference: Option<String> = None;
    let mut track_numbers = BTreeSet::new();

    for (index, raw_line) in contents.lines().enumerate() {
        let line_number = index + 1;
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        let (command, rest) = command_and_rest(line);
        match command.to_ascii_uppercase().as_str() {
            "REM" => {}
            "FILE" => match parse_file(rest) {
                Some(reference) => {
                    parsed.references.push(ParsedReference {
                        raw: reference.clone(),
                        line: line_number,
                    });
                    current_reference = Some(reference);
                }
                None => parsed.problems.push(problem(
                    ValidationProblemKind::MalformedDescriptor,
                    Some(line_number),
                )),
            },
            "TRACK" => match parse_track(rest) {
                Some((number, track_type)) => {
                    let Some(source_reference) = current_reference.clone() else {
                        parsed.problems.push(problem(
                            ValidationProblemKind::MalformedDescriptor,
                            Some(line_number),
                        ));
                        continue;
                    };

                    if !track_numbers.insert(number) {
                        parsed.problems.push(problem(
                            ValidationProblemKind::DuplicateTrack,
                            Some(line_number),
                        ));
                        continue;
                    }

                    let kind = cue_track_kind(&track_type);
                    if kind == TrackKind::Unknown {
                        parsed.problems.push(problem(
                            ValidationProblemKind::MalformedDescriptor,
                            Some(line_number),
                        ));
                    }

                    parsed.tracks.push(ParsedTrack {
                        number,
                        kind,
                        source_reference,
                        start_lba: None,
                        sector_size: cue_sector_size(&track_type),
                    });
                }
                None => parsed.problems.push(problem(
                    ValidationProblemKind::MalformedDescriptor,
                    Some(line_number),
                )),
            },
            _ => {}
        }
    }

    if parsed.references.is_empty() || parsed.tracks.is_empty() {
        parsed
            .problems
            .push(problem(ValidationProblemKind::MalformedDescriptor, None));
    }

    parsed
}

fn command_and_rest(line: &str) -> (&str, &str) {
    match line.find(char::is_whitespace) {
        Some(index) => (&line[..index], line[index..].trim()),
        None => (line, ""),
    }
}

fn parse_file(rest: &str) -> Option<String> {
    if let Some(quoted) = rest.strip_prefix('"') {
        let closing_quote = quoted.find('"')?;
        let filename = &quoted[..closing_quote];
        let file_type = quoted[closing_quote + 1..].trim();
        if filename.is_empty() || file_type.split_whitespace().count() != 1 {
            return None;
        }
        return Some(filename.to_owned());
    }

    let mut fields = rest.split_whitespace();
    let filename = fields.next()?;
    fields.next()?;
    if filename.is_empty() || fields.next().is_some() {
        return None;
    }
    Some(filename.to_owned())
}

fn parse_track(rest: &str) -> Option<(u32, String)> {
    let mut fields = rest.split_whitespace();
    let number = fields.next()?.parse().ok()?;
    let track_type = fields.next()?.to_ascii_uppercase();
    if fields.next().is_some() {
        return None;
    }
    Some((number, track_type))
}

fn cue_track_kind(track_type: &str) -> TrackKind {
    if track_type == "AUDIO" {
        TrackKind::Audio
    } else if track_type.contains("SUB") {
        TrackKind::Subchannel
    } else if track_type.starts_with("MODE") || track_type.starts_with("CDI") {
        TrackKind::Data
    } else {
        TrackKind::Unknown
    }
}

fn cue_sector_size(track_type: &str) -> Option<u32> {
    if track_type == "AUDIO" {
        Some(2352)
    } else {
        track_type.rsplit_once('/')?.1.parse().ok()
    }
}

fn problem(kind: ValidationProblemKind, line: Option<usize>) -> ValidationProblem {
    ValidationProblem::new(kind, line, None)
}
