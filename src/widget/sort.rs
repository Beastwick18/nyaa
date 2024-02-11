pub enum Sort {
    Date,
    Downloads,
    Seeders,
    Leechers,
    Name,
    Category,
}

pub struct SortPopup {
    pub sort: Sort,
}

impl Default for SortPopup {
    fn default() -> Self {
        return SortPopup { sort: Sort::Date };
    }
}
