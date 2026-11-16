use std::{collections::HashSet, fs, path::Path};

use orchestrata_application::solve_service::SolveService;
use orchestrata_recipes::load_recipe;
use orchestrata_solver::{Problem, verify_solution};

use super::recipe_catalog::{BUILTIN_RECIPES, all, find_example};

#[test]
fn every_built_in_recipe_is_valid_and_its_example_matches_a_verified_plan() {
    let metadata = all().expect("the built-in metadata must load");
    assert_eq!(metadata.len(), BUILTIN_RECIPES.len());
    let mut ids = HashSet::new();
    for builtin in BUILTIN_RECIPES {
        assert!(ids.insert(builtin.id), "duplicate recipe {}", builtin.id);
        assert_eq!(load_recipe(builtin.xml).unwrap().id(), builtin.id);
        assert_eq!(find_example(builtin.id), Some(builtin.problem));
        let problem: Problem = serde_json::from_str(builtin.problem)
            .unwrap_or_else(|error| panic!("invalid example {}: {error}", builtin.id));
        let expected: serde_json::Value = serde_json::from_str(builtin.expected).unwrap();
        let solution = SolveService::execute(&problem)
            .unwrap_or_else(|error| panic!("example {} failed: {error:?}", builtin.id))
            .unwrap_or_else(|| panic!("example {} was unexpectedly infeasible", builtin.id));
        verify_solution(&problem, &solution).unwrap();
        let actual = serde_json::to_value(solution).unwrap();
        for (key, value) in expected.as_object().unwrap() {
            assert_eq!(&actual[key], value, "recipe {}: {key}", builtin.id);
        }
        assert!(expected.get("values").is_some());
    }
    assert!(find_example("does-not-exist").is_none());
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../recipes");
    let source_ids: HashSet<String> = fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "xml"))
        .map(|path| path.file_stem().unwrap().to_str().unwrap().to_owned())
        .collect();
    assert_eq!(source_ids, ids.into_iter().map(str::to_owned).collect());
}

#[test]
fn retail_callout_changes_only_unavailable_choices_and_keeps_a_fixed_shift() {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../recipes/examples/retail-staffing-what-if");
    let mut cases = Vec::new();
    for phase in ["baseline", "adjusted"] {
        let path = folder.join(phase);
        let problem: Problem =
            serde_json::from_slice(&fs::read(path.join("problem.json")).unwrap()).unwrap();
        let expected: serde_json::Value =
            serde_json::from_slice(&fs::read(path.join("expected.json")).unwrap()).unwrap();
        let result = SolveService::execute(&problem).unwrap().unwrap();
        verify_solution(&problem, &result).unwrap();
        assert_eq!(serde_json::to_value(&result).unwrap(), expected);
        cases.push((problem, result));
    }
    let [(baseline, original), (updated, repaired)] = cases.try_into().ok().unwrap();
    assert_eq!(baseline.constraints, updated.constraints);
    assert_eq!(original.values[0], repaired.values[0]);
    for (old, new) in baseline.variables.iter().zip(&updated.variables) {
        assert_eq!(old.name, new.name);
        let without_absence: Vec<_> = old.domain.iter().copied().filter(|n| *n != 1).collect();
        assert_eq!(new.domain, without_absence);
    }
    assert_ne!(original.values, repaired.values);
}
