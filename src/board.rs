use std::alloc::Layout;

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
#[repr(u8)]
enum Size {
    Small = 0,
    Medium = 1,
    Large = 2,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Role {
    Star = 0 << 2,
    Ship = 1 << 2,
    White = 2 << 2,
    Black = 3 << 2,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Color {
    Red = 0 << 4,
    Yellow = 1 << 4,
    Green = 2 << 4,
    Blue = 3 << 4,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Ability {
    Attack = 0 << 4,
    Move = 1 << 4,
    Construct = 2 << 4,
    Transform = 3 << 4,
}

impl Ability {
    const fn for_color(color: Color) -> Ability {
        unsafe { std::mem::transmute(color) }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
#[repr(u8)]
enum Count {
    One = 1,
    Two = 2,
    Three = 3,
}

impl std::ops::Add for Count {
    type Output = Count;

    fn add(self, other: Count) -> Count {
        let result = self as u8 + other as u8;
        match result {
            1..=3 => unsafe { std::mem::transmute(result) },
            _ => unreachable!(),
        }
    }
}

impl std::ops::Sub for Count {
    type Output = Count;

    fn sub(self, other: Count) -> Count {
        let result = self as u8 - other as u8;
        match result {
            1..=3 => unsafe { std::mem::transmute(result) },
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Piece(u8);

impl Piece {
    const SIZE_MASK: u8 = 0b0000_0011;
    const ROLE_MASK: u8 = 0b0000_1100;
    const COLOR_MASK: u8 = 0b0011_0000;
    const COUNT_MASK: u8 = 0b1100_0000;
    const PAD_PIECE: Self = Self(0xFF);

    const fn new(size: Size, role: Role, color: Color, count: Count) -> Self {
        Self(size as u8 | role as u8 | color as u8 | ((count as u8) << 6))
    }

    const fn size(self) -> Size {
        unsafe { std::mem::transmute(self.0 & Self::SIZE_MASK) }
    }

    const fn role(self) -> Role {
        unsafe { std::mem::transmute(self.0 & Self::ROLE_MASK) }
    }

    const fn color(self) -> Color {
        unsafe { std::mem::transmute(self.0 & Self::COLOR_MASK) }
    }

    const fn count(self) -> Count {
        unsafe { std::mem::transmute((self.0 & Self::COUNT_MASK) >> 7) }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Key(u8);
impl Key {
    pub fn new(key: u8) -> Self {
        assert!(key < 36);
        return Self(key);
    }
}

enum MoveData {
    Attack { piece: Key },
    Move { piece: Key, system: Key },
    Construct { piece: Key },
    Transform { piece: Key, color: Color },
    Sacrifice { piece: Key },
}

pub struct Move(u16);
impl Move {
    const MOVE_TYPE_MASK: u16 = 0b1100_0000_0000_0000;
    const PIECE_KEY_MASK: u16 = 0b0011_1111_0000_0000;
    const SYS_KEY_COLOR_MASK: u16 = 0b0000_0000_1111_1111;

    const MOVE_TYPE_SHIFT: u16 = 14;
    const PIECE_KEY_SHIFT: u16 = 8;

    fn new(data: MoveData) -> Self {
        Self(match data {
            MoveData::Attack { piece_key } => piece_key as u16,
            MoveData::Move { piece_key, sys_key } => {
                (0b01 << Self::MOVE_TYPE_SHIFT)
                    | ((piece_key as u16) << Self::PIECE_KEY_SHIFT)
                    | (sys_key as u16)
            }
            MoveData::Construct { piece_key } => {
                (0b10 << Self::MOVE_TYPE_SHIFT) | (piece_key as u16)
            }
            MoveData::Transform { piece_key, color } => {
                (0b11 << Self::MOVE_TYPE_SHIFT)
                    | ((piece_key as u16) << Self::PIECE_KEY_SHIFT)
                    | (color as u16)
            }
        })
    }

    fn data(self) -> MoveData {
        let move_type = (self.0 & Self::MOVE_TYPE_MASK) >> Self::MOVE_TYPE_SHIFT;
        let piece_key = ((self.0 & Self::PIECE_KEY_MASK) >> Self::PIECE_KEY_SHIFT) as u8;
        let sys_key_color = (self.0 & Self::SYS_KEY_COLOR_MASK) as u8;

        match move_type {
            0b00 => MoveData::Attack { piece_key },
            0b01 => MoveData::Move {
                piece_key,
                sys_key: sys_key_color,
            },
            0b10 => MoveData::Construct { piece_key },
            0b11 => MoveData::Transform {
                piece_key,
                color: unsafe { std::mem::transmute(sys_key_color) },
            },
            _ => unreachable!(),
        }
    }
}

// Enumeration for Players
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Player {
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
    None,
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
            (p, Special::None) => (p.inv(), Special::None),
            (p, Special::Sacrifice(1, _)) => (p.inv(), Special::None),
            (p, Special::Sacrifice(v, a)) => (p, Special::Sacrifice(v - 1, a)),
            (p, Special::Star1) => (p, Special::Star2),
            (p, Special::Star2) => (p, Special::Ship),
            (Player::White, Special::Ship) => (Player::Black, Special::Star1),
            (Player::Black, Special::Ship) => (Player::White, Special::None),
        };
        Self { player, special }
    }
}

// 128bit multiply function
fn wymum(a: u64, b: u64) -> (u64, u64) {
    let r = (a as u128) * (b as u128);
    (r as u64, (r >> 64) as u64)
}

// multiply and xor mix function, aka MUM
fn wymix(a: u64, b: u64) -> u64 {
    let (a, b) = wymum(a, b);
    a ^ b
}

// wyhash64 function
fn wyhash64(a: u64, b: u64) -> u64 {
    let a = a ^ 0x2d358dccaa6c78a5;
    let b = b ^ 0x8bb84b93962eacc9;
    let (a, b) = wymum(a, b);
    wymix(a ^ 0x2d358dccaa6c78a5, b ^ 0x8bb84b93962eacc9)
}

pub struct BoardInner<T: ?Sized> {
    hash: u64,
    turn: Turn,
    pieces: T,
}

pub type Board = BoardInner<[Piece]>;

impl Board {
    pub fn new() -> Box<Board> {
        Box::new(BoardInner::<[Piece; 1]> {
            hash: 0,
            turn: Turn::initial(),
            pieces: [Piece::PAD_PIECE],
        })
    }
    pub fn moves(&self) -> Vec<Move> {

    }
}

impl Board {
    fn debug(&self) {
        println!("size {}", std::mem::size_of_val(self));
        println!("align {}", std::mem::align_of_val(self));
        println!("bytes {:?}", unsafe {
            &*std::ptr::slice_from_raw_parts(
                self as *const Board as *const u8,
                std::mem::size_of_val(self),
            )
        });
        println!("pieces len {}", self.pieces.len());
    }
}

impl Clone for Box<Board> {
    fn clone(&self) -> Self {
        let src = self.as_ref() as *const Board as *const u8;
        let src_size = std::mem::size_of_val(self.as_ref());
        let src_align = std::mem::align_of_val(self.as_ref());
        let needs_pad = self.pieces[self.pieces.len() - 1] != Piece::PAD_PIECE;
        let needs_space = needs_pad
            && unsafe {
                (&self.pieces[0] as *const Piece).add(self.pieces.len()) as *const u8
                    == src.add(src_size)
            };
        let dst_size = src_size + needs_space as usize;
        let dst =
            unsafe { std::alloc::alloc(Layout::from_size_align(dst_size, src_align).unwrap()) };
        unsafe {
            std::ptr::copy_nonoverlapping(src, dst, src_size);
        }
        let mut new_board = unsafe {
            Box::from_raw(core::slice::from_raw_parts_mut(
                dst,
                self.pieces.len() + needs_pad as usize,
            ) as *mut [u8] as *mut Board)
        };
        new_board.pieces[new_board.pieces.len() - 1] = Piece::PAD_PIECE;
        return new_board;
    }
}
