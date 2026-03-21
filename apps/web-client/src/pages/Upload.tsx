import Navbar from "@/components/Navbar";
import Footer from "@/components/Footer";
import MetadataUpload from "@/components/MetadataUpload";

const Upload = () => {
  return (
    <div className="min-h-screen bg-background">
      <Navbar />
      <div className="container mx-auto px-6 py-12">
        <MetadataUpload />
      </div>
      <Footer />
    </div>
  );
};

export default Upload;
