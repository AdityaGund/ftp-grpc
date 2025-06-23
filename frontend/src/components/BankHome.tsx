import { useState, useEffect } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Upload, FileText } from "lucide-react";
import { toast } from "sonner";
import { fetchFileInfo } from "@/lib/api";
import { useAuth } from "@/lib/AuthContext";
import FileUpload from "./FileUpload";
import FileInfoDisplay from "./ui/FileInfo";
// import BankFileInfo from "./BankFileInfo";

interface FileInfo {
  _id: string | object;
  name: string;
  path: string;
  sender_bank_id: string;
  receiver_bank_id: string;
  message: string;
  time_sent_at: string;
  time_received_at: string;
}

export function BankHome() {
  const [activeTab, setActiveTab] = useState<"upload" | "files">("upload");
  const [files, setFiles] = useState<FileInfo[]>([]);
  const [loading, setLoading] = useState<boolean>(false);
  const { username } = useAuth();
  const token = localStorage.getItem("jwt");

  const loadFileInfo = async () => {
    setLoading(true);
    try {
      const response = await fetchFileInfo(token);
      console.log("File info response:", response.data);
      setFiles(response.data.data || []);
    } catch (error) {
      console.error("Error fetching file info:", error);
      toast.error("Failed to load file information");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadFileInfo();
  }, []);

  const handleRefresh = () => {
    toast.success("Refreshing file information...");
    loadFileInfo();
  };

  return (
    <div className="w-full max-w-6xl mx-auto">
      <Card className="border-2 border-border shadow-md mb-6">
        <CardHeader className="border-b-2 border-border pb-4">
          <CardTitle className="text-xl">Bank Dashboard</CardTitle>
        </CardHeader>
        <CardContent className="p-6">
          <div className="flex space-x-2 mb-6">
            <Button
              variant={activeTab === "upload" ? "default" : "outline"}
              onClick={() => setActiveTab("upload")}
              className="flex items-center gap-2"
            >
              <Upload className="h-4 w-4" />
              File Upload
            </Button>
            <Button
              variant={activeTab === "files" ? "default" : "outline"}
              onClick={() => setActiveTab("files")}
              className="flex items-center gap-2"
            >
              <FileText className="h-4 w-4" />
              File History
            </Button>
          </div>
        </CardContent>
      </Card>

      {activeTab === "upload" && <FileUpload />}
      {activeTab === "files" && (
        // <BankFileInfo 
        //   files={files}
        //   loading={loading}
        //   onRefresh={handleRefresh}
        //   username={username}
        // />
        <FileInfoDisplay
          files={files}
          loading={loading}
          onRefresh={handleRefresh}
          title="File Transfer History"
          description={`View all file transfers for bank: ${username}`}
        />
      )}
    </div>
  );
}

export default BankHome; 