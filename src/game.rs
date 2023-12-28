// Importing necessary libraries and modules
use arrayvec::ArrayVec;
use once_cell::sync::Lazy;
use std::ops::{Index, IndexMut};
use std::str::FromStr;

// Constants
const PIECE_COUNT: usize = 36; // Total number of pieces on the board
const MOVE_COUNT: usize = 338; // Total number of possible moves

// Struct to represent a key (unique identifier for pieces)
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Key(u8);

// Implementation for Key structure
impl Key {
    // Function to get the size of a piece associated with a key
    fn size(self) -> Size {
        unsafe { std::mem::transmute((self.0 % 9) / 3) }
    }

    // Function to get the color of a piece associated with a key
    fn color(self) -> Color {
        unsafe { std::mem::transmute(self.0 / 9) }
    }
}

// Enumerations for Size and Color
#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
#[repr(u8)]
enum Size {
    Small = 0,
    Medium = 1,
    Large = 2,
}

impl Size {
    // Function to convert Size enum to a string
    fn to_str(&self) -> &'static str {
        match self {
            Size::Small => "small",
            Size::Medium => "medium",
            Size::Large => "large",
        }
    }

    // Function to create a Size enum from a string
    fn from_str(s: &str) -> Result<Self, ()> {
        match s.to_lowercase().as_str() {
            "small" => Ok(Size::Small),
            "medium" => Ok(Size::Medium),
            "large" => Ok(Size::Large),
            _ => Err(()),
        }
    }

    // Function to list available Sizes
    fn list() -> [Size; 3] {
        [Size::Small, Size::Medium, Size::Large]
    }

    // Function to get the number of turns to sacrifice based on size
    fn sacrifice_turns(self) -> u8 {
        self as u8 + 1
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Color {
    Red = 0,
    Yellow = 1,
    Green = 2,
    Blue = 3,
}

impl Color {
    // Function to convert Color enum to a string
    fn to_str(&self) -> &'static str {
        match self {
            Color::Red => "red",
            Color::Yellow => "yellow",
            Color::Green => "green",
            Color::Blue => "blue",
        }
    }

    // Function to create a Color enum from a string
    fn from_str(s: &str) -> Result<Self, ()> {
        match s.to_lowercase().as_str() {
            "red" => Ok(Color::Red),
            "yellow" => Ok(Color::Yellow),
            "green" => Ok(Color::Green),
            "blue" => Ok(Color::Blue),
            _ => Err(()),
        }
    }

    // Function to retrieve the list of available colors
    fn list() -> [Color; 4] {
        [Color::Red, Color::Yellow, Color::Green, Color::Blue]
    }
}

// Define the possible moves in the game
#[derive(Clone, Copy)]
enum Move {
    Attack(Key),
    Construct(Key),
    Transform(Key, Color),
    Sacrifice(Key),
    MoveInit(Key),
    MoveFinish(Key),
    Select(Size, Color),
    Catastrophe(Key),
    Pass,
}

impl ToString for Move {
    fn to_string(&self) -> String {
        match self {
            Move::Attack(key) => format!("attack {}", key.0),
            Move::Construct(key) => format!("construct {}", key.0),
            Move::Transform(key, color) => format!("transform {} {}", key.0, color.to_str()),
            Move::Sacrifice(key) => format!("sacrifice {}", key.0),
            Move::MoveInit(key) => format!("moveinit {}", key.0),
            Move::MoveFinish(key) => format!("movefinish {}", key.0),
            Move::Select(size, color) => format!("select {} {}", size.to_str(), color.to_str()),
            Move::Catastrophe(key) => format!("catastrophe {}", key.0),
            Move::Pass => "pass".to_string(),
        }
    }
}

impl FromStr for Move {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.trim().split_whitespace().collect();
        match parts.as_slice() {
            ["attack", key] => Ok(Move::Attack(Key(key.parse().or(Err(()))?))),
            ["construct", key] => Ok(Move::Construct(Key(key.parse().or(Err(()))?))),
            ["transform", key, color] => {
                let color = Color::from_str(color)?;
                Ok(Move::Transform(Key(key.parse().or(Err(()))?), color))
            }
            ["sacrifice", key] => Ok(Move::Sacrifice(Key(key.parse().or(Err(()))?))),
            ["moveinit", key] => Ok(Move::MoveInit(Key(key.parse().or(Err(()))?))),
            ["movefinish", key] => Ok(Move::MoveFinish(Key(key.parse().or(Err(()))?))),
            ["select", size, color] => {
                let size = Size::from_str(size)?;
                let color = Color::from_str(color)?;
                Ok(Move::Select(size, color))
            }
            ["catastrophe", key] => Ok(Move::Catastrophe(Key(key.parse().or(Err(()))?))),
            ["pass"] => Ok(Move::Pass),
            _ => Err(()),
        }
    }
}

