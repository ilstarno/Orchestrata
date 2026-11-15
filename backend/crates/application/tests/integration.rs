use orchestrata_application::{solve_error::SolveError, solve_service::SolveService};
use orchestrata_solver::{Constraint, Problem, Score, Variable, score};

fn assignment() -> Problem {
    Problem {
        variables: vec![
            Variable { name: "first".into(), domain: vec![0, 1] },
            Variable { name: "second".into(), domain: vec![0, 1] },
        ],
        constraints: vec![
            Constraint::AllDifferent { variables: vec![0, 1], hard_weight: 1 },
            Constraint::PreferValue { variable: 0, value: 1, soft_weight: 3 },
        ],
    }
}

#[test]
fn service_returns_independently_scored_solution() {
    let problem = assignment();
    let result = SolveService::execute(&problem).unwrap().unwrap();

    assert_eq!(result.values, vec![1, 0]);
    assert_eq!(result.score, Score { hard: 0, soft: 0 });
    assert_eq!(score(&problem, &result.values).unwrap(), result.score);
}

#[test]
fn service_rejects_duplicate_names() {
    let mut problem = assignment();
    problem.variables[1].name = problem.variables[0].name.clone();

    assert_eq!(SolveService::execute(&problem), Err(SolveError::InvalidInput));
}

#[tokio::test]
async fn asynchronous_and_standalone_execution_agree() {
    let problem = assignment();
    let direct = SolveService::execute(&problem).unwrap();
    let asynchronous = SolveService::execute_async(problem).await.unwrap();

    assert_eq!(direct, asynchronous);
}

#[test]
fn service_handles_large_search_spaces() {
    let problem = Problem {
        variables: (0..30).map(|index| Variable {
            name: format!("task_{index}"),
            domain: vec![0, 1],
        }).collect(),
        constraints: (0..30).map(|index| Constraint::PreferValue {
            variable: index,
            value: 1,
            soft_weight: 1,
        }).collect(),
    };

    let solution = SolveService::execute(&problem).unwrap().unwrap();
    assert_eq!(solution.values, vec![1; 30]);
    assert_eq!(score(&problem, &solution.values).unwrap(), solution.score);
}
