// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "forge-std/Test.sol";
import "../src/Contract.sol";

contract ContractTest is Test {
    Crowdfunding crowdfunding;

    function setUp() public {
        crowdfunding = new Crowdfunding();
    }

    function testCreateCampaign() public {
        crowdfunding.createCampaign("Test Campaign", "Test Description", 1000);
        (address creator, string memory title, , uint256 goal, , ) = crowdfunding.campaigns(1);
        assertEq(creator, address(this));
        assertEq(title, "Test Campaign");
        assertEq(goal, 1000);
    }

    function testFundCampaign() public {
        crowdfunding.createCampaign("Test Campaign", "Test Description", 1000);
        crowdfunding.fundCampaign{value: 500}(1);
        (, , , , uint256 fundsRaised, ) = crowdfunding.campaigns(1);
        assertEq(fundsRaised, 500);
    }
}