// Lazily initialize an array of all possible moves in the game
static MOVES: Lazy<[Move; MOVE_COUNT]> = Lazy::new(|| {
    // Initialize an array to store all possible moves
    let mut moves = [Move::Pass; MOVE_COUNT];
    let mut i = 0;

    // Generating all possible moves
    // attack
    for key in KeyRange::all() {
        moves[i] = Move::Attack(key);
        i += 1;
    }
    // construct
    for key in KeyRange::all() {
        moves[i] = Move::Construct(key);
        i += 1;
    }
    // transform
    for key in KeyRange::all() {
        for color in Color::list() {
            if color == key.color() {
                continue;
            }
            moves[i] = Move::Transform(key, color);
            i += 1;
        }
    }
    // sacrifice
    for key in KeyRange::all() {
        moves[i] = Move::Sacrifice(key);
        i += 1;
    }
    // move init
    for key in KeyRange::all() {
        moves[i] = Move::MoveInit(key);
        i += 1;
    }
    // move finish
    for key in KeyRange::all() {
        moves[i] = Move::MoveFinish(key);
        i += 1;
    }
    // selection
    for size in Size::list() {
        for color in Color::list() {
            moves[i] = Move::Select(size, color);
            i += 1;
        }
    }
    // trigger catastrophe
    for key in KeyRange::all() {
        moves[i] = Move::Catastrophe(key);
        i += 1;
    }
    // end turn
    moves[i] = Move::Pass;
    i += 1;
    // Asserting that the total number of generated moves matches the pre-defined count
    assert!(i == MOVE_COUNT);

    // Return the generated moves
    return moves;
});

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Ability {
    Attack = 0,
    Move = 1,
    Construct = 2,
    Transform = 3,
}

impl Ability {
    fn for_color(color: Color) -> Ability {
        return unsafe { std::mem::transmute(color) };
    }
}

// Enumeration for Players
#[derive(Clone, Copy, PartialEq, Eq)]
enum Player {
    White = 0,
    Black = 1,
}

// Implementation for Player enumeration
impl Player {
    // Function to get the inverse of a player (White -> Black, Black -> White)
    fn inv(self) -> Self {
        unsafe { std::mem::transmute((self as u8) ^ 1) }
    }
}

// Enumeration for special actions in the game
#[derive(Clone, Copy, PartialEq, Eq)]
enum Special {
    Move,
    Star1,
    Star2,
    Ship,
    Sacrifice(u8, Ability),
}

// Struct to represent a turn in the game
#[derive(Clone, Copy)]
struct Turn {
    player: Player,
    special: Special,
}

// Implementation for Turn structure
impl Turn {
    // Function to initialize the first turn of the game
    fn initial() -> Self {
        Self {
            player: Player::White,
            special: Special::Star1,
        }
    }

    // Function to calculate the next turn in the game
    fn next(self) -> Self {
        let (player, special) = match (self.player, self.special) {
            (p, Special::Move) => (p.inv(), Special::Move),
            (p, Special::Star1) => (p, Special::Star2),
            (p, Special::Star2) => (p, Special::Ship),
            (Player::White, Special::Ship) => (Player::Black, Special::Star1),
            (Player::Black, Special::Ship) => (Player::White, Special::Move),
            (p, Special::Sacrifice(1, _)) => (p.inv(), Special::Move),
            (p, Special::Sacrifice(v, a)) => (p, Special::Sacrifice(v - 1, a)),
        };
        Self { player, special }
    }
}

// Struct to represent a ship in the game
#[derive(Clone, Copy, PartialEq, Eq)]
struct Ship {
    parent: Key,
    sibling: Key,
    player: Player,
}

// Enumeration to represent different types of pieces in the game
#[derive(Clone, Copy, PartialEq, Eq)]
enum Piece {
    Bank,
    Star { child: Key },
    BinaryFirst { child: KeyMaybe, sibling: KeyMaybe },
    BinarySecond { sibling: Key },
    Ship(Ship),
}

// Struct to represent an optional Key (Some(Key) or None)
#[derive(Clone, Copy, PartialEq, Eq)]
struct KeyMaybe(i8);

// Implementation for KeyMaybe structure
impl KeyMaybe {
    // Function to create KeyMaybe with a key (Some(Key))
    fn some(key: Key) -> Self {
        Self(key.0 as i8)
    }

