import { motion } from "framer-motion";
import { Link } from "react-router-dom";
import ConnectWallet from "./ConnectWallet";
import { Terminal } from "lucide-react";

const Navbar = () => {
  return (
    <motion.nav
      className="fixed top-0 left-0 right-0 z-50 bg-black/90 border-b border-zinc-800 backdrop-blur-md"
      initial={{ opacity: 0, y: -10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
    >
      <div className="container mx-auto px-6 h-16 flex items-center justify-between">
        <Link to="/" className="flex items-center gap-3 group">
          <div className="w-8 h-8 bg-primary/10 border border-primary/50 flex items-center justify-center group-hover:bg-primary/20 transition-colors">
            <Terminal className="w-4 h-4 text-primary" />
          </div>
          <span className="font-black italic text-xl uppercase tracking-tighter text-white">
            Retro<span className="text-primary">Sync</span>
          </span>
        </Link>

        <div className="hidden sm:flex items-center gap-8">
          <a href="/#features" className="text-zinc-500 hover:text-primary transition-colors uppercase tracking-widest font-black text-[10px]">
            Capabilities
          </a>
          <a href="/#pricing" className="text-zinc-500 hover:text-primary transition-colors uppercase tracking-widest font-black text-[10px]">
            Economics
          </a>
          <Link to="/marketplace" className="text-zinc-500 hover:text-primary transition-colors uppercase tracking-widest font-black text-[10px]">
            Exchange
          </Link>
          <Link to="/upload" className="bg-primary/10 border border-primary/50 px-4 py-1.5 text-primary hover:bg-primary/20 transition-all font-black uppercase tracking-widest text-[10px]">
            [ Secure_Upload ]
          </Link>
        </div>
        
        <ConnectWallet />
      </div>
    </motion.nav>
  );
};

export default Navbar;
