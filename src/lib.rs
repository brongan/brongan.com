extern crate fixedbitset;
extern crate rand;
extern crate web_sys;
mod utils;

use rand::distributions::{Bernoulli, Distribution};
use rand::thread_rng;
use std::default::Default;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

pub struct Timer<'a> {
    name: &'a str,
}

impl<'a> Timer<'a> {
    pub fn new(name: &'a str) -> Timer<'a> {
        web_sys::console::time_with_label(name);
        Timer { name }
    }
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        web_sys::console::time_end_with_label(self.name);
    }
}

#[wasm_bindgen]
pub fn set_panic_hook() {
    utils::set_panic_hook();
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Alive,
    Dead,
}

impl Cell {
    fn toggle(&mut self) {
        *self = match *self {
            Cell::Dead => Cell::Alive,
            Cell::Alive => Cell::Dead,
        };
    }
}

#[wasm_bindgen]
#[derive(Default)]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
    _next: Vec<Cell>,
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

    /// Get the dead and alive values of the entire universe.
    pub fn get_cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }

    /// Set cells to be alive in a universe by passing the row and column
    /// of each cell as an array.
    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        self.cells.clear();
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells[idx] = Cell::Alive;
        }
    }
}

use std::fmt;

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..self.width * self.height {
            let symbol = match self.cells[i as usize] {
                Cell::Alive => '◼',
                Cell::Dead => '◻',
            };
            write!(f, "{}", symbol)?;
        }
        writeln!(f)?;
        Ok(())
    }
}

/// Public methods, exported to JavaScript.
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
                        (Cell::Alive, x) if x < 2 => Cell::Dead,
                        // Rule 2: Any live cell with two or three live neighbours
                        // lives on to the next generation.
                        (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                        // Rule 3: Any live cell with more than three live
                        // neighbours dies, as if by overpopulation.
                        (Cell::Alive, x) if x > 3 => Cell::Dead,
                        // Rule 4: Any dead cell with exactly three live neighbours
                        // becomes a live cell, as if by reproduction.
                        (Cell::Dead, 3) => Cell::Alive,
                        // All other cells remain in the same state.
                        (otherwise, _) => otherwise,
                    };

                    self._next[idx] = next_cell;
                }
            }
        }
        //        let _timer = Timer::new("swap cells");
        std::mem::swap(&mut self.cells, &mut self._next);
    }

    pub fn new(width: u32, height: u32) -> Universe {
        let size = (width * height) as usize;
        Universe {
            width,
            height,
            cells: Universe::create_cells(size),
            _next: Vec::with_capacity(size),
        }
    }

    fn create_cells(size: usize) -> Vec<Cell> {
        let mut rng = thread_rng();
        let d = Bernoulli::new(0.5).unwrap();
        (0..size)
            .map(|_| match d.sample(&mut rng) {
                true => Cell::Alive,
                false => Cell::Dead,
            })
            .collect()
    }

    pub fn reset(&mut self) {
        log!("Resetting Universe");
        self.cells = Universe::create_cells((self.width * self.height) as usize);
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

    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        log!("Toggling Cell at: ({}, {})", row, column);
        let idx = self.get_index(row, column);
        self.cells[idx].toggle();
    }

    pub fn insert_pulsar(&mut self, row: u32, column: u32) {
        log!("Inserting Pulsar at: ({}, {})", row, column);
        for (delta_row, delta_col, value) in [
            (self.height - 4, 1, Cell::Alive),
            (self.height - 3, 1, Cell::Alive),
            (self.height - 2, 1, Cell::Alive),
            (self.height - 1, 1, Cell::Dead),
            (0, 1, Cell::Dead),
            (1, 1, Cell::Dead),
            (2, 1, Cell::Alive),
            (3, 1, Cell::Alive),
            (4, 1, Cell::Alive),
            (self.height - 4, 0, Cell::Alive),
            (self.height - 3, 0, Cell::Dead),
            (self.height - 2, 0, Cell::Alive),
            (self.height - 1, 0, Cell::Dead),
            (0, 0, Cell::Dead),
            (1, 0, Cell::Dead),
            (2, 0, Cell::Alive),
            (3, 0, Cell::Dead),
            (4, 0, Cell::Alive),
            (self.height - 4, self.width - 1, Cell::Alive),
            (self.height - 3, self.width - 1, Cell::Alive),
            (self.height - 2, self.width - 1, Cell::Alive),
            (self.height - 1, self.width - 1, Cell::Dead),
            (0, self.width - 1, Cell::Dead),
            (1, self.width - 1, Cell::Dead),
            (2, self.width - 1, Cell::Alive),
            (3, self.width - 1, Cell::Alive),
            (4, self.width - 1, Cell::Alive),
        ]
        .iter()
        .cloned()
        {
            let neighbor_row = (row + delta_row) % self.height;
            let neighbor_col = (column + delta_col) % self.width;
            let idx = self.get_index(neighbor_row, neighbor_col);
            self.cells[idx] = value;
        }
    }

    pub fn insert_glider(&mut self, row: u32, column: u32) {
        log!("Inserting Glider at: ({}, {})", row, column);
        for (delta_row, delta_col, value) in [
            (self.height - 1, 1, Cell::Alive),
            (0, 1, Cell::Alive),
            (1, 1, Cell::Alive),
            (self.height - 1, 0, Cell::Alive),
            (0, 0, Cell::Dead),
            (1, 0, Cell::Dead),
            (self.height - 1, self.width - 1, Cell::Dead),
            (0, self.width - 1, Cell::Alive),
            (1, self.width - 1, Cell::Dead),
        ]
        .iter()
        .cloned()
        {
            let neighbor_row = (row + delta_row) % self.height;
            let neighbor_col = (column + delta_col) % self.width;
            let idx = self.get_index(neighbor_row, neighbor_col);
            self.cells[idx] = value;
        }
    }
}
