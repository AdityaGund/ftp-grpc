import { Layout } from "@/components/Layout";
import { FileUpload } from "@/components/FileUpload";
import { Toaster } from "@/components/ui/sonner";

function App() {
  return (
    <>
      <Layout>
        <div className="flex justify-center items-center min-h-[calc(100vh-8rem)]">
          <FileUpload />
        </div>
      </Layout>
      <Toaster position="bottom-right" theme="system" />
    </>
  );
}

export default App;