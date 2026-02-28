use strum::{Display, EnumIs};

#[derive(Display, Debug, Clone, Copy, PartialEq, Eq, Default, EnumIs)]
pub enum SortDirection {
    #[default]
    Desc,
    Asc,
}
