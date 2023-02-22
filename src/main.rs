use std::fmt::Display;

use grid::Grid;
use itertools::Itertools;

fn main() {
    println!("{}", Board::random(8, 8));
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    grid: Grid<bool>,
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (row, col) in (0..self.grid.rows()).cartesian_product(0..self.grid.cols()) {
            write!(f, "{}", self.grid[row][col] as u32).unwrap();
            if col == self.grid.cols() - 1 {
                write!(f, "\n").unwrap();
            }
        }
        Ok(())
    }
}

impl Board {
    pub fn dead(rows: usize, cols: usize) -> Self {
        Board { grid: Grid::init(rows, cols, false) }
    }

    /// Creates a board of `rows` x `cols` with every cell initialized randomly.
    /// 
    /// `rows * cols` must be less than `usize::MAX`
    pub fn random(rows: usize, cols: usize) -> Self {
        let mut grid = Grid::new(rows, cols);
        grid.fill_with(|| rand::random::<bool>());
        Board { grid }
    }

    pub fn advance(&mut self) {
        let (rows, cols) = (self.grid.rows(), self.grid.cols());
        let new_state = (0..rows).cartesian_product(0..cols)
            .map(|(row, col)| match &self.count_live_neighbors(row, col) {
                0..=1 => false,                 // if alive, becomes dead; if dead, stays dead
                2 => *(&self.grid[row][col]),   // unchanged whether originally alive or dead
                3 => true,                      // if alive, stays alive; if dead, becomes alive
                _ => false,                     // more than 3 live neighbors becomes dead
            })
            .collect_vec();
        
        self.grid = Grid::from_vec(new_state, cols);
    }

    fn count_live_neighbors(&self, row: usize, col: usize) -> usize {
        // literally just binding to reduce the times writing self.grid
        let grid = &self.grid;

        // finds valid indices around the given coordinates
        // might be worth breaking off as an option if implementing toroidal board space
        let valid_rows = match row {
            0 => 0..=1usize,
            r if r == grid.rows() - 1 => (r-1)..=r,
            _ => (row - 1)..=(row + 1),
        };
        let valid_cols = match col {
            0 => 0..=1usize,
            r if r == grid.cols() - 1 => (r-1)..=r,
            _ => (col - 1)..=(col + 1),
        };

        valid_rows.cartesian_product(valid_cols)
            .filter(|i| i != &(row, col))
            .map(|(r, c)| grid[r][c])
            .filter(|&c| c)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_board_x_by_x<T: as_bool::AsBool>(vec: Vec<T>, x: usize) -> Board {
        let vec = vec.iter().map(|i| i.as_bool()).collect();
        Board { grid: Grid::from_vec(vec, x) }
    }

    #[test]
    fn dead_0_neighbors_stay_dead() {
        let mut board = create_board_x_by_x(vec![0;9], 3);
        let expected = board.clone();
        board.advance();
        assert_eq!(expected, board);
    }

    #[test]
    fn dead_3_neighbors_come_alive() {
        let mut board = create_board_x_by_x(vec![
            0, 0, 1,
            0, 1, 1,
            0, 0, 0,
        ], 3);

        let expected = create_board_x_by_x(vec![
            0, 1, 1,
            0, 1, 1,
            0, 0, 0,
        ], 3);

        board.advance();
        assert_eq!(expected, board);
    }

    #[test]
    fn alive_4_neighbors_die() {
        let mut board = create_board_x_by_x(vec![
            0, 1, 1,
            0, 1, 1,
            0, 0, 1,
        ], 3);

        let expected = create_board_x_by_x(vec![
            0, 1, 1,
            0, 0, 0,
            0, 1, 1,
        ], 3);

        board.advance();
        assert_eq!(expected, board);
    }

    #[test]
    fn common_still_lifes() {
        let still_lifes = vec![
            // block
            create_board_x_by_x(vec![
                0, 0, 0, 0,
                0, 1, 1, 0,
                0, 1, 1, 0,
                0, 0, 0, 0,
            ], 4),
            
            // beehive
            create_board_x_by_x(vec![
                0, 0, 0, 0, 0, 0,
                0, 0, 1, 1, 0, 0,
                0, 1, 0, 0, 1, 0,
                0, 0, 1, 1, 0, 0,
                0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0,
            ], 6),

            // boat
            create_board_x_by_x(vec![
                0, 0, 0, 0, 0, 
                0, 0, 1, 0, 0, 
                0, 1, 0, 1, 0, 
                0, 1, 1, 0, 0, 
                0, 0, 0, 0, 0, 
            ], 5),

        ];

        for mut board in still_lifes {
            let expected = board.clone();
    
            board.advance();
            assert_eq!(expected, board);
        }
    }

    #[test]
    fn simple_oscillators() {
        let mut blinker = create_board_x_by_x(vec![
            0, 0, 0, 0, 0, 
            0, 0, 1, 0, 0, 
            0, 0, 1, 0, 0, 
            0, 0, 1, 0, 0, 
            0, 0, 0, 0, 0, 
        ], 5);

        let expected = create_board_x_by_x(vec![
            0, 0, 0, 0, 0, 
            0, 0, 0, 0, 0, 
            0, 1, 1, 1, 0, 
            0, 0, 0, 0, 0, 
            0, 0, 0, 0, 0, 
        ], 5);

        blinker.advance();
        assert_eq!(expected, blinker);

        let mut toad = create_board_x_by_x(vec![
            0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
            0, 0, 1, 1, 1, 0,
            0, 1, 1, 1, 0, 0,
            0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
        ], 6);
        
        let expected = create_board_x_by_x(vec![
            0, 0, 0, 0, 0, 0,
            0, 0, 0, 1, 0, 0,
            0, 1, 0, 0, 1, 0,
            0, 1, 0, 0, 1, 0,
            0, 0, 1, 0, 0, 0,
            0, 0, 0, 0, 0, 0,
        ], 6);

        toad.advance();
        assert_eq!(expected, toad);
    }
}
