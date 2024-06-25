require("dotenv").config();
require("@nomicfoundation/hardhat-ignition-ethers");

// Initial checks
if (
  !process.env.RPC_URL ||
  process.env.RPC_URL == "" ||
  !process.env.PRIVATE_KEY ||
  process.env.PRIVATE_KEY == ""
) {
  console.error(
    "The following environment variables are needed (set them in .env): RPC_URL, PRIVATE_KEY"
  );
  return;
}

const RPC_URL = process.env.RPC_URL;
const PRIVATE_KEY = process.env.PRIVATE_KEY;

module.exports = {
  solidity: "0.8.18",
  paths: {
    sources: "./external_contracts",
  },
  networks: {
    arb_sepolia: {
      url: RPC_URL,
      accounts: [PRIVATE_KEY],
      chainId: 421614,
    },
  },
};
