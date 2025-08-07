/*! Endpoints that deal with the Krist compatability API as defined in
    the [Krist docs](https://krist.dev/docs/). Each struct handles
    interactions with a specific endpoint defined in the docs, alongside
    the [`Endpoint`](super::Endpoint), [`Paginated`](super::Paginated)
    and [`PaginatedEndpoint`](super::PaginatedEndpoint) traits to set
    parameters and query the API.


    Endpoints such as the [get latest KristWallet version] and [get recent
    changes to the Krist project] endpoints are ommitted since they are
    eitheir irrelevant or not present in the `Kromer2` implementation of the `Krist` API.

    [get latest KristWallet version]: <https://krist.dev/docs/#api-MiscellaneousGroup-GetWalletVersion>
    [get recent changes to the Krist project]: <https://krist.dev/docs/#api-MiscellaneousGroup-GetWhatsNew>
*/

pub use addresses::*;
pub use misc::*;

mod addresses;
mod misc;
