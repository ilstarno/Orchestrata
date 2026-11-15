use orchestrata_solver::{Constraint, Problem, Variable, solve_heuristic};

use crate::OrcStatus;

#[repr(C)]
pub struct OrcVariable {
    pub domain: *const i64,
    pub domain_len: usize,
}

#[repr(C)]
pub struct OrcConstraint {
    pub kind: u32,
    pub left: usize,
    pub right: usize,
    pub value: i64,
    pub weight: i64,
}

#[repr(C)]
pub struct OrcScore {
    pub hard: i64,
    pub soft: i64,
}

#[unsafe(no_mangle)]
pub extern "C" fn orc_abi_version() -> u32 {
    1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn orc_solve(
    variables: *const OrcVariable,
    variable_count: usize,
    constraints: *const OrcConstraint,
    constraint_count: usize,
    max_evaluations: u64,
    output: *mut i64,
    output_capacity: usize,
    score: *mut OrcScore,
) -> OrcStatus {
    std::panic::catch_unwind(|| unsafe {
        solve_inner(
            variables,
            variable_count,
            constraints,
            constraint_count,
            max_evaluations,
            output,
            output_capacity,
            score,
        )
    })
    .unwrap_or(OrcStatus::InternalError)
}

unsafe fn solve_inner(
    variables: *const OrcVariable,
    variable_count: usize,
    constraints: *const OrcConstraint,
    constraint_count: usize,
    max_evaluations: u64,
    output: *mut i64,
    output_capacity: usize,
    score: *mut OrcScore,
) -> OrcStatus {
    if variable_count > 0 && variables.is_null()
        || constraint_count > 0 && constraints.is_null()
        || output_capacity < variable_count
        || variable_count > 0 && output.is_null()
        || score.is_null()
        || variable_count > isize::MAX as usize / size_of::<OrcVariable>()
        || constraint_count > isize::MAX as usize / size_of::<OrcConstraint>()
    {
        return OrcStatus::InvalidArgument;
    }

    let raw_variables = if variable_count == 0 {
        &[][..]
    } else {
        unsafe { std::slice::from_raw_parts(variables, variable_count) }
    };
    let raw_constraints = if constraint_count == 0 {
        &[][..]
    } else {
        unsafe { std::slice::from_raw_parts(constraints, constraint_count) }
    };

    let mut parsed_variables = Vec::with_capacity(variable_count);
    for (index, variable) in raw_variables.iter().enumerate() {
        if variable.domain_len == 0
            || variable.domain.is_null()
            || variable.domain_len > isize::MAX as usize / size_of::<i64>()
        {
            return OrcStatus::InvalidProblem;
        }
        let domain = unsafe { std::slice::from_raw_parts(variable.domain, variable.domain_len) };
        parsed_variables.push(Variable {
            name: index.to_string(),
            domain: domain.to_vec(),
        });
    }

    let mut parsed_constraints = Vec::with_capacity(constraint_count);
    for constraint in raw_constraints {
        let parsed = match constraint.kind {
            1 => Constraint::NotEqualOffset {
                left: constraint.left,
                right: constraint.right,
                offset: constraint.value,
                hard_weight: constraint.weight,
            },
            2 => Constraint::PreferValue {
                variable: constraint.left,
                value: constraint.value,
                soft_weight: constraint.weight,
            },
            _ => return OrcStatus::InvalidProblem,
        };
        parsed_constraints.push(parsed);
    }

    let problem = Problem {
        variables: parsed_variables,
        constraints: parsed_constraints,
    };
    let solution = match solve_heuristic(&problem, max_evaluations) {
        Ok(Some(solution)) => solution,
        Ok(None) => return OrcStatus::ExecutionFailed,
        Err(_) => return OrcStatus::InvalidProblem,
    };

    if variable_count > 0 {
        unsafe {
            std::ptr::copy_nonoverlapping(solution.values.as_ptr(), output, variable_count);
        }
    }
    unsafe {
        score.write(OrcScore {
            hard: solution.score.hard,
            soft: solution.score.soft,
        });
    }
    OrcStatus::Ok
}
