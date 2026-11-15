use crate::{Error, Problem, Solution, score, validate};

pub fn solve_heuristic(problem: &Problem, max_evaluations: u64) -> Result<Option<Solution>, Error> {
    validate(problem)?;

    if max_evaluations == 0 {
        return Ok(None);
    }

    let mut values: Vec<i64> = problem.variables.iter().map(|variable| variable.domain[0]).collect();
    let mut current = score(problem, &values)?;
    let mut best_values = values.clone();
    let mut best_score = current;
    let mut evaluated = 1_u64;

    while evaluated < max_evaluations {
        let mut selected: Option<(usize, i64, crate::Score)> = None;

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

                if candidate_score.better_than(current)
                    && selected.as_ref().is_none_or(|(_, _, chosen)| candidate_score.better_than(*chosen))
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

        if current.better_than(best_score) {
            best_values.clone_from(&values);
            best_score = current;
        }
    }

    Ok(Some(Solution {
        values: best_values,
        score: best_score,
        evaluated,
    }))
}
