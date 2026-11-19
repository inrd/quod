//! Semantic types for the phase-1 checker (`spec/typing.md` §2).

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntTy {
    U8,
    U16,
    U32,
    U64,
    Usize,
    I8,
    I16,
    I32,
    I64,
    Isize,
}

impl IntTy {
    pub fn from_name(s: &str) -> Option<IntTy> {
        Some(match s {
            "u8" => IntTy::U8,
            "u16" => IntTy::U16,
            "u32" => IntTy::U32,
            "u64" => IntTy::U64,
            "usize" => IntTy::Usize,
            "i8" => IntTy::I8,
            "i16" => IntTy::I16,
            "i32" => IntTy::I32,
            "i64" => IntTy::I64,
            "isize" => IntTy::Isize,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            IntTy::U8 => "u8",
            IntTy::U16 => "u16",
            IntTy::U32 => "u32",
            IntTy::U64 => "u64",
            IntTy::Usize => "usize",
            IntTy::I8 => "i8",
            IntTy::I16 => "i16",
            IntTy::I32 => "i32",
            IntTy::I64 => "i64",
            IntTy::Isize => "isize",
        }
    }

    pub fn signed(self) -> bool {
        matches!(
            self,
            IntTy::I8 | IntTy::I16 | IntTy::I32 | IntTy::I64 | IntTy::Isize
        )
    }

    pub fn is_pointer_sized(self) -> bool {
        matches!(self, IntTy::Usize | IntTy::Isize)
    }

    /// Smallest width this type can have on any target (`usize`: 32).
    pub fn min_bits(self) -> u32 {
        match self {
            IntTy::U8 | IntTy::I8 => 8,
            IntTy::U16 | IntTy::I16 => 16,
            IntTy::U32 | IntTy::I32 | IntTy::Usize | IntTy::Isize => 32,
            IntTy::U64 | IntTy::I64 => 64,
        }
    }

    /// Largest width this type can have on any target (`usize`: 64).
    pub fn max_bits(self) -> u32 {
        if self.is_pointer_sized() {
            64
        } else {
            self.min_bits()
        }
    }

    /// Value range for a given width.
    fn range(self, bits: u32) -> (i128, i128) {
        if self.signed() {
            (-(1i128 << (bits - 1)), (1i128 << (bits - 1)) - 1)
        } else {
            (0, (1i128 << bits) - 1)
        }
    }

    /// Range guaranteed on every target (used for static checks).
    pub fn portable_range(self) -> (i128, i128) {
        self.range(self.min_bits())
    }

    /// Range of any value on some target.
    pub fn widest_range(self) -> (i128, i128) {
        self.range(self.max_bits())
    }

    pub fn fits(self, v: i128) -> bool {
        let (lo, hi) = self.portable_range();
        lo <= v && v <= hi
    }

    /// Every value of `self` on every target fits `to` on every target.
    pub fn widens_to(self, to: IntTy) -> bool {
        let (slo, shi) = self.widest_range();
        let (dlo, dhi) = to.portable_range();
        dlo <= slo && shi <= dhi
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Ty {
    Int(IntTy),
    Bool,
    #[default]
    Unit,
    Struct(String),
    Array(Box<Ty>, u64),
    Slice(Box<Ty>),
    Ref(bool, Box<Ty>),
}

impl Ty {
    pub fn int(&self) -> Option<IntTy> {
        match self {
            Ty::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn contains_ref(&self) -> bool {
        match self {
            Ty::Ref(..) => true,
            Ty::Array(e, _) | Ty::Slice(e) => e.contains_ref(),
            _ => false,
        }
    }
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ty::Int(i) => f.write_str(i.name()),
            Ty::Bool => f.write_str("bool"),
            Ty::Unit => f.write_str("()"),
            Ty::Struct(n) => f.write_str(n),
            Ty::Array(e, n) => write!(f, "[{e}; {n}]"),
            Ty::Slice(e) => write!(f, "[{e}]"),
            Ty::Ref(m, t) => write!(f, "&{}{t}", if *m { "mut " } else { "" }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widening() {
        assert!(IntTy::U32.widens_to(IntTy::Usize));
        assert!(IntTy::Usize.widens_to(IntTy::U64));
        assert!(!IntTy::U64.widens_to(IntTy::Usize));
        assert!(!IntTy::Usize.widens_to(IntTy::U32));
        assert!(IntTy::U8.widens_to(IntTy::I16));
        assert!(!IntTy::I8.widens_to(IntTy::U64));
        assert!(IntTy::I32.widens_to(IntTy::Isize));
        assert!(IntTy::U16.widens_to(IntTy::Isize));
        assert!(!IntTy::U32.widens_to(IntTy::Isize));
    }

    #[test]
    fn portable() {
        assert!(IntTy::Usize.fits(4_294_967_295));
        assert!(!IntTy::Usize.fits(4_294_967_296));
        assert!(IntTy::I8.fits(-128));
        assert!(!IntTy::I8.fits(128));
    }
}
