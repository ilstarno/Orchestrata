use serde::Deserialize;
use utoipa::IntoParams;

use super::{recipe_grid_direction::RecipeGridDirection, recipe_grid_sort::RecipeGridSort};

const DEFAULT_PAGE_SIZE: u32 = 25;
const MAX_PAGE_SIZE: u32 = 100;
const MAX_PAGE: u32 = 100_000;
const MAX_FILTER_BYTES: usize = 100;

#[derive(Deserialize, IntoParams)]
#[serde(default, deny_unknown_fields)]
#[into_params(parameter_in = Query)]
pub struct RecipeGridQuery {
    /// One-based page index. Defaults to 1. Pages after the last item are empty.
    #[param(minimum = 1, maximum = 100000, default = 1, required = false)]
    pub page: u32,
    /// Maximum rows returned. Defaults to 25.
    #[param(minimum = 1, maximum = 100, default = 25, required = false)]
    pub page_size: u32,
    /// Allowlisted sort field. Defaults to id.
    #[param(default = RecipeGridSort::default, required = false)]
    pub sort: RecipeGridSort,
    /// Defaults to asc. Rows with equal titles always use ascending ID order.
    #[param(default = RecipeGridDirection::default, required = false)]
    pub direction: RecipeGridDirection,
    /// Literal, case-insensitive substring of ID, title or scenario. At most 100 UTF-8 bytes.
    #[param(max_length = 100, default = "", required = false)]
    pub q: String,
}

impl Default for RecipeGridQuery {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: DEFAULT_PAGE_SIZE,
            sort: RecipeGridSort::default(),
            direction: RecipeGridDirection::default(),
            q: String::new(),
        }
    }
}

impl RecipeGridQuery {
    pub fn is_valid(&self) -> bool {
        (1..=MAX_PAGE).contains(&self.page)
            && (1..=MAX_PAGE_SIZE).contains(&self.page_size)
            && self.q.len() <= MAX_FILTER_BYTES
            && !self.q.chars().any(char::is_control)
    }

    pub fn offset(&self) -> usize {
        ((self.page - 1) * self.page_size) as usize
    }
}
