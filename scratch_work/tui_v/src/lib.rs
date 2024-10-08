// ANCHOR: new-imports
pub use color_eyre::{
    eyre::{bail, WrapErr},
    Result,
};
// ANCHOR_END: new-imports
pub use bracket_pathfinding::prelude::{Point as BracketPoint, Rect as BracketRect, *};
pub use interslavic::*;
use rand::{
    distributions::{Distribution, Uniform},
    rngs::SmallRng,
    SeedableRng,
};
pub use ratatui::style::Color as RatColor;
pub use ratatui::text::Span;
pub use ratatui::{
    backend::CrosstermBackend,
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    crossterm::{
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::{Alignment, Flex, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{
        block::{Position, Title},
        Block, Borders, Paragraph, Widget,
    },
    Frame, Terminal,
};
use ratatui::{prelude::*, widgets::*};
pub use rstar::{Envelope, PointDistance, RTree, RTreeObject, SelectionFunction, AABB};
pub use std::collections::HashMap;
pub use std::collections::HashSet;
pub use strum::Display;
use strum::IntoEnumIterator;
use strum::{EnumCount, EnumIter, FromRepr};

pub use std::io::{self, stdout, Stdout};

mod components;
pub use components::*;

mod tui;
pub use tui::*;
mod actions;
pub use actions::*;
mod app;
pub use app::*;

mod gamemap;
pub use gamemap::*;
mod complexplanet;
pub use complexplanet::*;
mod typedefs;
pub use typedefs::*;
mod items_creatures;
pub use items_creatures::*;
mod voxel;
pub use voxel::*;

/*mod components;
pub use components::*; */

mod blockable_trait;
pub use blockable_trait::*;

mod typeenums;
pub use typeenums::*;

mod graphic_trait;
pub use graphic_trait::*;

mod color_trait;
pub use color_trait::*;

mod input;
pub use input::*;

mod equipment_traits;
pub use equipment_traits::*;
