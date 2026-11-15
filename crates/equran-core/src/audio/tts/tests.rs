//! Runtime, splitting and worker-protocol tests.

use super::config::SupertonicConfig;
#[cfg(unix)]
use super::config::status_from_config;
use super::text::{sanitize_for_speech, split_text_for_tts};
use super::*;

#[test]
fn resolves_supertonic_script_and_falls_back_to_system_python() {
    let config = SupertonicConfig::from_env_or_project_roots(&[PathBuf::from("/tmp/equran-cli")]);

    assert_eq!(
        config.script,
        PathBuf::from("/tmp/equran-cli/tts/tts_supertonic.py")
    );
    assert_eq!(config.python, PathBuf::from("python3"));
}

#[test]
fn prefers_project_venv_python_when_present() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let missing_root = temp_dir.path().join("missing");
    let resource_root = temp_dir.path().join("resources");
    let tts_dir = resource_root.join("tts");
    let venv_bin = tts_dir.join(".venv/bin");
    std::fs::create_dir_all(&venv_bin).expect("venv bin should be created");
    std::fs::write(tts_dir.join("tts_supertonic.py"), "").expect("script should be created");
    std::fs::write(venv_bin.join("python"), "").expect("python should be created");

    let config =
        SupertonicConfig::from_env_or_project_roots(&[missing_root, resource_root.clone()]);

    assert_eq!(config.script, resource_root.join("tts/tts_supertonic.py"));
    assert_eq!(config.python, resource_root.join("tts/.venv/bin/python"));
}

// The interpreter stubs below are POSIX shell scripts, so the tests that run
// them only apply on Unix. The path-resolution and text tests stay portable.
#[cfg(unix)]
#[test]
fn ignores_gpu_venv_that_cannot_import_supertonic() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let root = temp_dir.path().to_path_buf();
    let tts_dir = root.join("tts");
    let gpu_bin = tts_dir.join(".venv-gpu/bin");
    std::fs::create_dir_all(&gpu_bin).expect("gpu venv bin should be created");
    std::fs::write(tts_dir.join("tts_supertonic.py"), "").expect("script should be created");
    // An interpreter that cannot import the package must not be selected.
    std::fs::write(gpu_bin.join("python"), "#!/bin/sh\nexit 1\n")
        .expect("python stub should be created");
    std::fs::set_permissions(
        gpu_bin.join("python"),
        std::os::unix::fs::PermissionsExt::from_mode(0o755),
    )
    .expect("python stub should be executable");

    let config = SupertonicConfig::from_env_or_project_roots(&[root]);

    assert_eq!(
        config.gpu_python,
        Some(tts_dir.join(".venv-gpu/bin/python"))
    );
    assert_eq!(config.synthesis_python(), Path::new("python3"));
}

#[test]
fn reports_missing_script() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let engine =
        TtsEngine::with_project_root(temp_dir.path().join("cache"), temp_dir.path().to_path_buf());

    assert_eq!(engine.voice_status(), TtsVoiceStatus::MissingScript);
}

#[cfg(unix)]
#[test]
fn reports_missing_runtime_when_python_cannot_import_supertonic() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let root = temp_dir.path();
    let script = root.join("tts_supertonic.py");
    std::fs::write(&script, "").expect("script should be created");

    let failing_python = root.join("python-failing");
    std::fs::write(&failing_python, "#!/bin/sh\nexit 1\n").expect("python should be created");
    make_executable(&failing_python);

    let config = SupertonicConfig {
        python: failing_python,
        gpu_python: None,
        script,
        uses_project_venv: false,
        voice: DEFAULT_VOICE.to_owned(),
        total_steps: DEFAULT_TOTAL_STEPS,
        device: DEFAULT_DEVICE.to_owned(),
        idle_timeout_secs: DEFAULT_IDLE_TIMEOUT_SECS,
    };

    assert_eq!(status_from_config(&config), TtsVoiceStatus::MissingRuntime);
}

#[cfg(unix)]
#[test]
fn reports_ready_when_python_imports_supertonic() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let root = temp_dir.path();
    let script = root.join("tts_supertonic.py");
    std::fs::write(&script, "").expect("script should be created");

    let working_python = root.join("python-working");
    std::fs::write(&working_python, "#!/bin/sh\nexit 0\n").expect("python should be created");
    make_executable(&working_python);

    let config = SupertonicConfig {
        python: working_python,
        gpu_python: None,
        script,
        uses_project_venv: false,
        voice: DEFAULT_VOICE.to_owned(),
        total_steps: DEFAULT_TOTAL_STEPS,
        device: DEFAULT_DEVICE.to_owned(),
        idle_timeout_secs: DEFAULT_IDLE_TIMEOUT_SECS,
    };

    assert_eq!(status_from_config(&config), TtsVoiceStatus::Ready);
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = std::fs::metadata(path)
        .expect("metadata should be readable")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).expect("permissions should be set");
}

