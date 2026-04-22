// SPDX-License-Identifier: MIT
pragma solidity ^0.8.26;

import { Ownable } from "@openzeppelin/contracts/access/Ownable.sol";
import { Ownable2Step } from "@openzeppelin/contracts/access/Ownable2Step.sol";
import { AGCToken } from "./AGCToken.sol";
import { IAGCHook } from "./interfaces/IAGCHook.sol";
import { ISettlementRouter } from "./interfaces/ISettlementRouter.sol";
import { IStabilityVault } from "./interfaces/IStabilityVault.sol";
import { IXAGCVault } from "./interfaces/IXAGCVault.sol";
import { AGCDataTypes } from "./libraries/AGCDataTypes.sol";
import { PolicyEngine } from "./PolicyEngine.sol";

contract PolicyController is Ownable2Step {
    error Unauthorized();
    error InvalidMintDistribution();
    error InvalidRecipient();
    error EpochTooSoon();
    error InvalidEpoch();
    error InvalidTreasuryBuybackParams();
    error NoPendingTreasuryBuyback();

    event KeeperUpdated(address indexed keeper, bool allowed);
    event PolicyParametersUpdated(uint16 normalBandBps, uint16 stressedBandBps, uint64 epochDuration);
    event MintDistributionUpdated(
        uint16 xagcBps,
        uint16 growthProgramsBps,
        uint16 lpBps,
        uint16 integratorsBps,
        uint16 treasuryBps
    );
    event SettlementRecipientsUpdated(address growthPrograms, address lp, address integrators);
    event GrowthProgramsEnabledUpdated(bool enabled);
    event EpochSettled(
        uint64 indexed epochId,
        AGCDataTypes.Regime indexed regime,
        uint256 anchorNextX18,
        uint256 mintBudgetAcp,
        uint256 buybackBudgetQuoteX18
    );
    event TreasuryBuybackQueued(
        uint64 indexed epochId, uint256 quoteBudgetX18, uint256 rawUsdcBudget
    );
    event PendingTreasuryBuybackExecuted(
        uint256 usdcSpent, uint256 agcBurned, uint256 pendingTreasuryBuybackUsdcAfter
    );

    struct Dependencies {
        AGCToken agcToken;
        IAGCHook hookContract;
        IStabilityVault stabilityVault;
        IXAGCVault xagcVault;
        ISettlementRouter router;
        PolicyEngine policyEngine;
    }

    AGCToken public immutable agc;
    IAGCHook public immutable hook;
    IStabilityVault public immutable treasuryVault;
    IXAGCVault public immutable xagcVault;
    ISettlementRouter public immutable settlementRouter;
    PolicyEngine public immutable policyEngine;

    AGCDataTypes.PolicyParams public policyParams;
    AGCDataTypes.MintDistribution public mintDistribution;
    AGCDataTypes.SettlementRecipients public settlementRecipients;
    AGCDataTypes.Regime public regime;

    uint256 public anchorPriceX18;
    uint256 public premiumPersistenceEpochs;
    uint256 public lastGrossBuyQuoteX18;
    uint256 public lastCoverageBps;
    uint256 public lastExitPressureBps;
    uint256 public lastVolatilityBps;
    uint256 public lastPremiumBps;
    uint256 public lastLockedShareBps;
    uint256 public lastLockFlowBps;

    uint64 public lastSettledEpoch;
    uint64 public lastSettlementTimestamp;
    uint64 public recoveryCooldownEpochsRemaining;
    uint64 public mintWindowDay;

    uint256 public mintedInCurrentDay;
    uint256 public pendingTreasuryBuybackUsdc;
    uint256 public lastXagcDepositTotalAcp;
    uint256 public lastXagcRedemptionTotalAcp;
    uint256 internal buybackExecutionNonce;

    bool public growthProgramsEnabled = true;

    mapping(address keeper => bool allowed) public keepers;
    mapping(uint64 epochId => AGCDataTypes.EpochResult result) public epochResult;

    AGCDataTypes.EpochResult internal _lastEpochResult;

    constructor(
        address admin,
        Dependencies memory deps,
        uint256 initialAnchorPriceX18,
        AGCDataTypes.PolicyParams memory params,
        AGCDataTypes.MintDistribution memory distribution
    ) Ownable(admin) {
        agc = deps.agcToken;
        hook = deps.hookContract;
        treasuryVault = deps.stabilityVault;
        xagcVault = deps.xagcVault;
        settlementRouter = deps.router;
        policyEngine = deps.policyEngine;

        anchorPriceX18 = initialAnchorPriceX18;
        policyParams = params;
        regime = AGCDataTypes.Regime.Neutral;
        settlementRecipients = AGCDataTypes.SettlementRecipients({
            growthPrograms: admin,
            lp: admin,
            integrators: admin
        });

        _setMintDistribution(distribution);
    }

    modifier onlyKeeperOrOwner() {
        if (msg.sender != owner() && !keepers[msg.sender]) revert Unauthorized();
        _;
    }

    function setKeeper(
        address keeper,
        bool allowed
    ) external onlyOwner {
        keepers[keeper] = allowed;
        emit KeeperUpdated(keeper, allowed);
    }

    function setPolicyParams(
        AGCDataTypes.PolicyParams calldata params
    ) external onlyOwner {
        policyParams = params;
        emit PolicyParametersUpdated(
            params.normalBandBps, params.stressedBandBps, params.policyEpochDuration
        );
    }

    function setMintDistribution(
        AGCDataTypes.MintDistribution calldata distribution
    ) external onlyOwner {
        _setMintDistribution(distribution);
    }

    function setSettlementRecipients(
        AGCDataTypes.SettlementRecipients calldata recipients
    ) external onlyOwner {
        if (
            recipients.growthPrograms == address(0) || recipients.lp == address(0)
                || recipients.integrators == address(0)
        ) revert InvalidRecipient();

        settlementRecipients = recipients;
        emit SettlementRecipientsUpdated(
            recipients.growthPrograms, recipients.lp, recipients.integrators
        );
    }

    function setGrowthProgramsEnabled(
        bool enabled
    ) external onlyOwner {
        growthProgramsEnabled = enabled;
        emit GrowthProgramsEnabledUpdated(enabled);
    }

    function previewEpoch(
        AGCDataTypes.ExternalMetrics calldata externalMetrics
    ) external view returns (AGCDataTypes.EpochResult memory result) {
        return _deriveEpochResult(hook.previewEpochSnapshot(), externalMetrics);
    }

    function settleEpoch(
        AGCDataTypes.ExternalMetrics calldata externalMetrics
    ) external onlyKeeperOrOwner returns (AGCDataTypes.EpochResult memory result) {
        _validateSettlementWindow();
        _refreshMintWindow();

        AGCDataTypes.EpochSnapshot memory snapshot = hook.consumeEpochSnapshot();
        if (snapshot.epochId <= lastSettledEpoch) revert InvalidEpoch();

        result = _deriveEpochResult(snapshot, externalMetrics);
        _persistEpochSettlement(snapshot, result);
    }

    function lastEpochResult() external view returns (AGCDataTypes.EpochResult memory result) {
        return _lastEpochResult;
    }

    function circulatingFloat() public view returns (uint256) {
        uint256 totalSupply = agc.totalSupply();
        uint256 sequestered =
            agc.balanceOf(address(treasuryVault)) + agc.balanceOf(address(xagcVault));
        return totalSupply > sequestered ? totalSupply - sequestered : 0;
    }

    function executePendingTreasuryBuyback(
        uint256 usdcSpend,
        uint256 minAgcOut,
        uint160 sqrtPriceLimitX96
    ) external onlyKeeperOrOwner returns (uint256 agcBurned) {
        if (usdcSpend == 0 || minAgcOut == 0) revert InvalidTreasuryBuybackParams();

        uint256 spend =
            usdcSpend > pendingTreasuryBuybackUsdc ? pendingTreasuryBuybackUsdc : usdcSpend;
        if (spend == 0) revert NoPendingTreasuryBuyback();

        pendingTreasuryBuybackUsdc -= spend;
        unchecked {
            buybackExecutionNonce++;
        }

        bytes32 refId = keccak256(
            abi.encodePacked(
                "buyback", address(this), block.chainid, buybackExecutionNonce, spend, block.timestamp
            )
        );
        agcBurned =
            settlementRouter.executeTreasuryBuyback(spend, minAgcOut, sqrtPriceLimitX96, refId);

        emit PendingTreasuryBuybackExecuted(spend, agcBurned, pendingTreasuryBuybackUsdc);
    }

    function _deriveEpochResult(
        AGCDataTypes.EpochSnapshot memory snapshot,
        AGCDataTypes.ExternalMetrics calldata externalMetrics
    ) internal view returns (AGCDataTypes.EpochResult memory result) {
        AGCDataTypes.PolicyState memory state = AGCDataTypes.PolicyState({
            anchorPriceX18: anchorPriceX18,
            premiumPersistenceEpochs: premiumPersistenceEpochs,
            lastGrossBuyQuoteX18: lastGrossBuyQuoteX18,
            mintedTodayAcp: mintedInCurrentDay,
            lastRegime: regime,
            recoveryCooldownEpochsRemaining: recoveryCooldownEpochsRemaining,
            floatSupplyAcp: circulatingFloat(),
            treasuryQuoteX18: _quoteToX18(treasuryVault.availableUsdc()),
            treasuryAcp: treasuryVault.availableAGC(),
            xagcTotalAssetsAcp: xagcVault.totalAssets()
        });

        AGCDataTypes.VaultFlows memory flows = AGCDataTypes.VaultFlows({
            xagcDepositsAcp: xagcVault.grossDepositsTotalAcp() - lastXagcDepositTotalAcp,
            xagcGrossRedemptionsAcp: xagcVault.grossRedemptionsTotalAcp()
                - lastXagcRedemptionTotalAcp
        });

        result = policyEngine.evaluateEpoch(
            snapshot,
            externalMetrics,
            state,
            flows,
            policyParams
        );
        result.mintAllocations = _allocateMint(result.mintBudgetAcp);
    }

    function _persistEpochSettlement(
        AGCDataTypes.EpochSnapshot memory snapshot,
        AGCDataTypes.EpochResult memory result
    ) internal {
        anchorPriceX18 = result.anchorNextX18;
        premiumPersistenceEpochs = result.premiumPersistenceEpochs;
        lastGrossBuyQuoteX18 = result.grossBuyQuoteX18;
        regime = result.regime;

        if (result.regime == AGCDataTypes.Regime.Defense) {
            recoveryCooldownEpochsRemaining = policyParams.recoveryCooldownEpochs;
        } else if (result.regime == AGCDataTypes.Regime.Recovery) {
            recoveryCooldownEpochsRemaining = recoveryCooldownEpochsRemaining > 0
                ? recoveryCooldownEpochsRemaining - 1
                : 0;
        } else {
            recoveryCooldownEpochsRemaining = 0;
        }

        hook.setRegime(result.regime);

        if (result.mintBudgetAcp > 0) {
            mintedInCurrentDay += result.mintBudgetAcp;
            if (result.mintAllocations.xagcMintAcp > 0 && xagcVault.totalSupply() == 0) {
                result.mintAllocations.treasuryMintAcp += result.mintAllocations.xagcMintAcp;
                result.mintAllocations.xagcMintAcp = 0;
            }
            _distributeExpansionMint(result.mintAllocations);
        }

        if (result.buybackBudgetQuoteX18 > 0) {
            uint256 rawBudget = _quoteFromX18(result.buybackBudgetQuoteX18);
            pendingTreasuryBuybackUsdc += rawBudget;
            emit TreasuryBuybackQueued(snapshot.epochId, result.buybackBudgetQuoteX18, rawBudget);
        }

        lastSettledEpoch = snapshot.epochId;
        lastSettlementTimestamp = uint64(block.timestamp);
        lastCoverageBps = result.reserveCoverageBps;
        lastExitPressureBps = result.exitPressureBps;
        lastVolatilityBps = result.realizedVolatilityBps;
        lastPremiumBps = result.premiumBps;
        lastLockedShareBps = result.lockedShareBps;
        lastLockFlowBps = result.lockFlowBps;
        lastXagcDepositTotalAcp = xagcVault.grossDepositsTotalAcp();
        lastXagcRedemptionTotalAcp = xagcVault.grossRedemptionsTotalAcp();

        epochResult[snapshot.epochId] = result;
        _lastEpochResult = result;

        emit EpochSettled(
            snapshot.epochId,
            result.regime,
            result.anchorNextX18,
            result.mintBudgetAcp,
            result.buybackBudgetQuoteX18
        );
    }

    function _refreshMintWindow() internal {
        uint64 currentDay = uint64(block.timestamp / 1 days);
        if (mintWindowDay != currentDay) {
            mintWindowDay = currentDay;
            mintedInCurrentDay = 0;
        }
    }

    function _validateSettlementWindow() internal view {
        if (
            lastSettlementTimestamp != 0
                && block.timestamp < lastSettlementTimestamp + policyParams.policyEpochDuration
        ) revert EpochTooSoon();
    }

    function _allocateMint(
        uint256 mintBudgetAcp
    ) internal view returns (AGCDataTypes.MintAllocation memory allocation) {
        allocation.xagcMintAcp =
            mintBudgetAcp * mintDistribution.xagcBps / AGCDataTypes.BPS;
        allocation.growthProgramsMintAcp =
            mintBudgetAcp * mintDistribution.growthProgramsBps / AGCDataTypes.BPS;
        allocation.lpMintAcp = mintBudgetAcp * mintDistribution.lpBps / AGCDataTypes.BPS;
        allocation.integratorsMintAcp =
            mintBudgetAcp * mintDistribution.integratorsBps / AGCDataTypes.BPS;
        allocation.treasuryMintAcp = mintBudgetAcp - allocation.xagcMintAcp
            - allocation.growthProgramsMintAcp - allocation.lpMintAcp
            - allocation.integratorsMintAcp;

        if (!growthProgramsEnabled) {
            allocation.treasuryMintAcp += allocation.growthProgramsMintAcp;
            allocation.growthProgramsMintAcp = 0;
        }
    }

    function _distributeExpansionMint(
        AGCDataTypes.MintAllocation memory allocation
    ) internal {
        if (allocation.xagcMintAcp > 0) {
            agc.mint(address(xagcVault), allocation.xagcMintAcp);
        }
        if (allocation.growthProgramsMintAcp > 0) {
            agc.mint(settlementRecipients.growthPrograms, allocation.growthProgramsMintAcp);
        }
        if (allocation.lpMintAcp > 0) {
            agc.mint(settlementRecipients.lp, allocation.lpMintAcp);
        }
        if (allocation.integratorsMintAcp > 0) {
            agc.mint(settlementRecipients.integrators, allocation.integratorsMintAcp);
        }
        if (allocation.treasuryMintAcp > 0) {
            agc.mint(address(treasuryVault), allocation.treasuryMintAcp);
        }
    }

    function _setMintDistribution(
        AGCDataTypes.MintDistribution memory distribution
    ) internal {
        uint256 total = uint256(distribution.xagcBps) + distribution.growthProgramsBps
            + distribution.lpBps + distribution.integratorsBps + distribution.treasuryBps;
        if (total != AGCDataTypes.BPS || distribution.xagcBps == 0) {
            revert InvalidMintDistribution();
        }

        mintDistribution = distribution;
        emit MintDistributionUpdated(
            distribution.xagcBps,
            distribution.growthProgramsBps,
            distribution.lpBps,
            distribution.integratorsBps,
            distribution.treasuryBps
        );
    }

    function _quoteToX18(
        uint256 rawUsdc
    ) internal pure returns (uint256) {
        return rawUsdc * AGCDataTypes.QUOTE_SCALE;
    }

    function _quoteFromX18(
        uint256 normalizedQuoteX18
    ) internal pure returns (uint256) {
        return normalizedQuoteX18 / AGCDataTypes.QUOTE_SCALE;
    }
}
