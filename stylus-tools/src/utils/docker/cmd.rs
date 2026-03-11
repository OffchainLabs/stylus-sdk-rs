// Copyright 2025-2026, Offchain Labs, Inc.
// For licensing, see https://github.com/OffchainLabs/stylus-sdk-rs/blob/main/licenses/COPYRIGHT.md

//! Run commands from the Docker CLI.

use std::{
    ffi::OsStr,
    path::Path,
    process::{Command, Stdio},
    str,
};

use super::{error::DockerError, json};

const DOCKER_PROGRAM: &str = "docker";

/// Build a Docker image using a Dockerfile from an existing file.
pub fn build_with_file(tag: &str, dockerfile_path: &Path) -> Result<(), DockerError> {
    info!(@grey, "Building Docker image: {} (using Dockerfile: {})", tag, dockerfile_path.display());

    let mut child = Command::new(DOCKER_PROGRAM)
        .arg("build")
        .args(["--tag", tag])
        .args([
            "--file",
            dockerfile_path.to_str().ok_or_else(|| {
                DockerError::Docker(format!(
                    "Dockerfile path is not valid UTF-8: {}",
                    dockerfile_path.display()
                ))
            })?,
        ])
        .arg(".")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(DockerError::CommandExecution)?;

    let status = child.wait().map_err(DockerError::WaitFailure)?;

    if !status.success() {
        let detail = match status.code() {
            Some(code) => format!("exit code: {code}"),
            None => "terminated by signal".to_string(),
        };
        return Err(DockerError::Docker(format!(
            "Docker build failed with {detail}"
        )));
    }

    info!(@grey, "Docker image built successfully: {}", tag);
    Ok(())
}

/// Check if a Docker image exists on Docker Hub.
pub fn image_exists_on_hub(image_name: &str) -> Result<bool, DockerError> {
    // Use docker manifest inspect to check if the image exists on the registry
    let output = Command::new(DOCKER_PROGRAM)
        .arg("manifest")
        .arg("inspect")
        .arg(image_name)
        .output()
        .map_err(DockerError::CommandExecution)?;

    // If the command succeeds, the image exists
    let exists = output.status.success();

    Ok(exists)
}

/// Check if a specific Docker image exists locally.
/// Returns true if the exact image:tag combination exists locally.
pub fn image_exists_locally(image_name: &str) -> Result<bool, DockerError> {
    let output = Command::new(DOCKER_PROGRAM)
        .arg("images")
        .args(["--format", "json"])
        .arg(image_name)
        .output()
        .map_err(DockerError::CommandExecution)?;

    let success = output.status.success();
    if !success {
        return Err(DockerError::from_stderr(output.stderr));
    }

    // Parse the JSON output to check if any images match the exact image:tag
    let images: Vec<json::Image> = output
        .stdout
        .split(|b| *b == b'\n')
        .filter(|slice| !slice.is_empty()) // Filter out empty lines
        .map(|slice| serde_json::from_slice(slice).map_err(DockerError::Json))
        .collect::<Result<Vec<_>, _>>()?;

    // Check if any image matches the exact repository:tag combination
    let exists = images.iter().any(|image| {
        let full_name = if image.tag == "<none>" {
            image.repository.clone()
        } else {
            format!("{}:{}", image.repository, image.tag)
        };
        full_name == image_name
    });

    Ok(exists)
}

fn invalid_input(msg: impl Into<String>) -> DockerError {
    DockerError::InvalidVolumeInput(msg.into())
}

/// Returns true if the string contains any ASCII control character
/// (bytes 0x00–0x1F and 0x7F, including null, newline, carriage return,
/// tab, escape, etc.). No legitimate filesystem path or Docker argument
/// should contain these.
fn has_control_chars(s: &str) -> bool {
    s.bytes().any(|b| b.is_ascii_control())
}