#[test]
fn splits_tafsir_text_by_paragraphs_and_sentences() {
    let text = "Paragraf pertama cukup pendek. Paragraf pertama kalimat kedua.\n\nParagraf kedua yang juga pendek.";
    let chunks = TtsEngine::split_narration_text(text);

    assert_eq!(chunks.len(), 2);
    assert!(chunks[0].contains("kalimat kedua"));
    assert_eq!(chunks[1], "Paragraf kedua yang juga pendek.");
}

#[test]
fn splits_long_tafsir_paragraph_by_sentence_limit() {
    let text = "Kalimat pertama cukup panjang untuk contoh. Kalimat kedua cukup panjang untuk contoh. Kalimat ketiga cukup panjang untuk contoh.";
    let chunks = split_text_for_tts(text, 78);

    assert!(chunks.len() > 1);
    assert!(chunks.iter().all(|chunk| !chunk.trim().is_empty()));
}

#[test]
fn keeps_whole_text_when_no_boundary_exists() {
    let chunks = split_text_for_tts("tanpa titik sama sekali", 700);

    assert_eq!(chunks, vec!["tanpa titik sama sekali".to_owned()]);
}

#[test]
fn splits_one_oversized_sentence_at_word_boundaries() {
    let text = "word ".repeat(200);
    let text = text.trim();
    let chunks = split_text_for_tts(text, 100);

    assert!(chunks.len() > 1);
    assert!(chunks.iter().all(|chunk| chunk.chars().count() <= 100));
    assert_eq!(chunks.join(" "), text);
}

#[test]
fn sanitize_drops_honorifics_and_arabic_script() {
    let cleaned = sanitize_for_speech("The Companions ؓ said: قُلْ هُوَ اللّٰهُ and the Prophet ﷺ agreed.");

    assert_eq!(cleaned, "The Companions said: and the Prophet agreed.");
}

#[test]
fn sanitize_drops_urdu_letters_and_zero_width_joiners() {
    let cleaned = sanitize_for_speech("Hasan and Mujahid رحمۃ اللہ علیہما say\u{200c} that");

    assert_eq!(cleaned, "Hasan and Mujahid say that");
}

#[test]
fn sanitize_removes_ornate_ayah_brackets_with_their_digits() {
    let cleaned = sanitize_for_speech("Recite ﴿98﴾ now");

    assert_eq!(cleaned, "Recite now");
}

#[test]
fn sanitize_removes_brackets_emptied_by_dropped_script() {
    let cleaned = sanitize_for_speech("Jibra'il (عليه السلام) came to him");

    assert_eq!(cleaned, "Jibra'il came to him");
}

#[test]
fn sanitize_normalizes_typographic_punctuation() {
    let cleaned = sanitize_for_speech("The word „salm‟ is written salm′ here");

    assert_eq!(cleaned, "The word \"salm\" is written salm' here");
}

#[test]
fn sanitize_is_idempotent() {
    let once = sanitize_for_speech("Abu Bakr ؓ said ﻻ إله unless the Wise");
    let twice = sanitize_for_speech(&once);

    assert_eq!(once, twice);
}

#[test]
fn tafsir_chunks_never_contain_arabic_script() {
    let text = "He was guided by the verse قُلْ هُوَ اللّٰهُ أَحَدٌ and by the Companions ؓ. Then he \
                explained the phrase in detail, and the audience understood it, and the lesson \
                stayed with them for the rest of their lives.";

    let chunks = TtsEngine::split_narration_text(text);

    assert!(!chunks.is_empty());
    for chunk in chunks {
        assert!(!chunk.is_empty());
        assert!(
            !chunk.chars().any(|character| {
                matches!(
                    character as u32,
                    0x0600..=0x06FF | 0x0750..=0x077F | 0xFB50..=0xFDFF | 0xFE70..=0xFEFF
                )
            }),
            "chunk still holds script the voice cannot read: {chunk}"
        );
    }
}

/// Writes a stand-in interpreter for tests.
///
/// It satisfies the `import supertonic` readiness probe and otherwise runs
/// the host Python, so the tests exercise the service protocol instead of
/// whatever packages the machine happens to have installed.
#[cfg(unix)]
fn write_stub_python(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    std::fs::write(
        path,
        "#!/bin/sh\nif [ \"$1\" = \"-c\" ]; then exit 0; fi\nexec python3 \"$@\"\n",
    )
    .expect("stub interpreter should be written");

    let mut permissions = std::fs::metadata(path)
        .expect("stub should exist")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).expect("stub should be executable");
}

#[test]
fn builds_unique_temporary_paths() {
    let target = Path::new("/tmp/supertonic_id_001_001.wav");
    let first = temporary_path(target);
    let second = temporary_path(target);

    assert_ne!(first, second);
    assert_eq!(first.parent(), target.parent());
    assert!(first.to_string_lossy().ends_with(".part"));
}

