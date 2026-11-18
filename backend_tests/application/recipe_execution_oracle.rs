use orchestrata_application::recipe_execution::PackageRun;
use serde_json::Value;

// Exhaustive fixture oracle, independent of Engine compilation and optimization.
pub fn check(records: &str, result: &PackageRun, delay: bool) {
    let input: Value = serde_json::from_str(records).unwrap();
    let shift = input.get("work_orders").is_some();
    let (jobs, staff, edges, id, slots, active, job, person, weight, ceiling) = if shift {
        (
            "work_orders",
            "technicians",
            "eligibility",
            "id",
            "capacity_slots",
            "available",
            "work_order_id",
            "technician_id",
            if delay { "delay_minutes" } else { "cost_cents" },
            3,
        )
    } else {
        (
            "orders",
            "crew",
            "options",
            "key",
            "slots",
            "active",
            "task",
            "person",
            if delay { "delay" } else { "price" },
            1,
        )
    };
    let jobs = input[jobs].as_array().unwrap();
    let staff = input[staff].as_array().unwrap();
    let edges = input[edges].as_array().unwrap();
    let capacity: Vec<usize> = staff
        .iter()
        .map(|worker| {
            if worker[active].as_bool().unwrap() {
                (worker[slots].as_u64().unwrap() as usize).min(ceiling)
            } else {
                0
            }
        })
        .collect();
    let choices: Vec<Vec<(usize, u64)>> = jobs
        .iter()
        .map(|item| {
            edges
                .iter()
                .filter(|edge| edge[job] == item[id])
                .map(|edge| {
                    let resource = staff
                        .iter()
                        .position(|worker| worker[id] == edge[person])
                        .unwrap();
                    (resource, edge[weight].as_u64().unwrap())
                })
                .collect()
        })
        .collect();
    let mut best = (0, u64::MAX);
    enumerate(&choices, 0, &mut capacity.clone(), (0, 0), &mut best);

    assert_eq!(result.solution.assignments.len(), jobs.len());
    let mut remaining = capacity;
    let mut actual = (0, 0);
    for (index, assignment) in result.solution.assignments.iter().enumerate() {
        assert_eq!(assignment.item_id, jobs[index][id].as_str().unwrap());
        if let Some(resource_id) = &assignment.resource_id {
            let resource = staff
                .iter()
                .position(|worker| worker[id] == *resource_id)
                .unwrap();
            assert!(
                remaining[resource] > 0,
                "unavailable or over-capacity assignment"
            );
            remaining[resource] -= 1;
            let weight = choices[index]
                .iter()
                .find(|(r, _)| *r == resource)
                .unwrap()
                .1;
            actual.0 += 1;
            actual.1 += weight;
        }
    }
    assert_eq!(
        actual, best,
        "complete assignment must reach the independently enumerated optimum"
    );
    assert_eq!(result.solution.assigned, actual.0);
    assert_eq!(result.solution.minimized_edge_sum, Some(actual.1));
}

fn enumerate(
    choices: &[Vec<(usize, u64)>],
    item: usize,
    remaining: &mut [usize],
    score: (usize, u64),
    best: &mut (usize, u64),
) {
    if item == choices.len() {
        if score.0 > best.0 || (score.0 == best.0 && score.1 < best.1) {
            *best = score;
        }
        return;
    }
    enumerate(choices, item + 1, remaining, score, best);
    for &(resource, weight) in &choices[item] {
        if remaining[resource] == 0 {
            continue;
        }
        remaining[resource] -= 1;
        enumerate(
            choices,
            item + 1,
            remaining,
            (score.0 + 1, score.1 + weight),
            best,
        );
        remaining[resource] += 1;
    }
}
