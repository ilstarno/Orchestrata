use orchestrata_solver::{Constraint, Problem, Score, Variable, score, solve_heuristic};

#[test]
fn optimizes_without_enumerating_the_search_space() {
    let variables = (0..30).map(|index| Variable {
        name: format!("task_{index}"),
        domain: vec![0, 1],
    }).collect();
    let constraints = (0..30).map(|index| Constraint::PreferValue {
        variable: index,
        value: 1,
        soft_weight: 1,
    }).collect();
    let problem = Problem { variables, constraints };

    let solution = solve_heuristic(&problem, 2_000).unwrap().unwrap();
    assert_eq!(solution.values, vec![1; 30]);
    assert_eq!(solution.score, Score { hard: 0, soft: 0 });
    assert_eq!(score(&problem, &solution.values).unwrap(), solution.score);
    assert!(solution.evaluated < 2_000);
}

#[test]
fn preserves_hard_score_priority() {
    let problem = Problem {
        variables: vec![
            Variable { name: "a".into(), domain: vec![0, 1] },
            Variable { name: "b".into(), domain: vec![0, 1] },
        ],
        constraints: vec![
            Constraint::AllDifferent { variables: vec![0, 1], hard_weight: 1 },
            Constraint::PreferValue { variable: 0, value: 0, soft_weight: 100 },
            Constraint::PreferValue { variable: 1, value: 0, soft_weight: 100 },
        ],
    };
    let solution = solve_heuristic(&problem, 20).unwrap().unwrap();
    assert_eq!(solution.score.hard, 0);
    assert_eq!(score(&problem, &solution.values).unwrap(), solution.score);
}

#[test]
fn respects_evaluation_budget() {
    let problem = Problem {
        variables: vec![Variable { name: "a".into(), domain: vec![0, 1] }],
        constraints: vec![],
    };
    assert!(solve_heuristic(&problem, 0).unwrap().is_none());
    assert!(solve_heuristic(&problem, 1).unwrap().unwrap().evaluated <= 1);
}
