#![feature(const_eval_limit)]
#![feature(const_mut_refs)]
#![feature(int_roundings)]
#![feature(str_split_whitespace_remainder)]
#![const_eval_limit = "5000000"]

pub mod movegen;
pub mod position;
pub mod search;
pub mod uci;
pub mod zobrist;
