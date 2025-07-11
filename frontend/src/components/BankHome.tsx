import { useState, useEffect } from "react";
import { Upload, FileText } from "lucide-react";
import { toast } from "sonner";
import { getErrorMessage } from "@/lib/utils";
import { fetchFileInfo } from "@/lib/api";
import { useAuth } from "@/lib/AuthContext";
import FileUpload from "./FileUpload";
import FileInfoDisplay from "./ui/FileInfo";
import { TabContainer } from "./ui/TabContainer";

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
      toast.error(getErrorMessage(error));
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

  const tabs = [
    {
      id: "upload",
      label: "File Upload",
      icon: Upload,
      content: <FileUpload />
    },
    {
      id: "files",
      label: "File History",
      icon: FileText,
      content: (
        <FileInfoDisplay
          files={files}
          loading={loading}
          onRefresh={handleRefresh}
          title="File Transfer History"
          description={`Complete file transfer records for bank: ${username}`}
        />
      )
    }
  ];

  return (
    <div className="w-full max-w-6xl mx-auto">
      <TabContainer 
        tabs={tabs} 
        defaultTab="upload"
      />
    </div>
  );
}

export default BankHome; 