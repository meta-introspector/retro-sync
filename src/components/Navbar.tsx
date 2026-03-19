import { motion } from "framer-motion";

const Navbar = () => {
  return (
    <motion.nav
      className="fixed top-0 left-0 right-0 z-50 glass"
      initial={{ opacity: 0, y: -20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5 }}
    >
      <div className="container mx-auto px-6 h-16 flex items-center justify-between">
        <div className="font-bold text-xl text-gradient-primary">Retrosync</div>
        <div className="hidden sm:flex items-center gap-8 text-sm text-muted-foreground">
          <a href="#features" className="hover:text-foreground transition-colors">Features</a>
          <a href="#architecture" className="hover:text-foreground transition-colors">Architecture</a>
          <a href="#compliance" className="hover:text-foreground transition-colors">Compliance</a>
        </div>
        <button className="px-4 py-2 rounded-lg bg-primary text-primary-foreground text-sm font-semibold hover:brightness-110 transition-all">
          Launch App
        </button>
      </div>
    </motion.nav>
  );
};

export default Navbar;
