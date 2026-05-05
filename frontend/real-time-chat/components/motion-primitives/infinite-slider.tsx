"use client";

import { motion } from "framer-motion";
import { ReactNode } from "react";

export function InfiniteSlider({ children }: { children: ReactNode }) {
  return (
    <motion.div
      className="flex gap-6"
      animate={{ x: ["0%", "-50%"] }}
      transition={{
        repeat: Infinity,
        ease: "linear",
        duration: 20,
      }}
    >
      {children}
    </motion.div>
  );
}
