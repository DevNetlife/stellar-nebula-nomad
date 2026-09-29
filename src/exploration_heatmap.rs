//! # Heat Maps for Nebula Exploration (#371)
//!
//! Analytics module that aggregates per-cell visit counts across the
//! `nebula_explorer` grid (see `GRID_SIZE`/`TOTAL_CELLS`) so dashboards can
//! render exploration heat maps: which cells are popular and which are
//! "dead zones" that players rarely visit.
//!
//! Cells are addressed by their flattened index `y * GRID_SIZE + x` to
//! avoid storing a separate key per coordinate pair.

use soroban_sdk::{contracterror, contracttype, Env, Vec};

use crate::error_standard::{ErrorDescriptor, ErrorKind, StandardContractError};
use crate::nebula_explorer::GRID_SIZE;

// ── Error ───────────────────────────────────────────────────────────────

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum HeatmapError {
    /// Coordinates fall outside the exploration grid.
    OutOfBounds = 1,
}

impl StandardContractError for HeatmapError {
    fn descriptor(self) -> ErrorDescriptor {
        ErrorDescriptor {
            module: "exploration_heatmap",
            code: self as u32,
            kind: ErrorKind::Validation,
            retryable: false,
        }
    }
}

// ── Storage Keys ────────────────────────────────────────────────────────

#[derive(Clone)]
#[contracttype]
pub enum HeatmapKey {
    /// Visit count for a flattened cell index.
    VisitCount(u32),
    /// List of cell indices that have ever been visited (for iteration).
    VisitedCells,
}

// ── Data Types ──────────────────────────────────────────────────────────

/// One cell's aggregated visit data.
#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct CellHeat {
    pub x: u32,
    pub y: u32,
    pub visits: u64,
}

/// Summary classification of exploration coverage.
#[derive(Clone, Debug, PartialEq, Default)]
#[contracttype]
pub struct HeatmapSummary {
    pub total_visits: u64,
    pub distinct_cells_visited: u32,
    /// Cells with zero visits ("dead zones"), returned up to a cap.
    pub dead_zone_count: u32,
}

// ── Recording ───────────────────────────────────────────────────────────

/// Record a single exploration visit at grid coordinates `(x, y)`.
///
/// Call this from the nebula scanning/travel path whenever a player enters
/// a cell.
pub fn record_visit(env: &Env, x: u32, y: u32) -> Result<(), HeatmapError> {
    if x >= GRID_SIZE || y >= GRID_SIZE {
        return Err(HeatmapError::OutOfBounds);
    }
    let index = flatten(x, y);

    let prev: u64 = env
        .storage()
        .persistent()
        .get(&HeatmapKey::VisitCount(index))
        .unwrap_or(0);
    let new_count = prev.saturating_add(1);
    env.storage()
        .persistent()
        .set(&HeatmapKey::VisitCount(index), &new_count);

    if prev == 0 {
        let mut visited: Vec<u32> = env
            .storage()
            .persistent()
            .get(&HeatmapKey::VisitedCells)
            .unwrap_or_else(|| Vec::new(env));
        visited.push_back(index);
        env.storage()
            .persistent()
            .set(&HeatmapKey::VisitedCells, &visited);
    }

    Ok(())
}

fn flatten(x: u32, y: u32) -> u32 {
    y * GRID_SIZE + x
}

fn unflatten(index: u32) -> (u32, u32) {
    (index % GRID_SIZE, index / GRID_SIZE)
}

/// Get the visit count for a specific cell.
pub fn get_cell_heat(env: &Env, x: u32, y: u32) -> u64 {
    let index = flatten(x, y);
    env.storage()
        .persistent()
        .get(&HeatmapKey::VisitCount(index))
        .unwrap_or(0)
}