    // Function to create KeyMaybe representing None
    fn none() -> Self {
        Self(-1)
    }

    // Function to get the Key from KeyMaybe (if it exists)
    fn get(self) -> Option<Key> {
        match self.0 {
            -1 => None,
            v => Some(Key(v as u8)),
        }
    }

    // Function to check if KeyMaybe represents Some(Key)
    fn is_some(self) -> bool {
        self.0 != -1
    }

    // Function to check if KeyMaybe represents None
    fn is_none(self) -> bool {
        self.0 == -1
    }
}

// Iterator over sibling ships on the board
struct SiblingIter<'a> {
    board: &'a Board,
    home: Key,
    next: KeyMaybe,
}

// Implementation for the SiblingIter iterator
impl Iterator for SiblingIter<'_> {
    type Item = (Ship, Key);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next.get()?;
        let ship = match self.board[next] {
            Piece::Ship(ship) => ship,
            _ => unreachable!(),
        };
        self.next = match ship.sibling {
            v if v == self.home => KeyMaybe::none(),
            v => KeyMaybe::some(v),
        };
        Some((ship, next))
    }
}

// Struct to define a range of keys
struct KeyRange {
    next: u8,
    end: u8,
}

// Implementation for the KeyRange structure
impl KeyRange {
    // Function to create a new range of keys from start to end
    fn new(start: u8, end: u8) -> Self {
        Self { next: start, end }
    }

    // Function to create a range of keys with a specific color
    fn with_color(color: Color) -> Self {
        let start = color as u8 * 9;
        let end = start + 9;
        Self::new(start, end)
    }

    // Function to create a range of keys with a specific color and size
    fn with_color_and_size(color: Color, size: Size) -> Self {
        let start = (color as u8 * 9) + (size as u8) * 3;
        let end = start + 3;
        Self::new(start, end)
    }

    // Function to create a range of all keys
    fn all() -> Self {
        Self::new(0, PIECE_COUNT as u8)
    }
}

// Implementation for KeyRange iterator
impl Iterator for KeyRange {
    type Item = Key;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next >= self.end {
            return None;
        }
        let v = Some(Key(self.next));
        self.next += 1;
        v
    }
}

// Struct representing the game board with pieces
#[derive(Clone, Copy)]
struct Board {
    pieces: [Piece; PIECE_COUNT],
}

// Implementation for the Board structure
impl Board {
    // Function to create a new empty board
    const fn new() -> Self {
        Self {
            pieces: [Piece::Bank; PIECE_COUNT],
        }
    }

    // Function to create an iterator over sibling ships of a given key
    fn sibling_iter(&self, start: Key) -> SiblingIter {
        SiblingIter {
            board: self,
            home: start,
            next: KeyMaybe::some(start),
        }
    }
}

// Implementation for indexing the Board structure
impl Index<Key> for Board {
    type Output = Piece;

    fn index(&self, index: Key) -> &Self::Output {
        &self.pieces[index.0 as usize]
    }
}

// Implementation for mutable indexing the Board structure
impl IndexMut<Key> for Board {
    fn index_mut(&mut self, index: Key) -> &mut Self::Output {
        &mut self.pieces[index.0 as usize]
    }
}

struct Game {
    board: Board,
    turn: Turn,
    moving_piece: KeyMaybe,
    repetition_count: u8,
    wstar: KeyMaybe,
    bstar: KeyMaybe,
}

impl Game {
    // Constructor method to create a new game instance
    pub fn new() -> Self {
        // Initialization of game attributes
        return Self {
            board: Board::new(),            // Initialize the game board
            turn: Turn::initial(),          // Initialize the turn
            moving_piece: KeyMaybe::none(), // No moving piece initially
            repetition_count: 0,            // No repetitions initially
            wstar: KeyMaybe::none(),        // No star for white initially
            bstar: KeyMaybe::none(),        // No star for black initially
        };
    }

    fn force_catastrophes(&mut self) {
        for key in KeyRange::all() {
            _ = self.process_catastrophe(key);
        }
    }

    fn advance(&mut self) {
        let next_turn = self.turn.next();
        if next_turn.player != self.turn.player {
            self.force_catastrophes();
        }
        self.turn = next_turn;
    }

