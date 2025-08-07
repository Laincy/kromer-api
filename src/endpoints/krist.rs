/*! Endpoints that deal with the Krist compatability API as defined in
    the [Krist docs](https://krist.dev/docs/). Each struct handles
    interactions with a specific endpoint defined in the docs, alongside
    the [`Endpoint`](super::Endpoint), [`Paginated`](super::Paginated)
    and [`PaginatedEndpoint`](super::PaginatedEndpoint) traits to set
    parameters and query the API.
*/

pub use addresses::*;
pub use misc::*;

mod addresses;
mod misc;
