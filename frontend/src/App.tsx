import { Layout } from "@/components/Layout";
import { FileUpload } from "@/components/FileUpload";
import Login from "@/components/Login";
import { Toaster } from "@/components/ui/sonner";
import { Route, Routes } from "react-router-dom";

function App() {
  return (
    <>
      <Routes>
        <Route
          path="/"
          element={
            <Layout>
              <div className="flex justify-center items-center min-h-[calc(100vh-8rem)]">
                <Login />
              </div>
            </Layout>
          }
        />
        <Route
          path="/FileUpload"
          element={
            <Layout>
              <div className="flex justify-center items-center min-h-[calc(100vh-8rem)]">
                <FileUpload />
              </div>
            </Layout>
          }
        />
      </Routes>
      <Toaster position="bottom-right" theme="system" />
    </>
  );
}

export default App;