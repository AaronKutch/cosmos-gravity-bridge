use clarity::{address::Address as EthAddress, u256, Uint256};
use web30::{
    amm::{DAI_CONTRACT_ADDRESS, WETH_CONTRACT_ADDRESS},
    client::Web3,
    jsonrpc::error::Web3Error,
};

/// utility function, gets the price of a given ERC20 token in uniswap in WETH given the erc20 address and amount
pub async fn get_weth_price(
    token: EthAddress,
    amount: Uint256,
    pubkey: EthAddress,
    web3: &Web3,
) -> Result<Uint256, Web3Error> {
    if token == *WETH_CONTRACT_ADDRESS {
        return Ok(amount);
    } else if amount.is_zero() {
        return Ok(u256!(0));
    }

    // TODO: Make sure the market is not too thin
    web3.get_uniswap_price(
        pubkey,
        token,
        *WETH_CONTRACT_ADDRESS,
        None,
        amount,
        None,
        None,
    )
    .await
}

/// utility function, gets the price of a given ER20 token in uniswap in DAI given the erc20 address and amount
pub async fn get_dai_price(
    token: EthAddress,
    amount: Uint256,
    pubkey: EthAddress,
    web3: &Web3,
) -> Result<Uint256, Web3Error> {
    if token == *DAI_CONTRACT_ADDRESS {
        return Ok(amount);
    } else if amount.is_zero() {
        return Ok(u256!(0));
    }

    // TODO: Make sure the market is not too thin
    web3.get_uniswap_price(
        pubkey,
        token,
        *DAI_CONTRACT_ADDRESS,
        None,
        amount,
        None,
        None,
    )
    .await
}
