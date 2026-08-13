use std::collections::HashSet;
use std::ffi::OsString;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::domain::Operation;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CompressionCodec {
    None,
    Lzma,
    Zlib,
    Zstd,
    Huffman,
    Flac,
    CdLzma,
    CdZlib,
    CdZstd,
    CdFlac,
}

impl CompressionCodec {
    fn argument(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Lzma => "lzma",
            Self::Zlib => "zlib",
            Self::Zstd => "zstd",
            Self::Huffman => "huff",
            Self::Flac => "flac",
            Self::CdLzma => "cdlz",
            Self::CdZlib => "cdzl",
            Self::CdZstd => "cdzs",
            Self::CdFlac => "cdfl",
        }
    }

    fn supports(self, operation: Operation) -> bool {
        match operation {
            Operation::CreateCd => matches!(
                self,
                Self::None | Self::CdLzma | Self::CdZlib | Self::CdZstd | Self::CdFlac
            ),
            Operation::CreateDvd => matches!(
                self,
                Self::None | Self::Lzma | Self::Zlib | Self::Zstd | Self::Huffman | Self::Flac
            ),
            _ => false,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CreateOptions {
    pub hunk_size: Option<u32>,
    pub compression: Option<Vec<CompressionCodec>>,
    pub processors: Option<u16>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChdmanRequest {
    CreateCd {
        input: PathBuf,
        output: PathBuf,
        options: CreateOptions,
    },
    CreateDvd {
        input: PathBuf,
        output: PathBuf,
        options: CreateOptions,
    },
    ExtractCd {
        input: PathBuf,
        output: PathBuf,
        output_bin: PathBuf,
        split_bin: bool,
    },
    ExtractDvd {
        input: PathBuf,
        output: PathBuf,
    },
    Verify {
        input: PathBuf,
    },
    Info {
        input: PathBuf,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChdmanCommand {
    pub program: PathBuf,
    pub args: Vec<OsString>,
    pub operation: Operation,
}

impl ChdmanCommand {
    pub fn process(&self) -> Command {
        let mut command = Command::new(&self.program);
        command.args(&self.args);
        command
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommandBuildError {
    InputEqualsOutput(PathBuf),
    OutputPathsConflict(PathBuf),
    OutputExists(PathBuf),
    InvalidHunkSize {
        operation: Operation,
        size: u32,
    },
    InvalidProcessorCount,
    TooManyCodecs,
    DuplicateCodec(CompressionCodec),
    UnsupportedCodec {
        operation: Operation,
        codec: CompressionCodec,
    },
    NoneWithOtherCodecs,
    MissingTrackPlaceholder(PathBuf),
}

impl fmt::Display for CommandBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputEqualsOutput(path) => {
                write!(
                    formatter,
                    "input and output paths are identical: {}",
                    path.display()
                )
            }
            Self::OutputPathsConflict(path) => {
                write!(
                    formatter,
                    "multiple outputs use the same path: {}",
                    path.display()
                )
            }
            Self::OutputExists(path) => {
                write!(formatter, "output already exists: {}", path.display())
            }
            Self::InvalidHunkSize { operation, size } => {
                write!(formatter, "invalid hunk size {size} for {operation:?}")
            }
            Self::InvalidProcessorCount => {
                write!(formatter, "processor count must be at least one")
            }
            Self::TooManyCodecs => write!(formatter, "at most four compression codecs are allowed"),
            Self::DuplicateCodec(codec) => {
                write!(formatter, "duplicate compression codec: {codec:?}")
            }
            Self::UnsupportedCodec { operation, codec } => {
                write!(
                    formatter,
                    "compression codec {codec:?} is not valid for {operation:?}"
                )
            }
            Self::NoneWithOtherCodecs => {
                write!(
                    formatter,
                    "the none codec cannot be combined with other codecs"
                )
            }
            Self::MissingTrackPlaceholder(path) => write!(
                formatter,
                "split BIN output requires a %t track placeholder: {}",
                path.display()
            ),
        }
    }
}

impl std::error::Error for CommandBuildError {}

pub fn build_command(
    program: impl Into<PathBuf>,
    request: &ChdmanRequest,
) -> Result<ChdmanCommand, CommandBuildError> {
    let program = program.into();

    match request {
        ChdmanRequest::CreateCd {
            input,
            output,
            options,
        } => build_create(program, Operation::CreateCd, input, output, options),
        ChdmanRequest::CreateDvd {
            input,
            output,
            options,
        } => build_create(program, Operation::CreateDvd, input, output, options),
        ChdmanRequest::ExtractCd {
            input,
            output,
            output_bin,
            split_bin,
        } => {
            validate_output(input, output)?;
            if input == output_bin {
                return Err(CommandBuildError::InputEqualsOutput(output_bin.clone()));
            }
            if output == output_bin {
                return Err(CommandBuildError::OutputPathsConflict(output_bin.clone()));
            }
            if !split_bin && output_bin.exists() {
                return Err(CommandBuildError::OutputExists(output_bin.clone()));
            }
            if *split_bin && !contains_track_placeholder(output_bin) {
                return Err(CommandBuildError::MissingTrackPlaceholder(
                    output_bin.clone(),
                ));
            }

            let mut args = path_args("extractcd", input, output);
            args.push("-ob".into());
            args.push(output_bin.as_os_str().to_owned());
            if *split_bin {
                args.push("-sb".into());
            }
            Ok(ChdmanCommand {
                program,
                args,
                operation: Operation::ExtractCd,
            })
        }
        ChdmanRequest::ExtractDvd { input, output } => {
            validate_output(input, output)?;
            Ok(ChdmanCommand {
                program,
                args: path_args("extractdvd", input, output),
                operation: Operation::ExtractDvd,
            })
        }
        ChdmanRequest::Verify { input } => Ok(ChdmanCommand {
            program,
            args: vec!["verify".into(), "-i".into(), input.as_os_str().to_owned()],
            operation: Operation::Verify,
        }),
        ChdmanRequest::Info { input } => Ok(ChdmanCommand {
            program,
            args: vec![
                "info".into(),
                "-i".into(),
                input.as_os_str().to_owned(),
                "-v".into(),
            ],
            operation: Operation::Info,
        }),
    }
}

fn build_create(
    program: PathBuf,
    operation: Operation,
    input: &Path,
    output: &Path,
    options: &CreateOptions,
) -> Result<ChdmanCommand, CommandBuildError> {
    validate_output(input, output)?;
    validate_create_options(operation, options)?;

    let subcommand = match operation {
        Operation::CreateCd => "createcd",
        Operation::CreateDvd => "createdvd",
        _ => unreachable!("build_create only accepts create operations"),
    };
    let mut args = path_args(subcommand, input, output);
    if let Some(size) = options.hunk_size {
        args.push("-hs".into());
        args.push(size.to_string().into());
    }
    if let Some(codecs) = &options.compression {
        args.push("-c".into());
        args.push(
            codecs
                .iter()
                .map(|codec| codec.argument())
                .collect::<Vec<_>>()
                .join(",")
                .into(),
        );
    }
    if let Some(processors) = options.processors {
        args.push("-np".into());
        args.push(processors.to_string().into());
    }

    Ok(ChdmanCommand {
        program,
        args,
        operation,
    })
}

fn path_args(subcommand: &str, input: &Path, output: &Path) -> Vec<OsString> {
    vec![
        subcommand.into(),
        "-i".into(),
        input.as_os_str().to_owned(),
        "-o".into(),
        output.as_os_str().to_owned(),
    ]
}

fn validate_output(input: &Path, output: &Path) -> Result<(), CommandBuildError> {
    if input == output {
        return Err(CommandBuildError::InputEqualsOutput(output.to_owned()));
    }
    if output.exists() {
        return Err(CommandBuildError::OutputExists(output.to_owned()));
    }
    Ok(())
}

fn contains_track_placeholder(path: &Path) -> bool {
    let bytes = path.as_os_str().to_string_lossy();
    let bytes = bytes.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'%' {
            index += 1;
            continue;
        }
        let start = index;
        while index < bytes.len() && bytes[index] == b'%' {
            index += 1;
        }
        if (index - start) % 2 == 0 {
            continue;
        }
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        if bytes.get(index) == Some(&b't') {
            return true;
        }
    }
    false
}

fn validate_create_options(
    operation: Operation,
    options: &CreateOptions,
) -> Result<(), CommandBuildError> {
    if let Some(processors) = options.processors
        && processors == 0
    {
        return Err(CommandBuildError::InvalidProcessorCount);
    }

    if let Some(size) = options.hunk_size {
        let unit_size = match operation {
            Operation::CreateCd => 2_448,
            Operation::CreateDvd => 2_048,
            _ => unreachable!("only create operations have hunk sizes"),
        };
        if size == 0 || size > 1024 * 1024 || size % unit_size != 0 {
            return Err(CommandBuildError::InvalidHunkSize { operation, size });
        }
    }

    let Some(codecs) = &options.compression else {
        return Ok(());
    };
    if codecs.is_empty() || codecs.len() > 4 {
        return Err(CommandBuildError::TooManyCodecs);
    }
    if codecs.len() > 1 && codecs.contains(&CompressionCodec::None) {
        return Err(CommandBuildError::NoneWithOtherCodecs);
    }

    let mut seen = HashSet::new();
    for codec in codecs {
        if !seen.insert(*codec) {
            return Err(CommandBuildError::DuplicateCodec(*codec));
        }
        if !codec.supports(operation) {
            return Err(CommandBuildError::UnsupportedCodec {
                operation,
                codec: *codec,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_cd_preserves_paths_and_uses_argument_array() {
        let request = ChdmanRequest::CreateCd {
            input: "/games/Äther [Disc 1]/track set.cue".into(),
            output: "/output/Äther [Disc 1].chd".into(),
            options: CreateOptions {
                hunk_size: Some(9_792),
                compression: Some(vec![CompressionCodec::CdLzma, CompressionCodec::CdFlac]),
                processors: Some(4),
            },
        };

        let command = build_command("/opt/Hunk tools/chdman", &request).unwrap();

        assert_eq!(command.program, Path::new("/opt/Hunk tools/chdman"));
        assert_eq!(
            command.args,
            vec![
                "createcd",
                "-i",
                "/games/Äther [Disc 1]/track set.cue",
                "-o",
                "/output/Äther [Disc 1].chd",
                "-hs",
                "9792",
                "-c",
                "cdlz,cdfl",
                "-np",
                "4",
            ]
            .into_iter()
            .map(OsString::from)
            .collect::<Vec<_>>()
        );
        assert!(!command.args.iter().any(|arg| arg == "-f"));
    }

    #[test]
    fn all_six_operations_use_expected_subcommands() {
        let requests = [
            ChdmanRequest::CreateCd {
                input: "disc.cue".into(),
                output: "disc.chd".into(),
                options: CreateOptions::default(),
            },
            ChdmanRequest::CreateDvd {
                input: "disc.iso".into(),
                output: "dvd.chd".into(),
                options: CreateOptions::default(),
            },
            ChdmanRequest::ExtractCd {
                input: "disc.chd".into(),
                output: "disc.cue".into(),
                output_bin: "disc.bin".into(),
                split_bin: false,
            },
            ChdmanRequest::ExtractDvd {
                input: "dvd.chd".into(),
                output: "dvd.iso".into(),
            },
            ChdmanRequest::Verify {
                input: "disc.chd".into(),
            },
            ChdmanRequest::Info {
                input: "disc.chd".into(),
            },
        ];
        let expected = [
            "createcd",
            "createdvd",
            "extractcd",
            "extractdvd",
            "verify",
            "info",
        ];

        for (request, expected) in requests.iter().zip(expected) {
            let command = build_command("chdman", request).unwrap();
            assert_eq!(command.args.first(), Some(&OsString::from(expected)));
            assert!(!command.args.iter().any(|arg| arg == "-f"));
        }
    }

    #[test]
    fn rejects_overwrite_and_unsafe_option_combinations() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("source.iso");
        let output = directory.path().join("output.chd");
        std::fs::write(&output, b"existing").unwrap();

        let overwrite = ChdmanRequest::CreateDvd {
            input: source.clone(),
            output: output.clone(),
            options: CreateOptions::default(),
        };
        assert_eq!(
            build_command("chdman", &overwrite),
            Err(CommandBuildError::OutputExists(output))
        );

        let invalid_codec = ChdmanRequest::CreateCd {
            input: source,
            output: directory.path().join("new.chd"),
            options: CreateOptions {
                compression: Some(vec![CompressionCodec::Lzma]),
                ..CreateOptions::default()
            },
        };
        assert!(matches!(
            build_command("chdman", &invalid_codec),
            Err(CommandBuildError::UnsupportedCodec { .. })
        ));
    }

    #[test]
    fn split_bin_requires_track_placeholder() {
        let request = ChdmanRequest::ExtractCd {
            input: "disc.chd".into(),
            output: "disc.cue".into(),
            output_bin: "disc.bin".into(),
            split_bin: true,
        };

        assert!(matches!(
            build_command("chdman", &request),
            Err(CommandBuildError::MissingTrackPlaceholder(_))
        ));

        let valid_request = ChdmanRequest::ExtractCd {
            input: "disc.chd".into(),
            output: "disc.cue".into(),
            output_bin: "disc (Track %02t).bin".into(),
            split_bin: true,
        };
        assert!(build_command("chdman", &valid_request).is_ok());
    }
}
