import "@nomicfoundation/hardhat-toolbox";
import "@nomicfoundation/hardhat-toolbox";
import "@openzeppelin/hardhat-upgrades";
import { HardhatUserConfig } from "hardhat/config";

const ACCOUNTS_MNEMONIC = "test test test test test test test test test test test junk";

const config: HardhatUserConfig = {
    solidity: {
        version: "0.8.16",
        settings: {
            optimizer: {
                enabled: true,
                runs: 1000,
            },
        },
    },
    networks: {
        hardhat: {
            chainId: 2008,
            gasPrice: 0,
            initialBaseFeePerGas: 0,
            mining: {
                auto: true,
            },
        },
        anvil: {
            url: "http://localhost:8546",
            accounts: {
                mnemonic: ACCOUNTS_MNEMONIC,
            },
        },
        stratus: {
            url: "http://localhost:3000",
            accounts: {
                mnemonic: ACCOUNTS_MNEMONIC,
            },
        },
    },
};

export default config;
