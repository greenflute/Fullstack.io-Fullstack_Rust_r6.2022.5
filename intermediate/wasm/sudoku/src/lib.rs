use wasm_bindgen::prelude::*;

#[no_mangle]
static mut BOARD: [u8; SIZE] = [0; SIZE];

#[wasm_bindgen]
pub fn solve() -> bool {
    unsafe {
        let mut cells = Cells::new(&BOARD);
        cells.solve().is_ok()
    }
}

const SIZE: usize = 9 * 9;

pub struct Board([u8; SIZE]);

impl Board {
    pub fn new(board: [u8; SIZE]) -> Self {
        Board(board)
    }

    pub fn solve(self) -> Result<Self, ()> {
        let mut cells = Cells::new(&self.0);
        cells.solve()?;
        Ok(cells.as_board())
    }
}

#[derive(Clone, Copy, Debug)]
enum Cell {
    Fixed(u8),
    Guess(u8),
    Hole,
}

impl Cell {
    #[inline]
    fn as_val(self) -> u8 {
        match self {
            Cell::Fixed(x) | Cell::Guess(x) => x,
            Cell::Hole => 0,
        }
    }
}

struct Cells {
    inner: [Cell; SIZE],
    holes: [usize; SIZE],
    hole_count: usize,
}

impl Cells {
    fn new(board: &[u8; SIZE]) -> Self {
        let mut cells = [Cell::Hole; SIZE];
        let mut holes = [0; SIZE];
        let mut hole_count = 0;
        for r in 0..9 {
            for c in 0..9 {
                if board[9 * r + c] > 0 {
                    cells[9 * r + c] = Cell::Fixed(board[9 * r + c]);
                } else {
                    holes[hole_count] = 9 * r + c;
                    hole_count += 1;
                }
            }
        }
        Cells {
            inner: cells,
            holes,
            hole_count,
        }
    }

    fn solve(&mut self) -> Result<(), ()> {
        let mut hole_index = 0;
        while hole_index < self.hole_count {
            hole_index = self.update_hole(hole_index)?;
        }
        Ok(())
    }

    fn update_hole(&mut self, idx: usize) -> Result<usize, ()> {
        let pos = self.holes[idx];
        let cell = self.inner[pos];
        let valid = self.get_valid(pos);
        let mut value = cell.as_val() + 1;
        while value <= 9 && !valid[value as usize] {
            value += 1;
        }
        if value > 9 {
            if idx == 0 {
                return Err(());
            }
            self.inner[pos] = Cell::Hole;
            Ok(idx - 1)
        } else {
            self.inner[pos] = Cell::Guess(value);
            Ok(idx + 1)
        }
    }

    fn as_board(self) -> Board {
        let mut board = [0; SIZE];
        for r in 0..9 {
            for c in 0..9 {
                board[9 * r + c] = self.inner[9 * r + c].as_val();
            }
        }
        Board(board)
    }

    fn get_valid(&self, pos: usize) -> [bool; 10] {
        let mut result = [true; 10];
        let row_idx = pos / 9;
        let col_idx = pos % 9;
        let corners = [0usize, 3, 6, 27, 30, 33, 54, 57, 60];
        let corner = corners[(row_idx / 3) * 3 + (col_idx / 3)];

        for x in 0..9 {
            result[self.inner[(9 * x + col_idx)].as_val() as usize] = false;
            result[self.inner[(9 * row_idx + x)].as_val() as usize] = false;
        }
        for r in 0..3 {
            for c in 0..3 {
                result[self.inner[corner + 9 * r + c].as_val() as usize] = false;
            }
        }
        result
    }
}

impl PartialEq for Board {
    fn eq(&self, other: &Board) -> bool {
        for r in 0..9 {
            for c in 0..9 {
                if self.0[9 * r + c] != other.0[9 * r + c] {
                    return false;
                }
            }
        }
        true
    }
}