    // Method to attempt an attack on a specific key on the board
    fn process_attack(&mut self, tkey: Key) -> bool {
        // Check if a piece is already in motion
        if self.moving_piece.is_some() {
            return false;
        }

        // Check if the current turn allows an attack
        let is_sacrifice = match self.turn.special {
            Special::Move => false,
            Special::Sacrifice(_, Ability::Attack) => true,
            _ => return false,
        };

        // Retrieve ship information at the targeted key
        let tship = match self.board[tkey] {
            Piece::Ship(ship) if ship.player != self.turn.player => ship,
            _ => return false,
        };

        // Check conditions for potential attack
        let mut has_color = is_sacrifice || tship.parent.color() == Color::Red;
        let mut attack_size = Size::Small;
        for (sship, skey) in self.board.sibling_iter(tkey) {
            if sship.player != self.turn.player {
                // Check for invalid sibling ship configurations
                if skey < tkey && skey.size() == tkey.size() && skey.color() == tkey.color() {
                    return false;
                }
                continue;
            }
            // Update conditions for attack based on sibling ships
            has_color |= skey.color() == Color::Red;
            attack_size = attack_size.max(skey.size());
        }

        // Final validation for a successful attack
        if !has_color || attack_size < tkey.size() {
            return false;
        }

        // Update the attacked ship's owner and switch turns
        self.board[tkey] = Piece::Ship(Ship {
            player: self.turn.player,
            ..tship
        });
        self.advance();
        true // Attack successful
    }

    // Method to attempt ship construction on a specific key
    fn process_construct(&mut self, tkey: Key) -> bool {
        // Check if a piece is already in motion
        if self.moving_piece.is_some() {
            return false;
        }

        // Check if the current turn allows a sacrifice for construction
        let is_sacrifice = match self.turn.special {
            Special::Move => false,
            Special::Sacrifice(_, Ability::Construct) => true,
            _ => return false,
        };

        // Retrieve ship information at the targeted key
        let tship = match self.board[tkey] {
            Piece::Ship(ship) if ship.player == self.turn.player => ship,
            _ => return false,
        };

        // Check conditions for potential construction
        let mut has_color = is_sacrifice || tship.parent.color() == Color::Green;
        // Check sibling ships and update conditions
        for (sship, skey) in self.board.sibling_iter(tkey) {
            if sship.player != self.turn.player {
                continue;
            }
            if skey < tkey && skey.color() == tkey.color() {
                return false;
            }
            has_color |= skey.color() == Color::Green;
        }

        // Final validation for successful construction
        if !has_color {
            return false;
        }

        // Find an available key for construction and update the board
        let nkey =
            match KeyRange::with_color(tkey.color()).find(|&key| self.board[key] == Piece::Bank) {
                Some(v) => v,
                None => return false,
            };
        // 1) tkey -> tkey.next
        // 2) tkey -> nkey -> tkey.next
        self.board[tkey] = Piece::Ship(Ship {
            parent: tship.parent,
            sibling: nkey,
            player: self.turn.player,
        });
        self.board[nkey] = Piece::Ship(Ship {
            parent: tship.parent,
            sibling: tship.sibling,
            player: self.turn.player,
        });
        self.advance();
        true // Construction successful
    }

    // Method to attempt ship transformation at a specific key to a given color
    fn process_transform(&mut self, tkey: Key, tcolor: Color) -> bool {
        // Check if a piece is already in motion
        if self.moving_piece.is_some() {
            return false;
        }

        // Check if the current turn allows a sacrifice for transformation
        let is_sacrifice = match self.turn.special {
            Special::Move => false,
            Special::Sacrifice(_, Ability::Transform) => true,
            _ => return false,
        };

        // Retrieve ship information at the targeted key
        let tship = match self.board[tkey] {
            Piece::Ship(ship) if ship.player == self.turn.player => ship,
            _ => return false,
        };

        // Check conditions for potential transformation
        let mut has_color = is_sacrifice || tship.parent.color() == Color::Blue;
        let mut pkey = tkey;
        let mut pship = tship;
        for (sship, skey) in self.board.sibling_iter(tkey) {
            pkey = skey;
            pship = sship;
            if sship.player != self.turn.player {
                continue;
            }
            if skey < tkey && skey.color() == tkey.color() && skey.size() == tkey.size() {
                return false;
            }
            has_color |= skey.color() == Color::Blue;
        }

        // Final validation for successful transformation
        if !has_color {
            return false;
        }

        // Find an available key for transformation and update the board
        let nkey = match KeyRange::with_color_and_size(tcolor, tkey.size())
            .find(|&key| self.board[key] == Piece::Bank)
        {
            Some(v) => v,
            None => return false,
        };
        assert!(pship.sibling == tkey);
        // 1) pkey -> tkey -> tkey.next
        // 2) pkey -> nkey -> tkey.next
        self.board[pkey] = Piece::Ship(Ship {
            sibling: nkey,
            ..pship
        });
        self.board[tkey] = Piece::Bank;
        self.board[nkey] = Piece::Ship(tship);
        self.advance();
        true // Transformation successful
    }

