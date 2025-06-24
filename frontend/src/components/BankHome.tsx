import { useState, useEffect } from "react";
import { Upload, FileText, Send, Download } from "lucide-react";
import { toast } from "sonner";
import { fetchFileInfoSent, fetchFileInfoReceived } from "@/lib/api";
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
  const [sentFiles, setSentFiles] = useState<FileInfo[]>([]);
  const [recvFiles, setRecvFiles] = useState<FileInfo[]>([]);
  const [loadingSent, setLoadingSent] = useState<boolean>(false);
  const [loadingRecv, setLoadingRecv] = useState<boolean>(false);
  const { username } = useAuth();
  const token = localStorage.getItem("jwt");

  const loadSent = async () => {
    setLoadingSent(true);
    try {
      const response = await fetchFileInfoSent(token);
      setSentFiles(response.data.data || []);
    } catch (error) {
      console.error("Error fetching sent file info:", error);
      toast.error("Failed to load sent file information");
    } finally {
      setLoadingSent(false);
    }
  };

  const loadReceived = async () => {
    setLoadingRecv(true);
    try {
      const response = await fetchFileInfoReceived(token);
      setRecvFiles(response.data.data || []);
    } catch (error) {
      console.error("Error fetching received file info:", error);
      toast.error("Failed to load received file information");
    } finally {
      setLoadingRecv(false);
    }
  };

  useEffect(() => {
    loadSent();
    loadReceived();
  }, []);

  const handleRefreshSent = () => {
    toast.success("Refreshing sent files...");
    loadSent();
  };

  const handleRefreshRecv = () => {
    toast.success("Refreshing received files...");
    loadReceived();
  };

  const historyTabs = [
    {
      id: "sent",
      label: "Sent",
      icon: Send,
      content: (
        <FileInfoDisplay
          files={sentFiles}
          loading={loadingSent}
          onRefresh={handleRefreshSent}
          title="Sent Files"
          description={`Files sent by bank: ${username}`}
        />
      )
    },
    {
      id: "received",
      label: "Received",
      icon: Download,
      content: (
        <FileInfoDisplay
          files={recvFiles}
          loading={loadingRecv}
          onRefresh={handleRefreshRecv}
          title="Received Files"
          description={`Files received by bank: ${username}`}
        />
      )
    }
  ];

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
        <TabContainer tabs={historyTabs} defaultTab="sent" />
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