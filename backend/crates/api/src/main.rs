use axum::{Json, Router, http::StatusCode, routing::{get, post}};
use orchestrata_solver::{Problem, Solution, solve};
use serde::Serialize;

#[derive(Serialize)]
struct Health { status: &'static str }

async fn health() -> Json<Health> { Json(Health { status: "ok" }) }

#[derive(Serialize)]
struct SolveResponse { solution: Option<Solution> }

async fn solve_problem(Json(problem): Json<Problem>)
    -> Result<Json<SolveResponse>, (StatusCode, &'static str)>
{
    let solution = solve(&problem, 100_000)
        .map_err(|_| (StatusCode::UNPROCESSABLE_ENTITY, "Invalid problem or search space too large"))?;
    Ok(Json(SolveResponse { solution }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/solve", post(solve_problem));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
