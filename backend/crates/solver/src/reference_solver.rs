use crate::{Error, Problem, Score, Solution, score, validate};

pub fn solve(problem: &Problem, max_evaluations: u64) -> Result<Option<Solution>, Error> {
    validate(problem)?;

    let mut combinations = 1_u64;
    for variable in &problem.variables {
        let size = u64::try_from(variable.domain.len()).map_err(|_| Error::TooManyAssignments)?;
        combinations = combinations.checked_mul(size).ok_or(Error::TooManyAssignments)?;
    }

    if combinations > max_evaluations {
        return Err(Error::TooManyAssignments);
    }

    let mut values = vec![0; problem.variables.len()];
    let mut positions = vec![0_usize; problem.variables.len()];
    let mut best: Option<Solution> = None;
    let mut evaluated = 0_u64;

    loop {
        for (index, variable) in problem.variables.iter().enumerate() {
            values[index] = variable.domain[positions[index]];
        }

        evaluated += 1;
        let current = score(problem, &values)?;
        if best.as_ref().is_none_or(|old| current.better_than(old.score)) {
            best = Some(Solution {
                values: values.clone(),
                score: current,
                evaluated,
            });
        }

        let mut next = positions.len();
        while next > 0 {
            next -= 1;
            positions[next] += 1;
            if positions[next] < problem.variables[next].domain.len() {
                break;
            }
            positions[next] = 0;
        }

        if next == 0 && positions[0] == 0 {
            break;
        }
    }

    if let Some(solution) = &mut best {
        solution.evaluated = evaluated;
    }

    Ok(best)
}
