import { motion } from "framer-motion";
import { Link } from "react-router-dom";
import ConnectWallet from "./ConnectWallet";
import { Terminal, Menu, X } from "lucide-react";
import { useState } from "react";

const Navbar = () => {
  const [mobileOpen, setMobileOpen] = useState(false);

  return (
    <motion.nav
      className="fixed top-0 left-0 right-0 z-50 bg-background/90 border-b border-border backdrop-blur-md"
      initial={{ opacity: 0, y: -10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.4 }}
    >
      <div className="container mx-auto px-6 h-16 flex items-center justify-between">
        <Link to="/" className="flex items-center gap-3 group">
          <div className="w-8 h-8 bg-primary/10 border border-primary/50 flex items-center justify-center group-hover:bg-primary/20 transition-colors">
            <Terminal className="w-4 h-4 text-primary" />
          </div>
          <span className="font-bold text-lg tracking-tight text-foreground">
            Retro<span className="text-primary">Sync</span>
          </span>
        </Link>

        {/* Desktop nav */}
        <div className="hidden md:flex items-center gap-8">
          <a href="/#features" className="text-muted-foreground hover:text-primary transition-colors text-sm font-medium">
            Capabilities
          </a>
          <a href="/#pricing" className="text-muted-foreground hover:text-primary transition-colors text-sm font-medium">
            Economics
          </a>
          <Link to="/marketplace" className="text-muted-foreground hover:text-primary transition-colors text-sm font-medium">
            Exchange
          </Link>
          <Link to="/upload" className="bg-primary/10 border border-primary/40 px-4 py-1.5 text-primary hover:bg-primary/20 transition-all text-sm font-medium">
            Secure Upload
          </Link>
        </div>

        <div className="flex items-center gap-3">
          <ConnectWallet />
          {/* Mobile hamburger */}
          <button
            className="md:hidden p-2 text-muted-foreground hover:text-foreground transition-colors"
            onClick={() => setMobileOpen(!mobileOpen)}
          >
            {mobileOpen ? <X className="w-5 h-5" /> : <Menu className="w-5 h-5" />}
          </button>
        </div>
      </div>

      {/* Mobile menu */}
      {mobileOpen && (
        <motion.div
          className="md:hidden border-t border-border bg-background/95 backdrop-blur-md px-6 py-6 space-y-4"
          initial={{ opacity: 0, y: -10 }}
          animate={{ opacity: 1, y: 0 }}
        >
          <a href="/#features" className="block text-muted-foreground hover:text-primary transition-colors text-sm font-medium" onClick={() => setMobileOpen(false)}>
            Capabilities
          </a>
          <a href="/#pricing" className="block text-muted-foreground hover:text-primary transition-colors text-sm font-medium" onClick={() => setMobileOpen(false)}>
            Economics
          </a>
          <Link to="/marketplace" className="block text-muted-foreground hover:text-primary transition-colors text-sm font-medium" onClick={() => setMobileOpen(false)}>
            Exchange
          </Link>
          <Link to="/upload" className="block bg-primary/10 border border-primary/40 px-4 py-2 text-primary text-sm font-medium text-center" onClick={() => setMobileOpen(false)}>
            Secure Upload
          </Link>
        </motion.div>
      )}
    </motion.nav>
  );
};

export default Navbar;
