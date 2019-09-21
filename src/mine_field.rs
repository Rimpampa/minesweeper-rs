extern crate rand;
use rand::distributions::Distribution;
use rand::distributions::Uniform;

/// Possible state of any tile:
/// * `Normal`: an untouched tile
///
/// * `Digged`: a tile that has been digged
///
/// * `Flagged`: a tile that has a flag on top
#[derive(PartialEq, Clone, Copy)]
pub enum TileState {
    Normal,
    Digged,
    Flagged,
}

#[derive(Clone, Copy)]
pub enum TileNeighbour {
    Upper = 0,
    UpperRight = 1,
    Right = 2,
    LowerRight = 3,
    Lower = 4,
    LowerLeft = 5,
    Left = 6,
    UpperLeft = 7,
}

// If the bombs near counter of a tile is 9 means that there is a bomb there
const BOMB: u8 = 9;

#[derive(Clone)]
pub struct Tile {
    state: TileState,
    near_bombs: u8,
    near: [bool; 8], // true - digged | false - normal
}

impl Tile {
    // Creates a new `Tile`
    pub fn new() -> Tile {
        Tile {
            state: TileState::Normal,
            near_bombs: 0,
            near: [false; 8],
        }
    }

    /// Tells whether or not the `Tile` has been touched
    pub fn is_normal(&self) -> bool {
        self.state == TileState::Normal
    }

    /// Tells whether or not the `Tile` has been digged
    pub fn is_digged(&self) -> bool {
        self.state == TileState::Digged
    }

    /// Tells whether or not the `Tile` has a flag on top
    pub fn is_flagged(&self) -> bool {
        self.state == TileState::Flagged
    }

    /// Tells whether or not a bomb is hidden inside the `Tile`
    pub fn has_bomb(&self) -> bool {
        self.near_bombs == BOMB
    }

    /// Tells how many adjacent tiles have a bomb hidden inside
    pub fn near_bombs(&self) -> u8 {
        self.near_bombs
    }

    /// Returns the state of the `Tile`
    pub fn state(&self) -> TileState {
        self.state
    }

    /// Puts a bomb in the `Tile`
    pub fn put_bomb(&mut self) {
        self.near_bombs = BOMB;
    }

    /// Digs the `Tile` if it's hidden and returns whether or not it got digged
    pub fn dig(&mut self) -> bool {
        if self.is_normal() {
            self.state = TileState::Digged;
            true
        } else {
            false
        }
    }

    // pub fn dig_no_check(&mut self) {
    //     self.state = TileState::Digged;
    // }

    /// Puts a flag on the `Tile`
    pub fn flag(&mut self) -> bool {
        use TileState::*;
        self.state = match self.state {
            Normal => Flagged,
            Flagged => Normal,
            Digged => return false,
        };
        true
    }

    /// Set the flag at the `side` specified of the digged adjacent tiles list
    pub fn neighbour_digged(&mut self, side: TileNeighbour) {
        self.near[side as usize] = true;
    }

    /// Tells whether or not the `Tile` on the specified `side` has been digged
    pub fn is_neighbour_digged(&self, side: TileNeighbour) -> bool {
        self.near[side as usize]
    }

    /// Increments the counter of the bombs planted near the `Tile`
    pub fn another_bomb_near(&mut self) {
        if self.near_bombs != BOMB {
            self.near_bombs += 1;
        }
    }

