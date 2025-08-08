import { WagmiProvider } from "wagmi";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RainbowKitProvider, darkTheme } from "@rainbow-me/rainbowkit";
import { config } from "../utils/config";
import ConnectButton from "../components/ConnectButton";
import MintWidget from "../components/MintWidget";
import NFTGallery from "../components/NFTGallery";
import "@rainbow-me/rainbowkit/styles.css";

const queryClient = new QueryClient();

export default function Home() {
  return (
    <WagmiProvider config={config}>
      <QueryClientProvider client={queryClient}>
        <RainbowKitProvider theme={darkTheme()}>
          <div className="min-h-screen bg-[#0f1014] px-4 py-8">
            <ConnectButton />

            <div className="max-w-6xl mx-auto pt-16">
              <h1 className="text-4xl font-bold text-center text-white mb-2">
                Discover & Collect
              </h1>
              <h1 className="text-4xl font-bold text-center text-white mb-6">
                Extraordinary NFTs
              </h1>
              <p className="text-center text-gray-400 mb-8">
                Enter the world of digital art and collectibles. Explore unique
                NFTs created by artists worldwide.
              </p>

              <div className="flex justify-center gap-4 mb-12">
                <button className="bg-gradient-to-r from-pink-500 to-purple-500 text-white px-6 py-2 rounded-full">
                  Start Creating
                </button>
                <button className="border border-gray-600 text-white px-6 py-2 rounded-full">
                  Watch Demo
                </button>
              </div>

              <MintWidget />
              <NFTGallery />
            </div>
          </div>
        </RainbowKitProvider>
      </QueryClientProvider>
    </WagmiProvider>
  );
}