/// Validate a volume mount to prevent path traversal, Docker volume separator
/// injection, and control-character injection. The container path is checked
/// for colons, control characters, absoluteness, and `..` components. The host
/// path is checked for control characters, then canonicalized (resolving
/// symlinks and verifying existence), and the canonicalized result is checked
/// for volume-separator characters (`:` on Unix; `:` and `;` on Windows).
/// Returns the `host:container` string suitable for `docker run --volume`.
fn validate_volume(host_path: &str, container_path: &str) -> Result<String, DockerError> {
    // Validate container_path first (cheap string checks) before hitting the
    // filesystem, so malicious container paths are always rejected regardless
    // of host path resolution.
    if container_path.contains(':') {
        return Err(invalid_input(
            "container path contains ':' which could be interpreted as a volume separator",
        ));
    }
    if has_control_chars(container_path) {
        return Err(invalid_input("container path contains control characters"));
    }
    if !container_path.starts_with('/') {
        return Err(invalid_input("container path must be absolute"));
    }
    if container_path.split('/').any(|c| c == "..") {
        return Err(invalid_input(
            "container path must not contain '..' components",
        ));
    }

    // Validate raw host path for control characters before canonicalization,
    // preventing malformed paths from reaching filesystem operations or
    // appearing in error messages.
    if has_control_chars(host_path) {
        return Err(invalid_input("host path contains control characters"));
    }

    // Canonicalize host path (resolves symlinks, verifies existence).
    // Note: this is subject to TOCTOU — the path could change between
    // validation and Docker's use of it. This race is inherent to bind mounts
    // and would persist even with `--mount` syntax.
    let canonical = std::fs::canonicalize(host_path)
        .map_err(|e| invalid_input(format!("failed to resolve host path '{host_path}': {e}")))?;
    let canonical_str = canonical.to_str().ok_or_else(|| {
        invalid_input(format!(
            "volume host path is not valid UTF-8: {}",
            canonical.display()
        ))
    })?;
    // Reject colons to prevent them being interpreted as Docker volume
    // separators.
    #[cfg(unix)]
    if canonical_str.contains(':') {
        return Err(invalid_input(
            "host path contains ':' which could be interpreted as a volume separator",
        ));
    }
    // On Windows, NTFS prohibits colons in file/directory names, so the only
    // colon in a canonicalized path is the drive letter (e.g. `C:\...`). As
    // defense-in-depth, we still reject extra colons after the drive prefix.
    // Docker on Windows also accepts `;` as an alternative volume separator,
    // so we reject that too.
    // `std::fs::canonicalize` on Windows typically returns extended-length
    // paths like `\\?\C:\...`, so we strip that prefix before checking.
    #[cfg(windows)]
    {
        let path_for_check = canonical_str.strip_prefix(r"\\?\").unwrap_or(canonical_str);
        if path_for_check.contains(';') {
            return Err(invalid_input(
                "host path contains ';' which could be interpreted as a volume separator on Windows",
            ));
        }
        // After the drive letter prefix (e.g. "C:"), reject any additional colons.
        if path_for_check.len() > 2 && path_for_check[2..].contains(':') {
            return Err(invalid_input(
                "host path contains ':' which could be interpreted as a volume separator",
            ));
        }
    }
    Ok(format!("{canonical_str}:{container_path}"))
}

/// Run a command in a Docker container.
pub fn run(
    image: &str,
    network: Option<&str>,
    volumes: &[(&str, &str)],
    workdir: Option<&str>,
    args: impl IntoIterator<Item = impl AsRef<OsStr>>,
) -> Result<(), DockerError> {
    // TODO: builder pattern
    // TODO: --mount instead of --volume — `--mount type=bind,source=...,target=...`
    // uses key-value syntax, which would eliminate the colon/semicolon separator
    // checks and simplify the host-path portion of validate_volume. However,
    // `--mount` uses commas as delimiters, so paths containing commas would
    // need separate handling.
    let mut cmd = Command::new(DOCKER_PROGRAM);
    cmd.arg("run");
    if let Some(network) = network {
        if has_control_chars(network) {
            return Err(invalid_input("network name contains control characters"));
        }
        cmd.args(["--network", network]);
    }
    if let Some(workdir) = workdir {
        if has_control_chars(workdir) {
            return Err(invalid_input("workdir contains control characters"));
        }
        cmd.args(["--workdir", workdir]);
    }
    for (host_path, container_path) in volumes {
        let volume_arg = validate_volume(host_path, container_path)?;
        cmd.args(["--volume", &volume_arg]);
    }
    cmd.arg(image);
    cmd.args(args);
    let status = cmd
        .spawn()
        .map_err(DockerError::CommandExecution)?
        .wait()
        .map_err(DockerError::WaitFailure)?;
    if !status.success() {
        let detail = match status.code() {
            Some(code) => format!("exit code: {code}"),
            None => "terminated by signal".to_string(),
        };
        return Err(DockerError::Docker(format!(
            "Docker run failed with {detail} (see Docker output above for details)"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Returns the temp directory path as a string for use as a valid host path
    /// in tests.
    fn temp_host_path() -> String {
        std::env::temp_dir().to_str().unwrap().to_owned()
    }

    /// Returns the canonicalized temp directory path for expected-value
    /// assertions.
    fn canonical_temp() -> String {
        std::fs::canonicalize(std::env::temp_dir())
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned()
    }

    /// Asserts that `result` is an `InvalidVolumeInput` error whose message
    /// contains `expected_substr`.
    fn assert_invalid_volume(result: Result<String, DockerError>, expected_substr: &str) {
        let err = result.expect_err("expected InvalidVolumeInput error");
        assert!(
            matches!(err, DockerError::InvalidVolumeInput(_)),
            "expected InvalidVolumeInput, got: {err}"
        );
        assert!(
            err.to_string().contains(expected_substr),
            "expected error to contain {expected_substr:?}, got: {err}"
        );
    }

    // -- container path rejection tests --

    #[test]
    fn test_validate_volume_rejects_relative_container_path() {
        assert_invalid_volume(
            validate_volume(&temp_host_path(), "relative/path"),
            "must be absolute",
        );
    }

    #[test]
    fn test_validate_volume_rejects_empty_container_path() {
        assert_invalid_volume(validate_volume(&temp_host_path(), ""), "must be absolute");
    }

    #[test]
    fn test_validate_volume_rejects_dotdot_in_container_path() {
        let host = temp_host_path();
        for path in ["/foo/../etc", "/foo/bar/..", "/.."] {
            assert_invalid_volume(validate_volume(&host, path), "'..'");
        }
    }

    #[test]
    fn test_validate_volume_rejects_colon_in_container_path() {
        assert_invalid_volume(
            validate_volume(&temp_host_path(), "/foo:bar"),
            "container path contains ':'",
        );
    }

    #[test]
    fn test_validate_volume_rejects_control_chars_in_container_path() {
        let host = temp_host_path();
        for path in [
            "/foo\n/bar",
            "/foo\r/bar",
            "/foo\0/bar",
            "/foo\t/bar",
            "/foo\x1b/bar",
        ] {
            assert_invalid_volume(validate_volume(&host, path), "control characters");
        }
    }

    // -- host path rejection tests --

    #[test]
    fn test_validate_volume_rejects_control_chars_in_host_path() {
        for path in [
            "/tmp/foo\n/bar",
            "/tmp/foo\r/bar",
            "/tmp/foo\0/bar",
            "/tmp/foo\t/bar",
            "/tmp/foo\x1b/bar",
        ] {
            assert_invalid_volume(validate_volume(path, "/container"), "control characters");
        }
    }

    #[test]
    fn test_validate_volume_rejects_nonexistent_host_path() {
        assert_invalid_volume(
            validate_volume("/nonexistent/path/abc123", "/container"),
            "failed to resolve host path",
        );
    }

    #[test]
    fn test_validate_volume_rejects_empty_host_path() {
        assert_invalid_volume(
            validate_volume("", "/container"),
            "failed to resolve host path",
        );
    }

    #[test]
    #[cfg(unix)]
    fn test_validate_volume_rejects_colon_in_host_path() {
        let dir = std::env::temp_dir().join("test:colon");
        std::fs::create_dir_all(&dir).unwrap();
        let host = dir.to_str().unwrap();
        assert_invalid_volume(
            validate_volume(host, "/container"),
            "host path contains ':'",
        );
        std::fs::remove_dir(&dir).ok();
    }

    #[test]
    fn test_container_path_validated_before_host_path_resolution() {
        // Both paths are invalid, but the container path error should take
        // priority since container checks happen before filesystem access.
        assert_invalid_volume(
            validate_volume("/nonexistent/path", "relative/path"),
            "must be absolute",
        );
    }

    #[test]
    fn test_validate_volume_rejects_docker_options_injection_in_container_path() {
        // Docker volume syntax is `host:container:options` (e.g. `:ro`, `:rw`).
        // A colon in the container path could inject volume options.
        assert_invalid_volume(
            validate_volume(&temp_host_path(), "/container:ro"),
            "container path contains ':'",
        );
    }

    #[test]
    fn test_validate_volume_rejects_dotdot_at_path_start() {
        assert_invalid_volume(validate_volume(&temp_host_path(), "/../etc/shadow"), "'..'");
    }

    // -- acceptance tests --

    #[test]
    fn test_validate_volume_accepts_valid_paths() {
        let result = validate_volume(&temp_host_path(), "/container/path").unwrap();
        assert_eq!(result, format!("{}:/container/path", canonical_temp()));
    }

    #[test]
    fn test_validate_volume_accepts_root_container_path() {
        let result = validate_volume(&temp_host_path(), "/").unwrap();
        assert_eq!(result, format!("{}:/", canonical_temp()));
    }

    #[test]
    fn test_validate_volume_accepts_dotdot_substring_in_container_path() {
        // "bar..baz" contains ".." as a substring but NOT as a path component,
        // so it should be accepted (the check is component-based, not substring-based).
        let result = validate_volume(&temp_host_path(), "/foo/bar..baz").unwrap();
        assert_eq!(result, format!("{}:/foo/bar..baz", canonical_temp()));
    }

    #[test]
    fn test_validate_volume_canonicalizes_relative_host_path() {
        // A relative host path like "." should be canonicalized to the absolute CWD.
        let result = validate_volume(".", "/container").unwrap();
        let canonical = std::fs::canonicalize(".").unwrap();
        assert_eq!(result, format!("{}:/container", canonical.display()));
    }

    #[test]
    #[cfg(unix)]
    fn test_validate_volume_resolves_symlink_host_path() {
        let target = std::env::temp_dir();
        let link = target.join("test_symlink_validate_volume");
        let _ = std::fs::remove_file(&link);
        std::os::unix::fs::symlink(&target, &link).unwrap();
        let host = link.to_str().unwrap();
        let result = validate_volume(host, "/container").unwrap();
        // The result should use the resolved (canonical) path, not the symlink.
        assert_eq!(result, format!("{}:/container", canonical_temp()));
        std::fs::remove_file(&link).ok();
    }

    // -- integration tests via run() --

    #[test]
    fn test_run_rejects_malicious_volume() {
        // validate_volume should reject the nonexistent host path (via
        // canonicalize failing) before Docker is ever invoked.
        let err = run(
            "alpine:latest",
            None,
            &[("/nonexistent/path/abc123", "/container")],
            None,
            Vec::<String>::new(),
        )
        .unwrap_err();
        assert!(
            matches!(err, DockerError::InvalidVolumeInput(_)),
            "expected InvalidVolumeInput, got: {err}"
        );
    }

    #[test]
    fn test_run_rejects_traversal_volume() {
        let host = temp_host_path();
        let err = run(
            "alpine:latest",
            None,
            &[(&host, "/foo/../etc/shadow")],
            None,
            Vec::<String>::new(),
        )
        .unwrap_err();
        assert!(matches!(err, DockerError::InvalidVolumeInput(_)), "{err}");
        assert!(err.to_string().contains("'..'"), "{err}");
    }

    #[test]
    fn test_run_rejects_colon_in_container_path() {
        let host = temp_host_path();
        let err = run(
            "alpine:latest",
            None,
            &[(&host, "/evil:/host/root")],
            None,
            Vec::<String>::new(),
        )
        .unwrap_err();
        assert!(matches!(err, DockerError::InvalidVolumeInput(_)), "{err}");
        assert!(
            err.to_string().contains("container path contains ':'"),
            "{err}"
        );
    }
}
