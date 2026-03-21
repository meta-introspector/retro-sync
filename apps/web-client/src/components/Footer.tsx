import { motion } from "framer-motion";
import { Terminal, Shield, Github, Twitter, Cpu } from "lucide-react";

const Footer = () => {
  return (
    <footer className="bg-black border-t border-zinc-900 py-20 font-mono scanline">
      <div className="container mx-auto px-6">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-12 mb-16">
          <div className="col-span-1 md:col-span-2">
            <div className="flex items-center gap-3 mb-6">
              <div className="w-8 h-8 bg-primary/10 border border-primary/50 flex items-center justify-center">
                <Terminal className="w-4 h-4 text-primary" />
              </div>
              <span className="font-black italic text-xl uppercase tracking-tighter">RetroSync</span>
            </div>
            <p className="text-zinc-500 max-w-sm text-sm leading-tight mb-8">
              A decentralized distribution protocol. Built for artists who demand sovereignty. 
              Powered by BTFS and zero-knowledge cryptography.
            </p>
            <div className="flex gap-4">
              <a href="#" className="w-10 h-10 bg-zinc-950 border border-zinc-800 flex items-center justify-center hover:border-primary transition-colors">
                <Github className="w-4 h-4" />
              </a>
              <a href="#" className="w-10 h-10 bg-zinc-950 border border-zinc-800 flex items-center justify-center hover:border-primary transition-colors">
                <Twitter className="w-4 h-4" />
              </a>
            </div>
          </div>

          <div>
            <h4 className="font-black text-[10px] uppercase tracking-[0.3em] text-zinc-400 mb-6">Endpoints</h4>
            <ul className="space-y-4 text-sm text-zinc-600">
              <li><a href="/#features" className="hover:text-primary transition-colors italic">> capabilities.sys</a></li>
              <li><a href="/#pricing" className="hover:text-primary transition-colors italic">> economics.cfg</a></li>
              <li><a href="/marketplace" className="hover:text-primary transition-colors italic">> exchange.exe</a></li>
              <li><a href="/upload" className="hover:text-primary transition-colors italic">> secure_upload.bin</a></li>
            </ul>
          </div>

          <div>
            <h4 className="font-black text-[10px] uppercase tracking-[0.3em] text-zinc-400 mb-6">Security</h4>
            <ul className="space-y-4 text-sm text-zinc-600">
              <li className="flex items-center gap-2"><Shield className="w-3 h-3 text-primary" /> 256-bit AES</li>
              <li className="flex items-center gap-2"><Cpu className="w-3 h-3 text-primary" /> Groth16 Proofs</li>
              <li className="italic">No PII Stored</li>
              <li className="italic italic">Code is Law</li>
            </ul>
          </div>
        </div>

        <div className="flex flex-col md:flex-row justify-between items-center pt-12 border-t border-zinc-900 gap-6">
          <div className="text-[10px] font-bold text-zinc-700 uppercase tracking-widest">
            © 2026 RETROSYNC MEDIA GROUP — DISTRIBUTED UNDER AGPL-3.0
          </div>
          <div className="flex items-center gap-6">
            <span className="text-[10px] font-bold text-green-900 uppercase">System Status: Nominal</span>
            <span className="text-[10px] font-bold text-zinc-800 uppercase">v1.0.4-STABLE</span>
          </div>
        </div>
      </div>
    </footer>
  );
};

export default Footer;
