#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;

use hunk_lib::chdman::{APPROVED_CHDMAN_VERSION, ChdmanRequest, build_command, check_capabilities};

fn executable_script(directory: &std::path::Path, name: &str, source: &str) -> std::path::PathBuf {
    let path = directory.join(name);
    fs::write(&path, source).unwrap();
    let mut permissions = fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&path, permissions).unwrap();
    path
}

#[test]
fn capability_check_accepts_pinned_fake_process_even_with_usage_exit_status() {
    let directory = tempfile::tempdir().unwrap();
    let script = executable_script(
        directory.path(),
        "fake chdman [test]",
        "#!/bin/sh\n\
         printf '%s\\n' 'chdman - MAME Compressed Hunks of Data (CHD) manager 0.289' >&2\n\
         printf '%s\\n' 'info: info' 'verify: verify' 'createcd: create' 'createdvd: create' 'extractcd: extract' 'extractdvd: extract' >&2\n\
         exit 1\n",
    );

    let capabilities = check_capabilities(&script).unwrap();

    assert_eq!(capabilities.version, APPROVED_CHDMAN_VERSION);
    assert_eq!(capabilities.commands.len(), 6);
}

#[test]
fn spawned_command_preserves_spaces_brackets_and_unicode_without_a_shell() {
    let directory = tempfile::tempdir().unwrap();
    let script = executable_script(
        directory.path(),
        "argument recorder",
        "#!/bin/sh\nfor argument in \"$@\"; do printf '<%s>\\n' \"$argument\"; done\n",
    );
    let request = ChdmanRequest::Info {
        input: "/games/Äther [Disc 1]/track set.chd".into(),
    };
    let command = build_command(script, &request).unwrap();

    let output = command.process().output().unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "<info>\n<-i>\n</games/Äther [Disc 1]/track set.chd>\n<-v>\n"
    );
}

#[test]
#[ignore = "requires HUNK_CHDMAN to point to an explicitly built approved binary"]
fn approved_real_binary_reports_required_capabilities() {
    let path = std::env::var_os("HUNK_CHDMAN").expect("HUNK_CHDMAN is required");

    check_capabilities(std::path::Path::new(&path)).unwrap();
}
