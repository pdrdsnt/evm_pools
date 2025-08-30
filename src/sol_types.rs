alloy_sol_types::sol! {

type IHooks is address;
type Currency is address;
type PoolId is bytes32;

/// @notice Returns the key for identifying a pool
struct PoolKey {
    Currency currency0;
    Currency currency1;
    uint24 fee;
    int24 tickSpacing;
    IHooks hooks;

}

// Event signature from Uniswap V4 PoolManager
    event PoolInitialized(
        address currency0,
        address currency1,
        uint24 fee,
        int24 tickSpacing,
        address hooks,
        bytes32 poolId
    );
#[sol(rpc)]
interface IUniswapV2Factory {
        function getPair(address tokenA, address tokenB) external view returns (address pair);
        function createPair(address tokenA, address tokenB) external returns (address pair);
        function allPairs(uint) external view returns (address pair);
        function allPairsLength() external view returns (uint);

        event PairCreated(address indexed token0, address indexed token1, address pair, uint);
    }
#[sol(rpc)]
interface IUniswapV3Factory {
    event OwnerChanged(address indexed oldOwner, address indexed newOwner);
    event PoolCreated(
        address indexed token0,
        address indexed token1,
        uint24 indexed fee,
        int24 tickSpacing,
        address pool
    );

    event FeeAmountEnabled(uint24 indexed fee, int24 indexed tickSpacing);
    function owner() external view returns (address);
    function feeAmountTickSpacing(uint24 fee) external view returns (int24);

    function getPool(
        address tokenA,
        address tokenB,
        uint24 fee
    ) external view returns (address pool);

    function createPool(
        address tokenA,
        address tokenB,
        uint24 fee
    ) external returns (address pool);

    function setOwner(address _owner) external;

    function enableFeeAmount(uint24 fee, int24 tickSpacing) external;
}



#[sol(rpc)]
contract StateView{
    function getSlot0(PoolId poolId)
        external
        view
        returns (uint160 sqrtPriceX96, int24 tick, uint24 protocolFee, uint24 lpFee);

    function getTickInfo(PoolId poolId, int24 tick)
        external
        view
        returns (
            uint128 liquidityGross,
            int128 liquidityNet,
            uint256 feeGrowthOutside0X128,
            uint256 feeGrowthOutside1X128
        );

    function getTickLiquidity(PoolId poolId, int24 tick)
        external
        view
        returns (uint128 liquidityGross, int128 liquidityNet);
    function getLiquidity(PoolId poolId) external view returns (uint128 liquidity);
    function getTickBitmap(PoolId poolId, int16 tick) external view returns (uint256 tickBitmap);
    function getPositionInfo(PoolId poolId, address owner, int24 tickLower, int24 tickUpper, bytes32 salt)
        external
        view
            returns (uint128 liquidity, uint256 feeGrowthInside0LastX128, uint256 feeGrowthInside1LastX128);
    function getPositionInfo(PoolId poolId, bytes32 positionId)
        external
        view
        returns (uint128 liquidity, uint256 feeGrowthInside0LastX128, uint256 feeGrowthInside1LastX128);    /// @inheritdoc IStateView
    function getPositionLiquidity(PoolId poolId, bytes32 positionId) external view returns (uint128 liquidity);
    function getFeeGrowthInside(PoolId poolId, int24 tickLower, int24 tickUpper)
        external
        view
        returns (uint256 feeGrowthInside0X128, uint256 feeGrowthInside1X128);
}

#[sol(rpc)]
interface IUniswapV2Pair {
        function name() external view returns (string);
        function symbol() external view returns (string);
        function decimals() external view returns (uint8);
        function totalSupply() external view returns (uint256);
        function balanceOf(address owner) external view returns (uint256);
        function allowance(address owner, address spender) external view returns (uint256);
        function approve(address spender, uint256 value) external returns (bool);
        function transfer(address to, uint256 value) external returns (bool);
        function transferFrom(address from, address to, uint256 value) external returns (bool);

        function factory() external view returns (address);
        function token0() external view returns (address);
        function token1() external view returns (address);
        function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast);
        function mint(address to) external returns (uint liquidity);
        function burn(address to) external returns (uint amount0, uint amount1);
        function swap(uint amount0Out, uint amount1Out, address to, bytes calldata data) external;
        function skim(address to) external;
        function sync() external;

        event Approval(address indexed owner, address indexed spender, uint value);
        event Transfer(address indexed from, address indexed to, uint value);
        event Mint(address indexed sender, uint amount0, uint amount1);
        event Burn(address indexed sender, uint amount0, uint amount1, address indexed to);
        event Swap(
            address indexed sender,
            uint amount0In,
            uint amount1In,
            uint amount0Out,
            uint amount1Out,
            address indexed to
        );
        event Sync(uint112 reserve0, uint112 reserve1);
    }

