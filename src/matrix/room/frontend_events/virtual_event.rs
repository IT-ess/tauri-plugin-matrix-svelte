use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "kind"
)]
pub enum FrontendVirtualTimelineItem {
    /// A divider between messages of two days or months depending on the
    /// timeline configuration.
    DateDivider,

    /// The user's own read marker.
    ReadMarker,

    /// The timeline start, that is, an indication that we've seen all the
    /// events for that timeline.
    TimelineStart,
}
