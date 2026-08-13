use std::collections::BTreeSet;

use crate::domain::{TrackKind, ValidationProblem, ValidationProblemKind};

use super::{ParsedDescriptor, ParsedReference, ParsedTrack};

pub(super) fn parse(contents: &str) -> ParsedDescriptor {
    let mut parsed = ParsedDescriptor::default();
    let lines: Vec<(usize, &str)> = contents
        .lines()
        .enumerate()
        .filter_map(|(index, line)| {
            let line = line.trim();
            (!line.is_empty()).then_some((index + 1, line))
        })
        .collect();

    let Some((header_line, header)) = lines.first().copied() else {
        parsed
            .problems
            .push(problem(ValidationProblemKind::MalformedDescriptor, None));
        return parsed;
    };

    let Ok(expected_tracks) = header.parse::<usize>() else {
        parsed.problems.push(problem(
            ValidationProblemKind::MalformedDescriptor,
            Some(header_line),
        ));
        return parsed;
    };

    if expected_tracks == 0 {
        parsed.problems.push(problem(
            ValidationProblemKind::MalformedDescriptor,
            Some(header_line),
        ));
    }

    if lines.len().saturating_sub(1) != expected_tracks {
        parsed.problems.push(problem(
            ValidationProblemKind::TrackCountMismatch,
            Some(header_line),
        ));
    }

    let mut track_numbers = BTreeSet::new();
    for (line_number, line) in lines.into_iter().skip(1) {
        let Some(fields) = tokenize(line) else {
            parsed.problems.push(problem(
                ValidationProblemKind::MalformedDescriptor,
                Some(line_number),
            ));
            continue;
        };
        if fields.len() != 6 {
            parsed.problems.push(problem(
                ValidationProblemKind::MalformedDescriptor,
                Some(line_number),
            ));
            continue;
        }

        let values = (
            fields[0].parse::<u32>(),
            fields[1].parse::<u64>(),
            fields[2].parse::<u32>(),
            fields[3].parse::<u32>(),
            fields[5].parse::<u64>(),
        );
        let (Ok(number), Ok(start_lba), Ok(track_type), Ok(sector_size), Ok(_offset)) = values
        else {
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

        let kind = match track_type {
            0 => TrackKind::Audio,
            4 => TrackKind::Data,
            _ => {
                parsed.problems.push(problem(
                    ValidationProblemKind::MalformedDescriptor,
                    Some(line_number),
                ));
                TrackKind::Unknown
            }
        };
        let source_reference = fields[4].clone();
        parsed.references.push(ParsedReference {
            raw: source_reference.clone(),
            line: line_number,
        });
        parsed.tracks.push(ParsedTrack {
            number,
            kind,
            source_reference,
            start_lba: Some(start_lba),
            sector_size: Some(sector_size),
        });
    }

    if parsed.tracks.is_empty() {
        parsed
            .problems
            .push(problem(ValidationProblemKind::MalformedDescriptor, None));
    }

    parsed
}

fn tokenize(line: &str) -> Option<Vec<String>> {
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut quoted = false;
    let mut token_started = false;

    for character in line.chars() {
        match character {
            '"' => {
                quoted = !quoted;
                token_started = true;
            }
            character if character.is_whitespace() && !quoted => {
                if token_started {
                    tokens.push(std::mem::take(&mut token));
                    token_started = false;
                }
            }
            _ => {
                token.push(character);
                token_started = true;
            }
        }
    }

    if quoted {
        return None;
    }
    if token_started {
        tokens.push(token);
    }
    Some(tokens)
}

fn problem(kind: ValidationProblemKind, line: Option<usize>) -> ValidationProblem {
    ValidationProblem::new(kind, line, None)
}
