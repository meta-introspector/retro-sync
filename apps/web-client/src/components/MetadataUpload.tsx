import { useState } from "react";
import { motion } from "framer-motion";
import { Music, Upload, CheckCircle2, AlertCircle, Info, Terminal, Cpu } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Card, CardContent } from "@/components/ui/card";
import { useWallet } from "@/hooks/useWallet";

const MetadataUpload = () => {
  const { wallet } = useWallet();
  const [isUploading, setIsUploading] = useState(false);
  const [isSuccess, setIsSuccess] = useState(false);
  const [formData, setFormData] = useState({
    title: "",
    isrc: "",
    description: "",
  });

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => {
    const { id, value } = e.target;
    setFormData((prev) => ({ ...prev, [id]: value }));
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!wallet.connected) return;

    setIsUploading(true);
    // Simulate API call
    await new Promise((resolve) => setTimeout(resolve, 2000));
    console.log("Submitting metadata:", {
      ...formData,
      uploaderWallet: wallet.address,
      timestamp: new Date().toISOString(),
    });
    setIsUploading(false);
    setIsSuccess(true);
  };

  if (!wallet.connected) {
    return (
      <div className="flex flex-col items-center justify-center p-12 text-center border border-zinc-800 bg-zinc-950">
        <div className="w-16 h-16 bg-primary/10 border border-primary/50 flex items-center justify-center mb-6">
          <AlertCircle className="w-8 h-8 text-primary" />
        </div>
        <h2 className="text-2xl font-black italic uppercase mb-2 text-white italic tracking-tighter">Access Denied</h2>
        <p className="text-zinc-500 font-mono text-sm max-w-sm mb-8 leading-tight">
          > Error: Wallet_Not_Connected<br />
          > Action: Connect a valid TronLink or Coinbase wallet to access the upload portal.
        </p>
      </div>
    );
  }

  if (isSuccess) {
    return (
      <motion.div 
        className="flex flex-col items-center justify-center p-12 text-center border border-primary/50 bg-primary/5"
        initial={{ opacity: 0, scale: 0.9 }}
        animate={{ opacity: 1, scale: 1 }}
      >
        <div className="w-16 h-16 bg-primary border border-primary flex items-center justify-center mb-6">
          <CheckCircle2 className="w-8 h-8 text-primary-foreground" />
        </div>
        <h2 className="text-2xl font-black italic uppercase mb-2 text-white tracking-tighter">Transmission Successful</h2>
        <p className="text-zinc-400 font-mono text-sm max-w-sm mb-8 leading-tight">
          > Metadata registered on ledger.<br />
          > ID: <span className="text-primary">{wallet.address}</span><br />
          > All records are now immutable.
        </p>
        <button 
          onClick={() => setIsSuccess(false)}
          className="px-8 py-3 bg-zinc-900 border border-zinc-800 text-[10px] font-black uppercase tracking-widest hover:border-primary transition-all"
        >
          [ New_Transmission ]
        </button>
      </motion.div>
    );
  }

  return (
    <div className="max-w-2xl mx-auto py-8 font-mono">
      <Card className="bg-zinc-950 border border-zinc-800 rounded-none overflow-hidden relative">
        <div className="absolute top-0 right-0 p-4 opacity-10">
          <Cpu className="w-20 h-20 text-primary" />
        </div>

        <div className="p-8 border-b border-zinc-800 bg-zinc-900/30 flex items-center gap-4">
          <div className="p-3 bg-primary/10 border border-primary/50">
            <Music className="w-6 h-6 text-primary" />
          </div>
          <div>
            <h2 className="text-2xl font-black italic uppercase tracking-tighter">Upload Protocol</h2>
            <div className="text-[10px] text-zinc-500 font-bold uppercase tracking-[0.2em]">Secure_Metadata_Ingestion</div>
          </div>
        </div>

        <CardContent className="p-8 space-y-8">
          <div className="p-4 bg-primary/5 border border-primary/20 flex items-start gap-3">
            <Terminal className="w-5 h-5 text-primary mt-0.5 shrink-0" />
            <p className="text-[11px] text-zinc-400 leading-tight">
              <span className="text-primary font-bold">WARNING:</span> Identity protection active. Your artist name will NOT be requested or stored. All rights are bound to your wallet address: <span className="text-primary font-bold">{wallet.address.slice(0, 12)}...</span>
            </p>
          </div>

          <form onSubmit={handleSubmit} className="space-y-6">
            <div className="space-y-2">
              <Label htmlFor="title" className="text-[10px] font-black uppercase tracking-widest text-zinc-500">_song_title</Label>
              <Input 
                id="title" 
                placeholder="PROMPT: Enter Title" 
                className="bg-black border-zinc-800 rounded-none focus:border-primary transition-colors text-sm"
                value={formData.title}
                onChange={handleInputChange}
                required
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="isrc" className="text-[10px] font-black uppercase tracking-widest text-zinc-500">_isrc_code</Label>
              <Input 
                id="isrc" 
                placeholder="PROMPT: US-XXX-00-00000" 
                className="bg-black border-zinc-800 rounded-none focus:border-primary transition-colors text-sm font-mono"
                value={formData.isrc}
                onChange={handleInputChange}
                required
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="description" className="text-[10px] font-black uppercase tracking-widest text-zinc-500">_additional_data</Label>
              <Textarea 
                id="description" 
                placeholder="PROMPT: Optional Liner Notes" 
                className="bg-black border-zinc-800 rounded-none focus:border-primary transition-colors min-h-[100px] text-sm"
                value={formData.description}
                onChange={handleInputChange}
              />
            </div>

            <div className="pt-4">
              <div className="flex items-center justify-between mb-6 p-3 bg-zinc-900/50 border-l-2 border-primary">
                <span className="text-[10px] text-zinc-500 uppercase font-black tracking-widest">Signer_ID</span>
                <span className="text-[10px] font-mono text-primary font-bold truncate max-w-[200px]">{wallet.address}</span>
              </div>
              
              <button 
                type="submit" 
                className="w-full py-5 bg-primary text-primary-foreground font-black uppercase tracking-[0.2em] text-sm hover:bg-primary/90 shadow-[4px_4px_0px_0px_rgba(255,255,255,0.1)] active:translate-x-[2px] active:translate-y-[2px] active:shadow-none flex items-center justify-center gap-2"
                disabled={isUploading}
              >
                {isUploading ? (
                  <>
                    <div className="w-4 h-4 border-2 border-primary-foreground/20 border-t-primary-foreground rounded-full animate-spin" />
                    EXECUTING...
                  </>
                ) : (
                  <>
                    <Upload className="w-4 h-4" />
                    [ START_TRANSMISSION ]
                  </>
                )}
              </button>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
};

export default MetadataUpload;
