import { Terminal, Github, Twitter, Shield, Cpu } from "lucide-react";

const Footer = () => {
  return (
    <footer className="bg-background border-t border-border py-16 md:py-20">
      <div className="container mx-auto px-6">
        <div className="grid grid-cols-2 md:grid-cols-4 gap-10 mb-14">
          <div className="col-span-2">
            <div className="flex items-center gap-3 mb-5">
              <div className="w-8 h-8 bg-primary/10 border border-primary/50 flex items-center justify-center">
                <Terminal className="w-4 h-4 text-primary" />
              </div>
              <span className="font-bold text-lg tracking-tight">RetroSync</span>
            </div>
            <p className="text-muted-foreground max-w-sm text-sm leading-relaxed mb-6">
              Decentralized distribution protocol. Built for artists who demand sovereignty.
              Powered by BTFS and zero-knowledge cryptography.
            </p>
            <div className="flex gap-3">
              <a href="#" className="w-9 h-9 bg-card border border-border flex items-center justify-center hover:border-primary/50 transition-colors">
                <Github className="w-4 h-4 text-muted-foreground" />
              </a>
              <a href="#" className="w-9 h-9 bg-card border border-border flex items-center justify-center hover:border-primary/50 transition-colors">
                <Twitter className="w-4 h-4 text-muted-foreground" />
              </a>
            </div>
          </div>

          <div>
            <h4 className="text-xs uppercase tracking-wider text-muted-foreground font-medium mb-5">Navigate</h4>
            <ul className="space-y-3 text-sm text-muted-foreground">
              <li><a href="/#features" className="hover:text-primary transition-colors">Capabilities</a></li>
              <li><a href="/#pricing" className="hover:text-primary transition-colors">Economics</a></li>
              <li><a href="/marketplace" className="hover:text-primary transition-colors">Exchange</a></li>
              <li><a href="/upload" className="hover:text-primary transition-colors">Upload</a></li>
            </ul>
          </div>

          <div>
            <h4 className="text-xs uppercase tracking-wider text-muted-foreground font-medium mb-5">Security</h4>
            <ul className="space-y-3 text-sm text-muted-foreground">
              <li className="flex items-center gap-2"><Shield className="w-3 h-3 text-primary" /> 256-bit AES</li>
              <li className="flex items-center gap-2"><Cpu className="w-3 h-3 text-primary" /> Groth16 Proofs</li>
              <li>No PII Stored</li>
              <li><a href="/docs/whitepaper.md" className="hover:text-primary transition-colors">Whitepaper</a></li>
            </ul>
          </div>
        </div>

        <div className="flex flex-col md:flex-row justify-between items-center pt-10 border-t border-border gap-4">
          <div className="text-xs text-muted-foreground">
            © 2026 Retrosync Media Group — AGPL-3.0
          </div>
          <div className="flex items-center gap-4">
            <span className="text-xs text-primary/60 font-mono">Status: Nominal</span>
            <span className="text-xs text-muted-foreground font-mono">v1.0.4</span>
          </div>
        </div>
      </div>
    </footer>
  );
};

export default Footer;
