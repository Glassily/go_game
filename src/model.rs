mod board;
mod color;
mod point;
mod eye;


// 重新导出常用类型，方便外部使用
pub use board::Board;
pub use color::{Color, Move};
pub use point::Point;
