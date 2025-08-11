# Kromer API

The `kromer-api` crate provides a strongly typed interface for interacting with the [Kromer2](https://github.com/ReconnectedCC/kromer2) server. It omits backwards compatability parts of the API, such as those that deal with Krist mining.

Everything should be pretty well documented. For more info checkout the docs locally by running the following:

```bash
cargo doc --open
```

Or visit the Krist API documentatiot <https://krist.dev/docs/>

## Features

- [ ] Krist API
  - [x] Address Endpoints
    - [x] [Get an address](https://krist.dev/docs/#api-AddressGroup-GetAddress)
    - [x] [List addresses](https://krist.dev/docs/#api-AddressGroup-GetAddresses)
    - [x] [List richest](https://krist.dev/docs/#api-AddressGroup-GetRichAddresses)
    - [x] [Get recent transactions from an address](https://krist.dev/docs/#api-AddressGroup-GetAddressTransactions)
    - [x] [Get all names under an address](https://krist.dev/docs/#api-AddressGroup-GetAddressNames)
  - [x] Misc. Endpoints
    - [x] [Authenticate an address](https://krist.dev/docs/#api-MiscellaneousGroup-Login)
    - [x] [Get MOTD](https://krist.dev/docs/#api-MiscellaneousGroup-GetMOTD_+)
    - [x] [Get the money supply]("https://krist.dev/docs/#api-MiscellaneousGroup-GetMoneySupply")
  - [x] Name Endpoints
    - [x] [Get a name](https://krist.dev/docs/#api-NameGroup-GetName)
    - [x] [List names](https://krist.dev/docs/#api-NameGroup-GetNames)
    - [x] [List newest names](https://krist.dev/docs/#api-NameGroup-GetNewNames)
    - [x] [Get cost of a name](https://krist.dev/docs/#api-NameGroup-CheckName)
    - [x] [Check availability of a name](https://krist.dev/docs/#api-NameGroup-CheckName)
    - [x] [Register a name](https://krist.dev/docs/#api-NameGroup-RegisterName)
    - [x] [Transfer a name](https://krist.dev/docs/#api-NameGroup-TransferName)
    - [x] [Update a name](https://krist.dev/docs/#api-NameGroup-UpdateNamePOST)
  - [x] Transaction Endpoints
    - [x] [List all transactions](https://krist.dev/docs/#api-TransactionGroup-GetTransactions)
    - [x] [List latest transactions](https://krist.dev/docs/#api-TransactionGroup-GetLatestTransactions)
    - [x] [Get a transaction](https://krist.dev/docs/#api-TransactionGroup-GetTransaction)
    - [x] [Make a transaction](https://krist.dev/docs/#api-TransactionGroup-MakeTransaction)
  - The lookup API will be implemented under a feature flag once Kromer2 has made signifigant progress in their own implementation of the API
  - The websocket API will be implemented under a feature flag once the Krist API is feature complete
- [ ] Kromer2 API
  - [x] Wallet
    - [x] Get by UUID
    - [x] Get by name
  - [ ] Internal API (`internal` feature)
    - [ ] Create wallet
    - [ ] Give wallet money
    - [ ] Get wallet by UUID

## Ommissions

There are some notable things that I've left out of this crate becuase they are eitheir not needed for the Kromer2 API, or there are better ways to do it.

- **Make V2 Address**: Use the `Address::From<PrivateKey>` trait implementation instead
- **List Newest Names**: There are no new/unpaid names in Kromer2. This endpoint is purely for compatability
- **Redundant Name Update Endpoints**: Krist and kromer have multiple endpoints for updating the metadata of a name. Here, we use the `POST` endpoint only, not the `PUT` one
- **Misc. Mining**: Left out alot of things to do with mining of Krist since Kromer doesn't support this. If you still want to harm the environment, consider vanity address mining.
