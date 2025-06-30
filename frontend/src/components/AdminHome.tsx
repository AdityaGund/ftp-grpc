import React, { useState, useEffect } from "react";
import { toast } from "sonner";
import { useAuth } from "@/lib/AuthContext";
import { api } from "@/lib/api";
import FileInfoDisplay from "./ui/FileInfo";
import AdminUserManagement from "./ui/AdminUserManagement";
import { TabContainer } from "./ui/TabContainer";
import { FileText, User, Upload } from "lucide-react";
import FileUpload from "./FileUpload";

const AdminHome: React.FC = () => {
  useAuth();

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

  const [files, setFiles] = useState<FileInfo[]>([]);
  const [loadingFiles, setLoadingFiles] = useState<boolean>(false);

  const token = localStorage.getItem("jwt");

  // Fetch file information from admin server
  const loadFileInfo = async () => {
    setLoadingFiles(true);
    try {
      const response = await api.fetchAdminFileInfo(token);
      setFiles(response.data.data || []);
    } catch (error) {
      console.error("Error fetching admin file info", error);
      toast.error("Failed to load file history");
    } finally {
      setLoadingFiles(false);
    }
  };

  // Fetch once on mount
  useEffect(() => {
    loadFileInfo();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const handleRefreshFiles = () => {
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
      id: "userManagement",
      label: "User Management",
      icon: User,
      content: <AdminUserManagement token={token} />
    },
    {
      id: "files",
      label: "File History",
      icon: FileText,
      content: (
        <FileInfoDisplay
          files={files}
          loading={loadingFiles}
          onRefresh={handleRefreshFiles}
          title="Global File Transfer History"
          description="Comprehensive view of all file transfers across the entire banking network."
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
};

export default AdminHome;