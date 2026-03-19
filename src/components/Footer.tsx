const Footer = () => {
  return (
    <footer className="border-t border-border py-12">
      <div className="container mx-auto px-6 flex flex-col sm:flex-row items-center justify-between gap-4">
        <div className="font-bold text-lg text-gradient-primary">Retrosync</div>
        <div className="text-xs text-muted-foreground font-mono">
          AGPL-3.0 · Retrosync Media Group © {new Date().getFullYear()}
        </div>
        <div className="flex gap-6 text-sm text-muted-foreground">
          <a href="#" className="hover:text-primary transition-colors">Docs</a>
          <a href="#" className="hover:text-primary transition-colors">GitHub</a>
          <a href="#" className="hover:text-primary transition-colors">Contact</a>
        </div>
      </div>
    </footer>
  );
};

export default Footer;
