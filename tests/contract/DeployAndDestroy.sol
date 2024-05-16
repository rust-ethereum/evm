// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract DeployAndDestroy {
    constructor() {
        selfdestruct(payable(msg.sender));
    }
}
