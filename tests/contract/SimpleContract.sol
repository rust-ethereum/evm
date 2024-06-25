// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract SimpleContract {
    address public owner;

    // Constructor sets the owner of the contract
    constructor() {
        owner = msg.sender;
    }

    // Function to destroy the contract and send the remaining funds to the target address
    function destroy(address target) public {
         selfdestruct(payable(target));
    }
}