    #[rustfmt::skip]
    pub fn border_type(&self) -> u32 {
        
        match self.near {
            // Up | UpR  |   R  | LwR  |  Lw  | LwL  |   L  | UpL //
            [false, _    , false, _    , false, _    , false, _    ] => {  1 },
            [false, _    , true , _    , false, _    , false, _    ] => {  2 },
            [false, _    , false, _    , true , _    , false, _    ] => {  3 },
            [false, _    , false, _    , false, _    , true , _    ] => {  4 },
            [true , _    , false, _    , false, _    , false, _    ] => {  5 },
            [false, _    , false, _    , true , false, true , _    ] => {  6 },
            [true , _    , false, _    , false, _    , true , false] => {  7 },
            [true , false, true , _    , false, _    , false, _    ] => {  8 },
            [false, _    , true , false, true , _    , false, _    ] => {  9 },
            [true , _    , false, _    , true , _    , false, _    ] => { 10 },
            [false, _    , true , _    , false, _    , true , _    ] => { 11 },
            [true , false, true , false, true , _    , false, _    ] => { 12 },
            [false, _    , true , false, true , false, true , _    ] => { 13 },
            [true , _    , false, _    , true , false, true , false] => { 14 },
            [true , false, true , _    , false, _    , true , false] => { 15 },
            [true , false, true , false, true , false, true , false] => { 16 },
            [false, _    , false, _    , true , true , true , _    ] => { 17 },
            [true , _    , false, _    , false, _    , true , true ] => { 18 },
            [true , true , true , _    , false, _    , false, _    ] => { 19 },
            [false, _    , true , true , true , _    , false, _    ] => { 20 },
            [true , true , true , true , true , _    , false, _    ] => { 21 },
            [true , false, true , true , true , _    , false, _    ] => { 22 },
            [true , true , true , false, true , _    , false, _    ] => { 23 },
            [false, _    , true , true , true , true , true , _    ] => { 24 },
            [false, _    , true , false, true , true , true , _    ] => { 25 },
            [false, _    , true , true , true , false, true , _    ] => { 26 },
            [true , _    , false, _    , true , true , true , true ] => { 27 },
            [true , _    , false, _    , true , false, true , true ] => { 28 },
            [true , _    , false, _    , true , true , true , false] => { 29 },
            [true , true , true , _    , false, _    , true , true ] => { 30 },
            [true , true , true , _    , false, _    , true , false] => { 31 },
            [true , false, true , _    , false, _    , true , true ] => { 32 },
            [true , true , true , true , true , true , true , true ] => { 33 },
            [true , false, true , true , true , true , true , true ] => { 34 },
            [true , true , true , false, true , true , true , true ] => { 35 },
            [true , true , true , true , true , false, true , true ] => { 36 },
            [true , true , true , true , true , true , true , false] => { 37 },
            [true , false, true , false, true , true , true , true ] => { 38 },
            [true , true , true , false, true , false, true , true ] => { 39 },
            [true , true , true , true , true , false, true , false] => { 40 },
            [true , false, true , true , true , true , true , false] => { 41 },
            [true , false, true , false, true , false, true , true ] => { 42 },
            [true , true , true , false, true , false, true , false] => { 43 },
            [true , false, true , true , true , false, true , false] => { 44 },
            [true , false, true , false, true , true , true , false] => { 45 },
            [true , false, true , true , true , false, true , true ] => { 46 },
            [true , true , true , false, true , true , true , false] => { 47 },
        }
    }
}

pub struct MineField {
    grid: Vec<Vec<Tile>>,
    width: usize,
    height: usize,
}

impl MineField {
    /// Creates a new `MineField` with size: `width` x `height`
    pub fn new(width: usize, height: usize) -> MineField {
        MineField {
            grid: vec![vec![Tile::new(); height]; width],
            width,
            height,
        }
    }

    /// Hides a bomb inside the tile at `x`, `y`
    pub fn add_bomb_at(&mut self, x: usize, y: usize) -> bool {
        if !self.has_bomb(x, y) {
            self.grid[x][y].put_bomb();
            if x < self.width - 1 {
                self.grid[x + 1][y].another_bomb_near();

                if y > 0 {
                    self.grid[x + 1][y - 1].another_bomb_near();
                }
                if y < self.height - 1 {
                    self.grid[x + 1][y + 1].another_bomb_near();
                }
            }
            if x > 0 {
                self.grid[x - 1][y].another_bomb_near();

                if y > 0 {
                    self.grid[x - 1][y - 1].another_bomb_near();
                }
                if y < self.height - 1 {
                    self.grid[x - 1][y + 1].another_bomb_near();
                }
            }
            if y > 0 {
                self.grid[x][y - 1].another_bomb_near();
            }
            if y < self.height - 1 {
                self.grid[x][y + 1].another_bomb_near();
            }
            true
        } else {
            false
        }
    }

    pub fn gen_bombs(&mut self, number: usize, exclude: (usize, usize), radius: usize) {
        let rng = &mut rand::thread_rng();
        let distr_x = Uniform::new(0, self.width);
        let distr_y = Uniform::new(0, self.height);

        for _ in 0..number {
            let mut x = distr_x.sample(rng);
            let mut y = distr_y.sample(rng);

            while self.has_bomb(x, y)
                || (x < exclude.0 + radius
                    && x > exclude.0.saturating_sub(radius)
                    && y < exclude.1 + radius
                    && y > exclude.1.saturating_sub(radius))
            {
                if x == self.width - 1 {
                    x = 0;
                    if y == self.height - 1 {
                        y = 0;
                    } else {
                        y += 1;
                    }
                } else {
                    x += 1;
                }
            }
            self.add_bomb_at(x, y);
        }
    }

