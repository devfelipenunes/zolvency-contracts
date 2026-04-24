// scripts/deploy_verifier.js (EVM)
const hre = require("hardhat");

async function main() {
  const GATEWAY = process.env.AXELAR_GATEWAY_EVM || "0xe432150cce91c13a887f7D836923d5597adD8E31";
  const STELLAR_SOURCE = process.env.STELLAR_GITHUB_IDENTITY_CONTRACT;

  if (!STELLAR_SOURCE) {
    console.error("❌ STELLAR_GITHUB_IDENTITY_CONTRACT is not set in .env");
    process.exit(1);
  }

  console.log("🚀 Deploying ZolvencyVerifier to EVM...");
  console.log("Gateway:", GATEWAY);
  console.log("Authorized Stellar Source:", STELLAR_SOURCE);

  const Verifier = await hre.ethers.getContractFactory("ZolvencyVerifier");
  const verifier = await Verifier.deploy(GATEWAY, STELLAR_SOURCE);

  await verifier.deployed();

  console.log("✅ ZolvencyVerifier deployed to:", verifier.address);
  console.log("Save this to your .env: VERIFIER_CONTRACT_EVM=" + verifier.address);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
