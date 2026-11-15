use crate::{Error, Problem, Solution, score, validate};

pub fn solve_tabu(
    problem: &Problem,
    max_evaluations: u64,
    tenure: u64,
) -> Result<Option<Solution>, Error> {
    validate(problem)?;
    if max_evaluations == 0 {
        return Ok(None);
    }

    let mut values: Vec<i64> = problem.variables.iter().map(|variable| variable.domain[0]).collect();
    let mut current = score(problem, &values)?;
    let mut best = Solution {
        values: values.clone(),
        score: current,
        evaluated: 1,
    };
    let mut evaluated = 1_u64;
    let mut step = 0_u64;
    let mut tabu_until = vec![0_u64; values.len()];

    while evaluated < max_evaluations {
        let mut selected = None;

        for (index, variable) in problem.variables.iter().enumerate() {
            let original = values[index];
            for &candidate in &variable.domain {
                if candidate == original {
                    continue;
                }

                values[index] = candidate;
                let candidate_score = score(problem, &values);
                values[index] = original;
                let candidate_score = candidate_score?;
                evaluated += 1;

                let permitted = step >= tabu_until[index]
                    || candidate_score.better_than(best.score);

                if permitted
                    && selected.as_ref().is_none_or(
                        |(_, _, chosen): &(usize, i64, crate::Score)| {
                            candidate_score.better_than(*chosen)
                        },
                    )
                {
                    selected = Some((index, candidate, candidate_score));
                }

                if evaluated >= max_evaluations {
                    break;
                }
            }
            if evaluated >= max_evaluations {
                break;
            }
        }

        let Some((index, candidate, next_score)) = selected else {
            break;
        };
        values[index] = candidate;
        current = next_score;
        step += 1;
        tabu_until[index] = step.saturating_add(tenure);

        if current.better_than(best.score) {
            best.values.clone_from(&values);
            best.score = current;
        }
    }

    best.evaluated = evaluated;
    Ok(Some(best))
}