    pub fn dig(&mut self, x: usize, y: usize) -> Option<Vec<(usize, usize, bool)>> {
        let mut digging = Vec::new();
        let mut changed = Vec::new();
        let mut some = false;
        digging.push((x, y));
        while !digging.is_empty() {
            // Try to dig the tile
            if let Some((x, y)) = digging.pop() {
                let tile = &mut self.grid[x][y];
                if tile.dig() {
                    changed.push((x, y, true));
                    some = true;
                    // If this tile is digged
                    // Expand the hole if it has no bombs
                    if tile.near_bombs() == 0 {
                        if x < self.width - 1 {
                            digging.push((x + 1, y));
                            if y > 0 {
                                digging.push((x + 1, y - 1));
                            }
                            if y < self.height - 1 {
                                digging.push((x + 1, y + 1));
                            }
                        }
                        if x > 0 {
                            digging.push((x - 1, y));
                            if y > 0 {
                                digging.push((x - 1, y - 1));
                            }
                            if y < self.height - 1 {
                                digging.push((x - 1, y + 1));
                            }
                        }
                        if y > 0 {
                            digging.push((x, y - 1));
                        }
                        if y < self.height - 1 {
                            digging.push((x, y + 1));
                        }
                    }
                    if x < self.width - 1 {
                        if self.is_digged(x + 1, y) {
                            changed.push((x + 1, y, false));
                        }
                        self.grid[x + 1][y].neighbour_digged(TileNeighbour::Left);
                        if y > 0 {
                            if self.is_digged(x + 1, y - 1) {
                                changed.push((x + 1, y - 1, false));
                            }
                            self.grid[x + 1][y - 1].neighbour_digged(TileNeighbour::UpperLeft);
                        }
                        if y < self.height - 1 {
                            if self.is_digged(x + 1, y + 1) {
                                changed.push((x + 1, y + 1, false));
                            }
                            self.grid[x + 1][y + 1].neighbour_digged(TileNeighbour::LowerLeft);
                        }
                    }
                    if x > 0 {
                        if self.is_digged(x - 1, y) {
                            changed.push((x - 1, y, false));
                        }
                        self.grid[x - 1][y].neighbour_digged(TileNeighbour::Right);

                        if y > 0 {
                            if self.is_digged(x - 1, y - 1) {
                                changed.push((x - 1, y - 1, false));
                            }
                            self.grid[x - 1][y - 1].neighbour_digged(TileNeighbour::UpperRight);
                        }
                        if y < self.height - 1 {
                            if self.is_digged(x - 1, y + 1) {
                                changed.push((x - 1, y + 1, false));
                            }
                            self.grid[x - 1][y + 1].neighbour_digged(TileNeighbour::LowerRight);
                        }
                    }
                    if y > 0 {
                        if self.is_digged(x, y - 1) {
                            changed.push((x, y - 1, false));
                        }
                        self.grid[x][y - 1].neighbour_digged(TileNeighbour::Upper);
                    }
                    if y < self.height - 1 {
                        if self.is_digged(x, y + 1) {
                            changed.push((x, y + 1, false));
                        }
                        self.grid[x][y + 1].neighbour_digged(TileNeighbour::Lower);
                    }
                }
            }
        }
        if !some {
            None
        } else {
            Some(changed)
        }
    }

    pub fn update_all(&mut self) {
        for x in 0..self.width {
            for y in 0..self.height {
                if x < self.width - 1 {
                    self.grid[x + 1][y].neighbour_digged(TileNeighbour::Left);
                    if y > 0 {
                        self.grid[x + 1][y - 1].neighbour_digged(TileNeighbour::UpperLeft);
                    }
                    if y < self.height - 1 {
                        self.grid[x + 1][y + 1].neighbour_digged(TileNeighbour::LowerLeft);
                    }
                }
                if x > 0 {
                    self.grid[x - 1][y].neighbour_digged(TileNeighbour::Right);

                    if y > 0 {
                        self.grid[x - 1][y - 1].neighbour_digged(TileNeighbour::UpperRight);
                    }
                    if y < self.height - 1 {
                        self.grid[x - 1][y + 1].neighbour_digged(TileNeighbour::LowerRight);
                    }
                }
                if y > 0 {
                    self.grid[x][y - 1].neighbour_digged(TileNeighbour::Upper);
                }
                if y < self.height - 1 {
                    self.grid[x][y + 1].neighbour_digged(TileNeighbour::Lower);
                }
            }
        }
    }

    pub fn check_win(&self) -> bool {
        for x in 0..self.width {
            for y in 0..self.height {
                if self.has_bomb(x, y) && !self.is_flagged(x, y) {
                    return false;
                }
                if !self.has_bomb(x, y) && self.is_normal(x, y) {
                    return false;
                }
            }
        }
        true
    }

    pub fn flag(&mut self, x: usize, y: usize) -> bool {
        self.grid[x][y].flag()
    }

    pub fn bombs_near(&self, x: usize, y: usize) -> u8 {
        self.grid[x][y].near_bombs()
    }

    pub fn border_type(&self, x: usize, y: usize) -> u32 {
        self.grid[x][y].border_type()
    }

    pub fn is_normal(&self, x: usize, y: usize) -> bool {
        self.grid[x][y].is_normal()
    }

    pub fn is_digged(&self, x: usize, y: usize) -> bool {
        self.grid[x][y].is_digged()
    }

    pub fn is_flagged(&self, x: usize, y: usize) -> bool {
        self.grid[x][y].is_flagged()
    }

    pub fn has_bomb(&self, x: usize, y: usize) -> bool {
        self.grid[x][y].has_bomb()
    }

    /// Returns the `with` of the `MineField`
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns the `height` of the `MineField`
    pub fn height(&self) -> usize {
        self.height
    }
}