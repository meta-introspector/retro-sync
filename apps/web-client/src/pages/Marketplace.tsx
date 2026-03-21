import React, { useState } from 'react';
import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";
import FrequencyVisualizer from "@/components/FrequencyVisualizer";
import OwnershipHistory from "@/components/OwnershipHistory";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Tag, User, ShoppingBag, Shield, Cpu, Activity, Share2, Calendar, HardDrive } from "lucide-react";

const Marketplace = () => {
  const [activePlay, setActivePlay] = useState<number | null>(null);

  const listings = [
    {
      id: 1,
      title: "Resonance Pulse",
      genre: "Techno / Industrial",
      releaseDate: "2026-03-21",
      creator: "0x742d35Cc6634C0532925a3b844Bc454e4438f44e",
      owner: "0x89205A3A3b2A69De6Dbf7f01ED13B2108B2c43e7",
      isrc: "US-ABC-24-00001",
      band: 7,
      rarity: "Legendary",
      priceBtt: "150,000",
      seedingNodes: 124,
      history: [
        { from: "0x742d35Cc6634C0532925a3b844Bc454e4438f44e", to: "0x89205A3A3b2A69De6Dbf7f01ED13B2108B2c43e7", timestamp: "2026-03-22 14:30", type: "sale" as const, price: "150,000" },
        { from: "0x0000000000000000000000000000000000000000", to: "0x742d35Cc6634C0532925a3b844Bc454e4438f44e", timestamp: "2026-03-21 09:15", type: "mint" as const }
      ]
    },
    {
      id: 2,
      title: "Fractal Beats",
      genre: "IDM / Glitch",
      releaseDate: "2026-03-20",
      creator: "0x1234567890123456789012345678901234567890",
      owner: "0x1234567890123456789012345678901234567890",
      isrc: "GB-XYZ-24-00442",
      band: 3,
      rarity: "Common",
      priceBtt: "25,000",
      seedingNodes: 42,
      history: [
        { from: "0x0000000000000000000000000000000000000000", to: "0x1234567890123456789012345678901234567890", timestamp: "2026-03-20 18:22", type: "mint" as const }
      ]
    }
  ];

  return (
    <div className="min-h-screen bg-background text-foreground scanline font-mono">
      <Navbar />
      
      <main className="container mx-auto px-6 py-24">
        <div className="flex flex-col md:flex-row justify-between items-end gap-6 mb-12 border-b border-zinc-800 pb-12">
          <div className="space-y-4">
            <div className="flex items-center gap-2">
               <Activity className="w-4 h-4 text-primary animate-pulse" />
               <span className="text-[10px] font-bold text-primary uppercase tracking-[0.4em]">Node Network: Active</span>
            </div>
            <h1 className="text-5xl font-black italic tracking-tighter uppercase">
              Encrypted <span className="text-gradient-primary">Exchange</span>
            </h1>
            <p className="text-zinc-500 max-w-lg text-sm border-l-2 border-primary/20 pl-4 leading-tight">
              Peer-to-peer asset acquisition. All transactions settled on-chain. Sovereign ownership is the only standard.
            </p>
          </div>
          <div className="flex gap-2">
            <button className="px-4 py-2 border border-zinc-800 text-[10px] font-bold uppercase tracking-widest hover:bg-zinc-900 transition-colors">Sort: Yield</button>
            <button className="px-4 py-2 border border-zinc-800 text-[10px] font-bold uppercase tracking-widest hover:bg-zinc-900 transition-colors">Sort: Rarity</button>
          </div>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-12">
          {listings.map((item) => (
            <Card key={item.id} className="bg-zinc-950 border border-zinc-800 rounded-none overflow-hidden flex flex-col group hover:border-primary/50 transition-all duration-500 relative">
              {/* Corner Accents */}
              <div className="absolute top-0 right-0 w-8 h-8 border-t border-r border-zinc-800 group-hover:border-primary transition-colors" />
              <div className="absolute bottom-0 left-0 w-8 h-8 border-b border-l border-zinc-800 group-hover:border-primary transition-colors" />

              <div className="relative aspect-video bg-black p-6 flex flex-col justify-between overflow-hidden border-b border-zinc-800">
                <div className="absolute inset-0 opacity-20 pointer-events-none group-hover:opacity-40 transition-opacity">
                   <div className="absolute top-0 left-0 w-full h-full bg-[radial-gradient(circle_at_50%_50%,hsl(var(--primary)),transparent_70%)]" />
                </div>

                <div className="relative z-10 flex justify-between items-start">
                  <div className="flex flex-col gap-1">
                    <Badge className="bg-primary text-primary-foreground font-black uppercase rounded-none px-3 text-[10px]">
                      {item.rarity} Release
                    </Badge>
                    <div className="flex items-center gap-2 text-[10px] font-bold text-zinc-500 bg-black/80 px-2 py-1 border border-zinc-800">
                       <HardDrive className="w-3 h-3 text-primary" />
                       BTFS SEEDING: {item.seedingNodes} NODES
                    </div>
                  </div>
                  <div className="text-[10px] font-mono text-zinc-500 bg-black/80 px-2 py-1 border border-zinc-800">
                    ID: {item.isrc}
                  </div>
                </div>

                <div className="relative z-10">
                   <FrequencyVisualizer isPlaying={activePlay === item.id} />
                </div>
              </div>

              <CardContent className="p-8 space-y-8">
                <div className="flex justify-between items-start">
                  <div className="space-y-2">
                    <h2 className="text-3xl font-black italic uppercase group-hover:text-primary transition-colors tracking-tighter">
                      {item.title}
                    </h2>
                    <div className="flex items-center gap-4 text-[10px] text-zinc-500 font-bold uppercase tracking-[0.2em]">
                      <span className="flex items-center gap-1.5"><Tag className="w-3 h-3" /> {item.genre}</span>
                      <span className="flex items-center gap-1.5"><Calendar className="w-3 h-3" /> {item.releaseDate}</span>
                    </div>
                  </div>
                  <div className="text-right">
                    <div className="text-[9px] text-zinc-600 uppercase font-black tracking-widest mb-1">Contract Value</div>
                    <div className="text-2xl font-black text-primary font-mono">{item.priceBtt} BTT</div>
                  </div>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                   <div className="bg-zinc-900/30 p-4 border border-zinc-800 group-hover:border-zinc-700 transition-colors">
                      <div className="text-[9px] text-zinc-600 uppercase font-black mb-2 flex items-center gap-1">
                        <User className="w-2.5 h-2.5" /> Originator
                      </div>
                      <div className="text-xs font-mono text-zinc-400 truncate">{item.creator}</div>
                   </div>
                   <div className="bg-zinc-900/30 p-4 border border-zinc-800 group-hover:border-zinc-700 transition-colors">
                      <div className="text-[9px] text-zinc-600 uppercase font-black mb-2 flex items-center gap-1">
                        <ShoppingBag className="w-2.5 h-2.5" /> Current Holder
                      </div>
                      <div className="text-xs font-mono text-primary truncate">{item.owner}</div>
                   </div>
                </div>

                <div className="border-t border-zinc-900 pt-8">
                  <OwnershipHistory history={item.history} />
                </div>

                <div className="flex flex-col sm:flex-row gap-4 pt-4">
                  <button 
                    className={`flex-1 py-4 font-black uppercase tracking-widest text-xs transition-all border ${
                      activePlay === item.id 
                        ? "bg-primary text-primary-foreground border-primary" 
                        : "bg-zinc-950 text-foreground border-zinc-700 hover:border-primary"
                    }`}
                    onClick={() => setActivePlay(activePlay === item.id ? null : item.id)}
                  >
                    {activePlay === item.id ? "Kill Analysis" : "Analyze Audio"}
                  </button>
                  
                  <button 
                    className="flex-1 py-4 bg-primary text-primary-foreground font-black uppercase tracking-widest text-xs hover:bg-primary/90 shadow-[4px_4px_0px_0px_rgba(255,255,255,0.1)] active:translate-x-[2px] active:translate-y-[2px] active:shadow-none flex items-center justify-center gap-2"
                  >
                    <Share2 className="w-4 h-4" />
                    Acquire Contract
                  </button>
                </div>

                <div className="flex items-center justify-center gap-2 text-[9px] text-zinc-600 uppercase font-black pt-4 border-t border-zinc-900">
                   <Shield className="w-3 h-3 text-primary/40" />
                   Protocol Enforcement: Groth16 Active
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      </main>

      <Footer />
    </div>
  );
};

export default Marketplace;
