"use client";

import { ConnectButton } from "@rainbow-me/rainbowkit";

const CustomConnectButton = () => {
  if (typeof window === "undefined") return null;

  return (
    <div className="absolute top-4 right-4">
      <ConnectButton
        accountStatus={{
          smallScreen: "avatar",
          largeScreen: "full",
        }}
        showBalance={{
          smallScreen: false,
          largeScreen: true,
        }}
      />
    </div>
  );
};

export default CustomConnectButton;
