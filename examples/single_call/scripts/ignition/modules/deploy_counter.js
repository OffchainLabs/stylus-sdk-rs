const { buildModule } = require("@nomicfoundation/hardhat-ignition/modules");

module.exports = buildModule("deploy_counter", (m) => {
  const deployCounter = m.contract("Counter", []);

  m.call(deployCounter, "setNumber", [42]);

  return { deployCounter };
});
