# token-mapping-contract

This contract template is used for mappping an existed NEP-141 compatible token to another existed NEP-141 compatible token with a certain exchange rate.

## Use case in Octopus Network ecosystem

In Octoput Network ecosystem, it is used to mapping an existed wrapped ERC-20 token to an existed wrapped appchain token on demand. The so-called `wrapped ERC-20 token` is a wrapped token of an existed ERC-20 asset lived in Ethereum network. The contract for the `wrapped ERC-20 token` is created by Rainbow Bridge service of NEAR protocol.

The normal process to perform the mapping actions are as the followings:

- Deploy this contract for a certain appchain. This contract needs to be bonded to `wrapped ERC-20 token contract` and `wrapped appchain token contract` at construction time. Which means this contract is only working for the specific two tokens.
- The owner of `wrapped appchain token contract` should transfer a proper amount of `wrapped appchain token` to this contract, by calling function `ft_transfer_call` of `wrapped appchain token contract` with a certain message attached (for comfirming the intension). That is, the `token mapping contract` will lock enough amount of `wrapped appchain token` for the mapping actions.
- The holder of `wrapped ERC-20 token` transfer a part or all their tokens to this contract by calling the function `ft_transfer_call` of `wrapped ERC-20 token contract`. This call will trigger the callback function `ft_on_transfer` of this contract automatically. And in the callback function, the contract will transfer a proper amount of `wrapped appchain token` it locked, to the account which the `wrapped ERC-20 token` is transferred from. The exchange rate between `wrapped ERC-20 token` and `wrapped appchain token` is set in this contract.

The mapping from `wrapped ERC-20 token` to `wrapped appchain token` is restricted as an one-way mapping. This means the `token mapping contract` can only map `wrapped ERC-20 token` into `wrapped appchain token`, but not the opposite.

## Function specification

This contract stores the `trade in token contract` and `trade out token contract` at construction time. They can not be changed after the contract is initialized.

This contract is ownable. The owner account is the account who calls the function `new` to initialize this contract. And this contract implements the trait `Ownable` of `near-sdk-rs`.

This contract has a exchange rate from `trade in token` to `trade out token`, the default value is `100` (in percent). Only the owner can change the exchange rate.

This contract can receive deposit of `trade in token` and `trade out token` by using callback function `ft_on_transfer` with a certain message attached. Otherwise, the deposit should be refunded.

When receive deposit of `trade in token`, this contract will transfer a proper amount of `trade out token` to the sender of the transfer. The amount is calculated by the exchange rate.

This contract will NOT store the history of token mapping actions. That is, the result of the transfer of `trade out token` can only be viewed by NEAR explorer.

## Building

To build run:

```shell
./build.sh
```

## Testing

To test run:

```shell
cargo test --package wrapped-appchain-token -- --nocapture
```

## Deploy

To deploy run:

```shell
near dev-deploy
```

Init contract:

```shell
near call $WRAPPED_APPCHAIN_TOKEN new '{"owner_id":"$APPCHAIN_ANCHOR_CONTRACT_ID","premined_beneficiary":"$valid_account_id","premined_balance":"$premined_balance","metadata":{"spec":"ft-1.0.0","name":"TestToken","symbol":"TEST","decimals":18}}' --accountId $SIGNER
```

Set owner:

```bash
near call $WRAPPED_APPCHAIN_TOKEN set_owner '{"owner_id": "$APPCHAIN_ANCHOR_CONTRACT_ID"}' --accountId $SIGNER
```

Get owner:

```bash
near view $WRAPPED_APPCHAIN_TOKEN get_owner '{}'
```

## Prepare data link for icon of fungible token metadata

According to the NEAR protocol specification, the field `icon` of `FungibleTokenMetadata` must be a data URL if present. The actual value of this field can be obtained by the following steps:

* Prepare the original icon file, the recommended file type is `svg`.
* Use the free tool [SVGOMG](https://jakearchibald.github.io/svgomg/) to optimize the original icon file.
* Encode the optimized SVG file to base64 string. It can be done by following command on linux/macOS:

```shell
base64 <optimized icon file>.svg > <base64 encoding result>.txt
```

* The value of the field `icon` should be a string with fixed prefix `data:image/svg+xml;base64,` and concatenated with the base64 encoded string of the optimized SVG file.
