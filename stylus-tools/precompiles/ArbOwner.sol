// Copyright 2021-2024, Offchain Labs, Inc.
// For license information, see https://github.com/OffchainLabs/nitro-contracts/blob/main/LICENSE
// SPDX-License-Identifier: BUSL-1.1

pragma solidity >=0.4.21 <0.9.0;

/// @title Provides owners with tools for managing the rollup.
/// @notice Calls by non-owners will always revert.
/// Most of Arbitrum Classic's owner methods have been removed since they no longer make sense in Nitro:
/// - What were once chain parameters are now parts of ArbOS's state, and those that remain are set at genesis.
/// - ArbOS upgrades happen with the rest of the system rather than being independent
/// - Exemptions to address aliasing are no longer offered. Exemptions were intended to support backward compatibility for contracts deployed before aliasing was introduced, but no exemptions were ever requested.
/// Precompiled contract that exists in every Arbitrum chain at 0x0000000000000000000000000000000000000070.
#[sol(rpc)]
interface ArbOwner {
    /// @notice Add account as a chain owner
    function addChainOwner(
        address newOwner
    ) external;

    /// @notice Remove account from the list of chain owners
    function removeChainOwner(
        address ownerToRemove
    ) external;

    /// @notice See if the user is a chain owner
    function isChainOwner(
        address addr
    ) external view returns (bool);

    /// @notice Retrieves the list of chain owners
    function getAllChainOwners() external view returns (address[] memory);

    /// @notice Set how slowly ArbOS updates its estimate of the L1 basefee
    function setL1BaseFeeEstimateInertia(
        uint64 inertia
    ) external;

    /// @notice Set the L2 basefee directly, bypassing the pool calculus
    function setL2BaseFee(
        uint256 priceInWei
    ) external;

    /// @notice Set the minimum basefee needed for a transaction to succeed
    function setMinimumL2BaseFee(
        uint256 priceInWei
    ) external;

    /// @notice Set the computational speed limit for the chain
    function setSpeedLimit(
        uint64 limit
    ) external;

    /// @notice Set the maximum size a tx (and block) can be
    function setMaxTxGasLimit(
        uint64 limit
    ) external;

    /// @notice Set the L2 gas pricing inertia
    function setL2GasPricingInertia(
        uint64 sec
    ) external;

    /// @notice Set the L2 gas backlog tolerance
    function setL2GasBacklogTolerance(
        uint64 sec
    ) external;

    /// @notice Get the network fee collector
    function getNetworkFeeAccount() external view returns (address);

    /// @notice Get the infrastructure fee collector
    function getInfraFeeAccount() external view returns (address);

    /// @notice Set the network fee collector
    function setNetworkFeeAccount(
        address newNetworkFeeAccount
    ) external;

    /// @notice Set the infrastructure fee collector
    function setInfraFeeAccount(
        address newInfraFeeAccount
    ) external;

    /// @notice Upgrades ArbOS to the requested version at the requested timestamp
    function scheduleArbOSUpgrade(uint64 newVersion, uint64 timestamp) external;

    /// @notice Sets equilibration units parameter for L1 price adjustment algorithm
    function setL1PricingEquilibrationUnits(
        uint256 equilibrationUnits
    ) external;

    /// @notice Sets inertia parameter for L1 price adjustment algorithm
    function setL1PricingInertia(
        uint64 inertia
    ) external;

    /// @notice Sets reward recipient address for L1 price adjustment algorithm
    function setL1PricingRewardRecipient(
        address recipient
    ) external;

    /// @notice Sets reward amount for L1 price adjustment algorithm, in wei per unit
    function setL1PricingRewardRate(
        uint64 weiPerUnit
    ) external;

    /// @notice Set how much ArbOS charges per L1 gas spent on transaction data.
    function setL1PricePerUnit(
        uint256 pricePerUnit
    ) external;

    /// @notice Sets the base charge (in L1 gas) attributed to each data batch in the calldata pricer
    function setPerBatchGasCharge(
        int64 cost
    ) external;

    /// @notice Sets the Brotli compression level used for fast compression
    /// Available in ArbOS version 12 with default level as 1
    function setBrotliCompressionLevel(
        uint64 level
    ) external;

    /// @notice Sets the cost amortization cap in basis points
    function setAmortizedCostCapBips(
        uint64 cap
    ) external;

    /// @notice Releases surplus funds from L1PricerFundsPoolAddress for use
    function releaseL1PricerSurplusFunds(
        uint256 maxWeiToRelease
    ) external returns (uint256);

    /// @notice Sets the amount of ink 1 gas buys
    /// @param price the conversion rate (must fit in a uint24)
    function setInkPrice(
        uint32 price
    ) external;

    /// @notice Sets the maximum depth (in wasm words) a wasm stack may grow
    function setWasmMaxStackDepth(
        uint32 depth
    ) external;

    /// @notice Sets the number of free wasm pages a tx gets
    function setWasmFreePages(
        uint16 pages
    ) external;

    /// @notice Sets the base cost of each additional wasm page
    function setWasmPageGas(
        uint16 gas
    ) external;

    /// @notice Sets the maximum number of pages a wasm may allocate
    function setWasmPageLimit(
        uint16 limit
    ) external;

    /// @notice Sets the maximum size of the uncompressed wasm code in bytes
    function setWasmMaxSize(
        uint32 size
    ) external;

    /// @notice Sets the minimum costs to invoke a program
    /// @param gas amount of gas paid in increments of 256 when not the program is not cached
    /// @param cached amount of gas paid in increments of 64 when the program is cached
    function setWasmMinInitGas(uint8 gas, uint16 cached) external;

    /// @notice Sets the linear adjustment made to program init costs.
    /// @param percent the adjustment (100% = no adjustment).
    function setWasmInitCostScalar(
        uint64 percent
    ) external;

    /// @notice Sets the number of days after which programs deactivate
    function setWasmExpiryDays(
        uint16 _days
    ) external;

    /// @notice Sets the age a program must be to perform a keepalive
    function setWasmKeepaliveDays(
        uint16 _days
    ) external;

    /// @notice Sets the number of extra programs ArbOS caches during a given block
    function setWasmBlockCacheSize(
        uint16 count
    ) external;

    /// @notice Adds account as a wasm cache manager
    function addWasmCacheManager(
        address manager
    ) external;

    /// @notice Removes account from the list of wasm cache managers
    function removeWasmCacheManager(
        address manager
    ) external;

    /// @notice Sets serialized chain config in ArbOS state
    function setChainConfig(
        string calldata chainConfig
    ) external;

    /// @notice Sets the increased calldata price feature on or off (EIP-7623)
    /// Available in ArbOS version 40 with default as false
    function setCalldataPriceIncrease(
        bool enable
    ) external;

    /// Emitted when a successful call is made to this precompile
    event OwnerActs(bytes4 indexed method, address indexed owner, bytes data);
}
