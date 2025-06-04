// SPDX-License-Identifier: Apache-2.0

pragma solidity >=0.8.2 <0.9.0;

contract Storage {
    uint256 public number;

    function store(uint256 num) public {
        number = num;
    }
}

