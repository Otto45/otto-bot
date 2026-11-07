/// How long a cached prompt layer is kept at least: the API keeps it longer while requests keep reading it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheLifetime {
    ThirtyMinutes,
    TwelveHours,
}
