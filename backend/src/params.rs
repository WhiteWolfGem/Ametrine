use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    Asc,
    Desc,
}

impl SortDirection {
    pub fn to_sql(&self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

#[derive(Deserialize, Debug, Default)]
pub struct PaginationParams {
    pub limit: Option<String>,
    pub offset: Option<String>,
}
impl PaginationParams {
    pub fn limit(&self) -> i64 {
        self.limit
            .as_ref()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(100)
    }
    pub fn offset(&self) -> i64 {
        self.offset
            .as_ref()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0)
    }
}

#[derive(Deserialize, Debug, Default)]
pub struct SortParams<T> {
    #[serde(flatten)]
    pub pagination: PaginationParams,
    pub sort: Option<T>,
    pub sort_by: Option<SortDirection>,
}

impl<T> SortParams<T> {
    pub fn limit(&self) -> i64 {
        self.pagination.limit()
    }
    pub fn offset(&self) -> i64 {
        self.pagination.offset()
    }
    pub fn sort(&self) -> Option<&T> {
        self.sort.as_ref()
    }
    pub fn sort_by(&self) -> SortDirection {
        self.sort_by.unwrap_or(SortDirection::Asc)
    }
}

#[derive(Deserialize, Debug, Default)]
pub struct SearchParams<T> {
    #[serde(flatten)]
    pub sortable: SortParams<T>,
    pub search: Option<String>,
}

impl<T> SearchParams<T> {
    pub fn limit(&self) -> i64 {
        self.sortable.limit()
    }
    pub fn offset(&self) -> i64 {
        self.sortable.offset()
    }
    pub fn sort(&self) -> Option<&T> {
        self.sortable.sort()
    }
    pub fn sort_by(&self) -> SortDirection {
        self.sortable.sort_by()
    }
    pub fn search(&self) -> Option<&String> {
        self.search.as_ref()
    }
}
