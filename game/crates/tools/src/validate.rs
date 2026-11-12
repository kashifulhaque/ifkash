//! `lf-tools validate`: load the content, run every check, print the issues.

use std::path::Path;
use std::process::ExitCode;

use lf_content::validate::has_errors;
use lf_content::{Content, Portfolio, Severity, Validate};

/// Validates the data directory. Exits nonzero on any error; warnings alone pass.
pub fn run(data: &Path) -> ExitCode {
    let content = match Content::load_dir(data) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut issues = content.validate();
    // The portfolio is separate from Content (not hashed), so load and check it here.
    match Portfolio::load_dir(data) {
        Ok((portfolio, exported)) => {
            issues.extend(portfolio.validate(&exported));
            // The hub's NPCs must reach every portfolio entry (WP-068).
            issues.extend(lf_content::npc::check_facts(&content, &exported));
        }
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    }
    for issue in &issues {
        println!("{issue}");
    }
    let errors = issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .count();
    let warnings = issues.len() - errors;
    println!(
        "{} validated: {errors} error(s), {warnings} warning(s), content hash {:016x}",
        data.display(),
        content.content_hash()
    );
    if has_errors(&issues) {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
