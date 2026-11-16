use std::{collections::BTreeMap, fs, path::Path};

use orchestrata_recipes::{FieldKind, FieldValue, load_recipe};
use orchestrata_solver::Problem;
use serde_json::Value;

#[test]
fn fleet_examples_match_capacity_availability_and_recipe_payloads() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../recipes");
    for (recipe_id, output_name) in [
        ("fleet-pickup-delivery", "assigned-vehicles"),
        ("healthcare-supply-delivery", "assigned-vehicles"),
    ] {
        let xml = fs::read_to_string(root.join(format!("{recipe_id}.xml"))).unwrap();
        let recipe = load_recipe(&xml).unwrap();
        let files = root.join("examples").join(recipe_id);
        let inputs: BTreeMap<String, Value> =
            serde_json::from_slice(&fs::read(files.join("inputs.json")).unwrap()).unwrap();
        let problem: Problem =
            serde_json::from_slice(&fs::read(files.join("problem.json")).unwrap()).unwrap();
        let expected: Value =
            serde_json::from_slice(&fs::read(files.join("expected.json")).unwrap()).unwrap();
        let typed = recipe
            .inputs()
            .iter()
            .filter_map(|field| {
                inputs.get(&field.name).map(|value| {
                    let value = match field.kind {
                        FieldKind::StringList => FieldValue::StringList(
                            value
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|s| s.as_str().unwrap().to_owned())
                                .collect(),
                        ),
                        FieldKind::Integer => FieldValue::Integer(value.as_i64().unwrap()),
                        FieldKind::IntegerList => FieldValue::IntegerList(integers(value)),
                        _ => panic!("unexpected fleet field {}", field.name),
                    };
                    (field.name.clone(), value)
                })
            })
            .collect();
        recipe.validate_inputs(&typed).unwrap();
        let jobs = integers(&inputs["job-windows-minutes"]);
        let weights = integers(&inputs["job-weights-kg"]);
        let vehicles = integers(&inputs["vehicle-windows-minutes"]);
        let capacities = integers(&inputs["vehicle-capacities-kg"]);
        assert_eq!(problem.variables.len(), weights.len(), "{recipe_id}");
        assert_eq!(jobs.len(), weights.len() * 2, "{recipe_id}");
        assert_eq!(vehicles.len(), capacities.len() * 2, "{recipe_id}");
        let latest_start = jobs.as_chunks::<2>().0.iter().map(|w| w[0]).max().unwrap();
        let earliest_end = jobs.as_chunks::<2>().0.iter().map(|w| w[1]).min().unwrap();
        assert!(
            latest_start < earliest_end,
            "{recipe_id}: jobs must share an interval"
        );

        let mut pairs = Vec::new();
        for (job, window) in jobs.as_chunks::<2>().0.iter().enumerate() {
            assert!(window[0] < window[1] && weights[job] > 0);
            let valid: Vec<_> = vehicles
                .as_chunks::<2>()
                .0
                .iter()
                .enumerate()
                .filter_map(|(vehicle, available)| {
                    let fits = available[0] <= window[0]
                        && window[1] <= available[1]
                        && weights[job] <= capacities[vehicle];
                    fits.then(|| i64::try_from(vehicle).unwrap())
                })
                .collect();
            assert_eq!(
                problem.variables[job].domain, valid,
                "{recipe_id}: job {job}"
            );
            for vehicle in valid {
                pairs.extend([i64::try_from(job).unwrap(), vehicle]);
            }
        }
        assert_eq!(integers(&inputs["eligible-pairs"]), pairs);
        let payload = BTreeMap::from([(
            output_name.to_owned(),
            FieldValue::IntegerList(integers(&expected["values"])),
        )]);
        recipe.validate_outputs(&payload).unwrap();
    }
}

fn integers(value: &Value) -> Vec<i64> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap())
        .collect()
}
