use magicer::domain::value_objects::path::RelativePath;

#[test]
fn test_path_traversal_attacks() {
    let scenarios = [
        "../etc/passwd",
        "uploads/../../etc/passwd",
        "data/./../etc/passwd",
        "/etc/passwd",
    ];

    for scenario in scenarios {
        let result = RelativePath::new(scenario);
        assert!(result.is_err(), "Scenario '{}' should be rejected", scenario);
    }
}
