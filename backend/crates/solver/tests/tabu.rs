use orchestrata_solver::{Constraint, Problem, Score, Variable, score, solve_tabu};

#[test]
fn tabu_search_escapes_a_single_variable_local_optimum() {
    let problem = Problem {
        variables: vec![
            Variable { name: "a".into(), domain: vec![0, 1] },
            Variable { name: "b".into(), domain: vec![0, 1] },
        ],
        constraints: vec![
            Constraint::PreferValue { variable: 0, value: 0, soft_weight: 1 },
            Constraint::PreferValue { variable: 1, value: 0, soft_weight: 1 },
            Constraint::NotEqualOffset { left: 0, right: 1, offset: 0, hard_weight: 10 },
        ],
    };

    let solution = solve_tabu(&problem, 30, 2).unwrap().unwrap();
    assert_eq!(solution.score, Score { hard: 0, soft: -1 });
    assert_eq!(score(&problem, &solution.values).unwrap(), solution.score);
}

#[test]
fn tabu_search_obeys_zero_budget() {
    let problem = Problem { variables: vec![], constraints: vec![] };
    assert!(solve_tabu(&problem, 0, 3).unwrap().is_none());
}
