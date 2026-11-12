use std::path::{Path, PathBuf};

use lf_content::{Portfolio, PortfolioContent, Severity};

fn data_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

fn load(json: &str) -> (Portfolio, PortfolioContent) {
    let ron = std::fs::read_to_string(data_dir().join("portfolio.ron")).unwrap();
    (
        Portfolio::from_ron(&ron).unwrap(),
        PortfolioContent::from_json(json).unwrap(),
    )
}

#[test]
fn real_portfolio_has_no_issues() {
    let (p, c) = Portfolio::load_dir(data_dir()).expect("loads");
    let issues = p.validate(&c);
    assert!(issues.is_empty(), "{issues:#?}");
}

#[test]
fn missing_slug_is_an_error_naming_it() {
    let json = include_str!("fixtures/portfolio/missing_slug.json");
    let (p, c) = load(json);
    let issues = p.validate(&c);
    let err = issues
        .iter()
        .find(|i| i.severity == Severity::Error && i.message.contains("banana-cpp"))
        .unwrap_or_else(|| panic!("no error naming banana-cpp: {issues:#?}"));
    assert!(err.message.contains("projects"));
}

#[test]
fn unmapped_project_is_a_warning() {
    let json = include_str!("fixtures/portfolio/unmapped_project.json");
    let (p, c) = load(json);
    let issues = p.validate(&c);
    let warn = issues
        .iter()
        .find(|i| i.message.contains("brand-new-thing"))
        .unwrap_or_else(|| panic!("no warning: {issues:#?}"));
    assert_eq!(warn.severity, Severity::Warning);
    assert!(warn.message.contains("unmapped"));
}

#[test]
fn item_without_slug_is_an_error_naming_the_title() {
    let json = include_str!("fixtures/portfolio/null_slug.json");
    let (p, c) = load(json);
    let issues = p.validate(&c);
    assert!(
        issues
            .iter()
            .any(|i| i.severity == Severity::Error && i.message.contains("Slugless Project")),
        "{issues:#?}"
    );
}
