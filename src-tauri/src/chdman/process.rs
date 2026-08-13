use std::fmt;
use std::io;
use std::path::Path;
use std::process::Command;

pub const APPROVED_MAME_TAG: &str = "mame0289";
pub const APPROVED_MAME_COMMIT: &str = "f34f02505e32c1993c6a782b6814232cbfc74e36";
pub const APPROVED_CHDMAN_VERSION: &str = "0.289";

const REQUIRED_COMMANDS: [&str; 6] = [
    "createcd",
    "createdvd",
    "extractcd",
    "extractdvd",
    "verify",
    "info",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChdmanCapabilities {
    pub version: String,
    pub commands: Vec<String>,
}

#[derive(Debug)]
pub enum CapabilityError {
    Launch(io::Error),
    MissingVersion,
    UnsupportedVersion {
        expected: &'static str,
        actual: String,
    },
    MissingCommands(Vec<&'static str>),
}

impl fmt::Display for CapabilityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Launch(error) => write!(formatter, "could not launch bundled chdman: {error}"),
            Self::MissingVersion => write!(formatter, "chdman did not report a version"),
            Self::UnsupportedVersion { expected, actual } => {
                write!(formatter, "expected chdman {expected}, found {actual}")
            }
            Self::MissingCommands(commands) => {
                write!(
                    formatter,
                    "chdman is missing required commands: {}",
                    commands.join(", ")
                )
            }
        }
    }
}

impl std::error::Error for CapabilityError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Launch(error) => Some(error),
            _ => None,
        }
    }
}

pub fn check_capabilities(program: &Path) -> Result<ChdmanCapabilities, CapabilityError> {
    let output = Command::new(program)
        .output()
        .map_err(CapabilityError::Launch)?;
    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    parse_capabilities(&text)
}

fn parse_capabilities(output: &str) -> Result<ChdmanCapabilities, CapabilityError> {
    let version = output
        .lines()
        .find_map(|line| {
            line.split_once("CHD) manager ")
                .map(|(_, suffix)| suffix.split_whitespace().next().unwrap_or_default())
        })
        .filter(|value| !value.is_empty())
        .ok_or(CapabilityError::MissingVersion)?;
    if version != APPROVED_CHDMAN_VERSION {
        return Err(CapabilityError::UnsupportedVersion {
            expected: APPROVED_CHDMAN_VERSION,
            actual: version.to_owned(),
        });
    }

    let commands = REQUIRED_COMMANDS
        .into_iter()
        .filter(|command| contains_command(output, command))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let missing = REQUIRED_COMMANDS
        .into_iter()
        .filter(|required| !commands.iter().any(|command| command == required))
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(CapabilityError::MissingCommands(missing));
    }

    Ok(ChdmanCapabilities {
        version: version.to_owned(),
        commands,
    })
}

fn contains_command(output: &str, command: &str) -> bool {
    output.lines().any(|line| {
        line.split_whitespace()
            .any(|word| word.trim_end_matches(':') == command)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const COMPLETE_USAGE: &str = "\
chdman - MAME Compressed Hunks of Data (CHD) manager 0.289
Usage:
  info: displays information about a CHD
  verify: verifies a CHD's integrity
  createcd: create a CD CHD
  createdvd: create a DVD CHD
  extractcd: extract a CD CHD
  extractdvd: extract a DVD CHD
";

    #[test]
    fn accepts_only_the_pinned_version_and_required_commands() {
        let capabilities = parse_capabilities(COMPLETE_USAGE).unwrap();

        assert_eq!(capabilities.version, APPROVED_CHDMAN_VERSION);
        assert_eq!(capabilities.commands.len(), 6);
    }

    #[test]
    fn rejects_wrong_version() {
        let output = COMPLETE_USAGE.replace("0.289", "0.288");

        assert!(matches!(
            parse_capabilities(&output),
            Err(CapabilityError::UnsupportedVersion { .. })
        ));
    }

    #[test]
    fn rejects_missing_operation() {
        let output = COMPLETE_USAGE.replace("  extractdvd: extract a DVD CHD\n", "");

        assert!(matches!(
            parse_capabilities(&output),
            Err(CapabilityError::MissingCommands(commands)) if commands == vec!["extractdvd"]
        ));
    }

    #[test]
    fn approved_pin_matches_the_build_recipe() {
        let recipe = include_str!("../../../scripts/build-chdman.sh");
        let pin = include_str!("../../../scripts/mame-pin.sh");

        assert!(recipe.contains("mame-pin.sh"));
        assert!(pin.contains(APPROVED_MAME_TAG));
        assert!(pin.contains(APPROVED_MAME_COMMIT));
    }
}