    // Method to remove a ship from a key on the board and potentially a star associated with it
    fn remove_ship_and_maybe_star(
        &mut self,
        shkey: Key,      // The key where the ship to be removed is located
        shprvship: Ship, // The ship prior to the ship to be removed
        shprvkey: Key,   // The key of the ship prior to the ship to be removed
        shnxtkey: Key,   // The key of the ship after the ship to be removed
        stkey: Key,      // The key of the potential star associated with the ship
    ) {
        // Remove the ship from the current key on the board
        self.board[shkey] = Piece::Bank;

        // Check if the ship being removed is the only ship on the star, if it's a non-binary star, forget the star
        if shkey == shprvkey {
            // If the ship to be removed is the last ship on the star, handle the star accordingly
            self.board[stkey] = match self.board[stkey] {
                // If it's a star, forget it
                Piece::Star { .. } => Piece::Bank,
                // If it's a non-binary star, remove the ship association from the star
                Piece::BinaryFirst { sibling, .. } => Piece::BinaryFirst {
                    child: KeyMaybe::none(),
                    sibling,
                },
                _ => unreachable!(), // Error case, shouldn't happen
            };
        } else {
            // If the ship being removed is not the only ship on the star

            // Update the sibling ship pointers to bypass the ship being removed
            self.board[shprvkey] = Piece::Ship(Ship {
                sibling: shnxtkey, // Update the previous ship's sibling to skip the ship being removed
                ..shprvship        // Retain other ship attributes from the previous ship
            });

            // Update the association of the ship with the potential star
            self.board[stkey] = match self.board[stkey] {
                // If it's a star, update its child pointer
                Piece::Star { .. } => Piece::Star { child: shnxtkey },
                // If it's a non-binary star, update its child pointer and retain sibling information
                Piece::BinaryFirst { sibling, .. } => Piece::BinaryFirst {
                    child: KeyMaybe::some(shnxtkey),
                    sibling,
                },
                _ => unreachable!(), // Error case, shouldn't happen
            };
        }
    }

    // Method to attempt a ship sacrifice at a specific key
    fn process_sacrifice(&mut self, tkey: Key) -> bool {
        // Check if a piece is already in motion
        if self.moving_piece.is_some() {
            return false;
        }

        // Check if the current turn allows a sacrifice
        match self.turn.special {
            Special::Move => {}
            _ => return false,
        };

        // Retrieve ship information at the targeted key
        let tship = match self.board[tkey] {
            Piece::Ship(ship) if ship.player == self.turn.player => ship,
            _ => return false,
        };

        // Find the parent ship information for the targeted ship
        let mut pkey = tkey;
        let mut pship = tship;
        for (sship, skey) in self.board.sibling_iter(tkey) {
            pkey = skey;
            pship = sship;
            if sship.player != self.turn.player {
                continue;
            }
            if skey < tkey && skey.size() == tkey.size() && skey.color() == tkey.color() {
                return false;
            }
        }

        // Remove target from game
        self.remove_ship_and_maybe_star(tkey, pship, pkey, tship.sibling, tship.parent);
        self.turn.special = Special::Sacrifice(
            tkey.size().sacrifice_turns(),
            Ability::for_color(tkey.color()),
        );
        return true; // Sacrifice successful
    }

    // Method to attempt initiating a ship movement at a specific key
    fn process_move_init(&mut self, tkey: Key) -> bool {
        // Check if a piece is already in motion
        if self.moving_piece.is_some() {
            return false;
        }

        // Check if the current turn allows movement
        let is_sacrifice = match self.turn.special {
            Special::Move => false,
            Special::Sacrifice(_, Ability::Move) => true,
            _ => return false,
        };

        // Retrieve ship information at the targeted key
        let tship = match self.board[tkey] {
            Piece::Ship(ship) if ship.player == self.turn.player => ship,
            _ => return false,
        };

        // Check conditions for potential movement
        let mut has_color = is_sacrifice || tship.parent.color() == Color::Yellow;
        for (sship, skey) in self.board.sibling_iter(tkey) {
            if sship.player != self.turn.player {
                continue;
            }
            if skey < tkey && skey.size() == tkey.size() && skey.color() == tkey.color() {
                return false;
            }
            has_color |= skey.color() == Color::Yellow;
        }

        // Final validation for successful movement initiation
        if !has_color {
            return false;
        }

        // Set the moving piece and allow movement
        self.moving_piece = KeyMaybe::some(tkey);
        return true; // Movement initiation successful
    }

