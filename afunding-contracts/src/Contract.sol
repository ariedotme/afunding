// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;

contract Crowdfunding {
    struct Campaign {
        address creator;
        string title;
        string description;
        uint256 goal;
        uint256 fundsRaised;
        bool completed;
    }

    mapping(uint256 => Campaign) public campaigns;
    uint256 public campaignCount;

    event CampaignCreated(uint256 id, address creator, string title, string description, uint256 goal);
    event CampaignFunded(uint256 id, address funder, uint256 amount);

    function createCampaign(string memory _title, string memory _description, uint256 _goal) public {
        campaignCount++;
        campaigns[campaignCount] = Campaign({
            creator: msg.sender,
            title: _title,
            description: _description,
            goal: _goal,
            fundsRaised: 0,
            completed: false
        });

        emit CampaignCreated(campaignCount, msg.sender, _title, _description, _goal);
    }

    function fundCampaign(uint256 _id) public payable {
        Campaign storage campaign = campaigns[_id];
        require(!campaign.completed, "Campaign already completed");
        require(msg.value > 0, "Must send ETH");

        campaign.fundsRaised += msg.value;

        if (campaign.fundsRaised >= campaign.goal) {
            campaign.completed = true;
            payable(campaign.creator).transfer(campaign.fundsRaised);
        }

        emit CampaignFunded(_id, msg.sender, msg.value);
    }
}