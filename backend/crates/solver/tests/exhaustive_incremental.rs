use orchestrata_solver::{
    CompiledProblem, Constraint, Error, IncrementalScorer, Problem, Score, SearchOutcome,
    SearchStop, Variable, search_locally, verify_solution,
};

#[test]
fn every_tiny_assignment_and_trial_matches_pairwise_arithmetic() {
    for weight in [1, i64::MAX] {
        let problem = Problem {
            variables: (0..3).map(|i| variable(i, &[-1, 0, 1, 1])).collect(),
            constraints: vec![
                Constraint::AllDifferent {
                    variables: vec![0, 1, 0, 2],
                    hard_weight: weight,
                },
                Constraint::NotEqualOffset {
                    left: 0,
                    right: 1,
                    offset: 1,
                    hard_weight: weight,
                },
                Constraint::NotEqualOffset {
                    left: 1,
                    right: 2,
                    offset: i64::MAX,
                    hard_weight: weight,
                },
                Constraint::PreferValue {
                    variable: 0,
                    value: 1,
                    soft_weight: weight,
                },
                Constraint::PreferValue {
                    variable: 2,
                    value: -1,
                    soft_weight: weight,
                },
            ],
        };
        let compiled = CompiledProblem::new(&problem).unwrap();
        for a in -1..=1 {
            for b in -1..=1 {
                for c in -1..=1 {
                    let values = [a, b, c];
                    let mut scorer = IncrementalScorer::new(&compiled, &values).unwrap();
                    assert_eq!(scorer.score(), pairwise_score(&values, weight));
                    for index in 0..3 {
                        for candidate in [-1, 0, 1, 1] {
                            let mut changed = values;
                            changed[index] = candidate;
                            {
                                let trial = scorer.try_move(index, candidate).unwrap();
                                assert_eq!(trial.values(), changed);
                                assert_eq!(trial.score(), pairwise_score(&changed, weight));
                            }
                            assert_eq!(scorer.values(), values);
                            assert_eq!(scorer.score(), pairwise_score(&values, weight));
                        }
                    }
                    let mut changed = values;
                    changed[0] = 1;
                    scorer.try_move(0, 1).unwrap().commit();
                    assert_eq!(scorer.values(), changed);
                    assert_eq!(scorer.score(), pairwise_score(&changed, weight));
                }
            }
        }
    }
}

fn pairwise_score(values: &[i64; 3], weight: i64) -> Result<Score, Error> {
    let occurrences = [values[0], values[1], values[0], values[2]];
    let mut collisions = 0_i128;
    for left in 0..occurrences.len() {
        for right in left + 1..occurrences.len() {
            collisions += i128::from(occurrences[left] == occurrences[right]);
        }
    }
    collisions += i128::from(i128::from(values[0]) - i128::from(values[1]) == 1);
    collisions += i128::from(i128::from(values[1]) - i128::from(values[2]) == i128::from(i64::MAX));
    let misses = i128::from(values[0] != 1) + i128::from(values[2] != -1);
    Ok(Score {
        hard: i64::try_from(-collisions * i128::from(weight)).map_err(|_| Error::ScoreOverflow)?,
        soft: i64::try_from(-misses * i128::from(weight)).map_err(|_| Error::ScoreOverflow)?,
    })
}

#[test]
fn bounded_search_incumbents_belong_to_the_exhaustive_feasible_set() {
    let problem = Problem {
        variables: vec![variable(0, &[0, 1, 1]), variable(1, &[0, 1, 1])],
        constraints: vec![
            Constraint::AllDifferent {
                variables: vec![0, 1],
                hard_weight: 3,
            },
            Constraint::PreferValue {
                variable: 0,
                value: 1,
                soft_weight: 2,
            },
        ],
    };
    let feasible: Vec<_> = (0..=1)
        .flat_map(|a| {
            (0..=1).filter_map(move |b| {
                (a != b).then_some((
                    vec![a, b],
                    Score {
                        hard: 0,
                        soft: if a == 1 { 0 } else { -2 },
                    },
                ))
            })
        })
        .collect();
    for budget in 0..=20 {
        match search_locally(&problem, budget).unwrap() {
            SearchOutcome::FeasibleIncumbent { solution, stop } => {
                assert!(feasible.contains(&(solution.values.clone(), solution.score)));
                assert!(solution.evaluated <= budget);
                assert_eq!(verify_solution(&problem, &solution), Ok(()));
                if stop == SearchStop::EvaluationLimit {
                    assert_eq!(solution.evaluated, budget);
                }
            }
            SearchOutcome::NoFeasibleIncumbent { evaluated, stop } => {
                assert!(budget <= 1);
                assert_eq!(evaluated, budget);
                assert_eq!(stop, SearchStop::EvaluationLimit);
                assert!(!feasible.is_empty());
            }
        }
    }
}

fn variable(index: usize, domain: &[i64]) -> Variable {
    Variable {
        name: format!("v{index}"),
        domain: domain.to_vec(),
    }
}
