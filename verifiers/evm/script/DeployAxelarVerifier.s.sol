// packages/evm/script/DeployAxelarVerifier.s.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
import "forge-std/Script.sol";
import "../src/ZolvencyVerifierAxelar.sol";

contract DeployAxelarVerifier is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("EVM_PRIVATE_KEY");
        address gateway = 0xe432150cce91c13a887f7D836923d5597adD8E31; // Axelar Sepolia Gateway
        string memory stellarSource = vm.envString("STELLAR_IDENTITY_ADDRESS");

        vm.startBroadcast(deployerPrivateKey);
        ZolvencyVerifierAxelar verifier = new ZolvencyVerifierAxelar(gateway, "stellar", stellarSource);
        console.log("ZolvencyVerifierAxelar deployed to:", address(verifier));
        vm.stopBroadcast();
    }
}
