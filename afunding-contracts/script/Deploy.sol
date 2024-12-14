// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "forge-std/Script.sol";
import "../src/Contract.sol";

contract DeployScript is Script {
    function run() external {
        vm.startBroadcast();
        Crowdfunding crowdfunding = new Crowdfunding();
        console.log("Contract deployed at:", address(crowdfunding));
        vm.stopBroadcast();
    }
}