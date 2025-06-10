import { Layout } from "@/components/Layout";
import { FileUpload } from "@/components/FileUpload";
// import { TransferHistory } from "@/components/TransferHistory";
import { Toaster } from "@/components/ui/sonner";

function App() {
  return (
    <>
      <Layout>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
          <div className="flex justify-center">
            <FileUpload />
          </div>
          {/* <div>
            <TransferHistory />
          </div> */}
        </div>
      </Layout>
      <Toaster />
    </>
  );
}

export default App;