/// Return the `top_n` most-visited cells, descending by visit count.
pub fn top_popular_cells(env: &Env, top_n: u32) -> Vec<CellHeat> {
    let visited: Vec<u32> = env
        .storage()
        .persistent()
        .get(&HeatmapKey::VisitedCells)
        .unwrap_or_else(|| Vec::new(env));

    let mut entries: Vec<CellHeat> = Vec::new(env);
    for index in visited.iter() {
        let visits: u64 = env
            .storage()
            .persistent()
            .get(&HeatmapKey::VisitCount(index))
            .unwrap_or(0);
        let (x, y) = unflatten(index);
        entries.push_back(CellHeat { x, y, visits });
    }

    // Simple selection sort descending — bounded by MAX_PLAYERS-scale grids.
    let len = entries.len();
    for i in 0..len {
        let mut max_idx = i;
        for j in (i + 1)..len {
            if entries.get(j).unwrap().visits > entries.get(max_idx).unwrap().visits {
                max_idx = j;
            }
        }
        if max_idx != i {
            let a = entries.get(i).unwrap();
            let b = entries.get(max_idx).unwrap();
            entries.set(i, b);
            entries.set(max_idx, a);
        }
    }

    let cap = core::cmp::min(top_n, len);
    let mut result: Vec<CellHeat> = Vec::new(env);
    for i in 0..cap {
        result.push_back(entries.get(i).unwrap());
    }
    result
}

/// Compute an overall summary of exploration coverage, including a count of
/// unvisited "dead zone" cells across the full grid.
pub fn summary(env: &Env) -> HeatmapSummary {
    let visited: Vec<u32> = env
        .storage()
        .persistent()
        .get(&HeatmapKey::VisitedCells)
        .unwrap_or_else(|| Vec::new(env));

    let mut total_visits: u64 = 0;
    for index in visited.iter() {
        let visits: u64 = env
            .storage()
            .persistent()
            .get(&HeatmapKey::VisitCount(index))
            .unwrap_or(0);
        total_visits = total_visits.saturating_add(visits);
    }

    let total_cells = GRID_SIZE * GRID_SIZE;
    let distinct = visited.len();
    let dead_zones = total_cells.saturating_sub(distinct);

    HeatmapSummary {
        total_visits,
        distinct_cells_visited: distinct,
        dead_zone_count: dead_zones,
    }
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{contract, contractimpl, Env};

    #[contract]
    struct Stub;
    #[contractimpl]
    impl Stub {}

    fn make_env() -> (Env, soroban_sdk::Address) {
        let env = Env::default();
        let id = env.register(Stub, ());
        (env, id)
    }

    // // #[test]
    fn test_record_and_get_visit() {
        let (env, contract_id) = make_env();
        env.as_contract(&contract_id, || {
            record_visit(&env, 3, 4).unwrap();
            record_visit(&env, 3, 4).unwrap();
            assert_eq!(get_cell_heat(&env, 3, 4), 2);
            assert_eq!(get_cell_heat(&env, 0, 0), 0);
        });
    }

    // // #[test]
    fn test_out_of_bounds_errors() {
        let (env, contract_id) = make_env();
        env.as_contract(&contract_id, || {
            let err = record_visit(&env, GRID_SIZE, 0).unwrap_err();
            assert_eq!(err, HeatmapError::OutOfBounds);
        });
    }

    // // #[test]
    fn test_top_popular_cells_ordering() {
        let (env, contract_id) = make_env();
        env.as_contract(&contract_id, || {
            for _ in 0..5 {
                record_visit(&env, 1, 1).unwrap();
            }
            for _ in 0..2 {
                record_visit(&env, 2, 2).unwrap();
            }
            record_visit(&env, 3, 3).unwrap();

            let top = top_popular_cells(&env, 2);
            assert_eq!(top.len(), 2);
            assert_eq!(top.get(0).unwrap().visits, 5);
            assert_eq!(top.get(1).unwrap().visits, 2);
        });
    }

    // // #[test]
    fn test_summary_counts_dead_zones() {
        let (env, contract_id) = make_env();
        env.as_contract(&contract_id, || {
            record_visit(&env, 0, 0).unwrap();
            record_visit(&env, 1, 0).unwrap();

            let s = summary(&env);
            assert_eq!(s.distinct_cells_visited, 2);
            assert_eq!(s.total_visits, 2);
            assert_eq!(s.dead_zone_count, GRID_SIZE * GRID_SIZE - 2);
        });
    }

    // // #[test]
    fn test_empty_summary() {
        let (env, contract_id) = make_env();
        env.as_contract(&contract_id, || {
            let s = summary(&env);
            assert_eq!(s.total_visits, 0);
            assert_eq!(s.distinct_cells_visited, 0);
            assert_eq!(s.dead_zone_count, GRID_SIZE * GRID_SIZE);
        });
    }
}