    // Method to determine star sizes based on the provided key
    fn get_star_sizes(&self, tkey: Key) -> (Size, Size) {
        match self.board[tkey] {
            Piece::Star { .. } => (tkey.size(), tkey.size()), // If the key is a star, return its size twice
            Piece::BinaryFirst { sibling, .. } => match sibling.get() {
                None => (tkey.size(), tkey.size()), // If there's no sibling, return the key's size twice
                Some(skey) => match self.board[skey] {
                    Piece::BinarySecond { .. } => (tkey.size(), skey.size()), // If sibling is a BinarySecond, return sizes
                    _ => unreachable!(), // Unreachable if conditions don't match
                },
            },
            _ => unreachable!(), // Unreachable if conditions don't match
        }
    }

    // Method to complete a ship movement initiated in try_move_init
    fn process_move_finish(&mut self, tstar_key: Key) -> bool {
        // Retrieve the key of the moving piece
        let fkey = match self.moving_piece.get() {
            Some(v) => v,
            None => return false, // If there's no moving piece, exit with failure
        };

        // Retrieve the child key of the target star for movement
        let tstar_child_key = match self.board[tstar_key] {
            Piece::Star { child } => KeyMaybe::some(child),
            Piece::BinaryFirst { child, .. } => child,
            _ => return false, // If the target key is not a star or binary first, exit with failure
        };

        // Retrieve ship information of the moving piece
        let fship = match self.board[fkey] {
            Piece::Ship(ship) => ship,
            _ => unreachable!(), // Unreachable if the moving piece is not a ship
        };
        let fstar_key = fship.parent; // Retrieve the parent key of the moving ship

        // Get star sizes for the source and target stars
        let fsizes = self.get_star_sizes(fstar_key);
        let tsizes = self.get_star_sizes(tstar_key);

        // Check if ship movement is allowed based on star sizes
        if fsizes.0 == tsizes.0
            || fsizes.0 == tsizes.1
            || fsizes.1 == tsizes.0
            || fsizes.1 == tsizes.1
        {
            return false; // If sizes match, movement is not allowed, exit with failure
        }

        // Handle movement and update the board
        let (pship, pkey) = self.board.sibling_iter(fkey).last().unwrap(); // Retrieve sibling ship info
        self.remove_ship_and_maybe_star(fkey, pship, pkey, fship.sibling, fstar_key);

        // Update the board based on the existence of a child ship within the target star
        match tstar_child_key.get() {
            Some(tckey) => {
                // If the target star has a child ship
                let tcship = match self.board[tckey] {
                    Piece::Ship(ship) => ship, // Retrieve ship information of the target star's child
                    _ => unreachable!(),       // Unreachable if the target key doesn't hold a ship
                };

                // Move the moving piece (fkey) to become a sibling of the target star's child
                self.board[tckey] = Piece::Ship(Ship {
                    sibling: fkey, // The moving piece becomes a sibling of the target star's child ship
                    ..tcship       // Maintain other ship attributes from the target star's child
                });

                // Update the moving piece (fkey) to reflect its new parent and sibling relationships
                self.board[fkey] = Piece::Ship(Ship {
                    parent: tcship.parent, // Set the parent of the moving piece as the parent of the target star's child
                    sibling: tcship.sibling, // Set the sibling of the moving piece based on the target star's child's sibling
                    player: self.turn.player, // Update the player of the moving piece
                });
            }
            None => {
                // If the target star doesn't have a child ship
                self.board[tstar_key] = match self.board[tstar_key] {
                    // Assign the moving piece as the child ship of the target star
                    Piece::Star { .. } => Piece::Star {
                        child: fkey
                    },
                    Piece::BinaryFirst { sibling, .. } => Piece::BinaryFirst {
                        child: KeyMaybe::some(fkey),
                        sibling,
                    },
                    _ => unreachable!(), // Unreachable if the target key isn't of type BinaryFirst
                };

                // Update the moving piece (fkey) to reflect its new parent and sibling relationships
                self.board[fkey] = Piece::Ship(Ship {
                    parent: tstar_key,        // Set the parent of the moving piece as the target star
                    sibling: fkey,            // Set the sibling of the moving piece to itself
                    player: self.turn.player, // Update the player of the moving piece
                });
            }
        }

        self.advance(); // Move finished; advance turn
        true // Successful completion of ship movement
    }
    fn star_for(&mut self, player: Player) -> &mut KeyMaybe {
        match self.turn.player {
            Player::White => &mut self.wstar,
            Player::Black => &mut self.bstar,
        }
    }
    // Method to attempt piece selection of a specific size and color
    fn process_select(&mut self, size: Size, color: Color) -> bool {
        // Check if the current turn allows selection of a piece (Star1, Star2, or Ship)
        match self.turn.special {
            Special::Star1 | Special::Star2 | Special::Ship => {}
            _ => return false, // Exit with failure if selection isn't allowed in the current turn
        }

        // Find an available key of the specified size and color on the board
        let tkey = KeyRange::with_color_and_size(color, size)
            .find(|&key| self.board[key] == Piece::Bank) // Find an empty slot
            .unwrap(); // Unwrap the found key; assumes a valid key is always available

        // Perform selection based on the current special action of the turn
        match self.turn.special {
            Special::Star1 => {
                // Set the board at the chosen key as a BinaryFirst piece with no child or sibling
                self.board[tkey] = Piece::BinaryFirst {
                    child: KeyMaybe::none(),
                    sibling: KeyMaybe::none(),
                };
                // Set the star for the respective player to the chosen key
                *self.star_for(self.turn.player) = KeyMaybe::some(tkey);
            }
            Special::Star2 => {
                // Get the current star key for the respective player
                let star = self.star_for(self.turn.player).get().unwrap();
                // Set the board at the chosen key as a BinarySecond piece with the sibling as the current star
                self.board[tkey] = Piece::BinarySecond { sibling: star };
                // Set the board at the current star as a BinaryFirst piece with the chosen key as the sibling
                self.board[star] = Piece::BinaryFirst {
                    child: KeyMaybe::none(),
                    sibling: KeyMaybe::some(tkey),
                };
            }
            Special::Ship => {
                // Get the current star key for the respective player
                let star = self.star_for(self.turn.player).get().unwrap();
                // Set the board at the chosen key as a Ship with parent as the star, sibling as the chosen key, and player's turn
                self.board[tkey] = Piece::Ship(Ship {
                    parent: star,
                    sibling: tkey,
                    player: self.turn.player,
                });
                // Set the board at the star as a BinaryFirst piece with the chosen key as the child
                self.board[star] = match self.board[star] {
                    Piece::BinaryFirst { sibling, .. } => Piece::BinaryFirst {
                        child: KeyMaybe::some(tkey),
                        sibling,
                    },
                    _ => unreachable!(), // Unreachable if the star is not in the expected state
                };
            }
            _ => unreachable!(), // Unreachable if the current special action is unexpected
        }

        self.turn = self.turn.next(); // Advance to the next turn
        true // Successful completion of piece selection
    }

