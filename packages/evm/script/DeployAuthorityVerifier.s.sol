// packages/evm/script/DeployAuthorityVerifier.s.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;
import "forge-std/Script.sol";
import "../src/ZolvencyVerifierAuthority.sol";

contract DeployAuthorityVerifier is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("EVM_PRIVATE_KEY");
        address authority = vm.envAddress("AUTHORITY_ADDRESS");

        vm.startBroadcast(deployerPrivateKey);
        ZolvencyVerifierAuthority verifier = new ZolvencyVerifierAuthority(authority);
        console.log("ZolvencyVerifierAuthority deployed to:", address(verifier));
        vm.stopBroadcast();
    }
}
