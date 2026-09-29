//! Game mechanics and procedural generation constants.

/// Grid dimension for procedural nebula generation (16x16 layout).
/// Rationale: Fits within Soroban WASM memory constraints while offering 256 unique tiles per sector.
/// Unit: Grid coordinate count (tiles).
pub const NEBULA_GRID_SIZE: u32 = 16;

/// Total cells in a single nebula grid section (16 * 16 = 256).
/// Rationale: Product of width and height of the standard grid layout.
/// Unit: Cell count.
pub const NEBULA_TOTAL_CELLS: u32 = 256;

/// Maximum allowable upgrade level for ship NFTs.
/// Rationale: Prevents numerical overflow in attribute scaling polynomials and maintains combat balance.
/// Unit: Level index (1-based scalar).
pub const SHIP_MAX_UPGRADE_LEVEL: u32 = 50;

/// Base damage value for level 1 ship weapons.
/// Rationale: Standard baseline for ship-to-ship combat calculations.
/// Unit: Hit points (HP).
pub const BASE_WEAPON_DAMAGE: u64 = 100;

/// Maximum region ID supported for space exploration coordinates.
/// Rationale: Prevents integer overflow in coordinate offsets.
/// Unit: Region ID index.
pub const MAX_EXPLORATION_REGION_ID: u64 = 10_000;
