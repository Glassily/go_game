mod board;
mod color;
mod eye;
mod group;
mod mv;
mod point;

mod test_board;

// 重新导出常用类型，方便外部使用
pub use board::Board;
pub use color::Color;
pub use eye::{EyeAnalysis, EyeType};
pub use group::{EmptyRegion, GroupSet, GroupStatus};
pub use mv::Move;
pub use point::Point;
