const { Contract, Wallet, JsonRpcProvider, parseEther, parseUnits, formatEther } = require('ethers');
require('dotenv').config();

// Initial checks
if (
    (!process.env.RPC_URL || process.env.RPC_URL == "") ||
    (!process.env.CONTRACT_ADDRESS || process.env.CONTRACT_ADDRESS == "") ||
    (!process.env.PRIVATE_KEY || process.env.PRIVATE_KEY == "") 
) {
    console.error('The following environment variables are needed (set them in .env): RPC_URL, CONTRACT_ADDRESS, PRIVATE_KEY');
    return;
}

// ABI of the token (used functions)
const abi = [
    "function mint(uint256 value) external",
    "function mintTo(address to, uint256 value) external",

    // Read-Only Functions
    "function balanceOf(address owner) external view returns (uint256)",
];

// Address of the token
const address = process.env.CONTRACT_ADDRESS;

// Private key and ethers provider
const walletPrivateKey = process.env.PRIVATE_KEY;
const stylusRpcProvider = new JsonRpcProvider(process.env.RPC_URL);
const signer = new Wallet(walletPrivateKey, stylusRpcProvider);

// Amount of tokens to mint
const amountToMint = process.env.AMOUNT_TO_MINT || "1000";

// Main function
const main = async () => {
    // Presentation message
    console.log(`Minting ${amountToMint} tokens of the contract ${address} to account ${signer.address}`);

    // Connecting to the ERC-20 contract
    const erc20 = new Contract(address, abi, signer);

    // Current balance of user
    const currentBalance = await erc20.balanceOf(signer.address);
    console.log(`Current balance: ${formatEther(currentBalance)}`);

    // Minting tokens
    const mintTransaction = await erc20.mint(parseEther(amountToMint));
    await mintTransaction.wait();

    // Final balance of user
    const finalBalance = await erc20.balanceOf(signer.address);
    console.log(`Final balance: ${formatEther(finalBalance)}`);
}

main()
  .then(() => process.exit(0))
  .catch(error => {
    console.error(error)
    process.exit(1)
  });