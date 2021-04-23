extern crate fixedbitset;
extern crate rand;
extern crate web_sys;
mod utils;
mod webgl;

use fixedbitset::FixedBitSet;
use rand::distributions::{Bernoulli, Distribution};
use rand::thread_rng;
use std::default::Default;
use utils::Timer;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn set_panic_hook() {
    utils::set_panic_hook();
}

#[wasm_bindgen]
#[derive(Default)]
pub struct Universe {
    width: u32,
    height: u32,
    cells: FixedBitSet,
    _next: FixedBitSet,
}

#[wasm_bindgen]
pub struct WebGLRenderer {
    canvas: HtmlCanvasElement,
    universe: Universe,
    texture: Vec<u8>,
}

impl Universe {
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        let north = if row == 0 { self.height - 1 } else { row - 1 };
        let south = if row == self.height - 1 { 0 } else { row + 1 };

        let west = if column == 0 {
            self.width - 1
        } else {
            column - 1
        };

        let east = if column == self.width - 1 {
            0
        } else {
            column + 1
        };

        let nw = self.get_index(north, west);
        count += self.cells[nw] as u8;

        let n = self.get_index(north, column);
        count += self.cells[n] as u8;

        let ne = self.get_index(north, east);
        count += self.cells[ne] as u8;

        let w = self.get_index(row, west);
        count += self.cells[w] as u8;

        let e = self.get_index(row, east);
        count += self.cells[e] as u8;

        let sw = self.get_index(south, west);
        count += self.cells[sw] as u8;

        let s = self.get_index(south, column);
        count += self.cells[s] as u8;

        let se = self.get_index(south, east);
        count += self.cells[se] as u8;
        count
    }

    /// Set cells to be alive in a universe by passing the row and column
    /// of each cell as an array.
    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        self.cells.clear();
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells.set(idx, true);
        }
    }
}

use std::fmt;

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..self.width * self.height {
            let symbol = if !self.cells[i as usize] {
                '◻'
            } else {
                '◼'
            };
            write!(f, "{}", symbol)?;
        }
        writeln!(f)?;
        Ok(())
    }
}

#[wasm_bindgen]
impl Universe {
    pub fn tick(&mut self) {
        let _timer = Timer::new("Universe::tick");
        {
            let _timer = Timer::new("new generation");
            for row in 0..self.height {
                for col in 0..self.width {
                    let idx = self.get_index(row, col);
                    let cell = self.cells[idx];
                    let live_neighbors = self.live_neighbor_count(row, col);

                    let next_cell = match (cell, live_neighbors) {
                        // Rule 1: Any live cell with fewer than two live neighbours
                        // dies, as if caused by underpopulation.
                        (true, x) if x < 2 => false,
                        // Rule 2: Any live cell with two or three live neighbours
                        // lives on to the next generation.
                        (true, 2) | (true, 3) => true,
                        // Rule 3: Any live cell with more than three live
                        // neighbours dies, as if by overpopulation.
                        (true, x) if x > 3 => false,
                        // Rule 4: Any dead cell with exactly three live neighbours
                        // becomes a live cell, as if by reproduction.
                        (false, 3) => true,
                        // All other cells remain in the same state.
                        (otherwise, _) => otherwise,
                    };

                    self._next.set(idx, next_cell);
                }
            }
        }
        let _timer = Timer::new("swap cells");
        std::mem::swap(&mut self.cells, &mut self._next);
    }

    pub fn new(width: u32, height: u32) -> Universe {
        Universe {
            width,
            height,
            cells: Universe::create_cells(width, height),
            _next: FixedBitSet::with_capacity((width * height) as usize),
        }
    }

    fn create_cells(width: u32, height: u32) -> FixedBitSet {
        let mut rng = thread_rng();
        let size = (width * height) as usize;
        let d = Bernoulli::new(0.5).unwrap();
        let mut cells = FixedBitSet::with_capacity(size);
        (0..width * height).for_each(|i| cells.set(i as usize, d.sample(&mut rng)));
        cells
    }

    pub fn reset(&mut self) {
        log!("Resetting Universe");
        self.cells = Universe::create_cells(self.width, self.height);
    }

    pub fn kill_all(&mut self) {
        log!("Clearing Universe");
        self.cells.clear();
    }

    pub fn render(&self) -> String {
        self.to_string()
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn cells(&self) -> *const u32 {
        self.cells.as_slice().as_ptr()
    }

    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        log!("Toggling Cell at: ({}, {})", row, column);
        let idx = self.get_index(row, column);
        self.cells.toggle(idx);
    }

    pub fn is_alive(&self, idx: usize) -> bool {
        self.cells.contains(idx)
    }

    pub fn insert_pulsar(&mut self, row: u32, column: u32) {
        log!("Inserting Pulsar at: ({}, {})", row, column);
        for (delta_row, delta_col, value) in [
            (self.height - 4, 1, true),
            (self.height - 3, 1, true),
            (self.height - 2, 1, true),
            (self.height - 1, 1, false),
            (0, 1, false),
            (1, 1, false),
            (2, 1, true),
            (3, 1, true),
            (4, 1, true),
            (self.height - 4, 0, true),
            (self.height - 3, 0, false),
            (self.height - 2, 0, true),
            (self.height - 1, 0, false),
            (0, 0, false),
            (1, 0, false),
            (2, 0, true),
            (3, 0, false),
            (4, 0, true),
            (self.height - 4, self.width - 1, true),
            (self.height - 3, self.width - 1, true),
            (self.height - 2, self.width - 1, true),
            (self.height - 1, self.width - 1, false),
            (0, self.width - 1, false),
            (1, self.width - 1, false),
            (2, self.width - 1, true),
            (3, self.width - 1, true),
            (4, self.width - 1, true),
        ]
        .iter()
        .cloned()
        {
            let neighbor_row = (row + delta_row) % self.height;
            let neighbor_col = (column + delta_col) % self.width;
            let idx = self.get_index(neighbor_row, neighbor_col);
            self.cells.set(idx, value);
        }
    }

    pub fn insert_glider(&mut self, row: u32, column: u32) {
        log!("Inserting Glider at: ({}, {})", row, column);
        for (delta_row, delta_col, value) in [
            (self.height - 1, 1, true),
            (0, 1, true),
            (1, 1, true),
            (self.height - 1, 0, true),
            (0, 0, false),
            (1, 0, false),
            (self.height - 1, self.width - 1, false),
            (0, self.width - 1, true),
            (1, self.width - 1, false),
        ]
        .iter()
        .cloned()
        {
            let neighbor_row = (row + delta_row) % self.height;
            let neighbor_col = (column + delta_col) % self.width;
            let idx = self.get_index(neighbor_row, neighbor_col);
            self.cells.set(idx, value);
        }
    }
}
