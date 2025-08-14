//! Marker types for [`Client`](super::Client)

use serde::Serialize;

/// Implemented by valid [`Client`](super::Client) markers.
#[allow(private_bounds)]
pub trait ClientMarker: ClientMarkerSealed {}

impl<T: ClientMarkerSealed> ClientMarker for T {}

pub(super) trait ClientMarkerSealed: Send + Sync {}

/// A marker type denoting a [`Client`](super::Client) that cannot use internal endpoints
pub struct Basic;

impl ClientMarkerSealed for Basic {}

/// Used for paginating various Krist endpoints
#[derive(Debug, Serialize, Clone, Copy, Eq, PartialEq)]
pub struct Paginator {
    limit: usize,
    offset: usize,
}

impl Paginator {
    /// Create a new [`Self`]
    #[must_use]
    pub fn new(offset: usize, limit: usize) -> Self {
        Self {
            limit: limit.clamp(1, 1000),
            offset,
        }
    }

    /// Sets the offset of the paginator
    #[must_use]
    pub const fn offset(mut self, v: usize) -> Self {
        self.offset = v;
        self
    }

    /// Sets the limit of the paginator, clamped between 1 and 1000
    #[must_use]
    pub fn limit(mut self, v: usize) -> Self {
        self.limit = v.clamp(1, 1000);
        self
    }

    /// Sets the offset of `self`
    pub const fn set_offset(&mut self, v: usize) {
        self.offset = v;
    }

    /// Sets the limit of `self`, clamped between 1 and 1000
    pub fn set_limit(&mut self, v: usize) {
        self.limit = v.clamp(1, 1000);
    }

    /// Calculates the page that [`Self`] is on
    #[must_use]
    pub const fn page(&self) -> usize {
        (self.offset / self.limit) + 1
    }

    /// Increments the offset of [`Self`] by its limit
    pub const fn next_page(&mut self) {
        self.offset += self.limit;
    }

    /// Increments offset by `v`
    pub const fn increment_offset(&mut self, v: usize) {
        self.offset += v;
    }

    /// Decrements offset by `v`, stopping at 0
    pub const fn decrement_offset(&mut self, v: usize) {
        self.offset = self.offset.saturating_sub(v);
    }
}

impl Default for Paginator {
    fn default() -> Self {
        Self {
            limit: 50,
            offset: 0,
        }
    }
}
