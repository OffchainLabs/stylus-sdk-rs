const { Contract, Wallet, JsonRpcProvider } = require("ethers");
require("dotenv").config();

// Initial checks
if (
  !process.env.RPC_URL ||
  process.env.RPC_URL == "" ||
  !process.env.CONTRACT_ADDRESS ||
  process.env.CONTRACT_ADDRESS == "" ||
  !process.env.PRIVATE_KEY ||
  process.env.PRIVATE_KEY == ""
) {
  console.error(
    "The following environment variables are needed (set them in .env): RPC_URL, CONTRACT_ADDRESS, PRIVATE_KEY"
  );
  return;
}

// ABI of the token (used functions)
const abi = [
  "function mint() external",
  "function mintTo(address to) external",

  // Read-Only Functions
  "function balanceOf(address owner) external view returns (uint256)",
];

// Address of the token
const address = process.env.CONTRACT_ADDRESS;
// Transaction Explorer URL
const tx_explorer_url = process.env.TX_EXPLORER_URL;

// Private key and ethers provider
const walletPrivateKey = process.env.PRIVATE_KEY;
const stylusRpcProvider = new JsonRpcProvider(process.env.RPC_URL);
const signer = new Wallet(walletPrivateKey, stylusRpcProvider);

// Main function
const main = async () => {
  // Presentation message
  console.log(
    `Minting an NFT of the contract ${address} to account ${signer.address}`
  );

  // Connecting to the ERC-721 contract
  const erc721 = new Contract(address, abi, signer);

  // Current balance of user
  const currentBalance = await erc721.balanceOf(signer.address);
  console.log(`Current balance: ${currentBalance}`);

  // Minting tokens
  const mintTransaction = await erc721.mint();
  await mintTransaction.wait();
  console.log(`Transaction hash: ${mintTransaction.hash}`);
  console.log(`View tx on explorer: ${tx_explorer_url}${mintTransaction.hash}`);

  // Final balance of user
  const finalBalance = await erc721.balanceOf(signer.address);
  console.log(`Final balance: ${finalBalance}`);
};

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
