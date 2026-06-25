#![allow(dead_code)]

pub(crate) const TYPEFACE_BIAS: u8 = 128;

pub(crate) const FN_TEXT: u8 = TYPEFACE_BIAS + 1;
pub(crate) const FN_FUNCTION: u8 = TYPEFACE_BIAS + 2;
pub(crate) const FN_VARIABLE: u8 = TYPEFACE_BIAS + 3;
pub(crate) const FN_LC_GREEK: u8 = TYPEFACE_BIAS + 4;
pub(crate) const FN_UC_GREEK: u8 = TYPEFACE_BIAS + 5;
pub(crate) const FN_SYMBOL: u8 = TYPEFACE_BIAS + 6;
pub(crate) const FN_VECTOR: u8 = TYPEFACE_BIAS + 7;
pub(crate) const FN_NUMBER: u8 = TYPEFACE_BIAS + 8;
pub(crate) const FN_USER1: u8 = TYPEFACE_BIAS + 9;
pub(crate) const FN_USER2: u8 = TYPEFACE_BIAS + 10;
pub(crate) const FN_MT_EXTRA: u8 = TYPEFACE_BIAS + 11;
pub(crate) const FN_TEXT_FE: u8 = TYPEFACE_BIAS + 12;
pub(crate) const FN_EXPAND: u8 = TYPEFACE_BIAS + 22;
pub(crate) const FN_MARKER: u8 = TYPEFACE_BIAS + 23;
pub(crate) const FN_SPACE: u8 = TYPEFACE_BIAS + 24;

pub(crate) const EXPLICIT_FONT_NEG_1: u8 = TYPEFACE_BIAS - 1;
pub(crate) const EXPLICIT_FONT_NEG_2: u8 = TYPEFACE_BIAS - 2;
