use std::{fs, path::PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("src-tauri manifest should be nested under apps/desktop")
        .to_path_buf()
}

fn read_repo_file(path: &str) -> String {
    let path = repo_root().join(path);
    fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

#[test]
fn tauri_bundles_macos_dmg_for_release_distribution() {
    let tauri_config = read_repo_file("apps/desktop/src-tauri/tauri.conf.json");

    assert!(
        tauri_config.contains(r#""targets": ["deb", "rpm", "appimage", "nsis", "dmg"]"#),
        "Tauri release bundles must keep producing a macOS DMG installer"
    );
    assert!(
        tauri_config.contains(r#""productName": "Publisher""#),
        "The generated DMG should contain Publisher.app"
    );
}

#[test]
fn macos_build_verifies_dmg_before_uploading_artifact() {
    let workflow = read_repo_file(".github/workflows/ci.yml");

    assert!(
        workflow.contains("hdiutil verify"),
        "macOS release builds must fail when the generated DMG is corrupt or incomplete"
    );
    assert!(
        workflow.contains("bundle/dmg") && workflow.contains("*.dmg"),
        "DMG verification must run against the generated bundle/dmg artifact"
    );
}

#[test]
fn macos_build_mounts_dmg_and_checks_publisher_app_contents() {
    let workflow = read_repo_file(".github/workflows/ci.yml");

    assert!(
        workflow.contains("hdiutil attach"),
        "DMG validation must mount the image before accepting it"
    );
    assert!(
        workflow.contains("Publisher.app"),
        "Mounted DMG validation must assert Publisher.app is present"
    );
    assert!(
        workflow.contains("Contents/MacOS"),
        "Mounted DMG validation must assert Publisher.app contains a launchable executable"
    );
    assert!(
        workflow.contains("hdiutil detach"),
        "Mounted DMG validation must detach the image after inspection"
    );
}

#[test]
fn macos_build_launches_app_from_mounted_dmg_without_gatekeeper_corruption_error() {
    let workflow = read_repo_file(".github/workflows/ci.yml");

    assert!(
        workflow.contains("spctl"),
        "Release validation must check the downloaded app would not be rejected as corrupted"
    );
    assert!(
        workflow.contains("open ") || workflow.contains("open -"),
        "Release validation must launch Publisher.app from the mounted DMG"
    );
}

#[test]
fn nightly_release_uploads_only_dmg_assets_that_pass_validation() {
    let workflow = read_repo_file(".github/workflows/ci.yml");

    assert!(
        workflow.contains("Validate macOS DMG"),
        "Nightly releases must have an explicit DMG validation gate before upload"
    );
    assert!(
        workflow.contains("needs:") && workflow.contains("build-artifacts"),
        "Nightly release publication must depend on the artifact build and validation pipeline"
    );
    assert!(
        workflow.contains("validated") || workflow.contains("verified"),
        "Nightly upload logic must distinguish verified DMGs from unvalidated files"
    );
}
