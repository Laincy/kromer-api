# Kromer API

The `kromer-api` crate provides a strongly typed interface for interacting with the [Kromer2](https://github.com/ReconnectedCC/kromer2) server. It omits backwards compatability parts of the API, such as those that deal with Krist mining.

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
    - [x] [Get v2 address from private key](https://krist.dev/docs/#api-MiscellaneousGroup-MakeV2Address)
  - [ ] Name Endpoints
    - [x] [Get a name](https://krist.dev/docs/#api-NameGroup-GetName)
    - [x] [List names](https://krist.dev/docs/#api-NameGroup-GetNames)
    - [x] [List newest names](https://krist.dev/docs/#api-NameGroup-GetNewNames)
    - [x] [Get cost of a name](https://krist.dev/docs/#api-NameGroup-CheckName)
    - [x] [Check availability of a name](https://krist.dev/docs/#api-NameGroup-CheckName)
    - [ ] [Register a name](https://krist.dev/docs/#api-NameGroup-RegisterName)
    - [ ] [Transfer a name](https://krist.dev/docs/#api-NameGroup-TransferName)
    - [ ] [Update a name](https://krist.dev/docs/#api-NameGroup-UpdateNamePOST)
  - [ ] Transaction Endpoints
    - [ ] [List all transactions](https://krist.dev/docs/#api-TransactionGroup-GetTransactions)
    - [ ] [List latest transactions](https://krist.dev/docs/#api-TransactionGroup-GetLatestTransactions)
    - [ ] [Get a transaction](https://krist.dev/docs/#api-TransactionGroup-GetTransaction)
    - [ ] [Make a transaction](https://krist.dev/docs/#api-TransactionGroup-MakeTransaction)
  - [ ] The lookup API will be implemented under a feature flag once Kromer2 has made signifigant progress in their own implementation of the API
  - [ ] The websocket API will be implemented under a feature flag once the Krist API is feature complete
- [ ] Kromer2 API
  - [ ] Walet
    - [ ] Get by UUID
    - [ ] Get by name
  - [ ] Internal API (`internal` feature)
    - [ ] Create wallet
    - [ ] Give wallet money
    - [ ] Get wallet by UUID