#[cfg(unix)]
#[tokio::test]
async fn uses_persistent_service_protocol() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let root = temp_dir.path().to_path_buf();
    let tts_dir = root.join("tts");
    let venv_bin = tts_dir.join(".venv/bin");
    std::fs::create_dir_all(&venv_bin).expect("venv bin should be created");
    // A stub interpreter stands in for the real one: these tests exercise
    // the service protocol, not the machine's Python. Depending on a system
    // interpreter made them fail whenever the host had no supertonic wheel.
    write_stub_python(&venv_bin.join("python"));

    let service_log = root.join("service.log");
    let script = tts_dir.join("tts_supertonic.py");
    std::fs::write(
            &script,
            format!(
                r#"#!/usr/bin/env python3
import argparse, json, pathlib, sys
parser = argparse.ArgumentParser()
parser.add_argument('--service', action='store_true')
parser.add_argument('--text')
parser.add_argument('--output')
parser.add_argument('--lang')
parser.add_argument('--voice')
parser.add_argument('--steps')
parser.add_argument('--device')
parser.add_argument('--idle-timeout')
args = parser.parse_args()
if not args.service:
    sys.exit(17)
pathlib.Path({log:?}).write_text('started')
for line in sys.stdin:
    request = json.loads(line)
    pathlib.Path(request['output']).write_text(request['text'] + '|' + request['lang'] + '|' + request['voice'] + '|' + str(request['steps']))
    print(json.dumps({{'ok': True}}), flush=True)
"#,
                log = service_log.to_string_lossy()
            ),
        )
        .expect("script should be created");

    let engine = TtsEngine::with_project_roots(root.join("cache"), vec![root.clone()]);
    let output = root.join("out.wav");

    engine
        .synthesize_with_service("halo", "id", &output)
        .await
        .expect("service synthesis should succeed");

    assert_eq!(
        std::fs::read_to_string(&output).expect("output"),
        format!("halo|id|M2|{DEFAULT_TOTAL_STEPS}")
    );
    assert_eq!(
        std::fs::read_to_string(&service_log).expect("log"),
        "started"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn service_error_does_not_poison_next_request() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let root = temp_dir.path().to_path_buf();
    let tts_dir = root.join("tts");
    let venv_bin = tts_dir.join(".venv/bin");
    std::fs::create_dir_all(&venv_bin).expect("venv bin should be created");
    // A stub interpreter stands in for the real one: these tests exercise
    // the service protocol, not the machine's Python. Depending on a system
    // interpreter made them fail whenever the host had no supertonic wheel.
    write_stub_python(&venv_bin.join("python"));

    let marker = root.join("marker");
    let script = tts_dir.join("tts_supertonic.py");
    std::fs::write(
        &script,
        format!(
            r#"#!/usr/bin/env python3
import argparse, json, pathlib, sys
parser = argparse.ArgumentParser()
parser.add_argument('--service', action='store_true')
parser.add_argument('--text')
parser.add_argument('--output')
parser.add_argument('--lang')
parser.add_argument('--voice')
parser.add_argument('--device')
parser.add_argument('--idle-timeout')
args = parser.parse_args()
marker = pathlib.Path({marker:?})
if not marker.exists():
    marker.write_text('failed-once')
    print(json.dumps({{'ok': False, 'error': 'temporary failure'}}), flush=True)
    sys.exit(0)
for line in sys.stdin:
    request = json.loads(line)
    pathlib.Path(request['output']).write_text(request['text'])
    print(json.dumps({{'ok': True}}), flush=True)
"#,
            marker = marker.to_string_lossy()
        ),
    )
    .expect("script should be created");

    let engine = TtsEngine::with_project_roots(root.join("cache"), vec![root.clone()]);
    let first_output = root.join("first.wav");
    let second_output = root.join("second.wav");

    assert!(
        engine
            .synthesize_with_service("first", "id", &first_output)
            .await
            .is_err()
    );
    engine
        .synthesize_with_service("second", "id", &second_output)
        .await
        .expect("service should restart after failure");

    assert_eq!(
        std::fs::read_to_string(&second_output).expect("second output"),
        "second"
    );
}

#[cfg(unix)]
#[test]
fn forgets_the_cached_probe_answer_after_setup_installs_the_package() {
    use std::os::unix::fs::PermissionsExt;

    // A stand-in interpreter whose import result comes from a file, so the test
    // can make the package "appear" between two probes without touching Python.
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let python = temp_dir.path().join("python");
    std::fs::write(
        &python,
        "#!/bin/sh\nexit \"$(cat \"$(dirname \"$0\")/result\")\"\n",
    )
    .expect("interpreter stub should be written");
    let mut permissions = std::fs::metadata(&python)
        .expect("stub metadata")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&python, permissions).expect("stub should be executable");
    let result = temp_dir.path().join("result");
    std::fs::write(&result, "1").expect("result should be written");

    assert!(!config::supertonic_module_available(&python));
    std::fs::write(&result, "0").expect("result should be rewritten");
    // Memoized for the session: this is the state the panel used to get stuck in.
    assert!(!config::supertonic_module_available(&python));

    forget_supertonic_probes();

    assert!(config::supertonic_module_available(&python));
}