#[sol(rpc)]
contract V3Pool {
   function slot0()
        external
        view
        returns (
            uint160 sqrtPriceX96,
            int24 tick,
            uint16 observationIndex,
            uint16 observationCardinality,
            uint16 observationCardinalityNext,
            uint8 feeProtocol,
            bool unlocked
        );

   function ticks(int24 tick)
        external
        view
        returns (
            uint128 liquidityGross,
            int128 liquidityNet,
            uint256 feeGrowthOutside0X128,
            uint256 feeGrowthOutside1X128,
            int56 tickCumulativeOutside,
            uint160 secondsPerLiquidityOutsideX128,
            uint32 secondsOutside,
            bool initialized
        );

    function liquidity() external view returns (uint128);
    function tickBitmap(int16 wordPosition) external view returns (uint256);
    function factory() external view returns (address);
    function token0() external view returns (address);
    function token1() external view returns (address);
    function fee() external view returns (uint24);
    function tickSpacing() external view returns (int24);
    function maxLiquidityPerTick() external view returns (uint128);
}

#[sol(rpc)]
interface IERC20 {
        function name() external view returns (string);
        function symbol() external view returns (string);
        function decimals() external view returns (uint8);
        function totalSupply() external view returns (uint256);
        function balanceOf(address account) external view returns (uint256);
        function transfer(address to, uint256 amount) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256);
        function approve(address spender, uint256 amount) external returns (bool);
        function transferFrom(address from, address to, uint256 amount) external returns (bool);

        // Optional: ERC20 events
        event Transfer(address indexed from, address indexed to, uint256 value);
        event Approval(address indexed owner, address indexed spender, uint256 value);

}
#[sol(rpc)]
interface IERC721 {
        function balanceOf(address owner) external view returns (uint256);
        function ownerOf(uint256 tokenId) external view returns (address);
        function safeTransferFrom(address from, address to, uint256 tokenId) external;
        function transferFrom(address from, address to, uint256 tokenId) external;
        function approve(address to, uint256 tokenId) external;
        function getApproved(uint256 tokenId) external view returns (address);
        function setApprovalForAll(address operator, bool approved) external;
        function isApprovedForAll(address owner, address operator) external view returns (bool);

        event Transfer(address indexed from, address indexed to, uint256 indexed tokenId);
        event Approval(address indexed owner, address indexed approved, uint256 indexed tokenId);
        event ApprovalForAll(address indexed owner, address indexed operator, bool approved);
    }

#[sol(rpc)]
interface IERC165 {

    function supportsInterface(bytes4 interfaceId) public view virtual returns (bool) {
        return interfaceId == type(IERC165).interfaceId;
    }
}
interface ICurveMetaRegistry {
        // Registry Discovery
        function registry_length() external view returns (uint256);
        function pool_list(uint256 index) external view returns (address);
        function get_pool_name(address pool) external view returns (string);
        function is_meta(address pool) external view returns (bool);

        // Pool Introspection via RegistryHandler APIs
        function get_n_coins(address pool) external view returns (uint256);
        function get_coins(address pool) external view returns (address[8]);
        function get_underlying_coins(address pool) external view returns (address[8]);
        function get_balances(address pool) external view returns (uint256[8]);
        function get_underlying_balances(address pool) external view returns (uint256[8]);
        function get_coin_indices(address pool, address from, address to) external view returns (int128, int128, bool);
    }
    // StableSwap v1 (plain pool)
    interface ICurveV1PlainPool {
        // Core quoting / swapping
        function get_dy(int128 i, int128 j, uint256 dx) external view returns (uint256);
        function exchange(int128 i, int128 j, uint256 dx, uint256 min_dy) external;

        // Pool state
        function A() external view returns (uint256);                // amplification
        function fee() external view returns (uint256);             // swap fee (1e10 or 1e8 style, pool-dependent)
        function get_virtual_price() external view returns (uint256);

        // Balances & coins
        function balances(uint256 index) external view returns (uint256);
        function coins(uint256 index) external view returns (address);

        // Optional but common
        function admin_fee() external view returns (uint256);
        // Some pools expose N_COINS as an immut/const; not always callable.
    }
    interface ICurveV1Underlying {
        // Underlying coins (e.g., DAI/USDC/USDT beneath cTokens or meta setup)
        function underlying_coins(uint256 index) external view returns (address);

        // Underlying quoting / swapping
        function get_dy_underlying(int128 i, int128 j, uint256 dx) external view returns (uint256);
        function exchange_underlying(int128 i, int128 j, uint256 dx, uint256 min_dy) external;

        // Optional convenience for meta pools
        function base_pool() external view returns (address);
    }
interface ICurveV2CryptoPool {
        // Core quoting / swapping
        function get_dy(int128 i, int128 j, uint256 dx) external view returns (uint256);
        function exchange(int128 i, int128 j, uint256 dx, uint256 min_dy) external;

        // Coins & balances
        function coins(uint256 index) external view returns (address);
        function balances(uint256 index) external view returns (uint256);

        // Dynamic fee & parameters
        function fee() external view returns (uint256);        // current effective fee (computed)
        function gamma() external view returns (uint256);
        function mid_fee() external view returns (uint256);
        function out_fee() external view returns (uint256);
        function fee_gamma() external view returns (uint256);

        // Pricing / oracles
        function get_virtual_price() external view returns (uint256);
        function price_scale(uint256 index) external view returns (uint256);
        function price_oracle(uint256 index) external view returns (uint256);
        function last_prices(uint256 index) external view returns (uint256);
    }
 interface ICurveFactory {
        function pool_count() external view returns (uint256);
        function pool_list(uint256 index) external view returns (address);
    }
}