    // Method to attempt a catastrophic event at a specific key
    fn process_catastrophe(&mut self, shkey: Key) -> bool {
        // Retrieve ship information for the targeted key
        let shship = match self.board[shkey] {
            Piece::Ship(ship) => ship,
            _ => return false, // Exit if the targeted key doesn't hold a ship
        };

        // Define an enum to store ship and other key information for potential catastrophic removal
        #[derive(Clone, Copy)]
        enum CatEntry {
            Ship {
                me: Key,     // Current ship key
                pkey: Key,   // Previous ship key
                pship: Ship, // Previous ship information
                nkey: Key,   // Next ship key
            },
            Other(Key), // Other key types (like star, binary first, binary second)
        }

        // Create an array vector to store entries related to potential catastrophic removal
        let mut catlist: ArrayVec<CatEntry, PIECE_COUNT> = ArrayVec::new();

        // Analyze the ship's parent and surrounding structures for potential catastrophic removal conditions
        match self.board[shship.parent] {
            Piece::Star { .. } => {
                if shship.parent.color() == shkey.color() {
                    // If the ship's parent is a star of the same color as the ship, add it to catlist
                    catlist.push(CatEntry::Other(shship.parent));
                }
            }
            Piece::BinaryFirst { sibling, .. } => {
                if shship.parent.color() == shkey.color() {
                    // If the ship's parent is a binary first of the same color as the ship, add it to catlist
                    catlist.push(CatEntry::Other(shship.parent));
                }
                match sibling.get() {
                    Some(v) if v.color() == shkey.color() => {
                        // If the sibling of the binary first is of the same color as the ship, add it to catlist
                        catlist.push(CatEntry::Other(v));
                    }
                    _ => {}
                }
            }
            Piece::BinarySecond { sibling } => {
                if sibling.color() == shkey.color() {
                    // If the sibling of the binary second is of the same color as the ship, add it to catlist
                    catlist.push(CatEntry::Other(sibling));
                }
                if shship.parent.color() == shkey.color() {
                    // If the ship's parent is a binary second of the same color as the ship, add it to catlist
                    catlist.push(CatEntry::Other(shship.parent));
                }
            }
            _ => unreachable!(), // Unreachable if the ship's parent isn't in an expected state
        }

        // Iterate through the ship's siblings to analyze potential catastrophic removal conditions
        let mut pkey = shkey;
        let mut pship = shship;
        for (sship, skey) in self.board.sibling_iter(shship.sibling) {
            if skey.color() == shkey.color() {
                // If a sibling ship has the same color as the ship, add it to catlist
                catlist.push(CatEntry::Ship {
                    me: skey,
                    pkey,
                    pship,
                    nkey: sship.sibling,
                });
            }
            pkey = skey; // Update previous ship key for the next iteration
            pship = sship; // Update previous ship information for the next iteration
        }

        // Check if the conditions for catastrophic removal are not met
        if catlist.len() < 4 {
            return false; // Exit if there are insufficient ships/structures for a catastrophe
        }

        // Iterate through the catlist in reverse order to avoid invalidating `pkey` and `pship`
        for &centry in catlist.iter().rev() {
            match centry {
                CatEntry::Ship {
                    me,
                    pkey,
                    pship,
                    nkey,
                } => {
                    // Remove the ship and potentially the associated star structure
                    self.remove_ship_and_maybe_star(me, pship, pkey, nkey, pship.parent);
                }
                CatEntry::Other(key) => match self.board[key] {
                    Piece::Bank => {
                        // If the key is an empty slot, do nothing
                    }
                    Piece::Star { child } => {
                        // If the key holds a star, remove the star and associated sibling ships
                        // Remove the star
                        self.board[key] = Piece::Bank;
                        // Remove sibling ships associated with the star
                        ArrayVec::<Key, PIECE_COUNT>::from_iter(
                            self.board.sibling_iter(child).map(|(_, skey)| skey),
                        )
                        .iter()
                        .for_each(|&skey| self.board[skey] = Piece::Bank);
                    }
                    Piece::BinarySecond { sibling } => {
                        // If the key holds a binary second, remove it and update its sibling
                        self.board[key] = Piece::Bank; // Remove the binary second
                        match self.board[sibling] {
                            Piece::BinaryFirst { child, .. } => {
                                // Update the sibling of the binary first associated with the binary second
                                self.board[sibling] = Piece::BinaryFirst {
                                    child,
                                    sibling: KeyMaybe::none(),
                                }
                            }
                            _ => unreachable!(), // Unreachable if the binary second isn't in the expected state
                        }
                    }
                    Piece::BinaryFirst { child, sibling } => {
                        // If the key holds a binary first, remove it and update its sibling if present
                        self.board[key] = Piece::Bank; // Remove the binary first
                        match sibling.get() {
                            Some(v) => {
                                self.board[v] = Piece::BinaryFirst {
                                    child,
                                    sibling: KeyMaybe::none(),
                                }
                            }
                            None => {} // Do nothing if the sibling of binary first is absent
                        }
                    }
                    Piece::Ship(_) => unreachable!(), // Unreachable if the key unexpectedly holds a ship
                },
            }
        }

        true // Successful completion of the catastrophic event
    }

    fn process_pass(&mut self) {
        self.turn = Turn {
            player: self.turn.player.inv(),
            special: Special::Move,
        };
        self.force_catastrophes();
    }

    pub fn process_move(&mut self, m: Move) -> bool {
        let us = self.turn.player;
        let them = us.inv();
        match m {
            Move::Attack(tkey) => self.process_attack(tkey),
            Move::Construct(tkey) => self.process_construct(tkey),
            Move::Transform(tkey, color) => self.process_transform(tkey, color),
            Move::Sacrifice(tkey) => self.process_sacrifice(tkey),
            Move::MoveInit(tkey) => self.process_move_init(tkey),
            Move::MoveFinish(tkey) => self.process_move_finish(tkey),
            Move::Select(size, color) => self.process_select(size, color),
            Move::Catastrophe(tkey) => self.process_catastrophe(tkey),
            Move::Pass => {
                self.process_pass();
                true
            }
        }
    }
    pub fn process_move_idx(&mut self, i: usize) -> bool {
        return self.process_move(MOVES[i]);
    }
}
