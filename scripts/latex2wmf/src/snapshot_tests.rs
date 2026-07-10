use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::cli::serialize_metadata;
use crate::svg_backend::{SvgBackend, render_formula_svg};
use crate::wmf::svg_to_wmf;

const SNAPSHOT_FONT_SIZE_PT: f64 = 12.0;
static SNAPSHOT_LOCK: Mutex<()> = Mutex::new(());

/// Render one copied manuscript formula and compare its WMF and JSON bytes.
fn assert_formula_snapshots(sample_name: &str, backend: SvgBackend) {
    // Serializing these cases avoids loading many embedded Typst font sets at once.
    let _guard = SNAPSHOT_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sample_path = manifest_dir
        .join("samples")
        .join("manuscript")
        .join(format!("{sample_name}.tex"));
    let latex = fs::read_to_string(&sample_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", sample_path.display()));
    let result = render_formula_svg(&latex, SNAPSHOT_FONT_SIZE_PT, backend).and_then(|rendered| {
        let wmf = svg_to_wmf(&rendered.svg, rendered.width_pt, rendered.height_pt)?;
        let metadata = serialize_metadata(&rendered, backend, SNAPSHOT_FONT_SIZE_PT)?;
        Ok((wmf, metadata))
    });

    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(PathBuf::from("../snapshots").join(backend.as_str()));
    settings.set_prepend_module_to_snapshot(false);
    settings.bind(|| match result {
        Ok((wmf, metadata)) => {
            let wmf_snapshot_name = format!("{sample_name}.wmf");
            insta::assert_binary_snapshot!(wmf_snapshot_name.as_str(), wmf);
            let json_snapshot_name = format!("{sample_name}_metadata.json");
            insta::assert_binary_snapshot!(json_snapshot_name.as_str(), metadata);
        }
        // Unsupported formulas remain explicit cases; gaining support replaces
        // the error snapshot with a real WMF snapshot during review.
        Err(error) => {
            let snapshot_name = format!("{sample_name}.error");
            insta::assert_snapshot!(snapshot_name.as_str(), error);
        }
    });
}

/// Return sorted `eq_*.tex` stems from one sample directory.
fn sample_names(directory: &Path) -> Vec<String> {
    let mut names = fs::read_dir(directory)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", directory.display()))
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            let stem = path.file_stem()?.to_str()?;
            (path.extension().and_then(|value| value.to_str()) == Some("tex")
                && stem.starts_with("eq_"))
            .then(|| stem.to_string())
        })
        .collect::<Vec<_>>();
    names.sort();
    names
}

macro_rules! manuscript_snapshot_cases {
    ($($case:ident),+ $(,)?) => {
        const MANUSCRIPT_CASES: &[&str] = &[$(stringify!($case)),+];

        mod ratex {
            use super::*;

            $(
                #[doc = concat!("Compare the RaTeX WMF snapshot for `", stringify!($case), "`.")]
                #[test]
                fn $case() {
                    assert_formula_snapshots(stringify!($case), SvgBackend::Ratex);
                }
            )+
        }

        mod typst {
            use super::*;

            $(
                #[doc = concat!("Compare the Typst WMF snapshot for `", stringify!($case), "`.")]
                #[test]
                fn $case() {
                    assert_formula_snapshots(stringify!($case), SvgBackend::Typst);
                }
            )+
        }
    };
}

manuscript_snapshot_cases!(
    eq_007, eq_015, eq_030, eq_039, eq_053, eq_056, eq_069, eq_072, eq_077, eq_079, eq_086, eq_087,
    eq_094, eq_110, eq_131, eq_163, eq_165, eq_188, eq_194, eq_200, eq_202, eq_211, eq_217, eq_220,
    eq_221, eq_223, eq_270, eq_285, eq_900, eq_901, eq_902, eq_903, eq_904, eq_905, eq_906, eq_907,
    eq_908, eq_909, eq_910, eq_911, eq_912, eq_913, eq_914, eq_915, eq_916, eq_917, eq_918, eq_919,
    eq_920, eq_921, eq_922, eq_923, eq_924, eq_925, eq_926, eq_927, eq_928, eq_929, eq_930, eq_931,
    eq_932, eq_933, eq_934, eq_935,
);

/// Keep the copied corpus byte-identical to the canonical manuscript samples.
#[test]
fn copied_manuscript_samples_match_source_corpus() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let local_dir = manifest_dir.join("samples").join("manuscript");
    let source_dir = manifest_dir
        .parent()
        .expect("latex2wmf has a scripts parent")
        .join("mathtype-rust")
        .join("samples")
        .join("manuscript");
    let expected = MANUSCRIPT_CASES
        .iter()
        .map(|name| (*name).to_string())
        .collect::<Vec<_>>();
    assert_eq!(sample_names(&local_dir), expected);
    assert_eq!(sample_names(&source_dir), expected);

    for sample_name in MANUSCRIPT_CASES {
        let file_name = format!("{sample_name}.tex");
        let local = fs::read(local_dir.join(&file_name)).expect("copied sample should be readable");
        let source =
            fs::read(source_dir.join(&file_name)).expect("source sample should be readable");
        assert_eq!(local, source, "copied sample differs: {file_name}");
    }
}
