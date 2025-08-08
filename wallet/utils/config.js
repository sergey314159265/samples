import { http, createConfig } from "wagmi";
import { mainnet, sepolia } from "wagmi/chains";
import {
  injected,
  metaMask,
  walletConnect,
  coinbaseWallet,
} from "wagmi/connectors";

export const config = createConfig({
  chains: [sepolia, mainnet],
  transports: {
    [sepolia.id]: http(),
    [mainnet.id]: http(),
  },
  connectors: [
    injected(),
    walletConnect({
      projectId: process.env.NEXT_PUBLIC_PROJECT_ID || "YOUR_PROJECT_ID",
    }),
    metaMask(),
    coinbaseWallet({
      appName: "NFT Minting App",
    }),
  ],
});
