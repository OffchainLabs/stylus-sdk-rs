const {
  Contract,
  Wallet,
  JsonRpcProvider,
  parseEther,
  formatEther,
} = require("ethers");
require("dotenv").config();

// Initial checks
if (
  !process.env.RPC_URL ||
  process.env.RPC_URL == "" ||
  !process.env.SINGLE_CALL_CONTRACT_ADDRESS ||
  process.env.SINGLE_CALL_CONTRACT_ADDRESS == "" ||
  !process.env.COUNTER_CONTRACT_ADDRESS ||
  process.env.COUNTER_CONTRACT_ADDRESS == "" ||
  !process.env.PRIVATE_KEY ||
  process.env.PRIVATE_KEY == ""
) {
  console.error(
    "The following environment variables are needed (set them in .env): RPC_URL, SINGLE_CALL_CONTRACT_ADDRESS, COUNTER_CONTRACT_ADDRESS, PRIVATE_KEY"
  );
  return;
}

// ABI of the SingleCall contract
const ABI_SINGLE_CALL = [
  "function execute(address target, bytes data) external returns (bytes)",
];

// ABI of the Counter contract
const ABI_COUNTER = [
  "function number() external view returns (uint256)",
  "function setNumber(uint256 value) external",
  "function increment() external",
];

// Addresses for the contracts
const SINGLE_CALL_ADDRESS = process.env.SINGLE_CALL_CONTRACT_ADDRESS;
const COUNTER_ADDRESS = process.env.COUNTER_CONTRACT_ADDRESS;

// Transaction Explorer URL
const TX_EXPLORER_URL = process.env.TX_EXPLORER_URL;

// Private key and ethers provider
const WALLET_PRIVATE_KEY = process.env.PRIVATE_KEY;
const stylusRpcProvider = new JsonRpcProvider(process.env.RPC_URL);
const signer = new Wallet(WALLET_PRIVATE_KEY, stylusRpcProvider);

// Main function
const main = async () => {
  // // Presentation message
  console.log(
    `Incrementing the Counter contract at ${COUNTER_ADDRESS} via the SingleCall router at ${SINGLE_CALL_ADDRESS}`
  );

  // Connecting to the contracts
  const singleCall = new Contract(SINGLE_CALL_ADDRESS, ABI_SINGLE_CALL, signer);
  const counter = new Contract(COUNTER_ADDRESS, ABI_COUNTER, signer);

  // Current value for the Counter
  const currentCount = await counter.number();
  console.log(`Current count: ${currentCount}`);

  // Encode the function call data
  const encodedData = counter.interface.encodeFunctionData("increment");
  console.log(encodedData);

  // Route the calldata through the SingleCall contract to the Counter contract
  const incTransaction = await singleCall.execute(COUNTER_ADDRESS, encodedData);
  await incTransaction.wait();

  console.log(`Transaction hash: ${incTransaction.hash}`);
  console.log(`View tx on explorer: ${TX_EXPLORER_URL}${incTransaction.hash}`);

  // Check the Counter value again
  const updatedCount = await counter.number();
  console.log(`Updated count: ${updatedCount}`);
};

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
