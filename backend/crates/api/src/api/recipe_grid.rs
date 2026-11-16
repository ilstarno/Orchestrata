use crate::dto::{
    recipe_grid_column::RecipeGridColumn, recipe_grid_column_id::RecipeGridColumnId,
    recipe_grid_direction::RecipeGridDirection, recipe_grid_query::RecipeGridQuery,
    recipe_grid_response::RecipeGridResponse, recipe_grid_sort::RecipeGridSort,
    recipe_metadata_response::RecipeMetadataResponse,
};

const COLUMNS: [RecipeGridColumn; 3] = [
    RecipeGridColumn {
        id: RecipeGridColumnId::Id,
        label: "ID",
        sortable: true,
    },
    RecipeGridColumn {
        id: RecipeGridColumnId::Title,
        label: "Title",
        sortable: true,
    },
    RecipeGridColumn {
        id: RecipeGridColumnId::Scenario,
        label: "Scenario",
        sortable: false,
    },
];

pub(super) fn page(
    query: RecipeGridQuery,
    catalog: &'static [RecipeMetadataResponse],
) -> RecipeGridResponse {
    let filter = query.q.trim().to_lowercase();
    let mut matches: Vec<_> = catalog
        .iter()
        .filter(|recipe| contains_filter(recipe, &filter))
        .collect();

    matches.sort_unstable_by(|left, right| {
        let primary = match query.sort {
            RecipeGridSort::Id => left.id.cmp(&right.id),
            RecipeGridSort::Title => left.title.cmp(&right.title),
        };
        let ordered = match query.direction {
            RecipeGridDirection::Asc => primary,
            RecipeGridDirection::Desc => primary.reverse(),
        };
        ordered.then_with(|| left.id.cmp(&right.id))
    });

    RecipeGridResponse {
        columns: COLUMNS,
        page: query.page,
        page_size: query.page_size,
        total_items: matches.len(),
        items: matches
            .into_iter()
            .skip(query.offset())
            .take(query.page_size as usize)
            .map(Into::into)
            .collect(),
    }
}

fn contains_filter(recipe: &RecipeMetadataResponse, filter: &str) -> bool {
    filter.is_empty()
        || [&recipe.id, &recipe.title, &recipe.scenario]
            .iter()
            .any(|value| value.to_lowercase().contains(filter))
}