impl std::fmt::Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for r in 0..9 {
            for c in 0..9 {
                write!(f, "{}", self.0[9 * r + c])?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Board;

    #[test]
    fn it_works() {
        let board = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 4, 5, 6, 7, 8, 9, 1, 2, 3, 7, 8, 9, 1, 2, 3, 4, 5, 6, 2, 3,
            4, 5, 6, 7, 8, 9, 1, 5, 6, 7, 8, 9, 1, 2, 3, 4, 8, 9, 1, 2, 3, 4, 5, 6, 7, 3, 4, 5, 6,
            7, 8, 9, 1, 2, 6, 7, 8, 9, 1, 2, 3, 4, 5, 9, 1, 2, 3, 4, 5, 6, 7, 8,
        ];
        let board2 = board.clone();
        assert_eq!(Board(board).solve(), Ok(Board(board2)));
    }

    #[test]
    fn it_solves() {
        let board = [
            0, 2, 3, 4, 5, 6, 7, 8, 9, 4, 5, 6, 7, 8, 9, 1, 2, 3, 7, 8, 9, 1, 0, 3, 4, 5, 6, 2, 3,
            4, 5, 6, 7, 8, 9, 1, 5, 6, 7, 8, 9, 1, 2, 3, 4, 8, 9, 1, 2, 3, 4, 5, 6, 7, 3, 4, 5, 6,
            7, 8, 9, 1, 2, 6, 7, 8, 9, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 6, 7, 0,
        ];
        let exp = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 4, 5, 6, 7, 8, 9, 1, 2, 3, 7, 8, 9, 1, 2, 3, 4, 5, 6, 2, 3,
            4, 5, 6, 7, 8, 9, 1, 5, 6, 7, 8, 9, 1, 2, 3, 4, 8, 9, 1, 2, 3, 4, 5, 6, 7, 3, 4, 5, 6,
            7, 8, 9, 1, 2, 6, 7, 8, 9, 1, 2, 3, 4, 5, 9, 1, 2, 3, 4, 5, 6, 7, 8,
        ];
        assert_eq!(Board(board).solve(), Ok(Board(exp)));
    }

    #[test]
    fn it_solves_hard() {
        let board = [
            1, 0, 0, 8, 7, 5, 6, 0, 0, 0, 0, 0, 0, 0, 1, 9, 5, 8, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 2,
            0, 7, 0, 0, 0, 0, 6, 0, 0, 0, 2, 4, 6, 0, 0, 0, 4, 0, 0, 0, 0, 3, 0, 7, 0, 0, 9, 0, 0,
            0, 0, 0, 0, 0, 3, 6, 7, 5, 0, 0, 0, 0, 0, 0, 0, 1, 6, 8, 7, 0, 0, 4,
        ];
        let exp = [
            1, 4, 9, 8, 7, 5, 6, 2, 3, 7, 3, 2, 4, 6, 1, 9, 5, 8, 6, 8, 5, 3, 2, 9, 4, 1, 7, 9, 2,
            3, 7, 1, 8, 5, 4, 6, 5, 7, 8, 2, 4, 6, 1, 3, 9, 4, 1, 6, 9, 5, 3, 8, 7, 2, 8, 9, 4, 1,
            3, 2, 7, 6, 5, 3, 6, 7, 5, 9, 4, 2, 8, 1, 2, 5, 1, 6, 8, 7, 3, 9, 4,
        ];
        assert_eq!(Board(board).solve(), Ok(Board(exp)));
    }

    #[test]
    fn it_solves_evil() {
        let board = [
            0, 0, 0, 0, 0, 6, 0, 0, 0, 0, 5, 9, 0, 0, 0, 0, 0, 8, 2, 0, 0, 0, 0, 8, 0, 0, 0, 0, 4,
            5, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 6, 0, 0, 3, 0, 5, 4, 0, 0, 0, 3,
            2, 5, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        let exp = [
            1, 3, 8, 2, 4, 6, 5, 7, 9, 6, 5, 9, 1, 3, 7, 2, 4, 8, 2, 7, 4, 5, 9, 8, 1, 6, 3, 7, 4,
            5, 6, 8, 2, 3, 9, 1, 8, 1, 3, 4, 5, 9, 6, 2, 7, 9, 2, 6, 7, 1, 3, 8, 5, 4, 4, 8, 7, 3,
            2, 5, 9, 1, 6, 3, 6, 2, 9, 7, 1, 4, 8, 5, 5, 9, 1, 8, 6, 4, 7, 3, 2,
        ];
        assert_eq!(Board(board).solve(), Ok(Board(exp)));
    }
}
