// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "forge-std/Script.sol";
import "../contracts/ZolvencyVerifier.sol";

contract DeployVerifier is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("EVM_PRIVATE_KEY");
        address gateway = 0xe432150cce91c13a887f7D836923d5597adD8E31;
        string memory stellarSource = "CAKC4ZOYRNP5T43OURK4H7H6UIOZ4DDBBHQGD736JTVIS6FNUXTH5QEM";

        vm.startBroadcast(deployerPrivateKey);

        ZolvencyVerifier verifier = new ZolvencyVerifier(gateway, stellarSource);

        console.log("ZolvencyVerifier deployed to:", address(verifier));

        vm.stopBroadcast();
    }
}
